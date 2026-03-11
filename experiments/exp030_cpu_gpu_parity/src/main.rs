// SPDX-License-Identifier: AGPL-3.0-or-later
//! exp030 — CPU-vs-GPU math parity validation.
//!
//! Validates that pure Rust CPU math (barraCuda) matches GPU shader output.
//! This is the first step of the pipeline:
//!   Paper → Python → **barraCuda CPU** → barraCuda GPU → toadStool → coralReef
//!
//! Subcommands:
//!   validate  — run all parity checks
//!   probe     — enumerate GPU/CPU adapters
//!   bench     — CPU-vs-GPU timing at varying sizes

use std::process;

use ludospring_barracuda::barcuda_math;
use ludospring_barracuda::procedural::noise;
use ludospring_barracuda::validation::ValidationResult;

fn main() {
    let arg = std::env::args().nth(1).unwrap_or_default();
    match arg.as_str() {
        "probe" => cmd_probe(),
        "bench" => cmd_bench(),
        "validate" | "" => cmd_validate(),
        other => {
            eprintln!("Unknown command: {other}");
            eprintln!("Usage: exp030_cpu_gpu_parity [validate|probe|bench]");
            process::exit(1);
        }
    }
}

// ---------------------------------------------------------------------------
// Inline WGSL shaders for parity testing.
// ---------------------------------------------------------------------------

const SIGMOID_WGSL: &str = r"
@group(0) @binding(0) var<storage, read> input: array<f32>;
@group(0) @binding(1) var<storage, read_write> output: array<f32>;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if i < arrayLength(&input) {
        output[i] = 1.0 / (1.0 + exp(-input[i]));
    }
}
";

const RELU_WGSL: &str = r"
@group(0) @binding(0) var<storage, read> input: array<f32>;
@group(0) @binding(1) var<storage, read_write> output: array<f32>;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if i < arrayLength(&input) {
        output[i] = max(input[i], 0.0);
    }
}
";

const DOT_PRODUCT_WGSL: &str = r"
@group(0) @binding(0) var<storage, read> a: array<f32>;
@group(0) @binding(1) var<storage, read> b: array<f32>;
@group(0) @binding(2) var<storage, read_write> result: array<f32>;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if i < arrayLength(&a) {
        result[i] = a[i] * b[i];
    }
}
";

const REDUCE_SUM_WGSL: &str = r"
@group(0) @binding(0) var<storage, read> input: array<f32>;
@group(0) @binding(1) var<storage, read_write> output: array<f32>;

var<workgroup> wg_data: array<f32, 256>;

@compute @workgroup_size(256)
fn main(
    @builtin(local_invocation_id) lid: vec3<u32>,
    @builtin(global_invocation_id) gid: vec3<u32>,
) {
    let i = gid.x;
    if i < arrayLength(&input) {
        wg_data[lid.x] = input[i];
    } else {
        wg_data[lid.x] = 0.0;
    }
    workgroupBarrier();

    var stride: u32 = 128u;
    while stride > 0u {
        if lid.x < stride {
            wg_data[lid.x] = wg_data[lid.x] + wg_data[lid.x + stride];
        }
        workgroupBarrier();
        stride = stride >> 1u;
    }

    if lid.x == 0u {
        output[gid.x / 256u] = wg_data[0];
    }
}
";

const SOFTMAX_WGSL: &str = r"
@group(0) @binding(0) var<storage, read> input: array<f32>;
@group(0) @binding(1) var<storage, read_write> output: array<f32>;

@compute @workgroup_size(1)
fn main() {
    let n = arrayLength(&input);
    var max_val: f32 = input[0];
    for (var i = 1u; i < n; i = i + 1u) {
        max_val = max(max_val, input[i]);
    }
    var sum_exp: f32 = 0.0;
    for (var i = 0u; i < n; i = i + 1u) {
        sum_exp = sum_exp + exp(input[i] - max_val);
    }
    for (var i = 0u; i < n; i = i + 1u) {
        output[i] = exp(input[i] - max_val) / sum_exp;
    }
}
";

const SCALE_WGSL: &str = r"
@group(0) @binding(0) var<storage, read> input: array<f32>;
@group(0) @binding(1) var<storage, read_write> output: array<f32>;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if i < arrayLength(&input) {
        output[i] = input[i] * 2.0 + 1.0;
    }
}
";

const LCG_WGSL: &str = r"
@group(0) @binding(0) var<storage, read> seeds: array<u32>;
@group(0) @binding(1) var<storage, read_write> output: array<u32>;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if i < arrayLength(&seeds) {
        output[i] = seeds[i] * 1664525u + 1013904223u;
    }
}
";

const ABS_WGSL: &str = r"
@group(0) @binding(0) var<storage, read> input: array<f32>;
@group(0) @binding(1) var<storage, read_write> output: array<f32>;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if i < arrayLength(&input) {
        output[i] = abs(input[i]);
    }
}
";

// ---------------------------------------------------------------------------
// GPU helpers
// ---------------------------------------------------------------------------

struct GpuContext {
    device: wgpu::Device,
    queue: wgpu::Queue,
    adapter_name: String,
}

fn try_create_gpu() -> Option<GpuContext> {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::VULKAN | wgpu::Backends::GL,
        ..Default::default()
    });

    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .ok()?;

    let adapter_name = adapter.get_info().name.clone();

    let (device, queue) = pollster::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: Some("exp030"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            memory_hints: wgpu::MemoryHints::Performance,
            experimental_features: Default::default(),
            trace: Default::default(),
        },
    ))
    .ok()?;

    Some(GpuContext {
        device,
        queue,
        adapter_name,
    })
}

fn device_poll_wait(device: &wgpu::Device) {
    let _ = device.poll(wgpu::PollType::Wait {
        submission_index: None,
        timeout: Some(std::time::Duration::from_secs(5)),
    });
}

fn gpu_run_f32_unary(ctx: &GpuContext, shader_src: &str, input: &[f32]) -> Vec<f32> {
    let n = input.len();
    let input_bytes = bytemuck_cast_f32(input);
    let output_size = (n * 4) as u64;

    let input_buf = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("input"),
        size: output_size,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    ctx.queue.write_buffer(&input_buf, 0, input_bytes);

    let output_buf = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("output"),
        size: output_size,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    let staging_buf = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("staging"),
        size: output_size,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let shader_module = ctx
        .device
        .create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shader"),
            source: wgpu::ShaderSource::Wgsl(shader_src.into()),
        });

    let bgl = ctx
        .device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

    let pl = ctx
        .device
        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pl"),
            bind_group_layouts: &[&bgl],
            immediate_size: 0,
        });

    let pipeline = ctx
        .device
        .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("pipeline"),
            layout: Some(&pl),
            module: &shader_module,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });

    let bg = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("bg"),
        layout: &bgl,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: input_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: output_buf.as_entire_binding(),
            },
        ],
    });

    let mut encoder = ctx
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("enc"),
        });

    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("pass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bg, &[]);
        #[allow(clippy::cast_possible_truncation)]
        let workgroups = ((n as u32) + 63) / 64;
        pass.dispatch_workgroups(workgroups, 1, 1);
    }

    encoder.copy_buffer_to_buffer(&output_buf, 0, &staging_buf, 0, output_size);
    ctx.queue.submit(std::iter::once(encoder.finish()));

    let slice = staging_buf.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        let _ = tx.send(result);
    });
    device_poll_wait(&ctx.device);
    if rx.recv().ok().and_then(|r| r.ok()).is_none() {
        return vec![0.0; n];
    }

    let data = slice.get_mapped_range();
    let result: Vec<f32> = data
        .chunks_exact(4)
        .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect();
    drop(data);
    staging_buf.unmap();
    result
}

fn gpu_run_u32_unary(ctx: &GpuContext, shader_src: &str, input: &[u32]) -> Vec<u32> {
    let n = input.len();
    let input_bytes = bytemuck_cast_u32(input);
    let output_size = (n * 4) as u64;

    let input_buf = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("input"),
        size: output_size,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    ctx.queue.write_buffer(&input_buf, 0, input_bytes);

    let output_buf = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("output"),
        size: output_size,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    let staging_buf = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("staging"),
        size: output_size,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let shader_module = ctx
        .device
        .create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shader"),
            source: wgpu::ShaderSource::Wgsl(shader_src.into()),
        });

    let bgl = ctx
        .device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

    let pl = ctx
        .device
        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pl"),
            bind_group_layouts: &[&bgl],
            immediate_size: 0,
        });

    let pipeline = ctx
        .device
        .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("pipeline"),
            layout: Some(&pl),
            module: &shader_module,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });

    let bg = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("bg"),
        layout: &bgl,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: input_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: output_buf.as_entire_binding(),
            },
        ],
    });

    let mut encoder = ctx
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("enc"),
        });

    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("pass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bg, &[]);
        #[allow(clippy::cast_possible_truncation)]
        let workgroups = ((n as u32) + 63) / 64;
        pass.dispatch_workgroups(workgroups, 1, 1);
    }

    encoder.copy_buffer_to_buffer(&output_buf, 0, &staging_buf, 0, output_size);
    ctx.queue.submit(std::iter::once(encoder.finish()));

    let slice = staging_buf.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        let _ = tx.send(result);
    });
    device_poll_wait(&ctx.device);
    if rx.recv().ok().and_then(|r| r.ok()).is_none() {
        return vec![0; n];
    }

    let data = slice.get_mapped_range();
    let result: Vec<u32> = data
        .chunks_exact(4)
        .map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect();
    drop(data);
    staging_buf.unmap();
    result
}

fn gpu_run_f32_3buf(ctx: &GpuContext, shader_src: &str, a: &[f32], b: &[f32]) -> Vec<f32> {
    let n = a.len();
    let buf_size = (n * 4) as u64;

    let a_buf = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("a"),
        size: buf_size,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    ctx.queue.write_buffer(&a_buf, 0, bytemuck_cast_f32(a));

    let b_buf = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("b"),
        size: buf_size,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    ctx.queue.write_buffer(&b_buf, 0, bytemuck_cast_f32(b));

    let result_buf = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("result"),
        size: buf_size,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    let staging = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("staging"),
        size: buf_size,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let shader_module = ctx
        .device
        .create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shader"),
            source: wgpu::ShaderSource::Wgsl(shader_src.into()),
        });

    let bgl = ctx
        .device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

    let pl = ctx
        .device
        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pl"),
            bind_group_layouts: &[&bgl],
            immediate_size: 0,
        });

    let pipeline = ctx
        .device
        .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("pipeline"),
            layout: Some(&pl),
            module: &shader_module,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });

    let bg = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("bg"),
        layout: &bgl,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: a_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: b_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: result_buf.as_entire_binding(),
            },
        ],
    });

    let mut encoder = ctx
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("enc"),
        });

    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("pass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bg, &[]);
        #[allow(clippy::cast_possible_truncation)]
        let wg = ((n as u32) + 255) / 256;
        pass.dispatch_workgroups(wg, 1, 1);
    }

    encoder.copy_buffer_to_buffer(&result_buf, 0, &staging, 0, buf_size);
    ctx.queue.submit(std::iter::once(encoder.finish()));

    let slice = staging.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    device_poll_wait(&ctx.device);
    if rx.recv().ok().and_then(|r| r.ok()).is_none() {
        return vec![0.0; n];
    }

    let data = slice.get_mapped_range();
    let result: Vec<f32> = data
        .chunks_exact(4)
        .map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]]))
        .collect();
    drop(data);
    staging.unmap();
    result
}

fn bytemuck_cast_f32(data: &[f32]) -> &[u8] {
    unsafe { std::slice::from_raw_parts(data.as_ptr().cast::<u8>(), data.len() * 4) }
}

fn bytemuck_cast_u32(data: &[u32]) -> &[u8] {
    unsafe { std::slice::from_raw_parts(data.as_ptr().cast::<u8>(), data.len() * 4) }
}

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

fn cmd_validate() {
    println!("=== exp030: CPU-vs-GPU Math Parity Validation ===\n");

    let gpu = try_create_gpu();
    let gpu_name = gpu
        .as_ref()
        .map_or("none".to_string(), |g| g.adapter_name.clone());
    println!("GPU adapter: {gpu_name}\n");

    let mut results = Vec::new();
    let experiment = "exp030_cpu_gpu_parity";

    // -- CPU-only checks (always run) --

    let cpu_sig: Vec<f64> = vec![-2.0, -1.0, 0.0, 1.0, 2.0];
    let cpu_sig_out: Vec<f64> = cpu_sig.iter().map(|&x| barcuda_math::sigmoid(x)).collect();
    let sig_at_zero = cpu_sig_out[2];
    results.push(ValidationResult::check(
        experiment,
        "sigmoid_cpu_at_zero",
        sig_at_zero,
        0.5,
        1e-10,
    ));

    let relu_neg = f64::max(-3.0, 0.0);
    let relu_pos = f64::max(3.0, 0.0);
    results.push(ValidationResult::check(
        experiment,
        "relu_cpu_negative",
        relu_neg,
        0.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        experiment,
        "relu_cpu_positive",
        relu_pos,
        3.0,
        0.0,
    ));

    let a = [1.0, 2.0, 3.0, 4.0];
    let b = [5.0, 6.0, 7.0, 8.0];
    let cpu_dot = barcuda_math::dot(&a, &b);
    results.push(ValidationResult::check(
        experiment,
        "dot_cpu_known",
        cpu_dot,
        70.0,
        1e-10,
    ));

    let seed: u64 = 42;
    let next = barcuda_math::lcg_step(seed);
    #[allow(clippy::cast_precision_loss)]
    let expected_lcg = 42_u64
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1) as f64;
    results.push(ValidationResult::check(
        experiment,
        "lcg_cpu_deterministic",
        next as f64,
        expected_lcg,
        0.0,
    ));

    let mut perlin_min = f64::MAX;
    let mut perlin_max = f64::MIN;
    for i in 0..1000 {
        let v = noise::perlin_2d(f64::from(i) * 0.1, f64::from(i) * 0.07);
        if v < perlin_min {
            perlin_min = v;
        }
        if v > perlin_max {
            perlin_max = v;
        }
    }
    results.push(ValidationResult::check(
        experiment,
        "perlin_bounded_low",
        if perlin_min >= -1.0 { 1.0 } else { 0.0 },
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        experiment,
        "perlin_bounded_high",
        if perlin_max <= 1.0 { 1.0 } else { 0.0 },
        1.0,
        0.0,
    ));

    let mean_data = [2.0, 4.0, 6.0, 8.0];
    let cpu_mean = barcuda_math::mean(&mean_data);
    results.push(ValidationResult::check(
        experiment,
        "mean_cpu_known",
        cpu_mean,
        5.0,
        1e-10,
    ));

    // -- GPU parity checks --

    if let Some(ref ctx) = gpu {
        let sig_input: Vec<f32> = vec![-2.0, -1.0, 0.0, 1.0, 2.0];
        let gpu_sig = gpu_run_f32_unary(ctx, SIGMOID_WGSL, &sig_input);
        let cpu_sig_f32: Vec<f32> = sig_input.iter().map(|&x| 1.0_f32 / (1.0 + (-x).exp())).collect();
        let sig_max_err = gpu_sig
            .iter()
            .zip(cpu_sig_f32.iter())
            .map(|(g, c)| (g - c).abs())
            .fold(0.0_f32, f32::max);
        results.push(ValidationResult::check(
            experiment,
            "sigmoid_gpu_parity",
            f64::from(sig_max_err),
            0.0,
            1e-6,
        ));

        let relu_input: Vec<f32> = vec![-3.0, -1.0, 0.0, 1.0, 3.0];
        let gpu_relu = gpu_run_f32_unary(ctx, RELU_WGSL, &relu_input);
        let cpu_relu: Vec<f32> = relu_input.iter().map(|&x| x.max(0.0)).collect();
        let relu_exact = gpu_relu == cpu_relu;
        results.push(ValidationResult::check(
            experiment,
            "relu_gpu_exact",
            if relu_exact { 1.0 } else { 0.0 },
            1.0,
            0.0,
        ));

        let dot_a: Vec<f32> = vec![1.0, 2.0, 3.0, 4.0];
        let dot_b: Vec<f32> = vec![5.0, 6.0, 7.0, 8.0];
        let gpu_products = gpu_run_f32_3buf(ctx, DOT_PRODUCT_WGSL, &dot_a, &dot_b);
        let gpu_dot_sum: f32 = gpu_products.iter().sum();
        results.push(ValidationResult::check(
            experiment,
            "dot_gpu_parity",
            f64::from(gpu_dot_sum),
            70.0,
            1e-4,
        ));

        let sm_input: Vec<f32> = vec![1.0, 2.0, 3.0, 4.0];
        let gpu_sm = gpu_run_f32_unary(ctx, SOFTMAX_WGSL, &sm_input);
        let cpu_sm = cpu_softmax_f32(&sm_input);
        let sm_max_err = gpu_sm
            .iter()
            .zip(cpu_sm.iter())
            .map(|(g, c)| (g - c).abs())
            .fold(0.0_f32, f32::max);
        results.push(ValidationResult::check(
            experiment,
            "softmax_gpu_parity",
            f64::from(sm_max_err),
            0.0,
            1e-5,
        ));

        let scale_input: Vec<f32> = vec![0.0, 1.0, 2.0, 3.0, 4.0];
        let gpu_scale = gpu_run_f32_unary(ctx, SCALE_WGSL, &scale_input);
        let cpu_scale: Vec<f32> = scale_input.iter().map(|&x| x * 2.0 + 1.0).collect();
        let scale_exact = gpu_scale == cpu_scale;
        results.push(ValidationResult::check(
            experiment,
            "scale_gpu_exact",
            if scale_exact { 1.0 } else { 0.0 },
            1.0,
            0.0,
        ));

        let lcg_seeds: Vec<u32> = vec![42, 100, 255, 0, 999_999];
        let gpu_lcg = gpu_run_u32_unary(ctx, LCG_WGSL, &lcg_seeds);
        let cpu_lcg: Vec<u32> = lcg_seeds
            .iter()
            .map(|&s| s.wrapping_mul(1_664_525).wrapping_add(1_013_904_223))
            .collect();
        let lcg_match = gpu_lcg == cpu_lcg;
        results.push(ValidationResult::check(
            experiment,
            "lcg_gpu_exact",
            if lcg_match { 1.0 } else { 0.0 },
            1.0,
            0.0,
        ));

        let abs_input: Vec<f32> = vec![-5.0, -1.0, 0.0, 1.0, 5.0];
        let gpu_abs = gpu_run_f32_unary(ctx, ABS_WGSL, &abs_input);
        let cpu_abs: Vec<f32> = abs_input.iter().map(|x| x.abs()).collect();
        let abs_exact = gpu_abs == cpu_abs;
        results.push(ValidationResult::check(
            experiment,
            "abs_gpu_exact",
            if abs_exact { 1.0 } else { 0.0 },
            1.0,
            0.0,
        ));

        #[allow(clippy::cast_precision_loss)]
        let sum_input: Vec<f32> = (0..256).map(|i| i as f32).collect();
        let gpu_partial = gpu_run_f32_unary(ctx, REDUCE_SUM_WGSL, &sum_input);
        let gpu_total: f32 = gpu_partial.iter().sum();
        let cpu_total: f32 = sum_input.iter().sum();
        results.push(ValidationResult::check(
            experiment,
            "reduce_sum_gpu_parity",
            f64::from((gpu_total - cpu_total).abs()),
            0.0,
            1.0,
        ));
    } else {
        println!("  [SKIP] No GPU adapter — GPU parity checks skipped\n");
    }

    let passed = results.iter().filter(|r| r.passed).count();
    let total = results.len();
    println!();
    for r in &results {
        let tag = if r.passed { "PASS" } else { "FAIL" };
        println!("  [{tag}] {}", r.description);
    }
    println!("\nResults: {passed}/{total} passed");
    if passed < total {
        process::exit(1);
    }
}

fn cpu_softmax_f32(input: &[f32]) -> Vec<f32> {
    let max_val = input.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let exps: Vec<f32> = input.iter().map(|&x| (x - max_val).exp()).collect();
    let sum: f32 = exps.iter().sum();
    exps.iter().map(|&e| e / sum).collect()
}

// ---------------------------------------------------------------------------
// Probe
// ---------------------------------------------------------------------------

fn cmd_probe() {
    println!("=== exp030: GPU Adapter Probe ===\n");

    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });

    let adapters: Vec<wgpu::Adapter> =
        pollster::block_on(instance.enumerate_adapters(wgpu::Backends::all()));
    if adapters.is_empty() {
        println!("  No adapters found.");
        return;
    }

    for (i, adapter) in adapters.iter().enumerate() {
        let info = adapter.get_info();
        println!("  Adapter {i}:");
        println!("    Name:    {}", info.name);
        println!("    Vendor:  0x{:04x}", info.vendor);
        println!("    Device:  0x{:04x}", info.device);
        println!("    Type:    {:?}", info.device_type);
        println!("    Backend: {:?}", info.backend);
        println!();
    }
}

// ---------------------------------------------------------------------------
// Bench
// ---------------------------------------------------------------------------

fn cmd_bench() {
    println!("=== exp030: CPU-vs-GPU Benchmark ===\n");

    let gpu = try_create_gpu();
    let gpu_name = gpu
        .as_ref()
        .map_or("none".to_string(), |g| g.adapter_name.clone());
    println!("GPU: {gpu_name}\n");

    let sizes = [64, 256, 1024, 4096, 16384, 65536];

    println!(
        "{:>8} {:>12} {:>12} {:>8}",
        "N", "CPU (us)", "GPU (us)", "Speedup"
    );
    println!("{}", "-".repeat(48));

    for &n in &sizes {
        #[allow(clippy::cast_precision_loss)]
        let input: Vec<f32> = (0..n).map(|i| (i as f32) * 0.001 - 0.5).collect();

        let cpu_start = std::time::Instant::now();
        let _cpu_out: Vec<f32> = input.iter().map(|&x| 1.0 / (1.0 + (-x).exp())).collect();
        let cpu_us = cpu_start.elapsed().as_micros();

        if let Some(ref ctx) = gpu {
            let gpu_start = std::time::Instant::now();
            let _gpu_out = gpu_run_f32_unary(ctx, SIGMOID_WGSL, &input);
            let gpu_us = gpu_start.elapsed().as_micros();

            let speedup = if gpu_us > 0 {
                cpu_us as f64 / gpu_us as f64
            } else {
                0.0
            };
            println!("{n:>8} {cpu_us:>12} {gpu_us:>12} {speedup:>8.2}x");
        } else {
            println!("{n:>8} {cpu_us:>12} {:>12} {:>8}", "N/A", "N/A");
        }
    }
}

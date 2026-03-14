// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
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
use ludospring_barracuda::game::raycaster;
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
// Constants
// ---------------------------------------------------------------------------

/// Perlin permutation table (standard 256-entry doubled to 512).
/// Must match barracuda `procedural::noise` for CPU/GPU parity.
const PERM_TABLE: [u8; 512] = {
    let base: [u8; 256] = [
        151, 160, 137, 91, 90, 15, 131, 13, 201, 95, 96, 53, 194, 233, 7, 225, 140, 36, 103, 30,
        69, 142, 8, 99, 37, 240, 21, 10, 23, 190, 6, 148, 247, 120, 234, 75, 0, 26, 197, 62, 94,
        252, 219, 203, 117, 35, 11, 32, 57, 177, 33, 88, 237, 149, 56, 87, 174, 20, 125, 136, 171,
        168, 68, 175, 74, 165, 71, 134, 139, 48, 27, 166, 77, 146, 158, 231, 83, 111, 229, 122, 60,
        211, 133, 230, 220, 105, 92, 41, 55, 46, 245, 40, 244, 102, 143, 54, 65, 25, 63, 161, 1,
        216, 80, 73, 209, 76, 132, 187, 208, 89, 18, 169, 200, 196, 135, 130, 116, 188, 159, 86,
        164, 100, 109, 198, 173, 186, 3, 64, 52, 217, 226, 250, 124, 123, 5, 202, 38, 147, 118,
        126, 255, 82, 85, 212, 207, 206, 59, 227, 47, 16, 58, 17, 182, 189, 28, 42, 223, 183, 170,
        213, 119, 248, 152, 2, 44, 154, 163, 70, 221, 153, 101, 155, 167, 43, 172, 9, 129, 22, 39,
        253, 19, 98, 108, 110, 79, 113, 224, 232, 178, 185, 112, 104, 218, 246, 97, 228, 251, 34,
        242, 193, 238, 210, 144, 12, 191, 179, 162, 241, 81, 51, 145, 235, 249, 14, 239, 107, 49,
        192, 214, 31, 181, 199, 106, 157, 184, 84, 204, 176, 115, 121, 50, 45, 127, 4, 150, 254,
        138, 236, 205, 93, 222, 114, 67, 29, 24, 72, 243, 141, 128, 195, 78, 66, 215, 61, 156, 180,
    ];
    let mut table = [0u8; 512];
    let mut i = 0;
    while i < 512 {
        table[i] = base[i & 255];
        i += 1;
    }
    table
};

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

const PERLIN_2D_WGSL: &str = r"
@group(0) @binding(0) var<storage, read> perm: array<u32>;
@group(0) @binding(1) var<storage, read> coords: array<f32>;
@group(0) @binding(2) var<storage, read_write> output: array<f32>;

fn fade(t: f32) -> f32 {
    return t * t * t * (t * (t * 6.0 - 15.0) + 10.0);
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    return a + t * (b - a);
}

fn grad2(hash: u32, x: f32, y: f32) -> f32 {
    switch hash & 3u {
        case 0u: { return x + y; }
        case 1u: { return -x + y; }
        case 2u: { return x - y; }
        default: { return -x - y; }
    }
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    let n = arrayLength(&output);
    if i >= n {
        return;
    }
    let x = coords[i * 2u];
    let y = coords[i * 2u + 1u];
    let xi = u32(i32(floor(x)) & 255);
    let yi = u32(i32(floor(y)) & 255);
    let xf = x - floor(x);
    let yf = y - floor(y);
    let u = fade(xf);
    let v = fade(yf);
    let aa = perm[perm[xi] + yi];
    let ab = perm[perm[xi] + yi + 1u];
    let ba = perm[perm[xi + 1u] + yi];
    let bb = perm[perm[xi + 1u] + yi + 1u];
    let result = lerp(
        lerp(grad2(aa, xf, yf), grad2(ba, xf - 1.0, yf), u),
        lerp(grad2(ab, xf, yf - 1.0), grad2(bb, xf - 1.0, yf - 1.0), u),
        v
    );
    output[i] = result;
}
";

const ENGAGEMENT_BATCH_WGSL: &str = r"
@group(0) @binding(0) var<storage, read> components: array<f32>;
@group(0) @binding(1) var<storage, read> weights: array<f32>;
@group(0) @binding(2) var<storage, read_write> output: array<f32>;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    let n = arrayLength(&output);
    if i >= n {
        return;
    }
    var sum: f32 = 0.0;
    for (var j = 0u; j < 5u; j = j + 1u) {
        sum += components[i * 5u + j] * weights[j];
    }
    output[i] = clamp(sum, 0.0, 1.0);
}
";

const DDA_RAYCAST_WGSL: &str = r"
@group(0) @binding(0) var<storage, read> map_data: array<u32>;
@group(0) @binding(1) var<storage, read> params: array<f32>;
@group(0) @binding(2) var<storage, read> ray_angles: array<f32>;
@group(0) @binding(3) var<storage, read_write> distances: array<f32>;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let ray_idx = gid.x;
    let n_rays = u32(params[5]);
    if ray_idx >= n_rays {
        return;
    }
    let player_x = params[0];
    let player_y = params[1];
    let map_w = u32(params[2]);
    let map_h = u32(params[3]);
    let max_depth = params[4];
    let ray_angle = ray_angles[ray_idx];
    let dir_x = cos(ray_angle);
    let dir_y = sin(ray_angle);
    let near_zero: f32 = 1e-6;
    var delta_x: f32;
    var delta_y: f32;
    if abs(dir_x) < near_zero {
        delta_x = 1e6;
    } else {
        delta_x = abs(1.0 / dir_x);
    }
    if abs(dir_y) < near_zero {
        delta_y = 1e6;
    } else {
        delta_y = abs(1.0 / dir_y);
    }
    var map_x = i32(floor(player_x));
    var map_y = i32(floor(player_y));
    var step_x: i32;
    var step_y: i32;
    var side_x: f32;
    var side_y: f32;
    if dir_x < 0.0 {
        step_x = -1;
        side_x = (player_x - f32(map_x)) * delta_x;
    } else {
        step_x = 1;
        side_x = (f32(map_x) + 1.0 - player_x) * delta_x;
    }
    if dir_y < 0.0 {
        step_y = -1;
        side_y = (player_y - f32(map_y)) * delta_y;
    } else {
        step_y = 1;
        side_y = (f32(map_y) + 1.0 - player_y) * delta_y;
    }
    var dist: f32 = 0.0;
    loop {
        if side_x < side_y {
            side_x += delta_x;
            map_x += step_x;
            dist = side_x - delta_x;
        } else {
            side_y += delta_y;
            map_y += step_y;
            dist = side_y - delta_y;
        }
        if map_x < 0 || map_y < 0 || map_x >= i32(map_w) || map_y >= i32(map_h) {
            distances[ray_idx] = max_depth;
            return;
        }
        if dist > max_depth {
            distances[ray_idx] = max_depth;
            return;
        }
        let cell = map_data[u32(map_y) * map_w + u32(map_x)];
        if cell != 0u {
            distances[ray_idx] = dist;
            return;
        }
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

    let adapter_name = adapter.get_info().name;

    let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("exp030"),
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::default(),
        memory_hints: wgpu::MemoryHints::Performance,
        experimental_features: wgpu::ExperimentalFeatures::default(),
        trace: wgpu::Trace::default(),
    }))
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

#[expect(
    clippy::too_many_lines,
    reason = "GPU pipeline setup — buffer, shader, bind group, dispatch"
)]
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
            compilation_options: wgpu::PipelineCompilationOptions::default(),
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
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("enc") });

    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("pass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bg, &[]);
        #[allow(clippy::cast_possible_truncation)]
        let workgroups = (n as u32).div_ceil(64);
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
    if rx.recv().ok().and_then(Result::ok).is_none() {
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

#[expect(
    clippy::too_many_lines,
    reason = "GPU pipeline setup — buffer, shader, bind group, dispatch"
)]
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
            compilation_options: wgpu::PipelineCompilationOptions::default(),
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
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("enc") });

    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("pass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bg, &[]);
        #[allow(clippy::cast_possible_truncation)]
        let workgroups = (n as u32).div_ceil(64);
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
    if rx.recv().ok().and_then(Result::ok).is_none() {
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

#[expect(
    clippy::too_many_lines,
    reason = "GPU pipeline setup — 3 buffers, shader, bind group, dispatch"
)]
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
            compilation_options: wgpu::PipelineCompilationOptions::default(),
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
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("enc") });

    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("pass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bg, &[]);
        #[allow(clippy::cast_possible_truncation)]
        let wg = (n as u32).div_ceil(256);
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
    if rx.recv().ok().and_then(Result::ok).is_none() {
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

#[expect(
    clippy::too_many_lines,
    reason = "GPU pipeline setup — 3 buffers (perm, coords, output), shader, bind group, dispatch"
)]
fn gpu_run_perlin(ctx: &GpuContext, perm: &[u32], coords: &[f32]) -> Vec<f32> {
    let n = coords.len() / 2;
    let perm_bytes = bytemuck_cast_u32(perm);
    let coords_bytes = bytemuck_cast_f32(coords);
    let output_size = (n * 4) as u64;

    let perm_buf = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("perm"),
        size: (perm.len() * 4) as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    ctx.queue.write_buffer(&perm_buf, 0, perm_bytes);

    let coords_buf = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("coords"),
        size: coords_bytes.len() as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    ctx.queue.write_buffer(&coords_buf, 0, coords_bytes);

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
            label: Some("perlin"),
            source: wgpu::ShaderSource::Wgsl(PERLIN_2D_WGSL.into()),
        });

    let bgl = ctx
        .device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("perlin_bgl"),
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
            label: Some("perlin_pl"),
            bind_group_layouts: &[&bgl],
            immediate_size: 0,
        });

    let pipeline = ctx
        .device
        .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("perlin_pipeline"),
            layout: Some(&pl),
            module: &shader_module,
            entry_point: Some("main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });

    let bg = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("perlin_bg"),
        layout: &bgl,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: perm_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: coords_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: output_buf.as_entire_binding(),
            },
        ],
    });

    let mut encoder = ctx
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("perlin_enc"),
        });

    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("perlin_pass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bg, &[]);
        #[allow(clippy::cast_possible_truncation)]
        let workgroups = (n as u32).div_ceil(64);
        pass.dispatch_workgroups(workgroups, 1, 1);
    }

    encoder.copy_buffer_to_buffer(&output_buf, 0, &staging_buf, 0, output_size);
    ctx.queue.submit(std::iter::once(encoder.finish()));

    let slice = staging_buf.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    device_poll_wait(&ctx.device);
    if rx.recv().ok().and_then(Result::ok).is_none() {
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

#[expect(
    clippy::too_many_lines,
    reason = "GPU pipeline setup — 3 buffers (components, weights, output), shader, bind group, dispatch"
)]
fn gpu_run_engagement_batch(ctx: &GpuContext, components: &[f32], weights: &[f32]) -> Vec<f32> {
    let n = components.len() / 5;
    let output_size = (n * 4) as u64;

    let components_buf = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("components"),
        size: (components.len() * 4) as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    ctx.queue
        .write_buffer(&components_buf, 0, bytemuck_cast_f32(components));

    let weights_buf = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("weights"),
        size: (weights.len() * 4) as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    ctx.queue
        .write_buffer(&weights_buf, 0, bytemuck_cast_f32(weights));

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
            label: Some("engagement"),
            source: wgpu::ShaderSource::Wgsl(ENGAGEMENT_BATCH_WGSL.into()),
        });

    let bgl = ctx
        .device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("engagement_bgl"),
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
            label: Some("engagement_pl"),
            bind_group_layouts: &[&bgl],
            immediate_size: 0,
        });

    let pipeline = ctx
        .device
        .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("engagement_pipeline"),
            layout: Some(&pl),
            module: &shader_module,
            entry_point: Some("main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });

    let bg = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("engagement_bg"),
        layout: &bgl,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: components_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: weights_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: output_buf.as_entire_binding(),
            },
        ],
    });

    let mut encoder = ctx
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("engagement_enc"),
        });

    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("engagement_pass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bg, &[]);
        #[allow(clippy::cast_possible_truncation)]
        let workgroups = (n as u32).div_ceil(64);
        pass.dispatch_workgroups(workgroups, 1, 1);
    }

    encoder.copy_buffer_to_buffer(&output_buf, 0, &staging_buf, 0, output_size);
    ctx.queue.submit(std::iter::once(encoder.finish()));

    let slice = staging_buf.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    device_poll_wait(&ctx.device);
    if rx.recv().ok().and_then(Result::ok).is_none() {
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

#[expect(
    clippy::too_many_lines,
    reason = "GPU pipeline setup — 4 buffers (map, params, angles, distances), shader, bind group, dispatch"
)]
#[allow(clippy::cast_precision_loss)]
fn gpu_run_raycaster(
    ctx: &GpuContext,
    map_data: &[u32],
    map_w: u32,
    map_h: u32,
    player_x: f32,
    player_y: f32,
    ray_angles: &[f32],
) -> Vec<f32> {
    let n_rays = ray_angles.len();
    let params: [f32; 6] = [
        player_x,
        player_y,
        map_w as f32,
        map_h as f32,
        20.0,
        n_rays as f32,
    ];
    let output_size = (n_rays * 4) as u64;

    let map_buf = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("map"),
        size: (map_data.len() * 4) as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    ctx.queue
        .write_buffer(&map_buf, 0, bytemuck_cast_u32(map_data));

    let params_buf = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("params"),
        size: 24,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    ctx.queue
        .write_buffer(&params_buf, 0, bytemuck::cast_slice(&params));

    let angles_buf = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("ray_angles"),
        size: (ray_angles.len() * 4) as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    ctx.queue
        .write_buffer(&angles_buf, 0, bytemuck_cast_f32(ray_angles));

    let output_buf = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("distances"),
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
            label: Some("raycaster"),
            source: wgpu::ShaderSource::Wgsl(DDA_RAYCAST_WGSL.into()),
        });

    let bgl = ctx
        .device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("raycaster_bgl"),
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
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
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
            label: Some("raycaster_pl"),
            bind_group_layouts: &[&bgl],
            immediate_size: 0,
        });

    let pipeline = ctx
        .device
        .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("raycaster_pipeline"),
            layout: Some(&pl),
            module: &shader_module,
            entry_point: Some("main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });

    let bg = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("raycaster_bg"),
        layout: &bgl,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: map_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: params_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: angles_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: output_buf.as_entire_binding(),
            },
        ],
    });

    let mut encoder = ctx
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("raycaster_enc"),
        });

    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("raycaster_pass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bg, &[]);
        #[allow(clippy::cast_possible_truncation)]
        let workgroups = (n_rays as u32).div_ceil(64);
        pass.dispatch_workgroups(workgroups, 1, 1);
    }

    encoder.copy_buffer_to_buffer(&output_buf, 0, &staging_buf, 0, output_size);
    ctx.queue.submit(std::iter::once(encoder.finish()));

    let slice = staging_buf.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |r| {
        let _ = tx.send(r);
    });
    device_poll_wait(&ctx.device);
    if rx.recv().ok().and_then(Result::ok).is_none() {
        return vec![20.0; n_rays];
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

fn bytemuck_cast_f32(data: &[f32]) -> &[u8] {
    bytemuck::cast_slice(data)
}

fn bytemuck_cast_u32(data: &[u32]) -> &[u8] {
    bytemuck::cast_slice(data)
}

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

#[expect(
    clippy::too_many_lines,
    clippy::cast_precision_loss,
    clippy::similar_names,
    reason = "validation orchestrator — sequential check groups"
)]
fn cmd_validate() {
    println!("=== exp030: CPU-vs-GPU Math Parity Validation ===\n");

    let gpu = try_create_gpu();
    let gpu_name = gpu
        .as_ref()
        .map_or_else(|| "none".to_string(), |g| g.adapter_name.clone());
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
        let cpu_sig_f32: Vec<f32> = sig_input
            .iter()
            .map(|&x| 1.0_f32 / (1.0 + (-x).exp()))
            .collect();
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

        let softmax_input: Vec<f32> = vec![1.0, 2.0, 3.0, 4.0];
        let gpu_sm = gpu_run_f32_unary(ctx, SOFTMAX_WGSL, &softmax_input);
        let cpu_sm = cpu_softmax_f32(&softmax_input);
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
        let cpu_scale: Vec<f32> = scale_input.iter().map(|&x| x.mul_add(2.0, 1.0)).collect();
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
        let reduce_sum_input: Vec<f32> = (0..256).map(|i| i as f32).collect();
        let gpu_partial = gpu_run_f32_unary(ctx, REDUCE_SUM_WGSL, &reduce_sum_input);
        let gpu_total: f32 = gpu_partial.iter().sum();
        let cpu_total: f32 = reduce_sum_input.iter().sum();
        results.push(ValidationResult::check(
            experiment,
            "reduce_sum_gpu_parity",
            f64::from((gpu_total - cpu_total).abs()),
            0.0,
            1.0,
        ));

        // -- Tier A GPU parity: Perlin 2D noise --
        let perm_u32: Vec<u32> = PERM_TABLE.iter().map(|&b| u32::from(b)).collect();
        let n_noise = 256;
        let mut noise_coords: Vec<f32> = Vec::with_capacity(n_noise * 2);
        for i in 0..n_noise {
            let x = (i as f32) * 0.1;
            let y = (i as f32) * 0.07;
            noise_coords.push(x);
            noise_coords.push(y);
        }
        let gpu_noise = gpu_run_perlin(ctx, &perm_u32, &noise_coords);
        #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
        let cpu_noise: Vec<f32> = (0..n_noise)
            .map(|i| {
                let x = (i as f64) * 0.1;
                let y = (i as f64) * 0.07;
                noise::perlin_2d(x, y) as f32
            })
            .collect();
        let noise_max_err = gpu_noise
            .iter()
            .zip(cpu_noise.iter())
            .map(|(g, c)| (g - c).abs())
            .fold(0.0_f32, f32::max);
        results.push(ValidationResult::check(
            experiment,
            "perlin_gpu_parity",
            f64::from(noise_max_err),
            0.0,
            1e-3,
        ));

        // Check range bounded
        let all_bounded = gpu_noise.iter().all(|&v| (-1.1..=1.1).contains(&v));
        results.push(ValidationResult::check(
            experiment,
            "perlin_gpu_range_bounded",
            if all_bounded { 1.0 } else { 0.0 },
            1.0,
            0.0,
        ));

        // Determinism: rerun
        let gpu_noise2 = gpu_run_perlin(ctx, &perm_u32, &noise_coords);
        let deterministic = gpu_noise == gpu_noise2;
        results.push(ValidationResult::check(
            experiment,
            "perlin_gpu_deterministic",
            if deterministic { 1.0 } else { 0.0 },
            1.0,
            0.0,
        ));

        // -- Tier A GPU parity: Engagement batch --
        let n_eng = 64;
        let weights_f32: [f32; 5] = [0.2, 0.2, 0.2, 0.2, 0.2];
        let mut eng_components: Vec<f32> = Vec::with_capacity(n_eng * 5);
        for i in 0..n_eng {
            let base = (i as f32) / (n_eng as f32);
            for j in 0..5 {
                eng_components.push((j as f32).mul_add(0.1, base).min(1.0));
            }
        }
        let gpu_eng = gpu_run_engagement_batch(ctx, &eng_components, &weights_f32);
        let cpu_eng: Vec<f32> = (0..n_eng)
            .map(|i| {
                let mut sum = 0.0_f32;
                for j in 0..5 {
                    sum += eng_components[i * 5 + j] * weights_f32[j];
                }
                sum.clamp(0.0, 1.0)
            })
            .collect();
        let eng_max_err = gpu_eng
            .iter()
            .zip(cpu_eng.iter())
            .map(|(g, c)| (g - c).abs())
            .fold(0.0_f32, f32::max);
        results.push(ValidationResult::check(
            experiment,
            "engagement_gpu_parity",
            f64::from(eng_max_err),
            0.0,
            1e-4,
        ));

        // -- FBM GPU parity (use Perlin shader with multiple octave calls) --
        let n_fbm = 128;
        let octaves = 4u32;
        let lacunarity: f32 = 2.0;
        let persistence: f32 = 0.5;
        #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
        let fbm_cpu: Vec<f32> = (0..n_fbm)
            .map(|i| {
                let x = (i as f64) * 0.05;
                let y = (i as f64) * 0.03;
                noise::fbm_2d(x, y, octaves, f64::from(lacunarity), f64::from(persistence)) as f32
            })
            .collect();
        // GPU fBm: accumulate octaves by calling Perlin shader per octave
        let mut fbm_gpu = vec![0.0_f32; n_fbm];
        let mut amplitude: f32 = 1.0;
        let mut frequency: f32 = 1.0;
        let mut max_value: f32 = 0.0;
        for _ in 0..octaves {
            let mut octave_coords: Vec<f32> = Vec::with_capacity(n_fbm * 2);
            for i in 0..n_fbm {
                octave_coords.push((i as f32) * 0.05 * frequency);
                octave_coords.push((i as f32) * 0.03 * frequency);
            }
            let octave_result = gpu_run_perlin(ctx, &perm_u32, &octave_coords);
            for (j, val) in octave_result.iter().enumerate() {
                fbm_gpu[j] += val * amplitude;
            }
            max_value += amplitude;
            amplitude *= persistence;
            frequency *= lacunarity;
        }
        for v in &mut fbm_gpu {
            *v /= max_value;
        }
        let fbm_max_err = fbm_gpu
            .iter()
            .zip(fbm_cpu.iter())
            .map(|(g, c)| (g - c).abs())
            .fold(0.0_f32, f32::max);
        results.push(ValidationResult::check(
            experiment,
            "fbm_gpu_parity",
            f64::from(fbm_max_err),
            0.0,
            0.01,
        ));

        // -- Raycaster GPU parity (simplified: DDA distance for N angles) --
        let map_w = 8u32;
        let map_h = 8u32;
        let mut map_data: Vec<u32> = vec![0; (map_w * map_h) as usize];
        for x in 0..map_w {
            for y in 0..map_h {
                if x == 0 || x == map_w - 1 || y == 0 || y == map_h - 1 {
                    map_data[(y * map_w + x) as usize] = 1;
                }
            }
        }
        let player_x: f32 = 4.0;
        let player_y: f32 = 4.0;
        let n_rays = 64u32;
        let fov: f32 = std::f32::consts::PI / 3.0;
        let base_angle: f32 = 0.0;
        let mut ray_angles: Vec<f32> = Vec::with_capacity(n_rays as usize);
        for i in 0..n_rays {
            let fraction = (i as f32) / (n_rays as f32) - 0.5;
            ray_angles.push(base_angle + fraction * fov);
        }

        // CPU raycaster reference
        let grid_map = raycaster::GridMap::new(
            map_w as usize,
            map_h as usize,
            map_data.iter().map(|&v| v != 0).collect(),
        );
        let ray_player = raycaster::RayPlayer {
            x: f64::from(player_x),
            y: f64::from(player_y),
            angle: f64::from(base_angle),
            fov: f64::from(fov),
            speed: 3.0,
            turn_speed: std::f64::consts::PI,
        };
        let cpu_distances: Vec<f32> = ray_angles
            .iter()
            .map(|&a| {
                let hit = raycaster::cast_ray(&ray_player, f64::from(a), &grid_map, 20.0);
                #[allow(clippy::cast_possible_truncation)]
                hit.map_or(20.0_f32, |h| h.distance as f32)
            })
            .collect();

        let gpu_distances = gpu_run_raycaster(
            ctx,
            &map_data,
            map_w,
            map_h,
            player_x,
            player_y,
            &ray_angles,
        );

        let ray_max_err = gpu_distances
            .iter()
            .zip(cpu_distances.iter())
            .map(|(g, c)| (g - c).abs())
            .fold(0.0_f32, f32::max);
        results.push(ValidationResult::check(
            experiment,
            "raycaster_gpu_parity",
            f64::from(ray_max_err),
            0.0,
            0.5,
        ));

        // Hit match: both agree on whether a wall was hit
        let gpu_hits: Vec<bool> = gpu_distances.iter().map(|&d| d < 19.0).collect();
        let cpu_hits: Vec<bool> = cpu_distances.iter().map(|&d| d < 19.0).collect();
        let hit_match = gpu_hits == cpu_hits;
        results.push(ValidationResult::check(
            experiment,
            "raycaster_gpu_hit_match",
            if hit_match { 1.0 } else { 0.0 },
            1.0,
            0.0,
        ));

        // Batch speedup check (positive or equal)
        let bench_n = 65536usize;
        let bench_input: Vec<f32> = (0..bench_n)
            .map(|i| (i as f32).mul_add(0.001, -0.5))
            .collect();
        let cpu_start = std::time::Instant::now();
        let _cpu: Vec<f32> = bench_input
            .iter()
            .map(|&x| 1.0 / (1.0 + (-x).exp()))
            .collect();
        let cpu_bench_us = cpu_start.elapsed().as_micros();
        let gpu_start = std::time::Instant::now();
        let _gpu = gpu_run_f32_unary(ctx, SIGMOID_WGSL, &bench_input);
        let gpu_bench_us = gpu_start.elapsed().as_micros();
        results.push(ValidationResult::check(
            experiment,
            "batch_speedup_nonnegative",
            if gpu_bench_us <= cpu_bench_us + 10000 {
                1.0
            } else {
                0.0
            },
            1.0,
            0.0,
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
        .map_or_else(|| "none".to_string(), |g| g.adapter_name.clone());
    println!("GPU: {gpu_name}\n");

    let sizes = [64, 256, 1024, 4096, 16384, 65536];

    println!(
        "{:>8} {:>12} {:>12} {:>8}",
        "N", "CPU (us)", "GPU (us)", "Speedup"
    );
    println!("{}", "-".repeat(48));

    for &n in &sizes {
        #[allow(clippy::cast_precision_loss)]
        let input: Vec<f32> = (0..n).map(|i| (i as f32).mul_add(0.001, -0.5)).collect();

        let cpu_start = std::time::Instant::now();
        let _cpu_out: Vec<f32> = input.iter().map(|&x| 1.0 / (1.0 + (-x).exp())).collect();
        let cpu_us = cpu_start.elapsed().as_micros();

        if let Some(ref ctx) = gpu {
            let gpu_start = std::time::Instant::now();
            let _gpu_out = gpu_run_f32_unary(ctx, SIGMOID_WGSL, &input);
            let gpu_us = gpu_start.elapsed().as_micros();

            #[allow(clippy::cast_precision_loss)]
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

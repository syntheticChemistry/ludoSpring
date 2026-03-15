// SPDX-License-Identifier: AGPL-3.0-or-later
//! GPU context and compute shader runners for CPU/GPU parity validation.
//!
//! Shared helpers (`storage_entry`, `build_pipeline`, `dispatch_and_read`)
//! consolidate the repetitive wgpu pipeline boilerplate. Each public
//! `gpu_run_*` function remains a focused orchestrator.

use crate::shaders::{DDA_RAYCAST_WGSL, ENGAGEMENT_BATCH_WGSL, PERLIN_2D_WGSL};

pub struct GpuContext {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub adapter_name: String,
}

pub fn try_create_gpu() -> Option<GpuContext> {
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

// ── Shared helpers ──────────────────────────────────────────────────

const fn storage_entry(binding: u32, read_only: bool) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

fn create_storage_buf(ctx: &GpuContext, label: &str, size: u64, writable: bool) -> wgpu::Buffer {
    let usage = if writable {
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC
    } else {
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST
    };
    ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size,
        usage,
        mapped_at_creation: false,
    })
}

fn create_staging_buf(ctx: &GpuContext, size: u64) -> wgpu::Buffer {
    ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("staging"),
        size,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

fn build_pipeline(
    ctx: &GpuContext,
    label: &str,
    shader_src: &str,
    layout_entries: &[wgpu::BindGroupLayoutEntry],
) -> (wgpu::ComputePipeline, wgpu::BindGroupLayout) {
    let shader_module = ctx
        .device
        .create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(label),
            source: wgpu::ShaderSource::Wgsl(shader_src.into()),
        });
    let bgl = ctx
        .device
        .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some(label),
            entries: layout_entries,
        });
    let pl = ctx
        .device
        .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(label),
            bind_group_layouts: &[&bgl],
            immediate_size: 0,
        });
    let pipeline = ctx
        .device
        .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some(label),
            layout: Some(&pl),
            module: &shader_module,
            entry_point: Some("main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });
    (pipeline, bgl)
}

fn dispatch_and_read_f32(
    ctx: &GpuContext,
    pipeline: &wgpu::ComputePipeline,
    bg: &wgpu::BindGroup,
    output_buf: &wgpu::Buffer,
    n: usize,
    workgroup_size: u32,
    default_val: f32,
) -> Vec<f32> {
    let output_size = (n * 4) as u64;
    let staging_buf = create_staging_buf(ctx, output_size);

    let mut encoder = ctx
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("enc") });
    {
        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("pass"),
            timestamp_writes: None,
        });
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, bg, &[]);
        #[allow(clippy::cast_possible_truncation)]
        let workgroups = (n as u32).div_ceil(workgroup_size);
        pass.dispatch_workgroups(workgroups, 1, 1);
    }
    encoder.copy_buffer_to_buffer(output_buf, 0, &staging_buf, 0, output_size);
    ctx.queue.submit(std::iter::once(encoder.finish()));

    read_staging_f32(ctx, &staging_buf, n, default_val)
}

fn read_staging_f32(
    ctx: &GpuContext,
    staging_buf: &wgpu::Buffer,
    n: usize,
    default_val: f32,
) -> Vec<f32> {
    let slice = staging_buf.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        let _ = tx.send(result);
    });
    device_poll_wait(&ctx.device);
    if rx.recv().ok().and_then(Result::ok).is_none() {
        return vec![default_val; n];
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

fn read_staging_u32(ctx: &GpuContext, staging_buf: &wgpu::Buffer, n: usize) -> Vec<u32> {
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

// ── Public GPU runners ──────────────────────────────────────────────

pub fn gpu_run_f32_unary(ctx: &GpuContext, shader_src: &str, input: &[f32]) -> Vec<f32> {
    let n = input.len();
    let size = (n * 4) as u64;
    let entries = [storage_entry(0, true), storage_entry(1, false)];
    let (pipeline, bgl) = build_pipeline(ctx, "unary_f32", shader_src, &entries);

    let input_buf = create_storage_buf(ctx, "input", size, false);
    ctx.queue
        .write_buffer(&input_buf, 0, bytemuck::cast_slice(input));
    let output_buf = create_storage_buf(ctx, "output", size, true);

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
    dispatch_and_read_f32(ctx, &pipeline, &bg, &output_buf, n, 64, 0.0)
}

pub fn gpu_run_u32_unary(ctx: &GpuContext, shader_src: &str, input: &[u32]) -> Vec<u32> {
    let n = input.len();
    let size = (n * 4) as u64;
    let entries = [storage_entry(0, true), storage_entry(1, false)];
    let (pipeline, bgl) = build_pipeline(ctx, "unary_u32", shader_src, &entries);

    let input_buf = create_storage_buf(ctx, "input", size, false);
    ctx.queue
        .write_buffer(&input_buf, 0, bytemuck::cast_slice(input));
    let output_buf = create_storage_buf(ctx, "output", size, true);

    let staging_buf = create_staging_buf(ctx, size);

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
    encoder.copy_buffer_to_buffer(&output_buf, 0, &staging_buf, 0, size);
    ctx.queue.submit(std::iter::once(encoder.finish()));

    read_staging_u32(ctx, &staging_buf, n)
}

pub fn gpu_run_f32_3buf(ctx: &GpuContext, shader_src: &str, a: &[f32], b: &[f32]) -> Vec<f32> {
    let n = a.len();
    let size = (n * 4) as u64;
    let entries = [
        storage_entry(0, true),
        storage_entry(1, true),
        storage_entry(2, false),
    ];
    let (pipeline, bgl) = build_pipeline(ctx, "f32_3buf", shader_src, &entries);

    let a_buf = create_storage_buf(ctx, "a", size, false);
    ctx.queue.write_buffer(&a_buf, 0, bytemuck::cast_slice(a));
    let b_buf = create_storage_buf(ctx, "b", size, false);
    ctx.queue.write_buffer(&b_buf, 0, bytemuck::cast_slice(b));
    let output_buf = create_storage_buf(ctx, "result", size, true);

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
                resource: output_buf.as_entire_binding(),
            },
        ],
    });
    dispatch_and_read_f32(ctx, &pipeline, &bg, &output_buf, n, 256, 0.0)
}

pub fn gpu_run_perlin(ctx: &GpuContext, perm: &[u32], coords: &[f32]) -> Vec<f32> {
    let n = coords.len() / 2;
    let output_size = (n * 4) as u64;
    let entries = [
        storage_entry(0, true),
        storage_entry(1, true),
        storage_entry(2, false),
    ];
    let (pipeline, bgl) = build_pipeline(ctx, "perlin", PERLIN_2D_WGSL, &entries);

    let perm_buf = create_storage_buf(ctx, "perm", (perm.len() * 4) as u64, false);
    ctx.queue
        .write_buffer(&perm_buf, 0, bytemuck::cast_slice(perm));
    let coords_buf = create_storage_buf(ctx, "coords", (coords.len() * 4) as u64, false);
    ctx.queue
        .write_buffer(&coords_buf, 0, bytemuck::cast_slice(coords));
    let output_buf = create_storage_buf(ctx, "output", output_size, true);

    let bg = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("bg"),
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
    dispatch_and_read_f32(ctx, &pipeline, &bg, &output_buf, n, 64, 0.0)
}

pub fn gpu_run_engagement_batch(ctx: &GpuContext, components: &[f32], weights: &[f32]) -> Vec<f32> {
    let n = components.len() / 5;
    let output_size = (n * 4) as u64;
    let entries = [
        storage_entry(0, true),
        storage_entry(1, true),
        storage_entry(2, false),
    ];
    let (pipeline, bgl) = build_pipeline(ctx, "engagement", ENGAGEMENT_BATCH_WGSL, &entries);

    let comp_buf = create_storage_buf(ctx, "components", (components.len() * 4) as u64, false);
    ctx.queue
        .write_buffer(&comp_buf, 0, bytemuck::cast_slice(components));
    let wt_buf = create_storage_buf(ctx, "weights", (weights.len() * 4) as u64, false);
    ctx.queue
        .write_buffer(&wt_buf, 0, bytemuck::cast_slice(weights));
    let output_buf = create_storage_buf(ctx, "output", output_size, true);

    let bg = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("bg"),
        layout: &bgl,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: comp_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wt_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: output_buf.as_entire_binding(),
            },
        ],
    });
    dispatch_and_read_f32(ctx, &pipeline, &bg, &output_buf, n, 64, 0.0)
}

#[allow(clippy::cast_precision_loss)]
pub fn gpu_run_raycaster(
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
    let entries = [
        storage_entry(0, true),
        storage_entry(1, true),
        storage_entry(2, true),
        storage_entry(3, false),
    ];
    let (pipeline, bgl) = build_pipeline(ctx, "raycaster", DDA_RAYCAST_WGSL, &entries);

    let map_buf = create_storage_buf(ctx, "map", (map_data.len() * 4) as u64, false);
    ctx.queue
        .write_buffer(&map_buf, 0, bytemuck::cast_slice(map_data));
    let params_buf = create_storage_buf(ctx, "params", 24, false);
    ctx.queue
        .write_buffer(&params_buf, 0, bytemuck::cast_slice(&params));
    let angles_buf = create_storage_buf(ctx, "angles", (ray_angles.len() * 4) as u64, false);
    ctx.queue
        .write_buffer(&angles_buf, 0, bytemuck::cast_slice(ray_angles));
    let output_buf = create_storage_buf(ctx, "distances", output_size, true);

    let bg = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("bg"),
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
    dispatch_and_read_f32(ctx, &pipeline, &bg, &output_buf, n_rays, 64, 20.0)
}

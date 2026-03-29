// SPDX-License-Identifier: AGPL-3.0-or-later
//! GPU context and compute shader runners for CPU/GPU parity validation.
//!
//! Shared helpers (`storage_entry`, `build_pipeline`, `dispatch_and_read`)
//! consolidate the repetitive wgpu pipeline boilerplate. Each public
//! `gpu_run_*` function remains a focused orchestrator.

use crate::shaders::{
    DDA_RAYCAST_WGSL, ENGAGEMENT_BATCH_WGSL, FOG_OF_WAR_WGSL, PATHFIND_WAVEFRONT_WGSL,
    PERLIN_2D_WGSL, TILE_LIGHTING_WGSL,
};

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
    let mut usage = wgpu::BufferUsages::STORAGE;
    if writable {
        usage |= wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST;
    } else {
        usage |= wgpu::BufferUsages::COPY_DST;
    }
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
        #[expect(clippy::cast_possible_truncation, reason = "value bounded")]
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
        #[expect(clippy::cast_possible_truncation, reason = "value bounded")]
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

#[expect(clippy::cast_precision_loss, reason = "counts fit in f64 mantissa")]
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

// ── Uniform buffer helpers ─────────────────────────────────────────

const fn uniform_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

fn create_uniform_buf(ctx: &GpuContext, label: &str, data: &[u8]) -> wgpu::Buffer {
    let buf = ctx.device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: data.len() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    ctx.queue.write_buffer(&buf, 0, data);
    buf
}

fn dispatch_and_read_u32(
    ctx: &GpuContext,
    pipeline: &wgpu::ComputePipeline,
    bg: &wgpu::BindGroup,
    output_buf: &wgpu::Buffer,
    n: usize,
    workgroup_size: u32,
) -> Vec<u32> {
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
        #[expect(clippy::cast_possible_truncation, reason = "value bounded")]
        let workgroups = (n as u32).div_ceil(workgroup_size);
        pass.dispatch_workgroups(workgroups, 1, 1);
    }
    encoder.copy_buffer_to_buffer(output_buf, 0, &staging_buf, 0, output_size);
    ctx.queue.submit(std::iter::once(encoder.finish()));

    read_staging_u32(ctx, &staging_buf, n)
}

// ── Game shader GPU runners ────────────────────────────────────────

/// Fog-of-war: per-tile visibility from viewer position.
pub fn gpu_run_fog_of_war(
    ctx: &GpuContext,
    grid_w: u32,
    grid_h: u32,
    viewer_x: f32,
    viewer_y: f32,
    sight_radius_sq: f32,
    terrain: &[f32],
    prev_vis: &[u32],
) -> Vec<u32> {
    let n = (grid_w * grid_h) as usize;
    let mut params_bytes = Vec::with_capacity(24);
    params_bytes.extend_from_slice(&viewer_x.to_le_bytes());
    params_bytes.extend_from_slice(&viewer_y.to_le_bytes());
    params_bytes.extend_from_slice(&grid_w.to_le_bytes());
    params_bytes.extend_from_slice(&grid_h.to_le_bytes());
    params_bytes.extend_from_slice(&sight_radius_sq.to_le_bytes());
    params_bytes.extend_from_slice(&0_f32.to_le_bytes());
    let entries = [
        uniform_entry(0),
        storage_entry(1, true),
        storage_entry(2, true),
        storage_entry(3, false),
    ];
    let (pipeline, bgl) = build_pipeline(ctx, "fog", FOG_OF_WAR_WGSL, &entries);

    let params_buf = create_uniform_buf(ctx, "fog_params", &params_bytes);
    let terrain_buf = create_storage_buf(ctx, "fog_terrain", (n * 4) as u64, false);
    ctx.queue
        .write_buffer(&terrain_buf, 0, bytemuck::cast_slice(terrain));
    let prev_buf = create_storage_buf(ctx, "fog_prev", (n * 4) as u64, false);
    ctx.queue
        .write_buffer(&prev_buf, 0, bytemuck::cast_slice(prev_vis));
    let out_buf = create_storage_buf(ctx, "fog_out", (n * 4) as u64, true);

    let bg = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("fog_bg"),
        layout: &bgl,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: params_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: terrain_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: prev_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: out_buf.as_entire_binding(),
            },
        ],
    });
    dispatch_and_read_u32(ctx, &pipeline, &bg, &out_buf, n, 64)
}

/// Tile lighting: per-tile light intensity from point sources.
#[expect(clippy::cast_precision_loss, reason = "grid dims fit in f32")]
pub fn gpu_run_tile_lighting(
    ctx: &GpuContext,
    grid_w: u32,
    grid_h: u32,
    num_lights: u32,
    ambient: f32,
    terrain: &[f32],
    lights: &[[f32; 4]],
) -> Vec<f32> {
    let n = (grid_w * grid_h) as usize;

    let mut params_bytes = Vec::with_capacity(16);
    params_bytes.extend_from_slice(&grid_w.to_le_bytes());
    params_bytes.extend_from_slice(&grid_h.to_le_bytes());
    params_bytes.extend_from_slice(&num_lights.to_le_bytes());
    params_bytes.extend_from_slice(&ambient.to_le_bytes());

    let entries = [
        uniform_entry(0),
        storage_entry(1, true),
        storage_entry(2, true),
        storage_entry(3, false),
    ];
    let (pipeline, bgl) = build_pipeline(ctx, "lighting", TILE_LIGHTING_WGSL, &entries);

    let params_buf = create_uniform_buf(ctx, "light_params", &params_bytes);
    let terrain_buf = create_storage_buf(ctx, "light_terrain", (n * 4) as u64, false);
    ctx.queue
        .write_buffer(&terrain_buf, 0, bytemuck::cast_slice(terrain));

    let mut lights_flat: Vec<f32> = Vec::with_capacity(lights.len() * 4);
    for l in lights {
        lights_flat.extend_from_slice(l);
    }
    let lights_buf =
        create_storage_buf(ctx, "lights", (lights_flat.len() * 4).max(16) as u64, false);
    ctx.queue
        .write_buffer(&lights_buf, 0, bytemuck::cast_slice(&lights_flat));
    let out_buf = create_storage_buf(ctx, "light_out", (n * 4) as u64, true);

    let bg = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("light_bg"),
        layout: &bgl,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: params_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: terrain_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: lights_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: out_buf.as_entire_binding(),
            },
        ],
    });
    dispatch_and_read_f32(ctx, &pipeline, &bg, &out_buf, n, 64, 0.0)
}

/// Pathfind wavefront: single BFS expansion step on a 2D grid.
/// Returns the dist_map after one expansion from the given dist_map state.
pub fn gpu_run_pathfind_step(
    ctx: &GpuContext,
    grid_w: u32,
    grid_h: u32,
    current_dist: u32,
    terrain: &[f32],
    dist_map_in: &[u32],
) -> (Vec<u32>, u32) {
    let n = (grid_w * grid_h) as usize;

    let mut params_bytes = Vec::with_capacity(16);
    params_bytes.extend_from_slice(&grid_w.to_le_bytes());
    params_bytes.extend_from_slice(&grid_h.to_le_bytes());
    params_bytes.extend_from_slice(&current_dist.to_le_bytes());
    params_bytes.extend_from_slice(&0_u32.to_le_bytes());

    let entries = [
        uniform_entry(0),
        storage_entry(1, true),
        storage_entry(2, false),
        storage_entry(3, false),
    ];
    let (pipeline, bgl) = build_pipeline(ctx, "pathfind", PATHFIND_WAVEFRONT_WGSL, &entries);

    let params_buf = create_uniform_buf(ctx, "pf_params", &params_bytes);
    let terrain_buf = create_storage_buf(ctx, "pf_terrain", (n * 4) as u64, false);
    ctx.queue
        .write_buffer(&terrain_buf, 0, bytemuck::cast_slice(terrain));

    let dist_buf = create_storage_buf(ctx, "pf_dist", (n * 4) as u64, true);
    ctx.queue
        .write_buffer(&dist_buf, 0, bytemuck::cast_slice(dist_map_in));

    let frontier_buf = create_storage_buf(ctx, "pf_frontier", 4, true);
    ctx.queue
        .write_buffer(&frontier_buf, 0, &0_u32.to_le_bytes());

    let bg = ctx.device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("pf_bg"),
        layout: &bgl,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: params_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: terrain_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: dist_buf.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: frontier_buf.as_entire_binding(),
            },
        ],
    });

    let dist_result = dispatch_and_read_u32(ctx, &pipeline, &bg, &dist_buf, n, 64);

    let frontier_staging = create_staging_buf(ctx, 4);
    let mut encoder = ctx
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some("enc") });
    encoder.copy_buffer_to_buffer(&frontier_buf, 0, &frontier_staging, 0, 4);
    ctx.queue.submit(std::iter::once(encoder.finish()));
    let frontier_val = read_staging_u32(ctx, &frontier_staging, 1);

    (dist_result, frontier_val.first().copied().unwrap_or(0))
}

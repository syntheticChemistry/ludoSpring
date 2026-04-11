// SPDX-License-Identifier: AGPL-3.0-or-later
//! GPU compute IPC — forwards game GPU workloads to toadStool `compute.dispatch.submit`.

use serde_json::json;

use crate::game::engine::gpu::{FogOfWarParams, GpuOp, PointLight, TileLightingParams};
use crate::ipc::envelope::{JsonRpcError, JsonRpcRequest};
use crate::ipc::params::{
    GpuBatchRaycastParams, GpuFogOfWarParams, GpuPathfindParams, GpuPerlinTerrainParams,
    GpuTileLightingParams,
};
use crate::ipc::toadstool;
use crate::procedural::noise::perlin_perm_table_u32;

use super::{HandlerResult, parse_params, to_json};

const DEGRADE_REASON: &str = "compute dispatch unavailable — CPU fallback active";

fn degradation_value() -> serde_json::Value {
    json!({
        "available": false,
        "fallback": "cpu",
        "reason": DEGRADE_REASON,
    })
}

fn tile_count_or_err(id: &serde_json::Value, w: u32, h: u32) -> Result<u32, JsonRpcError> {
    if w == 0 || h == 0 {
        return Err(JsonRpcError::invalid_params(
            id,
            "grid_w and grid_h must be positive",
        ));
    }
    w.checked_mul(h)
        .ok_or_else(|| JsonRpcError::invalid_params(id, "grid_w * grid_h overflows u32"))
}

#[expect(
    clippy::cast_possible_truncation,
    reason = "f64→f32 narrowing is the function's purpose — GPU buffers are f32"
)]
fn f32_vec_from_opt_f64(
    id: &serde_json::Value,
    v: Option<Vec<f64>>,
    len: usize,
    fill: f32,
) -> Result<Vec<f32>, JsonRpcError> {
    match v {
        None => Ok(vec![fill; len]),
        Some(xs) if xs.len() == len => Ok(xs.iter().map(|&x| x as f32).collect()),
        Some(xs) => Err(JsonRpcError::invalid_params(
            id,
            &format!("expected {len} terrain values, got {}", xs.len()),
        )),
    }
}

fn u32_vec_from_opt(
    id: &serde_json::Value,
    v: Option<Vec<u32>>,
    len: usize,
    fill: u32,
) -> Result<Vec<u32>, JsonRpcError> {
    match v {
        None => Ok(vec![fill; len]),
        Some(xs) if xs.len() == len => Ok(xs),
        Some(xs) => Err(JsonRpcError::invalid_params(
            id,
            &format!("expected {len} u32 values, got {}", xs.len()),
        )),
    }
}

#[expect(
    clippy::needless_pass_by_value,
    reason = "u32 is Copy; serde_json::Value ownership is consumed by the toadstool call"
)]
fn dispatch_gpu(
    req: &JsonRpcRequest,
    op: GpuOp,
    element_count: u32,
    buffers: serde_json::Value,
) -> HandlerResult {
    let shader = op
        .wgsl_source()
        .ok_or_else(|| JsonRpcError::internal(&req.id, "GPU op has no embedded WGSL source"))?;
    let wg_n = op.workgroup_size();
    let workgroup_size = [wg_n, 1, 1];
    let dispatch_x = op.dispatch_count(element_count).max(1);
    let dispatch_size = [dispatch_x, 1, 1];

    let result =
        toadstool::dispatch_submit(shader, "main", workgroup_size, dispatch_size, &buffers)
            .map_err(|e| JsonRpcError::internal(&req.id, &e))?;

    if result.available {
        to_json(
            &req.id,
            json!({
                "available": true,
                "message": result.message,
                "data": result.data,
            }),
        )
    } else {
        to_json(&req.id, degradation_value())
    }
}

/// `game.gpu.fog_of_war`
#[expect(
    clippy::cast_possible_truncation,
    reason = "f64→f32 narrowing for GPU buffers — game coordinates are small"
)]
pub(super) fn handle_gpu_fog_of_war(req: &JsonRpcRequest) -> HandlerResult {
    let p: GpuFogOfWarParams = parse_params(req)?;
    let n = tile_count_or_err(&req.id, p.grid_w, p.grid_h)? as usize;
    let terrain = f32_vec_from_opt_f64(&req.id, p.terrain, n, 0.0)?;
    let prev_vis = u32_vec_from_opt(&req.id, p.prev_vis, n, 0)?;

    let fog = FogOfWarParams::new(
        p.viewer_x as f32,
        p.viewer_y as f32,
        p.grid_w,
        p.grid_h,
        p.sight_radius,
    );
    let uniform: Vec<f32> = fog.as_uniform().to_vec();
    let op = GpuOp::FogOfWar;

    let buffers = json!({
        "ludospring_gpu_v1": true,
        "op": op.shader_name(),
        "uniform_f32": uniform,
        "storage": {
            "terrain": terrain,
            "prev_vis": prev_vis,
        }
    });
    dispatch_gpu(req, op, fog.tile_count(), buffers)
}

/// `game.gpu.tile_lighting`
#[expect(
    clippy::cast_possible_truncation,
    reason = "f64→f32 narrowing for GPU buffers — game coordinates and light params are small"
)]
pub(super) fn handle_gpu_tile_lighting(req: &JsonRpcRequest) -> HandlerResult {
    let p: GpuTileLightingParams = parse_params(req)?;
    let n = tile_count_or_err(&req.id, p.grid_w, p.grid_h)? as usize;
    let terrain = f32_vec_from_opt_f64(&req.id, p.terrain, n, 0.0)?;

    let lights: Vec<PointLight> = p
        .lights
        .iter()
        .map(|l| PointLight {
            x: l.x as f32,
            y: l.y as f32,
            intensity: l.intensity as f32,
            radius: l.radius as f32,
        })
        .collect();

    let tp = TileLightingParams {
        grid_w: p.grid_w,
        grid_h: p.grid_h,
        ambient: p.ambient as f32,
        lights,
    };

    let num_lights = tp.light_count();
    let mut lights_flat: Vec<f32> = Vec::with_capacity(32);
    for light in tp.lights.iter().take(8) {
        lights_flat.extend_from_slice(&[light.x, light.y, light.intensity, light.radius]);
    }
    lights_flat.resize(32, 0.0);

    let uniform = json!({
        "grid_w": tp.grid_w,
        "grid_h": tp.grid_h,
        "num_lights": num_lights,
        "ambient": tp.ambient,
    });

    let buffers = json!({
        "ludospring_gpu_v1": true,
        "op": GpuOp::TileLighting.shader_name(),
        "uniform": uniform,
        "storage": {
            "terrain": terrain,
            "lights_f32": lights_flat,
        }
    });

    dispatch_gpu(req, GpuOp::TileLighting, tp.tile_count(), buffers)
}

/// `game.gpu.pathfind`
pub(super) fn handle_gpu_pathfind(req: &JsonRpcRequest) -> HandlerResult {
    let p: GpuPathfindParams = parse_params(req)?;
    let n = tile_count_or_err(&req.id, p.grid_w, p.grid_h)? as usize;
    if p.start_x >= p.grid_w || p.start_y >= p.grid_h {
        return Err(JsonRpcError::invalid_params(
            &req.id,
            "start_x and start_y must be inside the grid",
        ));
    }

    let terrain = f32_vec_from_opt_f64(&req.id, p.terrain, n, 0.0)?;
    let current_dist = p.current_dist.unwrap_or(0);

    let dist_map = match &p.dist_map {
        None => {
            let start_idx = (p.start_y * p.grid_w + p.start_x) as usize;
            let mut dm = vec![u32::MAX; n];
            dm[start_idx] = 0;
            dm
        }
        Some(dm) if dm.len() == n => dm.clone(),
        Some(dm) => {
            return Err(JsonRpcError::invalid_params(
                &req.id,
                &format!("expected {} dist_map values, got {}", n, dm.len()),
            ));
        }
    };

    let buffers = json!({
        "ludospring_gpu_v1": true,
        "op": GpuOp::PathfindStep.shader_name(),
        "uniform": {
            "grid_w": p.grid_w,
            "grid_h": p.grid_h,
            "current_dist": current_dist,
        },
        "storage": {
            "terrain": terrain,
            "dist_map": dist_map,
            "frontier_u32": [0u32],
        }
    });

    dispatch_gpu(req, GpuOp::PathfindStep, p.grid_w * p.grid_h, buffers)
}

/// `game.gpu.perlin_terrain`
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    reason = "GPU buffers require f32; grid coords and seed offset are small values"
)]
pub(super) fn handle_gpu_perlin_terrain(req: &JsonRpcRequest) -> HandlerResult {
    let p: GpuPerlinTerrainParams = parse_params(req)?;
    let n = tile_count_or_err(&req.id, p.grid_w, p.grid_h)? as usize;
    let perm: Vec<u32> = perlin_perm_table_u32().to_vec();
    let seed_off = p.seed.unwrap_or(0) as f32 * 0.001;
    let mut coords = Vec::with_capacity(n * 2);
    for ty in 0..p.grid_h {
        for tx in 0..p.grid_w {
            coords.push(tx as f32 + seed_off);
            coords.push(ty as f32 + seed_off);
        }
    }
    let output = vec![0.0f32; n];

    let buffers = json!({
        "ludospring_gpu_v1": true,
        "op": GpuOp::PerlinTerrain.shader_name(),
        "storage": {
            "perm_u32": perm,
            "coords_f32": coords,
            "output_f32": output,
        }
    });

    dispatch_gpu(req, GpuOp::PerlinTerrain, n as u32, buffers)
}

/// `game.gpu.batch_raycast`
#[expect(
    clippy::cast_possible_truncation,
    reason = "f64→f32 narrowing for GPU buffers — game coordinates are small"
)]
pub(super) fn handle_gpu_batch_raycast(req: &JsonRpcRequest) -> HandlerResult {
    let p: GpuBatchRaycastParams = parse_params(req)?;
    let n = tile_count_or_err(&req.id, p.grid_w, p.grid_h)? as usize;
    let ray_count = p.origins_x.len();

    if p.origins_y.len() != ray_count || p.angles.len() != ray_count {
        return Err(JsonRpcError::invalid_params(
            &req.id,
            "origins_x, origins_y, and angles must have equal length",
        ));
    }
    if ray_count == 0 {
        return Err(JsonRpcError::invalid_params(
            &req.id,
            "at least one ray required",
        ));
    }

    let walls = f32_vec_from_opt_f64(&req.id, p.walls, n, 0.0)?;

    let mut ray_data: Vec<f32> = Vec::with_capacity(ray_count * 4);
    for i in 0..ray_count {
        let angle = p.angles[i];
        ray_data.push(p.origins_x[i] as f32);
        ray_data.push(p.origins_y[i] as f32);
        ray_data.push(angle.cos() as f32);
        ray_data.push(angle.sin() as f32);
    }
    let output = vec![0.0f32; ray_count];

    let buffers = serde_json::json!({
        "ludospring_gpu_v1": true,
        "op": GpuOp::BatchRaycast.shader_name(),
        "uniform": {
            "grid_w": p.grid_w,
            "grid_h": p.grid_h,
            "ray_count": ray_count,
        },
        "storage": {
            "walls_f32": walls,
            "rays_f32": ray_data,
            "output_f32": output,
        }
    });

    #[expect(
        clippy::cast_possible_truncation,
        reason = "ray count validated above, fits u32 for any practical grid"
    )]
    let count = ray_count as u32;
    dispatch_gpu(req, GpuOp::BatchRaycast, count, buffers)
}

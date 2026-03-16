// SPDX-License-Identifier: AGPL-3.0-or-later
// Tile lighting — GPU-parallel light propagation on a 2D tile grid.
//
// Computes light intensity at each tile from up to 8 point light sources.
// Light falls off as 1/distance² and is blocked by walls (terrain >= 0.9).
// The result is a per-tile f32 intensity (0.0 = dark, 1.0 = full light).
//
// Dispatch: one thread per tile (workgroup_size 64, dispatch ceil(tile_count/64)).
//
// Bindings:
//   0: params       [grid_w, grid_h, num_lights, ambient]
//   1: terrain      [f32 per tile — 1.0 = wall, 0.0 = open]
//   2: lights       [x, y, intensity, radius] × max 8 lights (32 floats)
//   3: out_light    [f32 per tile — computed light intensity]

struct Params {
    grid_w: u32,
    grid_h: u32,
    num_lights: u32,
    ambient: f32,
}

struct Light {
    x: f32,
    y: f32,
    intensity: f32,
    radius: f32,
}

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> terrain: array<f32>;
@group(0) @binding(2) var<storage, read> lights: array<Light>;
@group(0) @binding(3) var<storage, read_write> out_light: array<f32>;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let idx = gid.x;
    let total = params.grid_w * params.grid_h;
    if idx >= total {
        return;
    }

    // Walls emit no light and receive no light
    if terrain[idx] >= 0.9 {
        out_light[idx] = 0.0;
        return;
    }

    let tile_x = f32(idx % params.grid_w) + 0.5;
    let tile_y = f32(idx / params.grid_w) + 0.5;

    var total_light = params.ambient;

    let count = min(params.num_lights, 8u);
    for (var i = 0u; i < count; i = i + 1u) {
        let light = lights[i];
        let dx = tile_x - light.x;
        let dy = tile_y - light.y;
        let dist_sq = dx * dx + dy * dy;
        let radius_sq = light.radius * light.radius;

        if dist_sq < radius_sq {
            let attenuation = 1.0 / (1.0 + dist_sq);
            total_light += light.intensity * attenuation;
        }
    }

    out_light[idx] = clamp(total_light, 0.0, 1.0);
}

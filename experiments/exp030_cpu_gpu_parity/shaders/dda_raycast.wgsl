// SPDX-License-Identifier: AGPL-3.0-or-later
// DDA raycaster — per-column wall distance for a 2D grid map.
// Validated against barraCuda CPU `game::raycaster::cast_ray`.
// f32 GPU vs f64 CPU tolerance: < 0.5 (iterative DDA accumulation).

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

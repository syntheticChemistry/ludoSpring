// SPDX-License-Identifier: AGPL-3.0-or-later
// Pathfinding wavefront — single-step BFS expansion on a 2D tile grid.
//
// Each dispatch expands the BFS frontier by one step. The host loops
// until no new tiles are reached (frontier_count == 0) or a target
// is found. This is the GPU-parallel equivalent of Dijkstra's wavefront.
//
// Dispatch repeatedly: each call expands one ring of the wavefront.
// Suitable for large grids where CPU BFS is a bottleneck.
//
// Bindings:
//   0: params       [grid_w, grid_h, current_dist, _pad]
//   1: terrain      [f32 per tile — >= 0.9 = impassable]
//   2: dist_map     [u32 per tile — 0xFFFFFFFF = unvisited, else distance]
//   3: frontier     [atomic u32 — count of tiles expanded this step]

struct Params {
    grid_w: u32,
    grid_h: u32,
    current_dist: u32,
    _pad: u32,
}

@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> terrain: array<f32>;
@group(0) @binding(2) var<storage, read_write> dist_map: array<u32>;
@group(0) @binding(3) var<storage, read_write> frontier: atomic<u32>;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let idx = gid.x;
    let total = params.grid_w * params.grid_h;
    if idx >= total {
        return;
    }

    // Only process tiles at the current frontier distance
    if dist_map[idx] != params.current_dist {
        return;
    }

    let x = idx % params.grid_w;
    let y = idx / params.grid_w;
    let next_dist = params.current_dist + 1u;

    // Check 4 cardinal neighbors
    let offsets = array<vec2<i32>, 4>(
        vec2<i32>(0, -1), // North
        vec2<i32>(0, 1),  // South
        vec2<i32>(1, 0),  // East
        vec2<i32>(-1, 0)  // West
    );

    for (var i = 0u; i < 4u; i = i + 1u) {
        let nx = i32(x) + offsets[i].x;
        let ny = i32(y) + offsets[i].y;

        if nx < 0 || ny < 0 || nx >= i32(params.grid_w) || ny >= i32(params.grid_h) {
            continue;
        }

        let nidx = u32(ny) * params.grid_w + u32(nx);

        // Skip walls
        if terrain[nidx] >= 0.9 {
            continue;
        }

        // Only update unvisited tiles (avoid race conditions with atomicMin)
        if dist_map[nidx] == 0xFFFFFFFFu {
            dist_map[nidx] = next_dist;
            atomicAdd(&frontier, 1u);
        }
    }
}

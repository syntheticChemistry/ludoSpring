// SPDX-License-Identifier: AGPL-3.0-or-later
//! GPU compute dispatch parameter types (`game.gpu.*` methods).
//!
//! These map to toadStool-routed WGSL compute shaders: fog of war,
//! tile lighting, pathfinding, procedural terrain, and batch raycasting.

use serde::Deserialize;

/// Parameters for `game.gpu.fog_of_war` (toadStool compute dispatch).
#[derive(Debug, Deserialize)]
pub struct GpuFogOfWarParams {
    /// Grid width in tiles.
    pub grid_w: u32,
    /// Grid height in tiles.
    pub grid_h: u32,
    /// Viewer X (tile coordinates, may be fractional).
    pub viewer_x: f64,
    /// Viewer Y.
    pub viewer_y: f64,
    /// Sight radius in tiles.
    pub sight_radius: u32,
    /// Optional per-tile terrain (`1.0` = wall); defaults to open floor when omitted.
    #[serde(default)]
    pub terrain: Option<Vec<f64>>,
    /// Optional previous visibility per tile; defaults to unseen when omitted.
    #[serde(default)]
    pub prev_vis: Option<Vec<u32>>,
}

/// Point light for `game.gpu.tile_lighting` IPC.
#[derive(Debug, Deserialize)]
pub struct GpuPointLightParam {
    /// Light X (tile coordinates).
    pub x: f64,
    /// Light Y.
    pub y: f64,
    /// Intensity in \[0, 1\].
    pub intensity: f64,
    /// Radius in tiles.
    pub radius: f64,
}

/// Parameters for `game.gpu.tile_lighting`.
#[derive(Debug, Deserialize)]
pub struct GpuTileLightingParams {
    /// Grid width in tiles.
    pub grid_w: u32,
    /// Grid height in tiles.
    pub grid_h: u32,
    /// Ambient level \[0, 1\].
    pub ambient: f64,
    /// Active lights (up to 8 used).
    pub lights: Vec<GpuPointLightParam>,
    /// Optional terrain heights for line-of-sight shadowing.
    #[serde(default)]
    pub terrain: Option<Vec<f64>>,
}

/// Parameters for `game.gpu.pathfind` (one BFS wavefront step).
#[derive(Debug, Deserialize)]
pub struct GpuPathfindParams {
    /// Grid width in tiles.
    pub grid_w: u32,
    /// Grid height in tiles.
    pub grid_h: u32,
    /// Start tile X coordinate.
    pub start_x: u32,
    /// Start tile Y coordinate.
    pub start_y: u32,
    /// Distance ring the shader expands from (default `0` for a fresh search).
    #[serde(default)]
    pub current_dist: Option<u32>,
    /// Optional terrain heights for passability checks.
    #[serde(default)]
    pub terrain: Option<Vec<f64>>,
    /// Full distance map; when omitted, unvisited tiles start at `u32::MAX` and the start cell is seeded.
    #[serde(default)]
    pub dist_map: Option<Vec<u32>>,
}

/// Parameters for `game.gpu.perlin_terrain`.
#[derive(Debug, Deserialize)]
pub struct GpuPerlinTerrainParams {
    /// Grid width in tiles.
    pub grid_w: u32,
    /// Grid height in tiles.
    pub grid_h: u32,
    /// Optional coordinate offset seed for sample positions.
    #[serde(default)]
    pub seed: Option<u64>,
}

/// Parameters for `game.gpu.batch_raycast` (DDA batch line-of-sight).
#[derive(Debug, Deserialize)]
pub struct GpuBatchRaycastParams {
    /// Grid width in tiles.
    pub grid_w: u32,
    /// Grid height in tiles.
    pub grid_h: u32,
    /// Ray origin X positions (tile coordinates, fractional).
    pub origins_x: Vec<f64>,
    /// Ray origin Y positions.
    pub origins_y: Vec<f64>,
    /// Ray direction angles in radians.
    pub angles: Vec<f64>,
    /// Optional per-tile wall map (1.0 = solid, 0.0 = open); defaults to open.
    #[serde(default)]
    pub walls: Option<Vec<f64>>,
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::*;

    #[test]
    fn gpu_ipc_params_deserialize() {
        let _: GpuFogOfWarParams = serde_json::from_value(serde_json::json!({
            "grid_w": 8,
            "grid_h": 8,
            "viewer_x": 1.5,
            "viewer_y": 2.5,
            "sight_radius": 4
        }))
        .expect("fog");

        let _: GpuTileLightingParams = serde_json::from_value(serde_json::json!({
            "grid_w": 4,
            "grid_h": 4,
            "ambient": 0.1,
            "lights": [{"x": 1.0, "y": 1.0, "intensity": 0.8, "radius": 5.0}]
        }))
        .expect("light");

        let _: GpuPathfindParams = serde_json::from_value(serde_json::json!({
            "grid_w": 4,
            "grid_h": 4,
            "start_x": 0,
            "start_y": 0
        }))
        .expect("path");

        let _: GpuPerlinTerrainParams = serde_json::from_value(serde_json::json!({
            "grid_w": 16,
            "grid_h": 16,
            "seed": 42
        }))
        .expect("perlin");
    }
}

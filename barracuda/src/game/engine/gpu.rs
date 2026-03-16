// SPDX-License-Identifier: AGPL-3.0-or-later
//! GPU compute dispatch for 2D game engine operations.
//!
//! Defines the shader catalog, dispatch request types, and result types
//! for game-specific GPU compute operations. These are dispatched via:
//!
//! - **barraCuda in-process** — `ComputeDispatch` builder with wgpu
//! - **toadStool IPC** — `compute.submit` JSON-RPC with `Custom` job type
//! - **coralReef compile** — `shader.compile.wgsl` for native GPU binaries
//!
//! The engine never assumes which dispatch path is available. It discovers
//! GPU capability at runtime via Songbird (`compute.dispatch`, `science.gpu.dispatch`)
//! and falls back to CPU implementations in [`super::world`] when no GPU is found.
//!
//! # Shaders
//!
//! Source lives in `barracuda/shaders/game/`:
//! - `fog_of_war.wgsl` — per-tile visibility from viewer position
//! - `tile_lighting.wgsl` — point light propagation with 1/d² falloff
//! - `pathfind_wavefront.wgsl` — BFS expansion (one ring per dispatch)
//!
//! Plus inherited from barraCuda ecosystem:
//! - `perlin_2d.wgsl` — terrain generation (validated exp030)
//! - `dda_raycast.wgsl` — batch line-of-sight (validated exp030)

/// Embedded shader sources.
pub mod shaders {
    /// Fog of war — per-tile visibility computation.
    pub const FOG_OF_WAR: &str = include_str!("../../../shaders/game/fog_of_war.wgsl");

    /// Tile lighting — point light propagation.
    pub const TILE_LIGHTING: &str = include_str!("../../../shaders/game/tile_lighting.wgsl");

    /// Pathfinding wavefront — BFS expansion step.
    pub const PATHFIND_WAVEFRONT: &str =
        include_str!("../../../shaders/game/pathfind_wavefront.wgsl");
}

/// A named GPU compute operation the engine can dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GpuOp {
    /// Compute fog of war visibility for the entire grid.
    FogOfWar,
    /// Compute tile lighting from point light sources.
    TileLighting,
    /// Run one step of BFS pathfinding wavefront.
    PathfindStep,
    /// Generate Perlin 2D noise for terrain.
    PerlinTerrain,
    /// Batch DDA raycasting for line-of-sight.
    BatchRaycast,
}

impl GpuOp {
    /// WGSL source for this operation (if embedded).
    #[must_use]
    pub const fn wgsl_source(self) -> Option<&'static str> {
        match self {
            Self::FogOfWar => Some(shaders::FOG_OF_WAR),
            Self::TileLighting => Some(shaders::TILE_LIGHTING),
            Self::PathfindStep => Some(shaders::PATHFIND_WAVEFRONT),
            Self::PerlinTerrain | Self::BatchRaycast => None,
        }
    }

    /// Shader name for toadStool `Custom` job dispatch.
    #[must_use]
    pub const fn shader_name(self) -> &'static str {
        match self {
            Self::FogOfWar => "fog_of_war",
            Self::TileLighting => "tile_lighting",
            Self::PathfindStep => "pathfind_wavefront",
            Self::PerlinTerrain => "perlin_2d",
            Self::BatchRaycast => "dda_raycast",
        }
    }

    /// Workgroup size used by the shader.
    #[must_use]
    pub const fn workgroup_size(self) -> u32 {
        64
    }

    /// Calculate dispatch size (number of workgroups) for a given element count.
    #[must_use]
    pub const fn dispatch_count(self, element_count: u32) -> u32 {
        element_count.div_ceil(self.workgroup_size())
    }
}

/// Parameters for a fog of war GPU dispatch.
#[derive(Debug, Clone, Copy)]
pub struct FogOfWarParams {
    /// Viewer X position (tile coordinates, can be fractional).
    pub viewer_x: f32,
    /// Viewer Y position.
    pub viewer_y: f32,
    /// Grid width in tiles.
    pub grid_w: u32,
    /// Grid height in tiles.
    pub grid_h: u32,
    /// Sight radius squared (avoids sqrt in shader).
    pub sight_radius_sq: f32,
}

impl FogOfWarParams {
    /// Create from viewer position and sight radius.
    #[must_use]
    #[expect(
        clippy::cast_precision_loss,
        reason = "grid coords are small enough for f32"
    )]
    pub const fn new(
        viewer_x: f32,
        viewer_y: f32,
        grid_w: u32,
        grid_h: u32,
        sight_radius: u32,
    ) -> Self {
        Self {
            viewer_x,
            viewer_y,
            grid_w,
            grid_h,
            sight_radius_sq: (sight_radius * sight_radius) as f32,
        }
    }

    /// Total tile count.
    #[must_use]
    pub const fn tile_count(&self) -> u32 {
        self.grid_w * self.grid_h
    }

    /// Pack into a uniform buffer layout (6 × f32, matching the WGSL struct).
    #[must_use]
    pub const fn as_uniform(&self) -> [f32; 6] {
        [
            self.viewer_x,
            self.viewer_y,
            // Reinterpret u32 as f32 bits for the shader's u32 fields.
            // The shader reads grid_w/grid_h as u32 from the uniform struct.
            f32::from_bits(self.grid_w),
            f32::from_bits(self.grid_h),
            self.sight_radius_sq,
            0.0, // padding
        ]
    }
}

/// A point light source for tile lighting.
#[derive(Debug, Clone, Copy)]
pub struct PointLight {
    /// Light X position (tile coordinates).
    pub x: f32,
    /// Light Y position.
    pub y: f32,
    /// Light intensity (0.0..=1.0).
    pub intensity: f32,
    /// Light radius in tiles.
    pub radius: f32,
}

/// Parameters for tile lighting GPU dispatch.
#[derive(Debug, Clone)]
pub struct TileLightingParams {
    /// Grid width.
    pub grid_w: u32,
    /// Grid height.
    pub grid_h: u32,
    /// Ambient light level (0.0..=1.0).
    pub ambient: f32,
    /// Point light sources (max 8).
    pub lights: Vec<PointLight>,
}

impl TileLightingParams {
    /// Total tile count.
    #[must_use]
    pub const fn tile_count(&self) -> u32 {
        self.grid_w * self.grid_h
    }

    /// Number of active lights (clamped to 8).
    #[must_use]
    #[expect(
        clippy::cast_possible_truncation,
        reason = "clamped to 8, always fits u32"
    )]
    pub fn light_count(&self) -> u32 {
        self.lights.len().min(8) as u32
    }
}

/// Whether GPU compute is available (discovered at runtime).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuAvailability {
    /// No GPU compute found — use CPU fallback.
    None,
    /// barraCuda in-process (wgpu device available).
    InProcess,
    /// toadStool IPC (remote GPU dispatch via JSON-RPC).
    ToadStool,
    /// Both in-process and toadStool available.
    Both,
}

impl GpuAvailability {
    /// Whether any GPU compute path is available.
    #[must_use]
    pub const fn has_gpu(self) -> bool {
        !matches!(self, Self::None)
    }

    /// Whether in-process dispatch is available.
    #[must_use]
    pub const fn has_in_process(self) -> bool {
        matches!(self, Self::InProcess | Self::Both)
    }

    /// Whether toadStool IPC dispatch is available.
    #[must_use]
    pub const fn has_toadstool(self) -> bool {
        matches!(self, Self::ToadStool | Self::Both)
    }
}

/// Result of a GPU fog-of-war computation.
#[derive(Debug, Clone)]
pub struct FogResult {
    /// Per-tile visibility: 0 = hidden, 1 = explored, 2 = visible.
    pub visibility: Vec<u32>,
}

/// Result of a GPU tile-lighting computation.
#[derive(Debug, Clone)]
pub struct LightResult {
    /// Per-tile light intensity (0.0..=1.0).
    pub intensity: Vec<f32>,
}

/// Result of a GPU pathfinding step.
#[derive(Debug, Clone)]
pub struct PathfindResult {
    /// Per-tile distance from origin (u32::MAX = unreached).
    pub distances: Vec<u32>,
    /// Number of tiles expanded in this step.
    pub frontier_count: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shader_sources_embedded() {
        assert!(shaders::FOG_OF_WAR.contains("visibility"));
        assert!(shaders::TILE_LIGHTING.contains("out_light"));
        assert!(shaders::PATHFIND_WAVEFRONT.contains("dist_map"));
    }

    #[test]
    fn gpu_op_dispatch_count() {
        let op = GpuOp::FogOfWar;
        assert_eq!(op.dispatch_count(64), 1);
        assert_eq!(op.dispatch_count(65), 2);
        assert_eq!(op.dispatch_count(128), 2);
        assert_eq!(op.dispatch_count(0), 0);
    }

    #[test]
    fn fog_params_creation() {
        let params = FogOfWarParams::new(5.5, 3.5, 100, 80, 8);
        assert_eq!(params.tile_count(), 8000);
        assert!((params.sight_radius_sq - 64.0).abs() < f32::EPSILON);
    }

    #[test]
    fn fog_params_uniform_packing() {
        let params = FogOfWarParams::new(10.0, 20.0, 50, 40, 5);
        let uniform = params.as_uniform();
        assert!((uniform[0] - 10.0).abs() < f32::EPSILON);
        assert!((uniform[1] - 20.0).abs() < f32::EPSILON);
        assert!((uniform[4] - 25.0).abs() < f32::EPSILON); // 5² = 25
    }

    #[test]
    fn point_light_creation() {
        let light = PointLight {
            x: 5.0,
            y: 5.0,
            intensity: 0.8,
            radius: 10.0,
        };
        assert!((light.intensity - 0.8).abs() < f32::EPSILON);
    }

    #[test]
    fn lighting_params_light_count() {
        let params = TileLightingParams {
            grid_w: 50,
            grid_h: 50,
            ambient: 0.1,
            lights: vec![
                PointLight {
                    x: 5.0,
                    y: 5.0,
                    intensity: 1.0,
                    radius: 10.0,
                },
                PointLight {
                    x: 25.0,
                    y: 25.0,
                    intensity: 0.5,
                    radius: 8.0,
                },
            ],
        };
        assert_eq!(params.tile_count(), 2500);
        assert_eq!(params.light_count(), 2);
    }

    #[test]
    fn lighting_params_clamps_to_8() {
        let params = TileLightingParams {
            grid_w: 10,
            grid_h: 10,
            ambient: 0.0,
            lights: (0..12)
                .map(|i| PointLight {
                    x: i as f32,
                    y: 0.0,
                    intensity: 1.0,
                    radius: 5.0,
                })
                .collect(),
        };
        assert_eq!(params.light_count(), 8);
    }

    #[test]
    fn gpu_availability() {
        assert!(!GpuAvailability::None.has_gpu());
        assert!(GpuAvailability::InProcess.has_gpu());
        assert!(GpuAvailability::ToadStool.has_gpu());
        assert!(GpuAvailability::Both.has_gpu());

        assert!(GpuAvailability::InProcess.has_in_process());
        assert!(!GpuAvailability::ToadStool.has_in_process());
        assert!(GpuAvailability::Both.has_in_process());

        assert!(!GpuAvailability::InProcess.has_toadstool());
        assert!(GpuAvailability::ToadStool.has_toadstool());
        assert!(GpuAvailability::Both.has_toadstool());
    }

    #[test]
    fn gpu_op_shader_names() {
        assert_eq!(GpuOp::FogOfWar.shader_name(), "fog_of_war");
        assert_eq!(GpuOp::TileLighting.shader_name(), "tile_lighting");
        assert_eq!(GpuOp::PathfindStep.shader_name(), "pathfind_wavefront");
        assert_eq!(GpuOp::PerlinTerrain.shader_name(), "perlin_2d");
        assert_eq!(GpuOp::BatchRaycast.shader_name(), "dda_raycast");
    }

    #[test]
    fn gpu_op_wgsl_sources() {
        assert!(GpuOp::FogOfWar.wgsl_source().is_some());
        assert!(GpuOp::TileLighting.wgsl_source().is_some());
        assert!(GpuOp::PathfindStep.wgsl_source().is_some());
        // barraCuda-owned shaders are not embedded here
        assert!(GpuOp::PerlinTerrain.wgsl_source().is_none());
        assert!(GpuOp::BatchRaycast.wgsl_source().is_none());
    }
}

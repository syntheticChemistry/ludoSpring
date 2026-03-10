// SPDX-License-Identifier: AGPL-3.0-or-later

#![forbid(unsafe_code)]
#![deny(clippy::expect_used, clippy::unwrap_used)]

//! ludoSpring Forge — hardware discovery and dispatch for game science workloads.
//!
//! Forge discovers compute substrates (CPU, GPU, NPU) and routes ludoSpring
//! workloads to the best capable substrate. It leans on ToadStool/barraCuda
//! for GPU discovery and device management, and adds game-specific dispatch
//! logic (e.g., noise generation on GPU, WFC on CPU).
//!
//! # Design Principle
//!
//! Springs don't reference each other. ludoSpring doesn't import wetSpring.
//! Both lean on ToadStool independently — ludoSpring evolves game/interaction
//! shaders, wetSpring evolves bio shaders, and ToadStool absorbs both.

/// Game science workload types for dispatch routing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameWorkload {
    /// Perlin/simplex noise field generation.
    NoiseGeneration,
    /// Wave function collapse (constraint propagation).
    WaveFunctionCollapse,
    /// Physics tick (N-body, collision broadphase).
    PhysicsTick,
    /// Raycasting (screen-width ray batch).
    Raycasting,
    /// Engagement metric batch evaluation.
    MetricsBatch,
    /// Tufte analysis of UI layout.
    UiAnalysis,
}

/// Dispatch recommendation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Substrate {
    /// CPU (single-threaded reference).
    Cpu,
    /// GPU via barraCuda.
    Gpu,
}

/// Recommend a substrate for a given workload.
///
/// This is capability-based: if GPU is available, large parallel workloads
/// go to GPU. Sequential constraint propagation stays on CPU.
#[must_use]
pub const fn recommend_substrate(workload: GameWorkload, gpu_available: bool) -> Substrate {
    if !gpu_available {
        return Substrate::Cpu;
    }
    match workload {
        GameWorkload::NoiseGeneration | GameWorkload::PhysicsTick | GameWorkload::Raycasting => {
            Substrate::Gpu
        }
        GameWorkload::WaveFunctionCollapse
        | GameWorkload::MetricsBatch
        | GameWorkload::UiAnalysis => Substrate::Cpu,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noise_goes_to_gpu_when_available() {
        assert_eq!(
            recommend_substrate(GameWorkload::NoiseGeneration, true),
            Substrate::Gpu
        );
    }

    #[test]
    fn wfc_stays_on_cpu() {
        assert_eq!(
            recommend_substrate(GameWorkload::WaveFunctionCollapse, true),
            Substrate::Cpu
        );
    }

    #[test]
    fn everything_cpu_without_gpu() {
        assert_eq!(
            recommend_substrate(GameWorkload::NoiseGeneration, false),
            Substrate::Cpu
        );
    }
}

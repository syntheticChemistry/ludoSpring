// SPDX-License-Identifier: AGPL-3.0-or-later

//! Game workload taxonomy and [`GameWorkloadProfile`] presets for dispatch.

use crate::substrate::{Capability, SubstrateKind};

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
    /// Quantized int8 inference (NPC behavior, dialogue, predictions).
    QuantizedInference,
}

/// Workload profile describing required capabilities and substrate preference.
#[derive(Debug, Clone)]
pub struct GameWorkloadProfile {
    /// Profile identifier (e.g. `"noise_generation"`).
    pub name: String,
    /// Capabilities the substrate must provide.
    pub required: Vec<Capability>,
    /// Preferred substrate kind when multiple are capable.
    pub preferred_substrate: Option<SubstrateKind>,
}

impl GameWorkloadProfile {
    /// Noise generation: F32 + shaders, prefers GPU.
    #[must_use]
    pub fn noise_generation() -> Self {
        Self {
            name: "noise_generation".to_string(),
            required: vec![Capability::F32Compute, Capability::ShaderDispatch],
            preferred_substrate: Some(SubstrateKind::Gpu),
        }
    }

    /// Raycasting: F32 + shaders, prefers GPU.
    #[must_use]
    pub fn raycasting() -> Self {
        Self {
            name: "raycasting".to_string(),
            required: vec![Capability::F32Compute, Capability::ShaderDispatch],
            preferred_substrate: Some(SubstrateKind::Gpu),
        }
    }

    /// Physics tick: F32 + shaders, prefers GPU.
    #[must_use]
    pub fn physics_tick() -> Self {
        Self {
            name: "physics_tick".to_string(),
            required: vec![Capability::F32Compute, Capability::ShaderDispatch],
            preferred_substrate: Some(SubstrateKind::Gpu),
        }
    }

    /// WFC step: F32 only, prefers CPU (barrier sync).
    #[must_use]
    pub fn wfc_step() -> Self {
        Self {
            name: "wfc_step".to_string(),
            required: vec![Capability::F32Compute],
            preferred_substrate: Some(SubstrateKind::Cpu),
        }
    }

    /// Metrics batch: F32, prefers CPU.
    #[must_use]
    pub fn metrics_batch() -> Self {
        Self {
            name: "metrics_batch".to_string(),
            required: vec![Capability::F32Compute],
            preferred_substrate: Some(SubstrateKind::Cpu),
        }
    }

    /// UI analysis: F32, prefers CPU (serial layout analysis, no GPU benefit).
    #[must_use]
    pub fn ui_analysis() -> Self {
        Self {
            name: "ui_analysis".to_string(),
            required: vec![Capability::F32Compute],
            preferred_substrate: Some(SubstrateKind::Cpu),
        }
    }

    /// Fraud batch: F32, no preference.
    #[must_use]
    pub fn fraud_batch() -> Self {
        Self {
            name: "fraud_batch".to_string(),
            required: vec![Capability::F32Compute],
            preferred_substrate: None,
        }
    }

    /// Quantized inference: F32 + int8 inference, prefers NPU.
    #[must_use]
    pub fn quantized_inference() -> Self {
        Self {
            name: "quantized_inference".to_string(),
            required: vec![
                Capability::F32Compute,
                Capability::QuantizedInference { bits: 8 },
            ],
            preferred_substrate: Some(SubstrateKind::Npu),
        }
    }
}

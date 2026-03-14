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
//! Springs don't reference each other. `ludoSpring` doesn't import `wetSpring`.
//! Both lean on `ToadStool` independently — `ludoSpring` evolves game/interaction
//! shaders, `wetSpring` evolves bio shaders, and `ToadStool` absorbs both.

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

/// Dispatch recommendation (legacy, for exp031/exp033 compatibility).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Substrate {
    /// CPU (single-threaded reference).
    Cpu,
    /// GPU via barraCuda.
    Gpu,
}

/// Substrate kind for capability-based routing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubstrateKind {
    Cpu,
    Gpu,
    Npu,
}

/// Hardware capability flags for substrate matching.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Capability {
    F64Compute,
    F32Compute,
    ShaderDispatch,
    SimdVector,
    PcieTransfer,
    QuantizedInference { bits: u8 },
}

/// Rich substrate descriptor with capabilities and performance hints.
#[derive(Debug, Clone)]
pub struct SubstrateInfo {
    pub kind: SubstrateKind,
    pub name: String,
    pub capabilities: Vec<Capability>,
    pub flops_gflops: f64,
}

impl SubstrateInfo {
    /// Default CPU substrate (F64, F32, SIMD).
    #[must_use]
    pub fn default_cpu() -> Self {
        Self {
            kind: SubstrateKind::Cpu,
            name: "CPU".to_string(),
            capabilities: vec![
                Capability::F64Compute,
                Capability::F32Compute,
                Capability::SimdVector,
            ],
            flops_gflops: 200.0,
        }
    }

    /// Default GPU substrate (F32, shaders, `PCIe`).
    #[must_use]
    pub fn default_gpu() -> Self {
        Self {
            kind: SubstrateKind::Gpu,
            name: "GPU".to_string(),
            capabilities: vec![
                Capability::F32Compute,
                Capability::ShaderDispatch,
                Capability::PcieTransfer,
            ],
            flops_gflops: 29_000.0,
        }
    }

    /// Default NPU substrate (F32, quantized inference, `PCIe`).
    #[must_use]
    pub fn default_npu() -> Self {
        Self {
            kind: SubstrateKind::Npu,
            name: "NPU".to_string(),
            capabilities: vec![
                Capability::F32Compute,
                Capability::QuantizedInference { bits: 8 },
                Capability::PcieTransfer,
            ],
            flops_gflops: 50.0,
        }
    }
}

/// Workload profile describing required capabilities and substrate preference.
#[derive(Debug, Clone)]
pub struct GameWorkloadProfile {
    pub name: String,
    pub required: Vec<Capability>,
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

    /// UI analysis: F32, no preference.
    #[must_use]
    pub fn ui_analysis() -> Self {
        Self {
            name: "ui_analysis".to_string(),
            required: vec![Capability::F32Compute],
            preferred_substrate: None,
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

/// Routing decision with substrate and reason.
#[derive(Debug, Clone)]
pub struct Decision<'a> {
    pub substrate: &'a SubstrateInfo,
    pub reason: String,
}

/// Route a workload to the best capable substrate.
#[must_use]
pub fn route<'a>(
    workload: &GameWorkloadProfile,
    substrates: &'a [SubstrateInfo],
) -> Option<Decision<'a>> {
    let capable: Vec<&SubstrateInfo> = substrates
        .iter()
        .filter(|s| {
            workload
                .required
                .iter()
                .all(|req| s.capabilities.contains(req))
        })
        .collect();

    if capable.is_empty() {
        return None;
    }

    if let Some(preferred) = workload.preferred_substrate {
        if let Some(s) = capable.iter().find(|s| s.kind == preferred) {
            return Some(Decision {
                substrate: s,
                reason: "preferred substrate".to_string(),
            });
        }
    }

    let priority = [SubstrateKind::Gpu, SubstrateKind::Npu, SubstrateKind::Cpu];
    for kind in &priority {
        if let Some(s) = capable.iter().find(|s| s.kind == *kind) {
            return Some(Decision {
                substrate: s,
                reason: format!("{kind:?} selected by priority"),
            });
        }
    }

    capable.first().map(|s| Decision {
        substrate: s,
        reason: "last resort".to_string(),
    })
}

/// Return substrates in fallback order: GPU > NPU > CPU.
#[must_use]
pub fn fallback_chain(substrates: &[SubstrateInfo]) -> Vec<&SubstrateInfo> {
    let priority = [SubstrateKind::Gpu, SubstrateKind::Npu, SubstrateKind::Cpu];
    let mut result = Vec::new();
    for kind in &priority {
        for s in substrates {
            if s.kind == *kind {
                result.push(s);
            }
        }
    }
    result
}

/// Recommend a substrate for a given workload (legacy API for exp031/exp033).
///
/// Builds synthetic substrates internally and uses capability-based routing.
/// Returns `Substrate::Gpu` only when GPU is selected; NPU maps to `Substrate::Cpu`.
#[must_use]
pub fn recommend_substrate(workload: GameWorkload, gpu_available: bool) -> Substrate {
    let profile = match workload {
        GameWorkload::NoiseGeneration => GameWorkloadProfile::noise_generation(),
        GameWorkload::WaveFunctionCollapse => GameWorkloadProfile::wfc_step(),
        GameWorkload::PhysicsTick => GameWorkloadProfile::physics_tick(),
        GameWorkload::Raycasting => GameWorkloadProfile::raycasting(),
        GameWorkload::MetricsBatch => GameWorkloadProfile::metrics_batch(),
        GameWorkload::UiAnalysis => GameWorkloadProfile::ui_analysis(),
    };
    let mut substrates = vec![SubstrateInfo::default_cpu()];
    if gpu_available {
        substrates.push(SubstrateInfo::default_gpu());
    }
    match route(&profile, &substrates) {
        Some(d) if d.substrate.kind == SubstrateKind::Gpu => Substrate::Gpu,
        _ => Substrate::Cpu,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Legacy tests (backward compat) ---

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

    // --- Capability-based routing tests ---

    #[test]
    fn route_noise_to_gpu() {
        let profile = GameWorkloadProfile::noise_generation();
        let substrates = vec![SubstrateInfo::default_cpu(), SubstrateInfo::default_gpu()];
        let decision = route(&profile, &substrates);
        assert!(decision.is_some());
        let d = decision.unwrap();
        assert_eq!(d.substrate.kind, SubstrateKind::Gpu);
    }

    #[test]
    fn route_wfc_to_cpu() {
        let profile = GameWorkloadProfile::wfc_step();
        let substrates = vec![SubstrateInfo::default_cpu(), SubstrateInfo::default_gpu()];
        let decision = route(&profile, &substrates);
        assert!(decision.is_some());
        let d = decision.unwrap();
        assert_eq!(d.substrate.kind, SubstrateKind::Cpu);
    }

    #[test]
    fn route_npu_inference() {
        let profile = GameWorkloadProfile::quantized_inference();
        let substrates = vec![
            SubstrateInfo::default_cpu(),
            SubstrateInfo::default_gpu(),
            SubstrateInfo::default_npu(),
        ];
        let decision = route(&profile, &substrates);
        assert!(decision.is_some());
        let d = decision.unwrap();
        assert_eq!(d.substrate.kind, SubstrateKind::Npu);
    }

    #[test]
    fn route_fallback_chain_order() {
        let substrates = vec![
            SubstrateInfo::default_cpu(),
            SubstrateInfo::default_gpu(),
            SubstrateInfo::default_npu(),
        ];
        let chain = fallback_chain(&substrates);
        assert_eq!(chain.len(), 3);
        assert_eq!(chain[0].kind, SubstrateKind::Gpu);
        assert_eq!(chain[1].kind, SubstrateKind::Npu);
        assert_eq!(chain[2].kind, SubstrateKind::Cpu);
    }

    #[test]
    fn route_no_capable_substrate() {
        let profile = GameWorkloadProfile::quantized_inference();
        let substrates = vec![SubstrateInfo::default_cpu(), SubstrateInfo::default_gpu()];
        let decision = route(&profile, &substrates);
        assert!(decision.is_none());
    }

    #[test]
    fn route_preferred_honored() {
        let profile = GameWorkloadProfile::noise_generation();
        let substrates = vec![
            SubstrateInfo::default_cpu(),
            SubstrateInfo::default_gpu(),
            SubstrateInfo::default_npu(),
        ];
        let decision = route(&profile, &substrates);
        assert!(decision.is_some());
        let d = decision.unwrap();
        assert_eq!(d.substrate.kind, SubstrateKind::Gpu);
        assert_eq!(d.reason, "preferred substrate");
    }
}

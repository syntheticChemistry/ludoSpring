// SPDX-License-Identifier: AGPL-3.0-or-later

//! Capability-based routing, fallback ordering, and legacy per-workload substrate hints.

use crate::substrate::{SubstrateInfo, SubstrateKind};
use crate::workload::{GameWorkload, GameWorkloadProfile};

/// Dispatch recommendation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Substrate {
    /// CPU (general-purpose, SIMD).
    Cpu,
    /// GPU via barraCuda shader dispatch.
    Gpu,
    /// NPU for quantized inference workloads.
    Npu,
}

/// Routing decision with substrate and reason.
#[derive(Debug, Clone)]
pub struct Decision<'a> {
    /// Selected substrate for the workload.
    pub substrate: &'a SubstrateInfo,
    /// Human-readable selection rationale.
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

// ═══════════════════════════════════════════════════════════════════════════
// Legacy per-workload routing (preserved for backward compat)
// ═══════════════════════════════════════════════════════════════════════════

/// Recommend a substrate for a given workload, with full substrate discovery.
///
/// Builds synthetic substrates from available hardware flags and uses
/// capability-based routing. Maps `SubstrateKind` to `Substrate` 1:1.
#[must_use]
pub fn recommend_substrate(workload: GameWorkload, gpu_available: bool) -> Substrate {
    recommend_substrate_full(workload, gpu_available, false)
}

/// Extended recommendation that includes NPU discovery.
#[must_use]
pub fn recommend_substrate_full(
    workload: GameWorkload,
    gpu_available: bool,
    npu_available: bool,
) -> Substrate {
    let profile = match workload {
        GameWorkload::NoiseGeneration => GameWorkloadProfile::noise_generation(),
        GameWorkload::WaveFunctionCollapse => GameWorkloadProfile::wfc_step(),
        GameWorkload::PhysicsTick => GameWorkloadProfile::physics_tick(),
        GameWorkload::Raycasting => GameWorkloadProfile::raycasting(),
        GameWorkload::MetricsBatch => GameWorkloadProfile::metrics_batch(),
        GameWorkload::UiAnalysis => GameWorkloadProfile::ui_analysis(),
        GameWorkload::QuantizedInference => GameWorkloadProfile::quantized_inference(),
    };
    let mut substrates = vec![SubstrateInfo::default_cpu()];
    if gpu_available {
        substrates.push(SubstrateInfo::default_gpu());
    }
    if npu_available {
        substrates.push(SubstrateInfo::default_npu());
    }
    match route(&profile, &substrates) {
        Some(d) => match d.substrate.kind {
            SubstrateKind::Gpu => Substrate::Gpu,
            SubstrateKind::Npu => Substrate::Npu,
            SubstrateKind::Cpu => Substrate::Cpu,
        },
        None => Substrate::Cpu,
    }
}

// SPDX-License-Identifier: AGPL-3.0-or-later
//! NUCLEUS Atomic Composition — Tower, Node, and Nest atomics coordination.
//!
//! The NUCLEUS model composes primals into three hierarchical tiers:
//!
//! - **Tower** (electron): Specialist domain primals that produce domain signals.
//!   `ludoSpring` validates as the Tower Atomic for game science (bearDog, songbird, skunkBat).
//!
//! - **Node** (proton): Infrastructure primals that route, store, and transform signals.
//!   `toadStool` (compute dispatch), `coralReef` (state persistence), `barraCuda` (math engine).
//!
//! - **Nest** (neutron): Coordination primals that manage atomic composition lifetime.
//!   `biomeOS` (graph coordinator), `loamSpine` (certificate authority).
//!
//! # Mixed Hardware Coordination
//!
//! NUCLEUS atomics coordinate across hardware boundaries:
//! ```text
//! ┌─────────────────────────────────────────────────┐
//! │ biomeOS graph (Nest)                             │
//! │  ├─ NPU Node: toadStool quantized dispatch      │
//! │  │   └─→ PCIe direct ──→ GPU Node               │
//! │  ├─ GPU Node: toadStool shader dispatch          │
//! │  │   └─→ PCIe ──→ CPU Tower readback             │
//! │  ├─ CPU Tower: ludoSpring game logic             │
//! │  │   └─→ signal dispatch to Nest                 │
//! │  └─ CPU Node: coralReef state persistence        │
//! └─────────────────────────────────────────────────┘
//! ```
//!
//! This module validates the composition patterns and signal flow.

use crate::pipeline::BandTarget;
use crate::routing::route;
use crate::substrate::{SubstrateInfo, SubstrateKind};
use crate::workload::GameWorkloadProfile;

/// NUCLEUS atomic tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtomicTier {
    /// Tower (electron): domain specialist, produces signals.
    Tower,
    /// Node (proton): infrastructure, routes and transforms.
    Node,
    /// Nest (neutron): coordinator, manages composition lifetime.
    Nest,
}

/// A NUCLEUS atomic instance in a composition graph.
#[derive(Debug, Clone)]
pub struct Atomic {
    /// Primal name (e.g., "ludoSpring", "toadStool", "biomeOS").
    pub primal: String,
    /// Tier in the composition hierarchy.
    pub tier: AtomicTier,
    /// Hardware substrate this atomic is bound to.
    pub substrate: SubstrateKind,
    /// Capabilities this atomic provides to the composition.
    pub provides: Vec<String>,
    /// Capabilities this atomic requires from other atomics.
    pub requires: Vec<String>,
}

/// A signal flowing between atomics in the composition.
#[derive(Debug, Clone)]
pub struct Signal {
    /// Source atomic primal name.
    pub from: String,
    /// Destination atomic primal name.
    pub to: String,
    /// Signal payload type.
    pub payload: SignalPayload,
    /// Hardware transfer path.
    pub transfer: TransferPath,
}

/// Signal payload classification for routing decisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalPayload {
    /// Game state delta (positions, health, inventories).
    StateDelta,
    /// Compute result (noise field, physics step).
    ComputeResult,
    /// Inference output (NPC behavior, dialogue choice).
    InferenceOutput,
    /// Control signal (start/stop/pause).
    Control,
    /// Render command (draw calls, scene graph update).
    RenderCommand,
}

/// Hardware transfer path for a signal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferPath {
    /// Same hardware, zero-copy (shared memory).
    Local,
    /// PCIe bus transfer (CPU↔GPU, CPU↔NPU).
    Pcie,
    /// Direct device-to-device (NPU→GPU via PCIe P2P, bypassing CPU).
    DirectP2P,
    /// IPC socket (cross-process, used for NeuralAPI signals).
    Ipc,
}

/// Composition graph representing a running NUCLEUS atomic system.
#[derive(Debug, Clone)]
pub struct CompositionGraph {
    /// All atomics in the composition.
    pub atomics: Vec<Atomic>,
    /// All signals flowing between atomics.
    pub signals: Vec<Signal>,
}

/// Validation result for a composition graph.
#[derive(Debug, Clone)]
pub struct CompositionValidation {
    /// Whether the composition is valid.
    pub valid: bool,
    /// Validation errors found.
    pub errors: Vec<String>,
    /// Optimization suggestions.
    pub suggestions: Vec<String>,
    /// Total estimated frame-time contribution from transfers (ms).
    pub transfer_overhead_ms: f64,
}

impl CompositionGraph {
    /// Validate the composition graph for correctness and efficiency.
    #[must_use]
    pub fn validate(&self) -> CompositionValidation {
        let mut errors = Vec::new();
        let mut suggestions = Vec::new();

        self.check_tier_rules(&mut errors);
        self.check_signal_connectivity(&mut errors);
        self.check_hardware_paths(&mut errors, &mut suggestions);

        let transfer_overhead_ms = self.estimate_transfer_overhead();

        CompositionValidation {
            valid: errors.is_empty(),
            errors,
            suggestions,
            transfer_overhead_ms,
        }
    }

    fn check_tier_rules(&self, errors: &mut Vec<String>) {
        for signal in &self.signals {
            let from = self.atomics.iter().find(|a| a.primal == signal.from);
            let to = self.atomics.iter().find(|a| a.primal == signal.to);

            let (Some(from_atomic), Some(to_atomic)) = (from, to) else {
                errors.push(format!(
                    "signal references unknown atomic: {} → {}",
                    signal.from, signal.to
                ));
                continue;
            };

            if from_atomic.tier == AtomicTier::Tower && to_atomic.tier == AtomicTier::Tower {
                errors.push(format!(
                    "Tower→Tower signal forbidden (springs don't reference each other): {} → {}",
                    signal.from, signal.to
                ));
            }
        }
    }

    fn check_signal_connectivity(&self, errors: &mut Vec<String>) {
        let has_nest = self.atomics.iter().any(|a| a.tier == AtomicTier::Nest);
        if !has_nest {
            errors.push("composition requires at least one Nest atomic (coordinator)".into());
        }

        for atomic in &self.atomics {
            if atomic.tier == AtomicTier::Tower {
                let has_signal_to_node_or_nest = self.signals.iter().any(|s| {
                    s.from == atomic.primal
                        && self
                            .atomics
                            .iter()
                            .any(|a| a.primal == s.to && a.tier != AtomicTier::Tower)
                });
                if !has_signal_to_node_or_nest {
                    errors.push(format!(
                        "Tower atomic '{}' has no outgoing signal to Node/Nest",
                        atomic.primal
                    ));
                }
            }
        }
    }

    #[allow(
        clippy::match_same_arms,
        reason = "exhaustive transfer-path validation table"
    )]
    fn check_hardware_paths(&self, errors: &mut Vec<String>, suggestions: &mut Vec<String>) {
        for signal in &self.signals {
            let from = self.atomics.iter().find(|a| a.primal == signal.from);
            let to = self.atomics.iter().find(|a| a.primal == signal.to);

            let (Some(from_atomic), Some(to_atomic)) = (from, to) else {
                continue;
            };

            match (from_atomic.substrate, to_atomic.substrate, signal.transfer) {
                (SubstrateKind::Cpu, SubstrateKind::Cpu, TransferPath::Local) => {}
                (SubstrateKind::Gpu, SubstrateKind::Gpu, TransferPath::Local) => {}
                (SubstrateKind::Npu, SubstrateKind::Npu, TransferPath::Local) => {}
                (SubstrateKind::Cpu, SubstrateKind::Gpu, TransferPath::Pcie) => {}
                (SubstrateKind::Gpu, SubstrateKind::Cpu, TransferPath::Pcie) => {}
                (SubstrateKind::Cpu, SubstrateKind::Npu, TransferPath::Pcie) => {}
                (SubstrateKind::Npu, SubstrateKind::Cpu, TransferPath::Pcie) => {}
                (SubstrateKind::Npu, SubstrateKind::Gpu, TransferPath::Pcie) => {}
                (SubstrateKind::Gpu, SubstrateKind::Npu, TransferPath::Pcie) => {}
                (SubstrateKind::Npu, SubstrateKind::Gpu, TransferPath::DirectP2P) => {}
                (SubstrateKind::Gpu, SubstrateKind::Npu, TransferPath::DirectP2P) => {}
                (_, _, TransferPath::Ipc) => {}
                (from_sub, to_sub, path) => {
                    errors.push(format!(
                        "invalid transfer path {path:?} between {from_sub:?} and {to_sub:?} ({} → {})",
                        signal.from, signal.to
                    ));
                }
            }

            if from_atomic.substrate == SubstrateKind::Npu
                && to_atomic.substrate == SubstrateKind::Gpu
                && signal.transfer == TransferPath::Pcie
            {
                suggestions.push(format!(
                    "NPU→GPU signal '{}→{}' using PCIe could use DirectP2P to bypass CPU roundtrip",
                    signal.from, signal.to
                ));
            }
        }
    }

    #[allow(clippy::cast_precision_loss, reason = "signal count won't exceed 2^52")]
    fn estimate_transfer_overhead(&self) -> f64 {
        let mut total_ms = 0.0;
        for signal in &self.signals {
            let cost = match signal.transfer {
                TransferPath::Local => 0.0,
                TransferPath::Pcie => 0.065,
                TransferPath::DirectP2P => 0.04,
                TransferPath::Ipc => 0.15,
            };
            total_ms += cost;
        }
        total_ms
    }
}

/// Build the canonical ludoSpring mixed-hardware composition.
///
/// This represents the target deployment topology:
/// - ludoSpring (Tower, CPU): game logic, metrics, MDA
/// - barraCuda (Node, CPU/GPU): math engine, parity-validated
/// - toadStool (Node, GPU): compute dispatch, shader execution
/// - toadStool-npu (Node, NPU): quantized inference dispatch
/// - coralReef (Node, CPU): state persistence, event log
/// - biomeOS (Nest, CPU): graph coordination, signal routing
#[must_use]
pub fn canonical_composition() -> CompositionGraph {
    let atomics = vec![
        Atomic {
            primal: "ludoSpring".into(),
            tier: AtomicTier::Tower,
            substrate: SubstrateKind::Cpu,
            provides: vec!["game_logic".into(), "metrics".into(), "mda_analysis".into()],
            requires: vec!["compute_dispatch".into(), "state_persistence".into()],
        },
        Atomic {
            primal: "barraCuda".into(),
            tier: AtomicTier::Node,
            substrate: SubstrateKind::Cpu,
            provides: vec!["math_engine".into(), "noise_generation".into()],
            requires: vec!["compute_dispatch".into()],
        },
        Atomic {
            primal: "toadStool-gpu".into(),
            tier: AtomicTier::Node,
            substrate: SubstrateKind::Gpu,
            provides: vec!["compute_dispatch".into(), "shader_execution".into()],
            requires: vec![],
        },
        Atomic {
            primal: "toadStool-npu".into(),
            tier: AtomicTier::Node,
            substrate: SubstrateKind::Npu,
            provides: vec!["quantized_inference".into(), "npc_prediction".into()],
            requires: vec![],
        },
        Atomic {
            primal: "coralReef".into(),
            tier: AtomicTier::Node,
            substrate: SubstrateKind::Cpu,
            provides: vec!["state_persistence".into(), "event_log".into()],
            requires: vec![],
        },
        Atomic {
            primal: "biomeOS".into(),
            tier: AtomicTier::Nest,
            substrate: SubstrateKind::Cpu,
            provides: vec!["graph_coordination".into(), "signal_routing".into()],
            requires: vec![],
        },
    ];

    let signals = vec![
        Signal {
            from: "ludoSpring".into(),
            to: "biomeOS".into(),
            payload: SignalPayload::StateDelta,
            transfer: TransferPath::Ipc,
        },
        Signal {
            from: "ludoSpring".into(),
            to: "barraCuda".into(),
            payload: SignalPayload::Control,
            transfer: TransferPath::Local,
        },
        Signal {
            from: "barraCuda".into(),
            to: "toadStool-gpu".into(),
            payload: SignalPayload::ComputeResult,
            transfer: TransferPath::Pcie,
        },
        Signal {
            from: "toadStool-npu".into(),
            to: "toadStool-gpu".into(),
            payload: SignalPayload::InferenceOutput,
            transfer: TransferPath::DirectP2P,
        },
        Signal {
            from: "toadStool-gpu".into(),
            to: "ludoSpring".into(),
            payload: SignalPayload::ComputeResult,
            transfer: TransferPath::Pcie,
        },
        Signal {
            from: "ludoSpring".into(),
            to: "coralReef".into(),
            payload: SignalPayload::StateDelta,
            transfer: TransferPath::Local,
        },
        Signal {
            from: "biomeOS".into(),
            to: "toadStool-npu".into(),
            payload: SignalPayload::Control,
            transfer: TransferPath::Pcie,
        },
    ];

    CompositionGraph { atomics, signals }
}

/// Route workloads through the composition graph and verify band allocation.
#[must_use]
pub fn validate_workload_routing() -> Vec<(String, BandTarget)> {
    let workloads = vec![
        GameWorkloadProfile::noise_generation(),
        GameWorkloadProfile::physics_tick(),
        GameWorkloadProfile::wfc_step(),
        GameWorkloadProfile::quantized_inference(),
        GameWorkloadProfile::ui_analysis(),
    ];
    let substrates = vec![
        SubstrateInfo::default_cpu(),
        SubstrateInfo::default_gpu(),
        SubstrateInfo::default_npu(),
    ];

    let mut routing_decisions = Vec::new();
    for workload in &workloads {
        if let Some(decision) = route(workload, &substrates) {
            let band = match decision.substrate.kind {
                SubstrateKind::Cpu => BandTarget::Cpu,
                SubstrateKind::Gpu => BandTarget::GpuCompute,
                SubstrateKind::Npu => BandTarget::NpuCompute,
            };
            routing_decisions.push((workload.name.clone(), band));
        }
    }
    routing_decisions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_composition_valid() {
        let graph = canonical_composition();
        let result = graph.validate();
        assert!(result.valid, "errors: {:?}", result.errors);
    }

    #[test]
    fn canonical_has_all_tiers() {
        let graph = canonical_composition();
        assert!(graph.atomics.iter().any(|a| a.tier == AtomicTier::Tower));
        assert!(graph.atomics.iter().any(|a| a.tier == AtomicTier::Node));
        assert!(graph.atomics.iter().any(|a| a.tier == AtomicTier::Nest));
    }

    #[test]
    fn canonical_uses_direct_p2p_for_npu_gpu() {
        let graph = canonical_composition();
        let npu_gpu_signal = graph
            .signals
            .iter()
            .find(|s| s.from == "toadStool-npu" && s.to == "toadStool-gpu");
        assert!(npu_gpu_signal.is_some());
        assert_eq!(
            npu_gpu_signal.map(|s| s.transfer),
            Some(TransferPath::DirectP2P)
        );
    }

    #[test]
    fn tower_to_tower_forbidden() {
        let mut graph = canonical_composition();
        graph.atomics.push(Atomic {
            primal: "wetSpring".into(),
            tier: AtomicTier::Tower,
            substrate: SubstrateKind::Cpu,
            provides: vec!["bio_sim".into()],
            requires: vec![],
        });
        graph.signals.push(Signal {
            from: "ludoSpring".into(),
            to: "wetSpring".into(),
            payload: SignalPayload::Control,
            transfer: TransferPath::Local,
        });

        let result = graph.validate();
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("Tower→Tower")));
    }

    #[test]
    fn invalid_transfer_path_detected() {
        let mut graph = canonical_composition();
        graph.signals.push(Signal {
            from: "ludoSpring".into(),
            to: "toadStool-gpu".into(),
            payload: SignalPayload::Control,
            transfer: TransferPath::DirectP2P,
        });

        let result = graph.validate();
        assert!(!result.valid);
        assert!(
            result
                .errors
                .iter()
                .any(|e| e.contains("invalid transfer path"))
        );
    }

    #[test]
    fn pcie_suggestion_for_npu_gpu() {
        let mut graph = canonical_composition();
        let npu_gpu_idx = graph
            .signals
            .iter()
            .position(|s| s.from == "toadStool-npu" && s.to == "toadStool-gpu");
        if let Some(idx) = npu_gpu_idx {
            graph.signals[idx].transfer = TransferPath::Pcie;
        }

        let result = graph.validate();
        assert!(result.valid, "PCIe is valid, just suboptimal");
        assert!(
            result.suggestions.iter().any(|s| s.contains("DirectP2P")),
            "should suggest DirectP2P"
        );
    }

    #[test]
    fn transfer_overhead_positive() {
        let graph = canonical_composition();
        let result = graph.validate();
        assert!(result.transfer_overhead_ms > 0.0);
    }

    #[test]
    fn workload_routing_covers_all_substrates() {
        let routes = validate_workload_routing();
        assert!(routes.iter().any(|(_, b)| *b == BandTarget::Cpu));
        assert!(routes.iter().any(|(_, b)| *b == BandTarget::GpuCompute));
        assert!(routes.iter().any(|(_, b)| *b == BandTarget::NpuCompute));
    }

    #[test]
    fn no_nest_makes_composition_invalid() {
        let graph = CompositionGraph {
            atomics: vec![
                Atomic {
                    primal: "ludoSpring".into(),
                    tier: AtomicTier::Tower,
                    substrate: SubstrateKind::Cpu,
                    provides: vec!["game_logic".into()],
                    requires: vec![],
                },
                Atomic {
                    primal: "toadStool".into(),
                    tier: AtomicTier::Node,
                    substrate: SubstrateKind::Gpu,
                    provides: vec!["compute".into()],
                    requires: vec![],
                },
            ],
            signals: vec![Signal {
                from: "ludoSpring".into(),
                to: "toadStool".into(),
                payload: SignalPayload::Control,
                transfer: TransferPath::Pcie,
            }],
        };
        let result = graph.validate();
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("Nest atomic")));
    }
}

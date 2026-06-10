// SPDX-License-Identifier: AGPL-3.0-or-later
//! Health, readiness, and capability discovery handlers.

use crate::ipc::envelope::JsonRpcRequest;

use super::{HandlerResult, to_json};

pub(super) fn handle_health(req: &JsonRpcRequest) -> HandlerResult {
    to_json(
        &req.id,
        serde_json::json!({
            "status": "healthy",
            "name": crate::PRIMAL_NAME,
            "primal": crate::PRIMAL_NAME,
            "domain": crate::niche::NICHE_DOMAIN,
            "version": env!("CARGO_PKG_VERSION"),
            "capabilities": crate::niche::CAPABILITIES,
        }),
    )
}

/// `health.liveness` — Kubernetes-style liveness probe (coralReef Iter 51).
///
/// Returns immediately if the process is responsive. No external deps checked.
/// Response format per `SEMANTIC_METHOD_NAMING_STANDARD` v2.1:
/// `{"status": "alive"}`.
pub(super) fn handle_liveness(req: &JsonRpcRequest) -> HandlerResult {
    to_json(&req.id, serde_json::json!({"status": "alive"}))
}

/// `health.readiness` — Kubernetes-style readiness probe (healthSpring V32).
///
/// Reports whether subsystems are ready to accept science workloads.
pub(super) fn handle_readiness(req: &JsonRpcRequest) -> HandlerResult {
    let trio_available = crate::ipc::provenance::has_active_session();

    to_json(
        &req.id,
        serde_json::json!({
            "ready": true,
            "subsystems": {
                "science_dispatch": true,
                "provenance_trio": trio_available,
                "gpu_compute": cfg!(feature = "gpu"),
            }
        }),
    )
}

/// `health.version` — returns primal identity and build metadata.
///
/// Per barraCuda Sprint 69 trio consistency pattern: name, version, build target.
pub(super) fn handle_version(req: &JsonRpcRequest) -> HandlerResult {
    to_json(
        &req.id,
        serde_json::json!({
            "name": crate::PRIMAL_NAME,
            "version": env!("CARGO_PKG_VERSION"),
            "rust_version": env!("CARGO_PKG_RUST_VERSION"),
            "target": option_env!("TARGET").unwrap_or("unknown"),
        }),
    )
}

/// `health.drain` — graceful drain request.
///
/// Signals that this primal should stop accepting new work and finish in-flight
/// operations. For ludoSpring (validation-only, no persistent server), this is
/// a no-op acknowledgment.
pub(super) fn handle_drain(req: &JsonRpcRequest) -> HandlerResult {
    to_json(
        &req.id,
        serde_json::json!({
            "draining": true,
            "in_flight": 0,
        }),
    )
}

/// Signal acknowledgment — composition-level collapse signals (Wave 20).
///
/// biomeOS orchestrates the multi-step pipeline (e.g., `nest.commit` expands to
/// `event.append` → `crypto.sign` → `content.put` → `session.commit` → `braid.create`).
/// ludoSpring acknowledges receipt for observability and composition validation.
/// The actual work is delegated to the composed primals via the cell graph.
pub(super) fn handle_signal_ack(req: &JsonRpcRequest) -> HandlerResult {
    to_json(
        &req.id,
        serde_json::json!({
            "acknowledged": true,
            "signal": req.method,
            "primal": crate::niche::NICHE_NAME,
            "dispatch": "composition",
        }),
    )
}

/// `lifecycle.status` — discovery probe response (per Universal IPC Standard V3).
///
/// Returns `name`, `version`, `domain`, `capabilities`, and `status` so that
/// `probe_socket()` in the discovery module can identify this primal by capability.
pub(super) fn handle_lifecycle_status(req: &JsonRpcRequest) -> HandlerResult {
    to_json(
        &req.id,
        serde_json::json!({
            "name": crate::PRIMAL_NAME,
            "version": env!("CARGO_PKG_VERSION"),
            "domain": crate::niche::NICHE_DOMAIN,
            "status": "running",
            "capabilities": crate::niche::CAPABILITIES,
        }),
    )
}

pub(super) fn handle_capability_list(req: &JsonRpcRequest) -> HandlerResult {
    let mut response = crate::capability_domains::capability_list_response();
    response["operation_dependencies"] = crate::niche::operation_dependencies();
    response["cost_estimates"] = crate::niche::cost_estimates();
    to_json(&req.id, response)
}

/// `method.describe` — runtime method introspection.
///
/// Returns structured metadata for all available methods: name, domain,
/// required parameters, return type, and tier. Enables consumers
/// (esotericWebb, other springs) to discover game science capabilities
/// programmatically without prior knowledge of our method surface.
pub(super) fn handle_method_describe(req: &JsonRpcRequest) -> HandlerResult {
    to_json(&req.id, method_describe_response())
}

fn method_describe_response() -> serde_json::Value {
    serde_json::json!({
        "primal": crate::PRIMAL_NAME,
        "version": env!("CARGO_PKG_VERSION"),
        "method_count": METHOD_DESCRIPTORS.len(),
        "methods": METHOD_DESCRIPTORS.iter().map(|m| serde_json::json!({
            "name": m.name,
            "domain": m.domain,
            "tier": m.tier,
            "params": m.params,
            "returns": m.returns,
            "description": m.description,
        })).collect::<Vec<_>>(),
    })
}

struct MethodDescriptor {
    name: &'static str,
    domain: &'static str,
    tier: &'static str,
    params: &'static [&'static str],
    returns: &'static str,
    description: &'static str,
}

const METHOD_DESCRIPTORS: &[MethodDescriptor] = &[
    MethodDescriptor {
        name: "game.evaluate_flow",
        domain: "science",
        tier: "3-domain",
        params: &["challenge: f64", "skill: f64", "channel_width?: f64"],
        returns: "{ state: string, intensity: f64 }",
        description: "Csikszentmihalyi flow state evaluation given challenge/skill balance",
    },
    MethodDescriptor {
        name: "game.fitts_cost",
        domain: "science",
        tier: "3-domain",
        params: &[
            "distance: f64",
            "target_width: f64",
            "a?: f64",
            "b?: f64",
            "method?: string",
            "n?: u32",
        ],
        returns: "{ movement_time_ms: f64, index_of_difficulty: f64 }",
        description: "Fitts/Hick/Steering law cost prediction for interaction targets",
    },
    MethodDescriptor {
        name: "game.engagement",
        domain: "science",
        tier: "3-domain",
        params: &[
            "session_duration_s: f64",
            "action_count: u64",
            "exploration_breadth: u32",
            "challenge_seeking: u32",
            "retry_count: u32",
            "deliberate_pauses: u32",
        ],
        returns: "{ actions_per_minute: f64, exploration_rate: f64, composite_score: f64 }",
        description: "Composite engagement metrics from session telemetry",
    },
    MethodDescriptor {
        name: "game.generate_noise",
        domain: "science",
        tier: "3-domain",
        params: &[
            "x: f64",
            "y: f64",
            "z?: f64",
            "octaves?: u32",
            "lacunarity?: f64",
            "persistence?: f64",
        ],
        returns: "{ value: f64 }",
        description: "Perlin noise generation (2D/3D) with optional fBm octaves",
    },
    MethodDescriptor {
        name: "game.analyze_ui",
        domain: "science",
        tier: "3-domain",
        params: &["elements: [{ name, bounds, data_values, pixel_area, data_ink_area, critical }]"],
        returns: "{ data_ink_ratio: f64, density_score: f64, coverage: f64 }",
        description: "Tufte-inspired UI analysis: data-ink ratio, information density, spatial coverage",
    },
    MethodDescriptor {
        name: "game.accessibility",
        domain: "science",
        tier: "3-domain",
        params: &[
            "audio_cues: bool",
            "descriptions: bool",
            "braille: bool",
            "haptic: bool",
            "color_independent: bool",
            "scalable_text: bool",
        ],
        returns: "{ dimension: string, score: f64, issues: [string], strengths: [string] }",
        description: "Visual accessibility scoring per IGDA/XAG guidelines",
    },
    MethodDescriptor {
        name: "game.wfc_step",
        domain: "science",
        tier: "3-domain",
        params: &[
            "width: usize",
            "height: usize",
            "n_tiles: usize",
            "collapse?: [x, y, tile_id]",
        ],
        returns: "{ collapsed: bool, grid_state: string }",
        description: "Wave Function Collapse constraint propagation step",
    },
    MethodDescriptor {
        name: "game.difficulty_adjustment",
        domain: "science",
        tier: "3-domain",
        params: &["outcomes: [f64]", "target_success_rate?: f64"],
        returns: "{ adjustment: f64, direction: string }",
        description: "Dynamic difficulty adjustment from recent performance window",
    },
];

/// `lifecycle.composition` — runtime composition report.
///
/// Probes all proto-nucleate dependencies and returns a structured
/// report of live/absent status per primal. This is the composition
/// validation step: Python validated Rust, now Rust validates primal
/// composition patterns via IPC liveness.
pub(super) fn handle_composition(req: &JsonRpcRequest) -> HandlerResult {
    let report = crate::ipc::composition::composition_json()
        .map_err(|e| crate::ipc::envelope::JsonRpcError::internal(&req.id, &e))?;
    to_json(&req.id, report)
}

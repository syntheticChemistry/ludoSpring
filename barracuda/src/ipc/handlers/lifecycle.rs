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

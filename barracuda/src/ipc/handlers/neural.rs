// SPDX-License-Identifier: AGPL-3.0-or-later
//! Neural API shim and visualization **delegation** handlers for IPC tests and local development.
//!
//! When ludoSpring's [`super::dispatch`] is used as a stand-in for biomeOS's Neural
//! API (integration tests, single-process demos), lifecycle and capability methods mirror
//! the JSON-RPC surface that [`crate::ipc::NeuralBridge`] expects. Visualization-related
//! methods discover a peer that advertises `visualization.render` and forward work to
//! petalTongue via [`crate::visualization::VisualizationPushClient`]; when no peer is
//! available (or the `ipc` feature is off), handlers return structured **degraded**
//! payloads instead of silent acceptance.

use serde_json::json;

use crate::ipc::envelope::{JsonRpcError, JsonRpcRequest, RpcErrorBody};

use super::{HandlerResult, dispatch, parse_params, to_json};

/// `lifecycle.register` — accept registration payloads from [`crate::ipc::NeuralBridge`].
pub(super) fn handle_lifecycle_register(req: &JsonRpcRequest) -> HandlerResult {
    let _params: serde_json::Value = parse_params(req)?;
    to_json(
        &req.id,
        json!({
            "status": "registered",
            "domain": crate::niche::NICHE_DOMAIN,
            "name": crate::niche::NICHE_NAME,
        }),
    )
}

/// `capability.deregister` — accept deregistration from [`crate::ipc::NeuralBridge`].
pub(super) fn handle_capability_deregister(req: &JsonRpcRequest) -> HandlerResult {
    let _params: serde_json::Value = parse_params(req)?;
    to_json(
        &req.id,
        json!({
            "status": "deregistered",
            "domain": crate::niche::NICHE_DOMAIN,
        }),
    )
}

/// `capability.discover` — return a minimal provider list for discovery probes.
pub(super) fn handle_capability_discover(req: &JsonRpcRequest) -> HandlerResult {
    let _params: serde_json::Value = parse_params(req)?;
    to_json(
        &req.id,
        json!({
            "providers": [
                {
                    "name": crate::niche::NICHE_NAME,
                    "domain": crate::niche::NICHE_DOMAIN,
                }
            ]
        }),
    )
}

/// `capability.call` — route to an inner method via the same [`dispatch`] pipeline.
pub(super) fn handle_capability_call(req: &JsonRpcRequest) -> HandlerResult {
    let params: serde_json::Value = parse_params(req)?;
    let capability = params
        .get("capability")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| JsonRpcError::invalid_params(&req.id, "missing capability"))?;
    let operation = params
        .get("operation")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| JsonRpcError::invalid_params(&req.id, "missing operation"))?;
    let args = params.get("args").cloned().unwrap_or_else(|| json!({}));

    let method = if capability.contains('.') {
        capability.to_string()
    } else {
        format!("{capability}.{operation}")
    };

    if method == "capability.call" {
        return Err(JsonRpcError::invalid_params(
            &req.id,
            "recursive capability.call",
        ));
    }

    let inner = JsonRpcRequest {
        jsonrpc: "2.0".to_string(),
        method,
        params: Some(args),
        id: req.id.clone(),
    };

    let response_str = dispatch(&inner);
    let response: serde_json::Value = serde_json::from_str(&response_str)
        .map_err(|e| JsonRpcError::internal(&req.id, &format!("nested dispatch parse: {e}")))?;

    if let Some(result) = response.get("result") {
        return Ok(result.clone());
    }

    if let Some(err) = response.get("error") {
        let code_i64 = err
            .get("code")
            .and_then(serde_json::Value::as_i64)
            .unwrap_or(-32603);
        let code = i32::try_from(code_i64).unwrap_or(-32603);
        let message = err
            .get("message")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("nested rpc error")
            .to_string();
        return Err(JsonRpcError {
            jsonrpc: "2.0",
            error: RpcErrorBody { code, message },
            id: req.id.clone(),
        });
    }

    Err(JsonRpcError::internal(
        &req.id,
        "nested dispatch response missing result and error",
    ))
}

fn map_ipc_to_json_rpc(id: &serde_json::Value, err: crate::ipc::IpcError) -> JsonRpcError {
    match err {
        crate::ipc::IpcError::RpcError { code, message } => JsonRpcError {
            jsonrpc: "2.0",
            error: RpcErrorBody {
                code: i32::try_from(code).unwrap_or(-32603),
                message,
            },
            id: id.clone(),
        },
        other => JsonRpcError::internal(id, &other.to_string()),
    }
}

fn viz_delegated_ok(req: &JsonRpcRequest) -> HandlerResult {
    to_json(
        &req.id,
        json!({
            "delegated": true,
            "degraded": false,
            "accepted": true,
            "method": req.method,
            "domain": crate::niche::NICHE_DOMAIN,
        }),
    )
}

fn viz_degraded_peer(req: &JsonRpcRequest, reason: &str) -> HandlerResult {
    to_json(
        &req.id,
        json!({
            "delegated": false,
            "degraded": true,
            "accepted": false,
            "reason": reason,
            "method": req.method,
            "domain": crate::niche::NICHE_DOMAIN,
        }),
    )
}

/// petalTongue-style visualization JSON-RPC: capability discovery + [`crate::visualization::VisualizationPushClient`], or explicit degraded results.
pub(super) fn handle_visualization_delegation(req: &JsonRpcRequest) -> HandlerResult {
    let params = req.params.clone().unwrap_or_else(|| json!({}));
    let domain = crate::niche::NICHE_DOMAIN;
    let method = req.method.as_str();

    #[cfg(not(feature = "ipc"))]
    {
        return match method {
            "visualization.export" => to_json(
                &req.id,
                json!({
                    "ipc_delegated": false,
                    "degraded": true,
                    "status": "not_forwarded",
                    "note": "Visualization export is IPC-delegated to a visualization primal; the ipc feature is disabled in this build, so nothing was queued.",
                    "method": method,
                    "domain": domain,
                }),
            ),
            "visualization.validate" => to_json(
                &req.id,
                json!({
                    "degraded": true,
                    "visualization_peer": "unavailable",
                    "domain": domain,
                    "bindings_echo": params.get("bindings").cloned().unwrap_or_else(|| json!({})),
                    "state": {
                        "composition": "unknown",
                        "tufte_preflight": "skipped_build_without_ipc",
                    },
                }),
            ),
            "interaction.subscribe" => to_json(
                &req.id,
                json!({
                    "acknowledged": true,
                    "subscribed": false,
                    "degraded": true,
                    "detail": "interaction.subscribe is IPC-delegated; the ipc feature is disabled in this build.",
                    "session_id": params.get("session_id").cloned().unwrap_or(json!(null)),
                    "domain": domain,
                }),
            ),
            _ => viz_degraded_peer(
                req,
                "ipc feature disabled; visualization delegation unavailable",
            ),
        };
    }

    #[cfg(feature = "ipc")]
    {
        use crate::visualization::VisualizationPushClient;

        let client = match VisualizationPushClient::discover() {
            Ok(c) => c,
            Err(_) => {
                return match method {
                    "visualization.export" => to_json(
                        &req.id,
                        json!({
                            "ipc_delegated": false,
                            "degraded": true,
                            "status": "not_forwarded",
                            "note": "Export requests are normally queued to the visualization primal over IPC; no peer was discovered, so nothing was forwarded. ludoSpring does not write export files locally.",
                            "method": method,
                            "domain": domain,
                        }),
                    ),
                    "visualization.validate" => to_json(
                        &req.id,
                        json!({
                            "degraded": true,
                            "visualization_peer": "unavailable",
                            "domain": domain,
                            "bindings_echo": params.get("bindings").cloned().unwrap_or_else(|| json!({})),
                            "state": {
                                "composition": "unknown",
                                "tufte_preflight": "skipped_peer_unavailable",
                            },
                        }),
                    ),
                    "interaction.subscribe" => to_json(
                        &req.id,
                        json!({
                            "acknowledged": true,
                            "subscribed": false,
                            "degraded": true,
                            "detail": "Subscription is acknowledged locally but not established: no visualization primal was discovered.",
                            "session_id": params.get("session_id").cloned().unwrap_or(json!(null)),
                            "domain": domain,
                        }),
                    ),
                    _ => viz_degraded_peer(req, "no visualization-capable primal found"),
                };
            }
        };

        match method {
            "visualization.render" => {
                let session_id = params
                    .get("session_id")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("");
                let title = params
                    .get("title")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("Untitled");
                let data = params.get("data").cloned().unwrap_or_else(|| json!({}));
                client
                    .push_render(session_id, title, &data)
                    .map_err(|e| map_ipc_to_json_rpc(&req.id, e))?;
                viz_delegated_ok(req)
            }
            "visualization.render.stream" => {
                let session_id = params
                    .get("session_id")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("");
                let action = params
                    .get("action")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("append");
                let data = params.get("data").cloned().unwrap_or_else(|| json!({}));
                client
                    .push_stream(session_id, action, &data)
                    .map_err(|e| map_ipc_to_json_rpc(&req.id, e))?;
                viz_delegated_ok(req)
            }
            "visualization.render.scene" => {
                let session_id = params
                    .get("session_id")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("");
                let channel = params
                    .get("channel")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("default");
                let scene = params.get("scene").cloned().unwrap_or_else(|| json!({}));
                client
                    .push_scene(session_id, channel, &scene)
                    .map_err(|e| map_ipc_to_json_rpc(&req.id, e))?;
                viz_delegated_ok(req)
            }
            "visualization.render.dashboard" => {
                let session_id = params
                    .get("session_id")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("");
                let panels: Vec<serde_json::Value> = params
                    .get("panels")
                    .and_then(|p| p.as_array())
                    .map(|v| v.clone())
                    .unwrap_or_default();
                client
                    .push_dashboard(session_id, &panels)
                    .map_err(|e| map_ipc_to_json_rpc(&req.id, e))?;
                viz_delegated_ok(req)
            }
            "visualization.export" => {
                let session_id = params
                    .get("session_id")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("");
                let modality = params
                    .get("modality")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("svg");
                let peer_result = client
                    .export(session_id, modality)
                    .map_err(|e| map_ipc_to_json_rpc(&req.id, e))?;
                to_json(
                    &req.id,
                    json!({
                        "ipc_delegated": true,
                        "degraded": false,
                        "status": "forwarded_to_peer",
                        "note": "Export request was sent to the visualization primal over IPC. ludoSpring does not materialize export files locally; the peer handles queuing or generation.",
                        "session_id": session_id,
                        "modality": modality,
                        "peer": peer_result,
                        "domain": domain,
                    }),
                )
            }
            "visualization.validate" => {
                let bindings = params.get("bindings").cloned().unwrap_or_else(|| json!({}));
                let peer_validation = client
                    .validate(&bindings)
                    .map_err(|e| map_ipc_to_json_rpc(&req.id, e))?;
                to_json(
                    &req.id,
                    json!({
                        "delegated": true,
                        "degraded": false,
                        "domain": domain,
                        "visualization_peer": "available",
                        "peer_validation": peer_validation,
                    }),
                )
            }
            "interaction.subscribe" => {
                let session_id = params
                    .get("session_id")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("");
                let peer = client
                    .subscribe_interaction(session_id)
                    .map_err(|e| map_ipc_to_json_rpc(&req.id, e))?;
                to_json(
                    &req.id,
                    json!({
                        "acknowledged": true,
                        "subscribed": true,
                        "degraded": false,
                        "delegated": true,
                        "session_id": session_id,
                        "domain": domain,
                        "peer": peer,
                    }),
                )
            }
            _ => Err(JsonRpcError::internal(
                &req.id,
                "visualization dispatch: unexpected method",
            )),
        }
    }
}

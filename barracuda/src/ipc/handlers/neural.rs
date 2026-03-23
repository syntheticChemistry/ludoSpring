// SPDX-License-Identifier: AGPL-3.0-or-later
//! Neural API shim and visualization stubs for IPC tests and local development.
//!
//! When ludoSpring's [`super::dispatch`] is used as a stand-in for biomeOS's Neural
//! API (integration tests, single-process demos), these methods mirror the JSON-RPC
//! surface that [`crate::ipc::NeuralBridge`] and [`crate::visualization::VisualizationPushClient`]
//! expect.

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

/// Stub responses for petalTongue-style visualization JSON-RPC methods.
pub(super) fn handle_visualization_stub(req: &JsonRpcRequest) -> HandlerResult {
    to_json(
        &req.id,
        json!({
            "accepted": true,
            "method": req.method,
            "domain": crate::niche::NICHE_DOMAIN,
        }),
    )
}

// SPDX-License-Identifier: AGPL-3.0-or-later
//! JSON-RPC 2.0 protocol envelope types.
//!
//! These are protocol-level types, independent of any specific method.
//! They follow the JSON-RPC 2.0 specification (2010-03-26).

use serde::{Deserialize, Serialize};

/// JSON-RPC 2.0 request.
#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    /// Must be "2.0".
    pub jsonrpc: String,
    /// Method name (capability identifier).
    pub method: String,
    /// Method parameters.
    pub params: Option<serde_json::Value>,
    /// Request identifier (echoed in response).
    pub id: serde_json::Value,
}

/// JSON-RPC 2.0 success response.
#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    /// Always "2.0".
    pub jsonrpc: &'static str,
    /// Result payload.
    pub result: serde_json::Value,
    /// Echoed request identifier.
    pub id: serde_json::Value,
}

/// JSON-RPC 2.0 error response.
#[derive(Debug, Serialize)]
pub struct JsonRpcError {
    /// Always "2.0".
    pub jsonrpc: &'static str,
    /// Error object.
    pub error: RpcErrorBody,
    /// Echoed request identifier.
    pub id: serde_json::Value,
}

/// Error body inside a JSON-RPC error response.
#[derive(Debug, Serialize)]
pub struct RpcErrorBody {
    /// Error code (standard: -32600..-32603, application: -32000..-32099).
    pub code: i32,
    /// Human-readable message.
    pub message: String,
}

impl JsonRpcResponse {
    /// Construct a success response. Clones `id` from the request.
    #[must_use]
    pub fn ok(id: &serde_json::Value, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0",
            result,
            id: id.clone(),
        }
    }
}

/// Extract a human-readable error string from a raw JSON-RPC response object.
///
/// Handles the common pattern where callers receive a `serde_json::Value`
/// and need to check for an `"error"` field. Returns `Ok(result)` on success
/// or `Err(message)` if the response contains an error or no result.
///
/// Follows the healthSpring V29 `extract_rpc_error()` centralization pattern.
pub fn extract_rpc_result(response: &serde_json::Value) -> Result<serde_json::Value, String> {
    if let Some(error) = response.get("error") {
        let message = error
            .get("message")
            .and_then(|m| m.as_str())
            .unwrap_or("unknown");
        let code = error.get("code").and_then(|c| c.as_i64()).unwrap_or(0);
        return Err(format!("rpc error {code}: {message}"));
    }
    response
        .get("result")
        .cloned()
        .ok_or_else(|| "no result in response".to_owned())
}

impl JsonRpcError {
    /// Method not found (-32601). Clones `id` from the request.
    #[must_use]
    pub fn method_not_found(id: &serde_json::Value, method: &str) -> Self {
        Self {
            jsonrpc: "2.0",
            error: RpcErrorBody {
                code: -32601,
                message: format!("method not found: {method}"),
            },
            id: id.clone(),
        }
    }

    /// Invalid params (-32602). Clones `id` from the request.
    #[must_use]
    pub fn invalid_params(id: &serde_json::Value, detail: &str) -> Self {
        Self {
            jsonrpc: "2.0",
            error: RpcErrorBody {
                code: -32602,
                message: format!("invalid params: {detail}"),
            },
            id: id.clone(),
        }
    }

    /// Internal error (-32603). Clones `id` from the request.
    #[must_use]
    pub fn internal(id: &serde_json::Value, detail: &str) -> Self {
        Self {
            jsonrpc: "2.0",
            error: RpcErrorBody {
                code: -32603,
                message: format!("internal error: {detail}"),
            },
            id: id.clone(),
        }
    }
}

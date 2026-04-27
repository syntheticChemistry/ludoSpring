// SPDX-License-Identifier: AGPL-3.0-or-later
//! JSON-RPC 2.0 protocol envelope types and typed IPC error.
//!
//! These are protocol-level types, independent of any specific method.
//! They follow the JSON-RPC 2.0 specification (2010-03-26).
//!
//! [`IpcError`] replaces bare `String` errors across the IPC layer,
//! following the coralReef Iter 52 typed error pattern.

use std::borrow::Cow;

use serde::{Deserialize, Serialize};

// ── IPC Error Phase (primalSpring V094 pattern) ─────────────────────

/// Phase in which an IPC operation failed.
///
/// Absorbed from primalSpring `ecoPrimal/src/ipc/error.rs`. Annotating
/// errors with their phase enables smart retry logic: connect failures
/// are retriable, parse failures are not.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum IpcErrorPhase {
    /// Socket connection attempt.
    #[error("connect")]
    Connect,
    /// Sending the request payload.
    #[error("send")]
    Send,
    /// Waiting for / reading the response.
    #[error("receive")]
    Receive,
    /// Deserializing the response JSON.
    #[error("parse")]
    Parse,
    /// Timeout on any phase.
    #[error("timeout")]
    Timeout,
}

// ── Typed IPC Error ──────────────────────────────────────────────────

/// Structured error type for all IPC operations.
///
/// Each variant maps to a failure mode observed across ecosystem IPC
/// (connect, timeout, serialization, protocol, RPC error codes).
/// Evolved to `thiserror` per ecosystem standard (rhizoCrypt, bearDog,
/// airSpring, loamSpine pattern).
#[derive(Debug, thiserror::Error)]
pub enum IpcError {
    /// Socket connection failed (primal unreachable, path invalid).
    #[error("connect: {0}")]
    Connect(#[source] std::io::Error),
    /// Read/write timeout on an established connection.
    #[error("timeout: {0}")]
    Timeout(#[source] std::io::Error),
    /// I/O error during read, write, or stream clone.
    #[error("io: {0}")]
    Io(#[source] std::io::Error),
    /// JSON serialization or deserialization failed.
    #[error("serialization: {0}")]
    Serialization(String),
    /// JSON-RPC error response from the remote primal.
    #[error("rpc error {code}: {message}")]
    RpcError {
        /// JSON-RPC error code (-32600..-32603 standard, -32000..-32099 app).
        code: i64,
        /// Human-readable error message from the remote.
        message: String,
    },
    /// The response was valid JSON but contained no `result` or `error` field.
    #[error("no result in response")]
    NoResult,
    /// Discovery failed — no primal found for the requested capability.
    #[error("{0}")]
    NotFound(String),
}

impl IpcError {
    /// Whether a retry is likely to succeed (transient transport failure).
    #[must_use]
    pub const fn is_retriable(&self) -> bool {
        matches!(self, Self::Connect(_) | Self::Timeout(_))
    }

    /// Whether this error is recoverable (could succeed on a different endpoint).
    #[must_use]
    pub const fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::Connect(_) | Self::Timeout(_) | Self::Io(_) | Self::NotFound(_)
        )
    }

    /// Whether the failure is a connection-level problem (socket missing, refused).
    #[must_use]
    pub const fn is_connection_error(&self) -> bool {
        matches!(self, Self::Connect(_) | Self::NotFound(_))
    }

    /// Whether the failure looks like a timeout.
    #[must_use]
    pub const fn is_timeout_likely(&self) -> bool {
        matches!(self, Self::Timeout(_))
    }

    /// Whether the remote explicitly reported "method not found".
    #[must_use]
    pub const fn is_method_not_found(&self) -> bool {
        matches!(self, Self::RpcError { code: -32601, .. })
    }

    /// Whether this is a wire-level protocol violation.
    #[must_use]
    pub const fn is_protocol_error(&self) -> bool {
        matches!(self, Self::NoResult | Self::Serialization(_))
    }

    /// Whether this error should be treated as a graceful skip.
    ///
    /// Mirrors `primalspring::composition::is_skip_error`. Covers absent
    /// primals (connection refused, not found) and protocol mismatches
    /// (non-JSON-RPC response). A skip means "the capability is expected
    /// absent" and does not count as a test failure.
    #[must_use]
    pub const fn is_skip_error(&self) -> bool {
        self.is_connection_error() || self.is_protocol_error()
    }

    /// Wrap this error with a phase annotation.
    #[must_use]
    pub const fn in_phase(self, phase: IpcErrorPhase) -> PhasedIpcError {
        PhasedIpcError { phase, inner: self }
    }
}

impl From<serde_json::Error> for IpcError {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization(err.to_string())
    }
}

impl From<std::io::Error> for IpcError {
    fn from(err: std::io::Error) -> Self {
        classify_io_error(err)
    }
}

/// Classify a raw [`std::io::Error`] into a semantic [`IpcError`] variant.
///
/// Mirrors `primalspring::ipc::error::classify_io_error` — connection-refused
/// and not-found map to [`IpcError::Connect`], timeouts to [`IpcError::Timeout`],
/// everything else to [`IpcError::Io`].
#[must_use]
pub fn classify_io_error(err: std::io::Error) -> IpcError {
    match err.kind() {
        std::io::ErrorKind::ConnectionRefused | std::io::ErrorKind::NotFound => {
            IpcError::Connect(err)
        }
        std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock => IpcError::Timeout(err),
        _ => IpcError::Io(err),
    }
}

/// An [`IpcError`] annotated with the [`IpcErrorPhase`] where it occurred.
///
/// Absorbed from primalSpring `ecoPrimal/src/ipc/error.rs`. Enables
/// downstream code to make retry/fallback decisions based on both the
/// error kind and the communication phase.
#[derive(Debug, thiserror::Error)]
#[error("{phase}: {inner}")]
pub struct PhasedIpcError {
    /// Which phase of the IPC exchange failed.
    pub phase: IpcErrorPhase,
    /// The underlying error.
    pub inner: IpcError,
}

impl From<IpcError> for String {
    fn from(e: IpcError) -> Self {
        e.to_string()
    }
}

// ── Method Normalization (SPRING_COMPOSITION_PATTERNS §1 — MUST) ────

/// Known method-name prefixes that biomeOS or peer springs may prepend.
const METHOD_PREFIXES: &[&str] = &["ludospring.", "barracuda.", "biomeos.", "game.ludospring."];

/// Strip known spring/primal prefixes from a method name.
///
/// Iterates until stable — handles double-prefixed names like
/// `biomeos.ludospring.game.evaluate_flow`. Per `SPRING_COMPOSITION_PATTERNS`
/// §1, every spring's RPC dispatch MUST normalize before matching.
#[must_use]
pub fn normalize_method(method: &str) -> Cow<'_, str> {
    let mut m = method;
    loop {
        let before = m;
        for p in METHOD_PREFIXES {
            if let Some(stripped) = m.strip_prefix(p) {
                m = stripped;
            }
        }
        if m == before {
            break;
        }
    }
    if m == method {
        Cow::Borrowed(method)
    } else {
        Cow::Owned(m.to_owned())
    }
}

// ── Dispatch Outcome (groundSpring V112 / petalTongue V166 pattern) ─

/// Classifies the outcome of a JSON-RPC dispatch into one of three buckets.
///
/// Absorbed from groundSpring V112 and petalTongue V166. Replaces ad-hoc
/// `match` on `Result<Value, IpcError>` with a structured enum that
/// separates protocol-level failures from application-level errors.
#[derive(Debug)]
pub enum DispatchOutcome<T = serde_json::Value> {
    /// The call succeeded and returned a typed result.
    Ok(T),
    /// Protocol-level failure: socket unreachable, timeout, serialization,
    /// or the response was not valid JSON-RPC. Never the remote's fault.
    ProtocolError(IpcError),
    /// Application-level error: the remote primal explicitly returned a
    /// JSON-RPC error response (code + message).
    ApplicationError {
        /// JSON-RPC error code.
        code: i64,
        /// Human-readable error message from the remote primal.
        message: String,
    },
}

impl<T> DispatchOutcome<T> {
    /// Whether the dispatch succeeded.
    #[must_use]
    pub const fn is_ok(&self) -> bool {
        matches!(self, Self::Ok(_))
    }

    /// Convert to `Result`, merging both error variants.
    ///
    /// # Errors
    ///
    /// Returns [`IpcError`] for both protocol-level and application-level failures.
    pub fn into_result(self) -> Result<T, IpcError> {
        match self {
            Self::Ok(v) => Ok(v),
            Self::ProtocolError(e) => Err(e),
            Self::ApplicationError { code, message } => Err(IpcError::RpcError { code, message }),
        }
    }
}

impl DispatchOutcome<serde_json::Value> {
    /// Classify an `IpcError` into a `DispatchOutcome`.
    #[must_use]
    pub fn from_ipc_error(err: IpcError) -> Self {
        match err {
            IpcError::RpcError { code, message } => Self::ApplicationError { code, message },
            other => Self::ProtocolError(other),
        }
    }

    /// Classify a `Result<Value, IpcError>` into a `DispatchOutcome`.
    #[must_use]
    pub fn classify(result: Result<serde_json::Value, IpcError>) -> Self {
        match result {
            Ok(v) => Self::Ok(v),
            Err(e) => Self::from_ipc_error(e),
        }
    }
}

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

/// Extract the result from a raw JSON-RPC response object.
///
/// Handles the common pattern where callers receive a `serde_json::Value`
/// and need to check for an `"error"` field. Returns `Ok(result)` on success
/// or a typed [`IpcError`] if the response contains an error or no result.
///
/// Follows the healthSpring V29 `extract_rpc_error()` centralization pattern,
/// evolved to typed errors per coralReef Iter 52.
///
/// # Errors
///
/// Returns [`IpcError::RpcError`] if the response contains an `"error"` field,
/// or [`IpcError::NoResult`] if neither `"result"` nor `"error"` is present.
pub fn extract_rpc_result(response: &serde_json::Value) -> Result<serde_json::Value, IpcError> {
    if let Some(error) = response.get("error") {
        let message = error
            .get("message")
            .and_then(|m| m.as_str())
            .unwrap_or("unknown")
            .to_string();
        let code = error
            .get("code")
            .and_then(serde_json::Value::as_i64)
            .unwrap_or(0);
        return Err(IpcError::RpcError { code, message });
    }
    response.get("result").cloned().ok_or(IpcError::NoResult)
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
    pub fn internal(id: &serde_json::Value, detail: &(impl std::fmt::Display + ?Sized)) -> Self {
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

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
#[path = "envelope_tests.rs"]
mod tests;

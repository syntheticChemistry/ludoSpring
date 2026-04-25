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
mod tests {
    use super::*;
    use std::error::Error;

    #[test]
    fn extract_rpc_result_success() {
        let resp = serde_json::json!({"jsonrpc": "2.0", "result": {"ok": true}, "id": 1});
        let result = extract_rpc_result(&resp);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), serde_json::json!({"ok": true}));
    }

    #[test]
    fn extract_rpc_result_rpc_error() {
        let resp = serde_json::json!({
            "jsonrpc": "2.0",
            "error": {"code": -32601, "message": "method not found"},
            "id": 1
        });
        let err = extract_rpc_result(&resp).unwrap_err();
        assert!(matches!(err, IpcError::RpcError { code: -32601, .. }));
        assert!(err.to_string().contains("-32601"));
        assert!(err.to_string().contains("method not found"));
    }

    #[test]
    fn extract_rpc_result_no_result() {
        let resp = serde_json::json!({"jsonrpc": "2.0", "id": 1});
        let err = extract_rpc_result(&resp).unwrap_err();
        assert!(matches!(err, IpcError::NoResult));
    }

    #[test]
    fn extract_rpc_result_error_missing_message() {
        let resp = serde_json::json!({"error": {"code": -32000}, "id": 1});
        let err = extract_rpc_result(&resp).unwrap_err();
        match err {
            IpcError::RpcError { code, message } => {
                assert_eq!(code, -32000);
                assert_eq!(message, "unknown");
            }
            _ => panic!("expected RpcError"),
        }
    }

    #[test]
    fn ipc_error_display_formats() {
        let io_err = std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "refused");
        assert_eq!(IpcError::Connect(io_err).to_string(), "connect: refused");

        assert_eq!(
            IpcError::Serialization("bad json".into()).to_string(),
            "serialization: bad json"
        );

        assert_eq!(
            IpcError::RpcError {
                code: -32601,
                message: "not found".into()
            }
            .to_string(),
            "rpc error -32601: not found"
        );

        assert_eq!(IpcError::NoResult.to_string(), "no result in response");

        assert_eq!(IpcError::NotFound("no viz".into()).to_string(), "no viz");
    }

    #[test]
    fn ipc_error_to_string_conversion() {
        let err = IpcError::NoResult;
        let s: String = err.into();
        assert_eq!(s, "no result in response");
    }

    #[test]
    fn ipc_error_is_std_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::BrokenPipe, "pipe broke");
        let ipc_err = IpcError::Io(io_err);
        let as_error: &dyn std::error::Error = &ipc_err;
        assert!(as_error.source().is_some());
    }

    #[test]
    fn ipc_error_source_chain() {
        let inner = std::io::Error::new(std::io::ErrorKind::NotFound, "nf");
        let e = IpcError::Connect(inner);
        let src = e
            .source()
            .expect("Connect should chain to io::Error source");
        assert_eq!(src.to_string(), "nf");

        let inner = std::io::Error::new(std::io::ErrorKind::TimedOut, "to");
        let e = IpcError::Timeout(inner);
        let src = e
            .source()
            .expect("Timeout should chain to io::Error source");
        assert_eq!(src.to_string(), "to");

        let inner = std::io::Error::new(std::io::ErrorKind::BrokenPipe, "bp");
        let e = IpcError::Io(inner);
        let src = e.source().expect("Io should chain to io::Error source");
        assert_eq!(src.to_string(), "bp");
    }

    // ── IpcErrorPhase + classification tests ────────────────────────

    #[test]
    fn ipc_error_is_retriable() {
        let io = std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "refused");
        assert!(IpcError::Connect(io).is_retriable());
        let io = std::io::Error::new(std::io::ErrorKind::TimedOut, "timeout");
        assert!(IpcError::Timeout(io).is_retriable());
        assert!(!IpcError::NoResult.is_retriable());
        assert!(!IpcError::Serialization("bad".into()).is_retriable());
    }

    #[test]
    fn ipc_error_is_recoverable() {
        let io = std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "refused");
        assert!(IpcError::Connect(io).is_recoverable());
        assert!(IpcError::NotFound("x".into()).is_recoverable());
        assert!(!IpcError::NoResult.is_recoverable());
    }

    #[test]
    fn ipc_error_is_method_not_found() {
        assert!(
            IpcError::RpcError {
                code: -32601,
                message: "method not found".into()
            }
            .is_method_not_found()
        );
        assert!(
            !IpcError::RpcError {
                code: -32000,
                message: "app error".into()
            }
            .is_method_not_found()
        );
        assert!(!IpcError::NoResult.is_method_not_found());
    }

    #[test]
    fn ipc_error_is_connection_error() {
        let io = std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "refused");
        assert!(IpcError::Connect(io).is_connection_error());
        assert!(IpcError::NotFound("x".into()).is_connection_error());
        assert!(!IpcError::NoResult.is_connection_error());
        assert!(!IpcError::Serialization("bad".into()).is_connection_error());
    }

    #[test]
    fn ipc_error_is_timeout_likely() {
        let io = std::io::Error::new(std::io::ErrorKind::TimedOut, "timeout");
        assert!(IpcError::Timeout(io).is_timeout_likely());
        assert!(!IpcError::NoResult.is_timeout_likely());
        let io = std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "refused");
        assert!(!IpcError::Connect(io).is_timeout_likely());
    }

    #[test]
    fn ipc_error_is_protocol_error() {
        assert!(IpcError::NoResult.is_protocol_error());
        assert!(IpcError::Serialization("bad".into()).is_protocol_error());
        let io = std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "refused");
        assert!(!IpcError::Connect(io).is_protocol_error());
    }

    #[test]
    fn classify_io_error_connection_refused() {
        let io = std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "refused");
        let err = classify_io_error(io);
        assert!(matches!(err, IpcError::Connect(_)));
        assert!(err.is_connection_error());
    }

    #[test]
    fn classify_io_error_not_found() {
        let io = std::io::Error::new(std::io::ErrorKind::NotFound, "not found");
        let err = classify_io_error(io);
        assert!(matches!(err, IpcError::Connect(_)));
    }

    #[test]
    fn classify_io_error_timed_out() {
        let io = std::io::Error::new(std::io::ErrorKind::TimedOut, "timed out");
        let err = classify_io_error(io);
        assert!(matches!(err, IpcError::Timeout(_)));
        assert!(err.is_timeout_likely());
    }

    #[test]
    fn classify_io_error_would_block() {
        let io = std::io::Error::new(std::io::ErrorKind::WouldBlock, "would block");
        let err = classify_io_error(io);
        assert!(matches!(err, IpcError::Timeout(_)));
    }

    #[test]
    fn classify_io_error_other_becomes_io() {
        let io = std::io::Error::new(std::io::ErrorKind::BrokenPipe, "pipe");
        let err = classify_io_error(io);
        assert!(matches!(err, IpcError::Io(_)));
    }

    #[test]
    fn is_skip_error_for_connection_and_protocol() {
        assert!(IpcError::NotFound("gone".into()).is_skip_error());
        assert!(
            IpcError::Connect(std::io::Error::new(
                std::io::ErrorKind::ConnectionRefused,
                "refused"
            ))
            .is_skip_error()
        );
        assert!(IpcError::NoResult.is_skip_error());
        assert!(IpcError::Serialization("bad json".into()).is_skip_error());
        assert!(!IpcError::Io(std::io::Error::new(std::io::ErrorKind::BrokenPipe, "pipe"))
            .is_skip_error());
        assert!(!IpcError::RpcError {
            code: -32000,
            message: "app error".into()
        }
        .is_skip_error());
    }

    #[test]
    fn internal_accepts_ipc_error_display() {
        let ipc_err = IpcError::NotFound("beardog".into());
        let rpc_err = JsonRpcError::internal(&serde_json::json!(1), &ipc_err);
        assert!(rpc_err.error.message.contains("beardog"));
    }

    #[test]
    fn phased_ipc_error_display() {
        let err = IpcError::NoResult.in_phase(IpcErrorPhase::Receive);
        assert_eq!(err.to_string(), "receive: no result in response");
        assert_eq!(err.phase, IpcErrorPhase::Receive);
    }

    #[test]
    fn ipc_error_phase_display() {
        assert_eq!(IpcErrorPhase::Connect.to_string(), "connect");
        assert_eq!(IpcErrorPhase::Send.to_string(), "send");
        assert_eq!(IpcErrorPhase::Receive.to_string(), "receive");
        assert_eq!(IpcErrorPhase::Parse.to_string(), "parse");
        assert_eq!(IpcErrorPhase::Timeout.to_string(), "timeout");
    }

    // ── Method normalization tests ───────────────────────────────────

    #[test]
    fn normalize_method_passthrough() {
        assert_eq!(normalize_method("game.evaluate_flow"), "game.evaluate_flow");
        assert_eq!(normalize_method("health.liveness"), "health.liveness");
    }

    #[test]
    fn normalize_method_strips_prefix() {
        assert_eq!(
            normalize_method("ludospring.game.evaluate_flow"),
            "game.evaluate_flow"
        );
        assert_eq!(
            normalize_method("biomeos.game.evaluate_flow"),
            "game.evaluate_flow"
        );
    }

    #[test]
    fn normalize_method_strips_double_prefix() {
        assert_eq!(
            normalize_method("biomeos.ludospring.game.evaluate_flow"),
            "game.evaluate_flow"
        );
    }

    #[test]
    fn normalize_method_empty_and_unknown() {
        assert_eq!(normalize_method(""), "");
        assert_eq!(normalize_method("unknown.method"), "unknown.method");
    }

    // ── DispatchOutcome tests ────────────────────────────────────────

    #[test]
    fn dispatch_outcome_classify_ok() {
        let result: Result<serde_json::Value, IpcError> = Ok(serde_json::json!(42));
        let outcome = DispatchOutcome::classify(result);
        assert!(outcome.is_ok());
        assert!(outcome.into_result().is_ok());
    }

    #[test]
    fn dispatch_outcome_classify_protocol_error() {
        let result: Result<serde_json::Value, IpcError> = Err(IpcError::NoResult);
        let outcome = DispatchOutcome::classify(result);
        assert!(!outcome.is_ok());
        assert!(matches!(outcome, DispatchOutcome::ProtocolError(_)));
    }

    #[test]
    fn dispatch_outcome_classify_application_error() {
        let result: Result<serde_json::Value, IpcError> = Err(IpcError::RpcError {
            code: -32000,
            message: "app error".into(),
        });
        let outcome = DispatchOutcome::classify(result);
        assert!(!outcome.is_ok());
        assert!(matches!(
            outcome,
            DispatchOutcome::ApplicationError { code: -32000, .. }
        ));
    }

    #[test]
    fn dispatch_outcome_into_result_merges_errors() {
        let protocol = DispatchOutcome::<serde_json::Value>::ProtocolError(IpcError::NoResult);
        let err = protocol.into_result().unwrap_err();
        assert!(matches!(err, IpcError::NoResult));

        let app = DispatchOutcome::<serde_json::Value>::ApplicationError {
            code: -32001,
            message: "fail".into(),
        };
        let err = app.into_result().unwrap_err();
        assert!(matches!(err, IpcError::RpcError { code: -32001, .. }));
    }

    // ── Proptest fuzz (airSpring v0.8.7 pattern) ────────────────────

    mod proptest_fuzz {
        use super::*;
        use proptest::prelude::*;

        fn io_kind_from_byte(b: u8) -> std::io::ErrorKind {
            match b % 14 {
                0 => std::io::ErrorKind::NotFound,
                1 => std::io::ErrorKind::PermissionDenied,
                2 => std::io::ErrorKind::ConnectionRefused,
                3 => std::io::ErrorKind::ConnectionReset,
                4 => std::io::ErrorKind::ConnectionAborted,
                5 => std::io::ErrorKind::NotConnected,
                6 => std::io::ErrorKind::AddrInUse,
                7 => std::io::ErrorKind::AddrNotAvailable,
                8 => std::io::ErrorKind::BrokenPipe,
                9 => std::io::ErrorKind::AlreadyExists,
                10 => std::io::ErrorKind::WouldBlock,
                11 => std::io::ErrorKind::InvalidInput,
                12 => std::io::ErrorKind::InvalidData,
                _ => std::io::ErrorKind::Other,
            }
        }

        proptest! {
            #[test]
            fn extract_rpc_result_never_panics(json_str in "\\PC{0,200}") {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&json_str) {
                    let _ = extract_rpc_result(&val);
                }
            }

            #[test]
            fn extract_rpc_result_with_result_field_returns_ok(
                inner in prop::collection::hash_map("[a-z]{1,5}", 0i64..100, 0..3)
            ) {
                let val = serde_json::json!({"jsonrpc": "2.0", "result": inner, "id": 1});
                let res = extract_rpc_result(&val);
                prop_assert!(res.is_ok());
            }

            #[test]
            fn extract_rpc_result_with_error_field_returns_err(
                code in -32700i64..-31999,
                msg in "[a-z ]{0,30}"
            ) {
                let val = serde_json::json!({
                    "jsonrpc": "2.0",
                    "error": {"code": code, "message": msg},
                    "id": 1
                });
                let res = extract_rpc_result(&val);
                prop_assert!(res.is_err());
                match res.unwrap_err() {
                    IpcError::RpcError { code: c, message: m } => {
                        prop_assert_eq!(c, code);
                        prop_assert_eq!(m, msg);
                    }
                    other => prop_assert!(false, "expected RpcError, got {:?}", other),
                }
            }

            #[test]
            fn dispatch_outcome_classify_round_trips(code in -32700i64..-31999) {
                let rpc_err = IpcError::RpcError { code, message: "test".into() };
                let outcome = DispatchOutcome::from_ipc_error(rpc_err);
                let is_app_err = matches!(outcome, DispatchOutcome::ApplicationError { .. });
                prop_assert!(is_app_err, "expected ApplicationError, got {:?}", outcome);
            }

            #[test]
            fn fuzz_ipc_error_display(
                variant in 0u8..7u8,
                msg in "\\PC{0,200}",
                code in -200_000i64..200_000i64,
                kind_byte in 0u8..=255u8,
            ) {
                let kind = io_kind_from_byte(kind_byte);
                let err = match variant {
                    0 => IpcError::Connect(std::io::Error::new(kind, msg)),
                    1 => IpcError::Timeout(std::io::Error::new(kind, msg)),
                    2 => IpcError::Io(std::io::Error::new(kind, msg)),
                    3 => IpcError::Serialization(msg),
                    4 => IpcError::RpcError {
                        code,
                        message: msg,
                    },
                    5 => IpcError::NoResult,
                    _ => IpcError::NotFound(msg),
                };
                let _ = format!("{err}");
                let _ = err.to_string();
            }
        }
    }
}

// SPDX-License-Identifier: AGPL-3.0-or-later
// Tests for ipc/envelope.rs — extracted to keep envelope.rs under 450 lines.

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
    assert!(
        !IpcError::Io(std::io::Error::new(std::io::ErrorKind::BrokenPipe, "pipe")).is_skip_error()
    );
    assert!(
        !IpcError::RpcError {
            code: -32000,
            message: "app error".into()
        }
        .is_skip_error()
    );
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

// SPDX-License-Identifier: AGPL-3.0-or-later
//! BTSP relay — 4-step BearDog handshake for authenticated connections.
//!
//! Implements the sourDough BTSP Relay Pattern:
//! 1. Auto-detect: peek first line; if BTSP ClientHello, enter handshake
//! 2. Relay to BearDog via `btsp.session.create` / `btsp.session.verify`
//! 3. On success, transition to normal JSON-RPC dispatch
//! 4. On failure, send JSON error frame (never drop silently)
//!
//! See: `wateringHole/SOURDOUGH_BTSP_RELAY_PATTERN.md`

use std::io::{BufRead, Write};
use std::path::PathBuf;
use std::time::Duration;

use serde_json::{Value, json};
use tracing::info;

use crate::ipc::discovery;
use crate::ipc::envelope::IpcError;

const BTSP_TIMEOUT: Duration = Duration::from_secs(5);

/// Whether this primal requires BTSP authentication.
///
/// Active when `FAMILY_ID` is set to a non-empty, non-`"default"` value
/// and `BIOMEOS_INSECURE` is not set. Matches ecosystem standard.
pub fn btsp_required() -> bool {
    let insecure = std::env::var("BIOMEOS_INSECURE")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    if insecure {
        return false;
    }

    let fid = std::env::var("FAMILY_ID")
        .or_else(|_| std::env::var("BIOMEOS_FAMILY_ID"))
        .unwrap_or_default();

    !fid.is_empty() && fid != "default"
}

/// Check if a JSON line is a BTSP ClientHello.
fn is_btsp_client_hello(line: &str) -> bool {
    line.contains("\"protocol\"") && line.contains("\"btsp\"")
}

/// Standard base64 alphabet.
const B64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Base64-encode a byte slice (standard alphabet, with padding).
fn base64_encode(input: &[u8]) -> String {
    let mut out = String::with_capacity(input.len().div_ceil(3) * 4);
    for chunk in input.chunks(3) {
        let b0 = chunk[0];
        let b1 = chunk.get(1).copied().unwrap_or(0);
        let b2 = chunk.get(2).copied().unwrap_or(0);
        let triple = u32::from(b0) << 16 | u32::from(b1) << 8 | u32::from(b2);
        out.push(B64[(triple >> 18 & 0x3F) as usize] as char);
        out.push(B64[(triple >> 12 & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            out.push(B64[(triple >> 6 & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(B64[(triple & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}

/// Resolve the family_seed for BearDog: base64-encode the raw env string bytes.
fn resolve_family_seed() -> Option<String> {
    let raw = std::env::var("FAMILY_SEED")
        .or_else(|_| std::env::var("BEARDOG_FAMILY_SEED"))
        .or_else(|_| std::env::var("BIOMEOS_FAMILY_SEED"))
        .ok()?;
    Some(base64_encode(raw.trim().as_bytes()))
}

/// Find BearDog's socket via capability-based discovery.
fn discover_beardog() -> Option<PathBuf> {
    discovery::discover_by_capability("security").map(|ep| ep.socket)
}

/// Send a JSON-RPC request to BearDog and read the response.
///
/// Delegates transport to [`super::rpc_client::RpcClient`], then extracts
/// the `result` field with manual error handling (BearDog-specific).
fn beardog_call(socket: &std::path::Path, request: &Value) -> Result<Value, IpcError> {
    let client = super::rpc_client::RpcClient::new(socket, BTSP_TIMEOUT);
    let parsed = client.send_raw(request)?;

    if let Some(err) = parsed.get("error") {
        let code = err.get("code").and_then(Value::as_i64).unwrap_or(-32603);
        let message = err
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_owned();
        return Err(IpcError::RpcError { code, message });
    }

    parsed.get("result").cloned().ok_or(IpcError::NoResult)
}

/// Write a JSON line + newline to a writer.
fn write_json_line<W: Write>(writer: &mut W, value: &Value) -> Result<(), IpcError> {
    let mut msg = serde_json::to_string(value)?;
    msg.push('\n');
    writer.write_all(msg.as_bytes()).map_err(IpcError::Io)?;
    writer.flush().map_err(IpcError::Io)?;
    Ok(())
}

/// Write a JSON error frame to the client (never drop silently).
fn write_error_frame<W: Write>(writer: &mut W, reason: &str) -> Result<(), IpcError> {
    write_json_line(
        writer,
        &json!({
            "error": "handshake_failed",
            "reason": reason,
        }),
    )
}

/// Result of auto-detecting the first line from a client.
pub enum FirstLineResult {
    /// BTSP ClientHello detected — relay handshake needed.
    BtspHello(String),
    /// Plain JSON-RPC request — process normally.
    PlainJsonRpc(String),
}

/// Peek the first line and classify it.
///
/// # Errors
///
/// Returns `IpcError::Io` if the first line cannot be read, or
/// `IpcError::NoResult` if the first line is empty.
pub fn classify_first_line<R: BufRead>(reader: &mut R) -> Result<FirstLineResult, IpcError> {
    let mut first_line = String::new();
    reader.read_line(&mut first_line).map_err(IpcError::Io)?;

    if first_line.trim().is_empty() {
        return Err(IpcError::NoResult);
    }

    if is_btsp_client_hello(&first_line) {
        Ok(FirstLineResult::BtspHello(first_line))
    } else {
        Ok(FirstLineResult::PlainJsonRpc(first_line))
    }
}

/// Perform the full 4-step BTSP handshake relay via BearDog.
///
/// On success, the connection is authenticated and ready for JSON-RPC.
/// On failure, an error frame is sent to the client.
///
/// # Errors
///
/// Returns `IpcError::NotFound` if BearDog is not discoverable or the
/// family seed is missing, `IpcError::Serialization` for parse failures,
/// or propagates errors from `beardog_call`.
pub fn perform_handshake<R: BufRead, W: Write>(
    client_hello_line: &str,
    reader: &mut R,
    writer: &mut W,
) -> Result<(), IpcError> {
    let hello: Value = serde_json::from_str(client_hello_line)
        .map_err(|e| IpcError::Serialization(format!("parse ClientHello: {e}")))?;

    let client_ephemeral_pub = hello
        .get("client_ephemeral_pub")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_owned();

    let beardog_socket =
        discover_beardog().ok_or_else(|| IpcError::NotFound("BearDog not discovered".into()))?;

    let family_seed = resolve_family_seed()
        .ok_or_else(|| IpcError::NotFound("FAMILY_SEED / BEARDOG_FAMILY_SEED not set".into()))?;

    let create_result = beardog_call(
        &beardog_socket,
        &json!({
            "jsonrpc": "2.0",
            "method": "btsp.session.create",
            "params": { "family_seed": family_seed },
            "id": 1
        }),
    )
    .map_err(|e| {
        let _ = write_error_frame(
            writer,
            &format!("BearDog relay: session create failed: {e}"),
        );
        e
    })?;

    let session_token = create_result
        .get("session_token")
        .or_else(|| create_result.get("session_id"))
        .cloned()
        .unwrap_or(Value::Null);

    let server_hello = json!({
        "version": 1,
        "server_ephemeral_pub": create_result.get("server_ephemeral_pub").unwrap_or(&Value::Null),
        "challenge": create_result.get("challenge").unwrap_or(&Value::Null),
        "session_id": &session_token,
    });
    write_json_line(writer, &server_hello)?;
    info!("BTSP: ServerHello sent");

    let mut response_line = String::new();
    reader.read_line(&mut response_line).map_err(IpcError::Io)?;
    let challenge_resp: Value = serde_json::from_str(&response_line)
        .map_err(|e| IpcError::Serialization(format!("parse ChallengeResponse: {e}")))?;

    let verify_result = beardog_call(
        &beardog_socket,
        &json!({
            "jsonrpc": "2.0",
            "method": "btsp.session.verify",
            "params": {
                "session_token": &session_token,
                "response": challenge_resp.get("response").unwrap_or(&Value::Null),
                "client_ephemeral_pub": client_ephemeral_pub,
                "preferred_cipher": challenge_resp.get("preferred_cipher").unwrap_or(&Value::Null),
            },
            "id": 2
        }),
    )
    .map_err(|e| {
        let _ = write_error_frame(
            writer,
            &format!("BearDog relay: session verify failed: {e}"),
        );
        e
    })?;

    let verified = verify_result
        .get("verified")
        .and_then(Value::as_bool)
        .unwrap_or(false);

    if !verified {
        let _ = write_error_frame(writer, "BTSP verification failed");
        return Err(IpcError::RpcError {
            code: -32001,
            message: "BTSP verification failed".into(),
        });
    }

    let complete = json!({
        "status": "ok",
        "session_id": verify_result.get("session_id").unwrap_or(&session_token),
        "cipher": verify_result.get("cipher").unwrap_or(&Value::Null),
    });
    write_json_line(writer, &complete)?;
    info!("BTSP: handshake complete — session authenticated");

    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn is_btsp_client_hello_detects_btsp() {
        assert!(is_btsp_client_hello(
            r#"{"protocol":"btsp","version":1,"client_ephemeral_pub":"dGVzdA=="}"#
        ));
    }

    #[test]
    fn is_btsp_client_hello_rejects_plain_rpc() {
        assert!(!is_btsp_client_hello(
            r#"{"jsonrpc":"2.0","method":"health.check","id":1}"#
        ));
    }

    #[test]
    fn btsp_required_returns_bool() {
        let _result = btsp_required();
    }

    #[test]
    fn classify_btsp_hello() {
        let input = r#"{"protocol":"btsp","version":1,"client_ephemeral_pub":"dGVzdA=="}
"#;
        let mut reader = std::io::BufReader::new(input.as_bytes());
        let result = classify_first_line(&mut reader).unwrap();
        assert!(matches!(result, FirstLineResult::BtspHello(_)));
    }

    #[test]
    fn classify_plain_rpc() {
        let input = r#"{"jsonrpc":"2.0","method":"health.check","id":1}
"#;
        let mut reader = std::io::BufReader::new(input.as_bytes());
        let result = classify_first_line(&mut reader).unwrap();
        assert!(matches!(result, FirstLineResult::PlainJsonRpc(_)));
    }

    #[test]
    fn base64_encode_known_vectors() {
        assert_eq!(base64_encode(b""), "");
        assert_eq!(base64_encode(b"f"), "Zg==");
        assert_eq!(base64_encode(b"fo"), "Zm8=");
        assert_eq!(base64_encode(b"foo"), "Zm9v");
        assert_eq!(base64_encode(b"foobar"), "Zm9vYmFy");
        assert_eq!(base64_encode(b"Hello, World!"), "SGVsbG8sIFdvcmxkIQ==");
    }

    #[test]
    fn write_error_frame_produces_valid_json() {
        let mut buf = Vec::new();
        write_error_frame(&mut buf, "test error").unwrap();
        let s = String::from_utf8(buf).unwrap();
        let v: Value = serde_json::from_str(s.trim()).unwrap();
        assert_eq!(v["error"], "handshake_failed");
        assert_eq!(v["reason"], "test error");
    }
}

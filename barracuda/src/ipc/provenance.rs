// SPDX-License-Identifier: AGPL-3.0-or-later
//! Provenance trio integration for ludoSpring IPC.
//!
//! Wires game session lifecycle to the provenance trio (rhizoCrypt, LoamSpine,
//! sweetGrass) via capability-based discovery. Uses JSON-RPC over Unix socket,
//! either through Neural API `capability.call` or direct primal calls.
//!
//! ## Approach
//!
//! 1. **Discovery**: Resolve the Neural API socket via XDG-compliant paths
//!    (BIOMEOS_SOCKET_DIR, XDG_RUNTIME_DIR/biomeos, /tmp fallback). No
//!    hardcoded primal names — capability routing is delegated to the
//!    ecosystem.
//!
//! 2. **Protocol**: `capability.call` with `{ capability, operation, args }`
//!    routes to the appropriate primal (dag → rhizoCrypt, commit → LoamSpine,
//!    provenance → sweetGrass).
//!
//! 3. **Graceful degradation**: If the trio is unavailable (socket missing,
//!    connection refused, RPC error), handlers return success with
//!    `"provenance": "unavailable"` — game logic never fails for missing
//!    provenance.

use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

/// Result of a provenance operation; includes availability status.
#[derive(Debug)]
pub struct ProvenanceResult {
    /// Session ID, vertex ID, or braid ID from the trio.
    pub id: String,
    /// Whether the trio was available and the operation succeeded.
    pub available: bool,
    /// Additional data (e.g. merkle_root, commit_id, braid_id).
    pub data: serde_json::Value,
}

/// Resolve the Neural API socket path (XDG-compliant, no hardcoded primal names).
#[must_use]
fn neural_api_socket_path() -> Option<PathBuf> {
    let family_id = std::env::var("FAMILY_ID")
        .or_else(|_| std::env::var("BIOMEOS_FAMILY_ID"))
        .unwrap_or_else(|_| "default".to_string());

    let sock_name = format!("neural-api-{family_id}.sock");

    if let Ok(dir) = std::env::var("NEURAL_API_SOCKET") {
        let p = PathBuf::from(&dir);
        if p.exists() {
            return Some(p);
        }
    }

    if let Ok(dir) = std::env::var("BIOMEOS_SOCKET_DIR") {
        let p = PathBuf::from(dir).join(&sock_name);
        if p.exists() {
            return Some(p);
        }
    }

    if let Ok(xdg) = std::env::var("XDG_RUNTIME_DIR") {
        let p = PathBuf::from(xdg).join("biomeos").join(&sock_name);
        if p.exists() {
            return Some(p);
        }
    }

    let user = std::env::var("USER").unwrap_or_else(|_| "unknown".to_string());
    let fallback = PathBuf::from("/tmp").join(format!("biomeos-{user}"));
    let p = fallback.join(&sock_name);
    if p.exists() {
        return Some(p);
    }

    let p = PathBuf::from("/tmp").join(&sock_name);
    if p.exists() {
        return Some(p);
    }

    None
}

/// Send a capability.call request to the Neural API.
fn capability_call(
    socket_path: &std::path::Path,
    capability: &str,
    operation: &str,
    args: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    use std::os::unix::net::UnixStream;

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "capability.call",
        "params": {
            "capability": capability,
            "operation": operation,
            "args": args,
        },
        "id": 1
    });

    let request_str = serde_json::to_string(&request).map_err(|e| format!("serialize: {e}"))?;
    let mut stream = UnixStream::connect(socket_path).map_err(|e| format!("connect: {e}"))?;
    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(10)))
        .map_err(|e| format!("timeout: {e}"))?;
    stream
        .set_write_timeout(Some(std::time::Duration::from_secs(10)))
        .map_err(|e| format!("timeout: {e}"))?;

    stream
        .write_all(request_str.as_bytes())
        .map_err(|e| format!("write: {e}"))?;
    stream.write_all(b"\n").map_err(|e| format!("write: {e}"))?;
    stream.flush().map_err(|e| format!("flush: {e}"))?;
    stream
        .shutdown(std::net::Shutdown::Write)
        .map_err(|e| format!("shutdown: {e}"))?;

    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader
        .read_line(&mut line)
        .map_err(|e| format!("read: {e}"))?;

    let parsed: serde_json::Value =
        serde_json::from_str(line.trim()).map_err(|e| format!("parse: {e}"))?;

    if let Some(err) = parsed.get("error") {
        return Err(format!(
            "rpc error: {}",
            err.get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("unknown")
        ));
    }

    parsed
        .get("result")
        .cloned()
        .ok_or_else(|| "no result in response".to_string())
}

/// Whether any provenance session is currently active (trio reachable).
///
/// Used by `game.poll_telemetry` to report streaming vs idle status.
#[must_use]
pub fn has_active_session() -> bool {
    neural_api_socket_path().is_some()
}

/// Begin a game session in the provenance trio.
///
/// Creates a rhizoCrypt session via `dag.create_session`. Returns the session
/// ID or a fallback note if the trio is unavailable.
pub fn begin_game_session(session_name: &str) -> Result<ProvenanceResult, String> {
    let Some(socket) = neural_api_socket_path() else {
        return Ok(ProvenanceResult {
            id: format!("local-{}", uuid::Uuid::now_v7()),
            available: false,
            data: serde_json::json!({ "provenance": "unavailable" }),
        });
    };

    let args = serde_json::json!({
        "metadata": { "type": "game_session", "name": session_name },
        "session_type": { "Gaming": { "game_id": "ludospring" } },
        "description": session_name,
    });

    match capability_call(&socket, "dag", "create_session", &args) {
        Ok(result) => {
            let session_id = result
                .get("session_id")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();
            Ok(ProvenanceResult {
                id: session_id.clone(),
                available: true,
                data: serde_json::json!({ "session_id": session_id }),
            })
        }
        Err(_) => Ok(ProvenanceResult {
            id: format!("local-{}", uuid::Uuid::now_v7()),
            available: false,
            data: serde_json::json!({ "provenance": "unavailable" }),
        }),
    }
}

/// Record a game action in the provenance trio.
///
/// Appends a vertex via `dag.append_event`. Returns the vertex ID or a note
/// if unavailable.
pub fn record_game_action(
    session_id: &str,
    action: &serde_json::Value,
) -> Result<ProvenanceResult, String> {
    let Some(socket) = neural_api_socket_path() else {
        return Ok(ProvenanceResult {
            id: "unavailable".to_string(),
            available: false,
            data: serde_json::json!({ "provenance": "unavailable" }),
        });
    };

    let args = serde_json::json!({
        "session_id": session_id,
        "event": action,
    });

    match capability_call(&socket, "dag", "append_event", &args) {
        Ok(result) => {
            let vertex_id = result
                .get("vertex_id")
                .or_else(|| result.get("id"))
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();
            Ok(ProvenanceResult {
                id: vertex_id.clone(),
                available: true,
                data: serde_json::json!({ "vertex_id": vertex_id }),
            })
        }
        Err(_) => Ok(ProvenanceResult {
            id: "unavailable".to_string(),
            available: false,
            data: serde_json::json!({ "provenance": "unavailable" }),
        }),
    }
}

/// Complete a game session: dehydrate, commit, and attribute.
///
/// 1. `dag.dehydrate` — compute Merkle root and frontier
/// 2. `commit.session` — anchor to LoamSpine
/// 3. `provenance.create_braid` — attribute via sweetGrass
pub fn complete_game_session(session_id: &str) -> Result<serde_json::Value, String> {
    let Some(socket) = neural_api_socket_path() else {
        return Ok(serde_json::json!({
            "provenance": "unavailable",
            "session_id": session_id,
        }));
    };

    // Step 1: Dehydrate
    let dehydrate_args = serde_json::json!({ "session_id": session_id });
    let dehydration = match capability_call(&socket, "dag", "dehydrate", &dehydrate_args) {
        Ok(r) => r,
        Err(_) => {
            return Ok(serde_json::json!({
                "provenance": "unavailable",
                "session_id": session_id,
            }));
        }
    };

    let merkle_root = dehydration
        .get("merkle_root")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    // Step 2: Commit
    let commit_args = serde_json::json!({
        "summary": dehydration,
        "content_hash": merkle_root,
    });
    let commit_result = match capability_call(&socket, "commit", "session", &commit_args) {
        Ok(r) => r,
        Err(_) => {
            return Ok(serde_json::json!({
                "provenance": "partial",
                "session_id": session_id,
                "dehydrated": true,
                "committed": false,
            }));
        }
    };

    let commit_id = commit_result
        .get("commit_id")
        .or_else(|| commit_result.get("entry_id"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    // Step 3: Create braid (best-effort)
    let braid_args = serde_json::json!({
        "commit_ref": commit_id,
        "agents": [{
            "did": "did:key:ludospring-game",
            "role": "author",
            "contribution": 1.0
        }],
    });
    let braid_result = capability_call(&socket, "provenance", "create_braid", &braid_args);

    let braid_id = braid_result
        .ok()
        .and_then(|r| {
            r.get("braid_id")
                .or(r.get("id"))
                .and_then(|v| v.as_str())
                .map(str::to_string)
        })
        .unwrap_or_default();

    Ok(serde_json::json!({
        "provenance": "complete",
        "session_id": session_id,
        "merkle_root": merkle_root,
        "commit_id": commit_id,
        "braid_id": braid_id,
    }))
}

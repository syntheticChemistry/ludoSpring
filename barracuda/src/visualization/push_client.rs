// SPDX-License-Identifier: AGPL-3.0-or-later
//! Visualization push client — discovers any primal that advertises
//! `visualization.render` via capability-based discovery, then pushes
//! game science scenarios via JSON-RPC.
//!
//! Discovery priority:
//! 1. `VISUALIZATION_SOCKET` env var — explicit override
//! 2. Capability-based discovery (`ipc::discovery`) — finds any primal
//!    advertising `visualization.render` without knowing its name
//! 3. `$XDG_RUNTIME_DIR` + `/tmp` fallback scan for `*.sock` files
//!
//! This follows the "primal code only has self-knowledge" principle —
//! we never hardcode peer primal names.

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::time::Duration;

const VISUALIZATION_CAPABILITY: &str = "visualization.render";

/// Client for pushing visualization data to any viz-capable primal.
#[derive(Debug, Clone)]
pub struct VisualizationPushClient {
    socket: PathBuf,
}

impl VisualizationPushClient {
    /// Discover a live visualization-capable primal.
    ///
    /// Uses capability-based discovery first, then falls back to env var
    /// and directory scanning.
    ///
    /// # Errors
    ///
    /// Returns an error string if no visualization socket is found or connectable.
    pub fn discover() -> Result<Self, String> {
        if let Ok(explicit) = std::env::var("VISUALIZATION_SOCKET") {
            let path = PathBuf::from(&explicit);
            if Self::probe(&path) {
                return Ok(Self { socket: path });
            }
        }

        #[cfg(feature = "ipc")]
        {
            let registry = crate::ipc::discovery::discover_primals();
            if let Some(endpoint) = registry.find(VISUALIZATION_CAPABILITY) {
                if Self::probe(&endpoint.socket) {
                    return Ok(Self {
                        socket: endpoint.socket.clone(),
                    });
                }
            }
        }

        if let Ok(xdg) = std::env::var("XDG_RUNTIME_DIR") {
            let biomeos_dir = PathBuf::from(&xdg).join("biomeos");
            if let Some(sock) = Self::find_viz_sock_in(&biomeos_dir) {
                return Ok(Self { socket: sock });
            }
        }

        if let Some(sock) = Self::find_viz_sock_in(&PathBuf::from("/tmp")) {
            return Ok(Self { socket: sock });
        }

        Err("no visualization-capable primal found".into())
    }

    /// Push a visualization render request.
    ///
    /// # Errors
    ///
    /// Returns an error string if the connection or RPC call fails.
    pub fn push_render(
        &self,
        session_id: &str,
        title: &str,
        payload: &serde_json::Value,
    ) -> Result<(), String> {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": VISUALIZATION_CAPABILITY,
            "params": {
                "session_id": session_id,
                "title": title,
                "domain": "game",
                "data": payload,
            },
            "id": 1
        });
        self.send(&request)
    }

    /// Push a streaming update (append, `set_value`, or replace).
    ///
    /// # Errors
    ///
    /// Returns an error string if the connection or RPC call fails.
    pub fn push_stream(
        &self,
        session_id: &str,
        action: &str,
        payload: &serde_json::Value,
    ) -> Result<(), String> {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "visualization.render.stream",
            "params": {
                "session_id": session_id,
                "action": action,
                "data": payload,
            },
            "id": 1
        });
        self.send(&request)
    }

    fn send(&self, request: &serde_json::Value) -> Result<(), String> {
        let stream = UnixStream::connect(&self.socket).map_err(|e| format!("connect: {e}"))?;
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .map_err(|e| format!("timeout: {e}"))?;
        stream
            .set_write_timeout(Some(Duration::from_secs(5)))
            .map_err(|e| format!("timeout: {e}"))?;

        let mut writer = stream.try_clone().map_err(|e| format!("clone: {e}"))?;
        let mut msg = serde_json::to_string(request).map_err(|e| format!("serialize: {e}"))?;
        msg.push('\n');
        writer
            .write_all(msg.as_bytes())
            .map_err(|e| format!("write: {e}"))?;
        writer.flush().map_err(|e| format!("flush: {e}"))?;

        let mut reader = BufReader::new(stream);
        let mut response = String::new();
        reader
            .read_line(&mut response)
            .map_err(|e| format!("read: {e}"))?;

        let parsed: serde_json::Value =
            serde_json::from_str(&response).map_err(|e| format!("parse: {e}"))?;

        if let Some(error) = parsed.get("error") {
            return Err(format!("rpc error: {error}"));
        }
        Ok(())
    }

    fn probe(path: &std::path::Path) -> bool {
        UnixStream::connect(path)
            .and_then(|s| s.set_read_timeout(Some(Duration::from_millis(200))))
            .is_ok()
    }

    /// Scan a directory for any .sock file that responds to probing.
    fn find_viz_sock_in(dir: &std::path::Path) -> Option<PathBuf> {
        let entries = std::fs::read_dir(dir).ok()?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "sock") && Self::probe(&path) {
                return Some(path);
            }
        }
        None
    }
}

/// Type alias preserved for backward compatibility with existing callers.
pub type PetalTonguePushClient = VisualizationPushClient;

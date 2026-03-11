// SPDX-License-Identifier: AGPL-3.0-or-later
//! petalTongue push client — discovers the live viz socket and sends scenarios.
//!
//! Follows the wetSpring `PetalTonguePushClient` pattern: discover by socket,
//! push via `visualization.render`, fall back to JSON file export if the
//! socket is not available.

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::time::Duration;

/// Client for pushing visualization data to petalTongue.
#[derive(Debug, Clone)]
pub struct PetalTonguePushClient {
    socket: PathBuf,
}

impl PetalTonguePushClient {
    /// Discover a live petalTongue socket.
    ///
    /// Search order:
    /// 1. `PETALTONGUE_SOCKET` env var
    /// 2. `$XDG_RUNTIME_DIR/petaltongue/*.sock`
    /// 3. `/tmp/petaltongue-*.sock`
    ///
    /// # Errors
    ///
    /// Returns an error string if no socket is found or connectable.
    pub fn discover() -> Result<Self, String> {
        if let Ok(explicit) = std::env::var("PETALTONGUE_SOCKET") {
            let path = PathBuf::from(&explicit);
            if Self::probe(&path) {
                return Ok(Self { socket: path });
            }
        }

        if let Ok(xdg) = std::env::var("XDG_RUNTIME_DIR") {
            let dir = PathBuf::from(xdg).join("petaltongue");
            if let Some(sock) = Self::find_sock_in(&dir) {
                return Ok(Self { socket: sock });
            }
        }

        if let Some(sock) = Self::find_petaltongue_in_tmp() {
            return Ok(Self { socket: sock });
        }

        Err("no petalTongue socket found".into())
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
            "method": "visualization.render",
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

    fn find_sock_in(dir: &std::path::Path) -> Option<PathBuf> {
        let entries = std::fs::read_dir(dir).ok()?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "sock") && Self::probe(&path) {
                return Some(path);
            }
        }
        None
    }

    fn find_petaltongue_in_tmp() -> Option<PathBuf> {
        let entries = std::fs::read_dir("/tmp").ok()?;
        for entry in entries.flatten() {
            let path = entry.path();
            let name = path.file_name()?.to_str()?;
            if name.starts_with("petaltongue")
                && std::path::Path::new(name)
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("sock"))
                && Self::probe(&path)
            {
                return Some(path);
            }
        }
        None
    }
}

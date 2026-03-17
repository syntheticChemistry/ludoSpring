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
    /// Returns [`IpcError::NotFound`] if no visualization socket is found.
    pub fn discover() -> Result<Self, crate::ipc::IpcError> {
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

        if let Some(sock) = Self::find_viz_sock_in(&std::env::temp_dir()) {
            return Ok(Self { socket: sock });
        }

        Err(crate::ipc::IpcError::NotFound(
            "no visualization-capable primal found".into(),
        ))
    }

    /// Push a visualization render request.
    ///
    /// # Errors
    ///
    /// Returns a typed [`IpcError`](crate::ipc::IpcError) on failure.
    pub fn push_render(
        &self,
        session_id: &str,
        title: &str,
        payload: &serde_json::Value,
    ) -> Result<(), crate::ipc::IpcError> {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": VISUALIZATION_CAPABILITY,
            "params": {
                "session_id": session_id,
                "title": title,
                "domain": crate::niche::NICHE_DOMAIN,
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
    /// Returns a typed [`IpcError`](crate::ipc::IpcError) on failure.
    pub fn push_stream(
        &self,
        session_id: &str,
        action: &str,
        payload: &serde_json::Value,
    ) -> Result<(), crate::ipc::IpcError> {
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

    fn send(&self, request: &serde_json::Value) -> Result<(), crate::ipc::IpcError> {
        use crate::ipc::IpcError;

        let timeout = Duration::from_secs(crate::tolerances::RPC_TIMEOUT_SECS);
        let stream = UnixStream::connect(&self.socket).map_err(IpcError::Connect)?;
        stream
            .set_read_timeout(Some(timeout))
            .map_err(IpcError::Timeout)?;
        stream
            .set_write_timeout(Some(timeout))
            .map_err(IpcError::Timeout)?;

        let mut writer = stream.try_clone().map_err(IpcError::Io)?;
        let mut msg =
            serde_json::to_string(request).map_err(|e| IpcError::Serialization(e.to_string()))?;
        msg.push('\n');
        writer.write_all(msg.as_bytes()).map_err(IpcError::Io)?;
        writer.flush().map_err(IpcError::Io)?;

        let mut reader = BufReader::new(stream);
        let mut response = String::new();
        reader.read_line(&mut response).map_err(IpcError::Io)?;

        let parsed: serde_json::Value = serde_json::from_str(&response)
            .map_err(|e| IpcError::Serialization(e.to_string()))?;

        crate::ipc::extract_rpc_result(&parsed).map(|_| ())
    }

    fn probe(path: &std::path::Path) -> bool {
        UnixStream::connect(path)
            .and_then(|s| {
                s.set_read_timeout(Some(Duration::from_millis(
                    crate::tolerances::CONNECT_PROBE_TIMEOUT_MS,
                )))
            })
            .is_ok()
    }

    /// Push a scene graph to petalTongue for RPGPT game UI.
    ///
    /// Scene types from `rpgpt::scene` are serialized as JSON and routed
    /// to `visualization.render.scene`.
    ///
    /// # Errors
    ///
    /// Returns a typed [`IpcError`](crate::ipc::IpcError) on failure.
    pub fn push_scene(
        &self,
        session_id: &str,
        channel: &str,
        scene: &serde_json::Value,
    ) -> Result<(), crate::ipc::IpcError> {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "visualization.render.scene",
            "params": {
                "session_id": session_id,
                "channel": channel,
                "domain": crate::niche::NICHE_DOMAIN,
                "scene": scene,
            },
            "id": 1
        });
        self.send(&request)
    }

    /// Push a multi-panel dashboard (character sheet + map + narration + voices).
    ///
    /// # Errors
    ///
    /// Returns a typed [`IpcError`](crate::ipc::IpcError) on failure.
    pub fn push_dashboard(
        &self,
        session_id: &str,
        panels: &[serde_json::Value],
    ) -> Result<(), crate::ipc::IpcError> {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "visualization.render.dashboard",
            "params": {
                "session_id": session_id,
                "domain": crate::niche::NICHE_DOMAIN,
                "panels": panels,
            },
            "id": 1
        });
        self.send(&request)
    }

    /// Export a session replay (SVG timeline, audio archive).
    ///
    /// # Errors
    ///
    /// Returns a typed [`IpcError`](crate::ipc::IpcError) on failure.
    pub fn export(
        &self,
        session_id: &str,
        modality: &str,
    ) -> Result<serde_json::Value, crate::ipc::IpcError> {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "visualization.export",
            "params": {
                "session_id": session_id,
                "modality": modality,
            },
            "id": 1
        });
        self.send_with_result(&request)
    }

    /// Subscribe to interaction events (player clicks, key presses, selections).
    ///
    /// # Errors
    ///
    /// Returns a typed [`IpcError`](crate::ipc::IpcError) on failure.
    pub fn subscribe_interaction(
        &self,
        session_id: &str,
    ) -> Result<serde_json::Value, crate::ipc::IpcError> {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "interaction.subscribe",
            "params": {
                "session_id": session_id,
                "domain": crate::niche::NICHE_DOMAIN,
            },
            "id": 1
        });
        self.send_with_result(&request)
    }

    /// Run Tufte pre-flight validation on a game UI composition.
    ///
    /// # Errors
    ///
    /// Returns a typed [`IpcError`](crate::ipc::IpcError) on failure.
    pub fn validate(
        &self,
        bindings: &serde_json::Value,
    ) -> Result<serde_json::Value, crate::ipc::IpcError> {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "visualization.validate",
            "params": {
                "domain": crate::niche::NICHE_DOMAIN,
                "bindings": bindings,
            },
            "id": 1
        });
        self.send_with_result(&request)
    }

    fn send_with_result(
        &self,
        request: &serde_json::Value,
    ) -> Result<serde_json::Value, crate::ipc::IpcError> {
        use crate::ipc::IpcError;

        let timeout = Duration::from_secs(crate::tolerances::RPC_TIMEOUT_SECS);
        let stream = UnixStream::connect(&self.socket).map_err(IpcError::Connect)?;
        stream
            .set_read_timeout(Some(timeout))
            .map_err(IpcError::Timeout)?;
        stream
            .set_write_timeout(Some(timeout))
            .map_err(IpcError::Timeout)?;

        let mut writer = stream.try_clone().map_err(IpcError::Io)?;
        let mut msg =
            serde_json::to_string(request).map_err(|e| IpcError::Serialization(e.to_string()))?;
        msg.push('\n');
        writer.write_all(msg.as_bytes()).map_err(IpcError::Io)?;
        writer.flush().map_err(IpcError::Io)?;

        let mut reader = BufReader::new(stream);
        let mut response = String::new();
        reader.read_line(&mut response).map_err(IpcError::Io)?;

        let parsed: serde_json::Value = serde_json::from_str(&response)
            .map_err(|e| IpcError::Serialization(e.to_string()))?;

        crate::ipc::extract_rpc_result(&parsed)
    }

    /// Scan a directory for .sock files, verifying `visualization.render` capability.
    fn find_viz_sock_in(dir: &std::path::Path) -> Option<PathBuf> {
        let entries = std::fs::read_dir(dir).ok()?;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "sock")
                && Self::probe_with_capability(&path)
            {
                return Some(path);
            }
        }
        None
    }

    /// Probe a socket and verify it advertises `visualization.render`.
    fn probe_with_capability(path: &std::path::Path) -> bool {
        let Ok(stream) = UnixStream::connect(path) else {
            return false;
        };
        if stream
            .set_read_timeout(Some(Duration::from_millis(
                crate::tolerances::PROBE_TIMEOUT_MS,
            )))
            .is_err()
        {
            return false;
        }
        if stream
            .set_write_timeout(Some(Duration::from_millis(
                crate::tolerances::PROBE_TIMEOUT_MS,
            )))
            .is_err()
        {
            return false;
        }

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "lifecycle.status",
            "params": {},
            "id": 1
        });
        let Ok(mut msg) = serde_json::to_string(&request) else {
            return false;
        };
        msg.push('\n');

        let Ok(mut writer) = stream.try_clone() else {
            return false;
        };
        if writer.write_all(msg.as_bytes()).is_err() || writer.flush().is_err() {
            return false;
        }

        let mut reader = BufReader::new(stream);
        let mut response = String::new();
        if reader.read_line(&mut response).is_err() {
            return false;
        }

        let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&response) else {
            return false;
        };

        parsed
            .get("result")
            .and_then(|r| r.get("capabilities"))
            .and_then(|c| c.as_array())
            .is_some_and(|caps| {
                caps.iter()
                    .any(|c| c.as_str().is_some_and(|s| s.contains("visualization")))
            })
    }
}

/// Type alias preserved for backward compatibility with existing callers.
pub type PetalTonguePushClient = VisualizationPushClient;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discover_fails_without_viz_primal() {
        let result = VisualizationPushClient::discover();
        assert!(result.is_err());
    }

    #[test]
    fn probe_returns_false_for_nonexistent_path() {
        let result = VisualizationPushClient::probe(std::path::Path::new("/nonexistent.sock"));
        assert!(!result);
    }

    #[test]
    fn probe_with_capability_returns_false_for_nonexistent_path() {
        let result = VisualizationPushClient::probe_with_capability(std::path::Path::new(
            "/nonexistent.sock",
        ));
        assert!(!result);
    }

    #[test]
    fn find_viz_sock_nonexistent_dir_returns_none() {
        let result =
            VisualizationPushClient::find_viz_sock_in(std::path::Path::new("/nonexistent/dir"));
        assert!(result.is_none());
    }

    #[test]
    fn push_render_fails_without_connection() {
        let client = VisualizationPushClient {
            socket: PathBuf::from("/nonexistent.sock"),
        };
        let payload = serde_json::json!({"data": [1, 2, 3]});
        let result = client.push_render("sess-1", "Test Scene", &payload);
        assert!(result.is_err());
    }

    #[test]
    fn push_stream_fails_without_connection() {
        let client = VisualizationPushClient {
            socket: PathBuf::from("/nonexistent.sock"),
        };
        let payload = serde_json::json!({"value": 42});
        let result = client.push_stream("sess-1", "append", &payload);
        assert!(result.is_err());
    }

    #[test]
    fn push_scene_fails_without_connection() {
        let client = VisualizationPushClient {
            socket: PathBuf::from("/nonexistent.sock"),
        };
        let scene = serde_json::json!({"type": "dialogue"});
        let result = client.push_scene("sess-1", "rpgpt", &scene);
        assert!(result.is_err());
    }

    #[test]
    fn push_dashboard_fails_without_connection() {
        let client = VisualizationPushClient {
            socket: PathBuf::from("/nonexistent.sock"),
        };
        let panels = vec![serde_json::json!({"type": "map"})];
        let result = client.push_dashboard("sess-1", &panels);
        assert!(result.is_err());
    }

    #[test]
    fn type_alias_works() {
        let _: fn() -> Result<PetalTonguePushClient, _> = PetalTonguePushClient::discover;
    }
}

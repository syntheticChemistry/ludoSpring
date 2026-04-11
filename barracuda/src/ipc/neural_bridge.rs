// SPDX-License-Identifier: AGPL-3.0-or-later
//! Neural API bridge — typed IPC client for cross-primal capability routing.
//!
//! Follows the airSpring `NeuralBridge` pattern: discover the Neural API
//! socket at runtime, then route capability calls through biomeOS rather
//! than connecting directly to peer primals.
//!
//! # Usage
//!
//! ```rust,no_run
//! # fn main() -> Result<(), String> {
//! use ludospring_barracuda::ipc::NeuralBridge;
//!
//! let bridge = NeuralBridge::discover()?;
//!
//! // Route a capability call through biomeOS Neural API
//! let result = bridge.capability_call("crypto", "sign", &serde_json::json!({
//!     "data": "session-hash-abc123"
//! }))?;
//!
//! // Discover which primals serve a capability
//! let providers = bridge.discover_capability("visualization")?;
//! # Ok(())
//! # }
//! ```

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::time::Duration;

/// Default RPC timeout in seconds (overridable via `BIOMEOS_RPC_TIMEOUT_SECS`).
const DEFAULT_TIMEOUT_SECS: u64 = 5;

/// Typed client for the biomeOS Neural API.
///
/// All cross-primal communication flows through `capability.call` on the
/// Neural API socket. This primal never connects to peers directly — biomeOS
/// handles routing.
#[derive(Debug, Clone)]
pub struct NeuralBridge {
    socket: PathBuf,
    timeout: Duration,
}

impl NeuralBridge {
    /// Discover the Neural API socket using the XDG-compliant chain.
    ///
    /// # Errors
    ///
    /// Returns [`super::IpcError::NotFound`] if no Neural API socket is found.
    pub fn discover() -> Result<Self, super::envelope::IpcError> {
        let socket = crate::niche::resolve_neural_api_socket().ok_or_else(|| {
            super::envelope::IpcError::NotFound(
                "Neural API not found in any standard location".into(),
            )
        })?;

        let timeout_secs = std::env::var("BIOMEOS_RPC_TIMEOUT_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(DEFAULT_TIMEOUT_SECS);

        Ok(Self {
            socket,
            timeout: Duration::from_secs(timeout_secs),
        })
    }

    /// Create a bridge pointing at a specific socket (useful for testing).
    #[must_use]
    pub const fn with_socket(socket: PathBuf) -> Self {
        Self {
            socket,
            timeout: Duration::from_secs(DEFAULT_TIMEOUT_SECS),
        }
    }

    /// Bridge with explicit socket and RPC timeout (integration tests, tooling).
    #[must_use]
    pub const fn with_socket_and_timeout(socket: PathBuf, timeout: Duration) -> Self {
        Self { socket, timeout }
    }

    /// Whether the Neural API is reachable (connect + health check).
    #[must_use]
    pub fn is_available(&self) -> bool {
        self.rpc_send("health.check", &serde_json::json!({}))
            .is_ok()
    }

    /// Route a `capability.call` through the Neural API.
    ///
    /// biomeOS resolves the capability to the appropriate primal and forwards
    /// the call. This primal never needs to know which primal serves the
    /// capability.
    ///
    /// # Errors
    ///
    /// Returns a typed [`IpcError`](super::envelope::IpcError) on failure.
    pub fn capability_call(
        &self,
        capability: &str,
        operation: &str,
        args: &serde_json::Value,
    ) -> Result<serde_json::Value, super::envelope::IpcError> {
        let params = serde_json::json!({
            "capability": capability,
            "operation": operation,
            "args": args,
        });
        self.rpc_send("capability.call", &params)
    }

    /// Discover which primals serve a given capability.
    ///
    /// # Errors
    ///
    /// Returns a typed [`IpcError`](super::envelope::IpcError) on failure.
    pub fn discover_capability(
        &self,
        capability: &str,
    ) -> Result<serde_json::Value, super::envelope::IpcError> {
        let params = serde_json::json!({
            "capability": capability,
        });
        self.rpc_send("capability.discover", &params)
    }

    /// Register this primal's capabilities with the Neural API.
    ///
    /// Uses [`crate::niche`] for all identity and capability metadata.
    ///
    /// # Errors
    ///
    /// Returns a typed [`IpcError`](super::envelope::IpcError) on failure.
    pub fn register(
        &self,
        our_socket: &std::path::Path,
    ) -> Result<serde_json::Value, super::envelope::IpcError> {
        let mappings: serde_json::Value = crate::niche::SEMANTIC_MAPPINGS
            .iter()
            .map(|(short, full)| {
                (
                    (*short).to_string(),
                    serde_json::json!({
                        "provider": crate::niche::NICHE_NAME,
                        "method": full,
                    }),
                )
            })
            .collect::<serde_json::Map<String, serde_json::Value>>()
            .into();

        let params = serde_json::json!({
            "name": crate::niche::NICHE_NAME,
            "domain": crate::niche::NICHE_DOMAIN,
            "socket_path": our_socket.to_string_lossy(),
            "pid": std::process::id(),
            "capabilities": crate::niche::CAPABILITIES,
            "semantic_mappings": mappings,
            "operation_dependencies": crate::niche::operation_dependencies(),
            "cost_estimates": crate::niche::cost_estimates(),
        });

        self.rpc_send("lifecycle.register", &params)
    }

    /// Deregister this primal from the Neural API.
    ///
    /// # Errors
    ///
    /// Returns a typed [`IpcError`](super::envelope::IpcError) on failure.
    pub fn deregister(&self) -> Result<serde_json::Value, super::envelope::IpcError> {
        let params = serde_json::json!({
            "domain": crate::niche::NICHE_DOMAIN,
            "provider": crate::niche::NICHE_NAME,
        });
        self.rpc_send("capability.deregister", &params)
    }

    /// The resolved socket path for this bridge.
    #[must_use]
    pub fn socket_path(&self) -> &std::path::Path {
        &self.socket
    }

    fn rpc_send(
        &self,
        method: &str,
        params: &serde_json::Value,
    ) -> Result<serde_json::Value, super::envelope::IpcError> {
        use super::envelope::IpcError;

        let stream = UnixStream::connect(&self.socket).map_err(IpcError::Connect)?;
        stream
            .set_read_timeout(Some(self.timeout))
            .map_err(IpcError::Timeout)?;
        stream
            .set_write_timeout(Some(self.timeout))
            .map_err(IpcError::Timeout)?;

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1
        });

        let mut writer = stream.try_clone().map_err(IpcError::Io)?;
        let mut msg =
            serde_json::to_string(&request).map_err(|e| IpcError::Serialization(e.to_string()))?;
        msg.push('\n');
        writer.write_all(msg.as_bytes()).map_err(IpcError::Io)?;
        writer.flush().map_err(IpcError::Io)?;

        let mut reader = BufReader::new(stream);
        let mut response = String::new();
        reader.read_line(&mut response).map_err(IpcError::Io)?;

        let parsed: serde_json::Value =
            serde_json::from_str(&response).map_err(|e| IpcError::Serialization(e.to_string()))?;

        super::envelope::extract_rpc_result(&parsed)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn with_nonexistent_socket_is_unavailable() {
        let bridge = NeuralBridge::with_socket(PathBuf::from("/nonexistent/neural-api.sock"));
        assert!(!bridge.is_available());
    }

    #[test]
    fn with_socket_sets_path() {
        let socket_path = std::env::temp_dir().join(format!(
            "ludospring-test-neural-with-socket-sets-path-{}.sock",
            std::process::id()
        ));
        let bridge = NeuralBridge::with_socket(socket_path.clone());
        assert_eq!(bridge.socket_path().to_str(), socket_path.to_str());
    }

    #[test]
    fn timeout_default_is_5_seconds() {
        let socket_path = std::env::temp_dir().join(format!(
            "ludospring-test-neural-timeout-default-{}.sock",
            std::process::id()
        ));
        let bridge = NeuralBridge::with_socket(socket_path);
        assert_eq!(bridge.timeout, Duration::from_secs(5));
    }

    #[test]
    fn discover_matches_socket_resolution() {
        match crate::niche::resolve_neural_api_socket() {
            None => assert!(NeuralBridge::discover().is_err()),
            Some(path) => {
                let bridge = NeuralBridge::discover().expect("Neural API socket present");
                assert_eq!(bridge.socket_path(), path.as_path());
            }
        }
    }

    #[cfg(all(unix, feature = "ipc"))]
    mod unix_ipc {
        use super::*;
        use std::io::{BufRead, BufReader, Write};
        use std::os::unix::net::UnixListener;
        use std::thread;
        use std::time::Duration;

        #[test]
        fn connect_failure_maps_to_error() {
            let path = std::env::temp_dir().join(format!(
                "ludospring-neural-no-listener-{}.sock",
                std::process::id()
            ));
            let bridge = NeuralBridge::with_socket_and_timeout(path, Duration::from_secs(1));
            let err = bridge
                .capability_call("game", "evaluate_flow", &serde_json::json!({}))
                .expect_err("unreachable socket");
            assert!(
                matches!(err, crate::ipc::IpcError::Connect(_)),
                "expected connect error, got {err:?}"
            );
        }

        #[test]
        fn read_timeout_on_slow_peer() {
            let dir =
                std::env::temp_dir().join(format!("ludospring-neural-slow-{}", std::process::id()));
            std::fs::create_dir_all(&dir).expect("dir");
            let path = dir.join("neural.sock");
            let _ = std::fs::remove_file(&path);
            let listener = UnixListener::bind(&path).expect("bind");
            let path_clone = path.clone();
            let slow = thread::spawn(move || {
                if let Ok((stream, _)) = listener.accept() {
                    let mut line = String::new();
                    let mut br = BufReader::new(&stream);
                    let _ = br.read_line(&mut line);
                    thread::sleep(Duration::from_secs(10));
                }
                drop(listener);
                let _ = std::fs::remove_file(&path_clone);
                let _ = std::fs::remove_dir(&dir);
            });

            thread::sleep(Duration::from_millis(30));
            let bridge = NeuralBridge::with_socket_and_timeout(path, Duration::from_millis(80));
            let err = bridge
                .capability_call(
                    "game",
                    "evaluate_flow",
                    &serde_json::json!({"challenge": 0.5, "skill": 0.5}),
                )
                .expect_err("slow peer");
            assert!(
                matches!(
                    err,
                    crate::ipc::IpcError::Io(_) | crate::ipc::IpcError::Timeout(_)
                ),
                "got {err:?}"
            );

            let _ = slow.join();
        }

        #[test]
        fn malformed_response_line_is_serialization_error() {
            let dir = std::env::temp_dir()
                .join(format!("ludospring-neural-badjson-{}", std::process::id()));
            std::fs::create_dir_all(&dir).expect("dir");
            let path = dir.join("neural.sock");
            let _ = std::fs::remove_file(&path);
            let listener = UnixListener::bind(&path).expect("bind");
            let path_clone = path.clone();
            let bad = thread::spawn(move || {
                if let Ok((mut stream, _)) = listener.accept() {
                    let mut line = String::new();
                    let _ = BufReader::new(&stream).read_line(&mut line);
                    let _ = stream.write_all(b"not valid json rpc\n");
                    let _ = stream.flush();
                }
                drop(listener);
                let _ = std::fs::remove_file(&path_clone);
                let _ = std::fs::remove_dir(dir);
            });

            thread::sleep(Duration::from_millis(30));
            let bridge = NeuralBridge::with_socket_and_timeout(path, Duration::from_secs(2));
            let err = bridge
                .capability_call("x", "y", &serde_json::json!({}))
                .expect_err("bad json");
            assert!(
                matches!(err, crate::ipc::IpcError::Serialization(_)),
                "expected serialization error, got {err:?}"
            );
            let _ = bad.join();
        }

        #[test]
        fn rpc_error_response_maps() {
            let dir = std::env::temp_dir()
                .join(format!("ludospring-neural-rpcerr-{}", std::process::id()));
            std::fs::create_dir_all(&dir).expect("dir");
            let path = dir.join("neural.sock");
            let _ = std::fs::remove_file(&path);
            let listener = UnixListener::bind(&path).expect("bind");
            let path_clone = path.clone();
            let srv = thread::spawn(move || {
                if let Ok((mut stream, _)) = listener.accept() {
                    let mut line = String::new();
                    let _ = BufReader::new(&stream).read_line(&mut line);
                    let body = serde_json::json!({
                        "jsonrpc": "2.0",
                        "error": {"code": -32000, "message": "test rpc"},
                        "id": 1
                    });
                    let mut s = body.to_string();
                    s.push('\n');
                    let _ = stream.write_all(s.as_bytes());
                }
                drop(listener);
                let _ = std::fs::remove_file(&path_clone);
                let _ = std::fs::remove_dir(dir);
            });

            thread::sleep(Duration::from_millis(30));
            let bridge = NeuralBridge::with_socket_and_timeout(path, Duration::from_secs(2));
            let err = bridge
                .capability_call("a", "b", &serde_json::json!({}))
                .expect_err("rpc error");
            match err {
                crate::ipc::IpcError::RpcError { code, message } => {
                    assert_eq!(code, -32000);
                    assert_eq!(message, "test rpc");
                }
                other => panic!("expected RpcError, got {other:?}"),
            }
            let _ = srv.join();
        }
    }
}

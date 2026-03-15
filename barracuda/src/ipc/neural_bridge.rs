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
    /// Returns an error if no Neural API socket is found.
    pub fn discover() -> Result<Self, String> {
        let socket = crate::niche::resolve_neural_api_socket()
            .ok_or("Neural API not found in any standard location")?;

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
    /// Returns an error if the connection, serialization, or RPC call fails.
    pub fn capability_call(
        &self,
        capability: &str,
        operation: &str,
        args: &serde_json::Value,
    ) -> Result<serde_json::Value, String> {
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
    /// Returns an error if the discovery call fails.
    pub fn discover_capability(&self, capability: &str) -> Result<serde_json::Value, String> {
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
    /// Returns an error if the registration call fails.
    pub fn register(&self, our_socket: &std::path::Path) -> Result<serde_json::Value, String> {
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
    /// Returns an error if the deregistration call fails.
    pub fn deregister(&self) -> Result<serde_json::Value, String> {
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
    ) -> Result<serde_json::Value, String> {
        let stream = UnixStream::connect(&self.socket).map_err(|e| format!("connect: {e}"))?;
        stream
            .set_read_timeout(Some(self.timeout))
            .map_err(|e| format!("set_read_timeout: {e}"))?;
        stream
            .set_write_timeout(Some(self.timeout))
            .map_err(|e| format!("set_write_timeout: {e}"))?;

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1
        });

        let mut writer = stream.try_clone().map_err(|e| format!("clone: {e}"))?;
        let mut msg = serde_json::to_string(&request).map_err(|e| format!("serialize: {e}"))?;
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
            return Err(format!(
                "rpc error: {}",
                error
                    .get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or("unknown")
            ));
        }

        parsed
            .get("result")
            .cloned()
            .ok_or_else(|| "no result in response".to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_nonexistent_socket_is_unavailable() {
        let bridge = NeuralBridge::with_socket(PathBuf::from("/nonexistent/neural-api.sock"));
        assert!(!bridge.is_available());
    }

    #[test]
    fn with_socket_sets_path() {
        let bridge = NeuralBridge::with_socket(PathBuf::from("/tmp/test-neural.sock"));
        assert_eq!(bridge.socket_path().to_str(), Some("/tmp/test-neural.sock"));
    }

    #[test]
    fn timeout_default_is_5_seconds() {
        let bridge = NeuralBridge::with_socket(PathBuf::from("/tmp/test.sock"));
        assert_eq!(bridge.timeout, Duration::from_secs(5));
    }
}

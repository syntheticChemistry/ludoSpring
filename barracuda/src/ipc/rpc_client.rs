// SPDX-License-Identifier: AGPL-3.0-or-later
//! Shared Unix socket JSON-RPC 2.0 client.
//!
//! Deduplicates the connect → set_timeout → try_clone → write → read → parse
//! pattern that was previously inlined in `neural_bridge.rs`, `discovery/mod.rs`,
//! `btsp.rs`, and `visualization/push_client.rs`.

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;
use std::time::Duration;

use super::envelope::{IpcError, extract_rpc_result};

/// Minimal JSON-RPC 2.0 client over a Unix domain socket.
///
/// Each [`call`](Self::call) opens a fresh connection (primals use
/// short-lived sockets). The `timeout` governs both read and write.
#[derive(Debug)]
pub struct RpcClient {
    socket: std::path::PathBuf,
    timeout: Duration,
}

impl RpcClient {
    /// Create a client targeting the given socket path.
    #[must_use]
    pub fn new(socket: impl Into<std::path::PathBuf>, timeout: Duration) -> Self {
        Self {
            socket: socket.into(),
            timeout,
        }
    }

    /// Send a JSON-RPC call and return the parsed `result` field.
    ///
    /// Builds the envelope, writes it as newline-delimited JSON, reads one
    /// response line, and extracts the `result` (or converts an `error`
    /// field into [`IpcError::RpcError`]).
    ///
    /// # Errors
    ///
    /// Returns a typed [`IpcError`] on connect, I/O, serialization, or
    /// JSON-RPC error response.
    pub fn call(
        &self,
        method: &str,
        params: &serde_json::Value,
    ) -> Result<serde_json::Value, IpcError> {
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1
        });
        let parsed = self.send_raw(&request)?;
        extract_rpc_result(&parsed)
    }

    /// Send an already-formed JSON-RPC request and return the raw response.
    ///
    /// Useful when the caller constructs the full envelope (e.g. BTSP
    /// handshake frames that include their own `"method"` / `"params"`).
    ///
    /// # Errors
    ///
    /// Returns a typed [`IpcError`] on connect, I/O, or parse failure.
    pub fn send_raw(&self, request: &serde_json::Value) -> Result<serde_json::Value, IpcError> {
        let stream = UnixStream::connect(&self.socket).map_err(IpcError::Connect)?;
        stream
            .set_read_timeout(Some(self.timeout))
            .map_err(IpcError::Timeout)?;
        stream
            .set_write_timeout(Some(self.timeout))
            .map_err(IpcError::Timeout)?;

        let mut writer = stream.try_clone().map_err(IpcError::Io)?;
        let mut msg = serde_json::to_string(request)?;
        msg.push('\n');
        writer.write_all(msg.as_bytes()).map_err(IpcError::Io)?;
        writer.flush().map_err(IpcError::Io)?;

        let mut reader = BufReader::new(stream);
        let mut response = String::new();
        reader.read_line(&mut response).map_err(IpcError::Io)?;

        Ok(serde_json::from_str(&response)?)
    }

    /// The socket path this client targets.
    #[must_use]
    pub fn socket_path(&self) -> &Path {
        &self.socket
    }

    /// The configured timeout.
    #[must_use]
    pub const fn timeout(&self) -> Duration {
        self.timeout
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn connect_to_missing_socket_returns_connect_error() {
        let client = RpcClient::new(
            "/tmp/nonexistent-ludospring-test.sock",
            Duration::from_secs(1),
        );
        let err = client
            .call("health.liveness", &serde_json::json!({}))
            .unwrap_err();
        assert!(
            err.is_connection_error(),
            "expected connection error, got: {err}"
        );
    }
}

// SPDX-License-Identifier: AGPL-3.0-or-later
//! Unix domain socket server for JSON-RPC 2.0.
//!
//! Blocking I/O with std — no async runtime dependency.
//! Per wateringHole `UNIVERSAL_IPC_STANDARD_V3`: transport priority is
//! native Unix socket → TCP fallback.

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use super::envelope::JsonRpcRequest;
use super::handlers::dispatch;
use crate::PRIMAL_NAME;

use tracing::{error, info, warn};

/// Resolve the socket path using XDG-compliant priority:
///
/// 1. `LUDOSPRING_SOCK` — explicit override
/// 2. `BIOMEOS_SOCKET_DIR` — ecosystem convention
/// 3. `$XDG_RUNTIME_DIR/biomeos/ludospring.sock`
/// 4. `/tmp/ludospring.sock` — development fallback
fn resolve_socket_path() -> PathBuf {
    if let Ok(explicit) = std::env::var("LUDOSPRING_SOCK") {
        return PathBuf::from(explicit);
    }

    if let Ok(biomeos_dir) = std::env::var("BIOMEOS_SOCKET_DIR") {
        // Socket name derived from PRIMAL_NAME — no hardcoded peer names.
        return PathBuf::from(biomeos_dir).join(format!("{PRIMAL_NAME}.sock"));
    }

    if let Ok(xdg) = std::env::var("XDG_RUNTIME_DIR") {
        let biomeos_path = PathBuf::from(xdg).join("biomeos");
        if biomeos_path.is_dir() || std::fs::create_dir_all(&biomeos_path).is_ok() {
            return biomeos_path.join(format!("{PRIMAL_NAME}.sock"));
        }
    }

    // Socket name derived from PRIMAL_NAME — no hardcoded peer names.
    PathBuf::from("/tmp").join(format!("{PRIMAL_NAME}.sock"))
}

/// IPC server state.
pub struct IpcServer {
    socket_path: PathBuf,
}

impl IpcServer {
    /// Create a server with XDG-compliant socket path resolution.
    #[must_use]
    pub fn new() -> Self {
        Self {
            socket_path: resolve_socket_path(),
        }
    }

    /// Create a server bound to a specific path (useful for testing).
    #[must_use]
    pub fn with_path(path: impl Into<PathBuf>) -> Self {
        Self {
            socket_path: path.into(),
        }
    }

    /// Socket path this server will bind to.
    #[must_use]
    pub fn socket_path(&self) -> &Path {
        &self.socket_path
    }

    /// Run the server (blocking). Returns on I/O error.
    ///
    /// # Errors
    ///
    /// Returns `std::io::Error` if the socket cannot be bound or a
    /// connection produces an unrecoverable I/O error.
    pub fn run(&self) -> std::io::Result<()> {
        if self.socket_path.exists() {
            std::fs::remove_file(&self.socket_path)?;
        }

        let listener = UnixListener::bind(&self.socket_path)?;
        info!(path = %self.socket_path.display(), "IPC listening");

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    if let Err(e) = Self::handle_connection(&stream) {
                        warn!(error = %e, "IPC connection error");
                    }
                }
                Err(e) => error!(error = %e, "IPC accept error"),
            }
        }
        Ok(())
    }

    /// Run the server until `shutdown` is set to true.
    ///
    /// Uses non-blocking accept with polling to allow clean shutdown on SIGTERM.
    /// Returns when shutdown is requested or on I/O error.
    ///
    /// # Errors
    ///
    /// Returns `std::io::Error` if the socket cannot be bound.
    pub fn run_until(&self, shutdown: &AtomicBool) -> std::io::Result<()> {
        if self.socket_path.exists() {
            std::fs::remove_file(&self.socket_path)?;
        }

        let listener = UnixListener::bind(&self.socket_path)?;
        listener.set_nonblocking(true)?;

        while !shutdown.load(Ordering::Relaxed) {
            match listener.accept() {
                Ok((stream, _)) => {
                    if let Err(e) = Self::handle_connection(&stream) {
                        warn!(error = %e, "IPC connection error");
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(50));
                }
                Err(e) => error!(error = %e, "IPC accept error"),
            }
        }
        Ok(())
    }

    fn handle_connection(stream: &std::os::unix::net::UnixStream) -> std::io::Result<()> {
        let reader = BufReader::new(stream);
        let mut writer = stream;

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            let response = match serde_json::from_str::<JsonRpcRequest>(&line) {
                Ok(req) => dispatch(&req),
                Err(e) => serde_json::json!({
                    "jsonrpc": "2.0",
                    "error": {"code": -32700, "message": format!("parse error: {e}")},
                    "id": null
                })
                .to_string(),
            };

            writer.write_all(response.as_bytes())?;
            writer.write_all(b"\n")?;
            writer.flush()?;
        }
        Ok(())
    }
}

impl Default for IpcServer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::os::unix::net::UnixStream;

    #[test]
    fn server_round_trip() {
        let dir = std::env::temp_dir().join(format!("ludospring_test_{}", std::process::id()));
        std::fs::create_dir_all(&dir).ok();
        let sock = dir.join("test.sock");

        let sock_clone = sock.clone();
        let handle = std::thread::spawn(move || {
            let server = IpcServer::with_path(&sock_clone);
            let _ = server.run();
        });

        for _ in 0..50 {
            if sock.exists() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }

        if sock.exists() {
            let mut stream = UnixStream::connect(&sock)
                .unwrap_or_else(|e| panic!("connect to test socket: {e}"));
            let req = r#"{"jsonrpc":"2.0","method":"game.evaluate_flow","params":{"challenge":0.5,"skill":0.5},"id":42}"#;
            stream.write_all(req.as_bytes()).ok();
            stream.write_all(b"\n").ok();
            stream.flush().ok();

            stream.shutdown(std::net::Shutdown::Write).ok();

            let mut buf = String::new();
            stream.read_to_string(&mut buf).ok();
            assert!(buf.contains("flow"), "expected flow in response: {buf}");
        }

        std::fs::remove_file(&sock).ok();
        std::fs::remove_dir(&dir).ok();
        drop(handle);
    }
}

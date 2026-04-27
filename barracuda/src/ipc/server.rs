// SPDX-License-Identifier: AGPL-3.0-or-later
//! Unix domain socket server for JSON-RPC 2.0 with BTSP auto-detect.
//!
//! Blocking I/O with std — no async runtime dependency.
//! Per wateringHole `UNIVERSAL_IPC_STANDARD_V3`: transport priority is
//! native Unix socket → TCP fallback.
//!
//! BTSP relay: when `FAMILY_ID` is set, the server auto-detects BTSP
//! ClientHello on first line and relays the 4-step handshake through
//! BearDog before entering normal JSON-RPC dispatch.

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use super::btsp;
use super::envelope::JsonRpcRequest;
use super::handlers::dispatch;

use tracing::{debug, error, info, warn};

const ACCEPT_POLL_MS: u64 = 50;

/// Resolve the socket path using XDG-compliant priority.
///
/// Delegates to [`crate::niche::resolve_server_socket`] for the full chain.
fn resolve_socket_path() -> PathBuf {
    crate::niche::resolve_server_socket()
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
                    std::thread::sleep(Duration::from_millis(ACCEPT_POLL_MS));
                }
                Err(e) => error!(error = %e, "IPC accept error"),
            }
        }
        Ok(())
    }

    fn handle_connection(stream: &std::os::unix::net::UnixStream) -> std::io::Result<()> {
        let mut reader = BufReader::new(stream);
        let mut writer = stream;

        if btsp::btsp_required() {
            match btsp::classify_first_line(&mut reader) {
                Ok(btsp::FirstLineResult::BtspHello(hello_line)) => {
                    debug!("BTSP ClientHello detected — relaying handshake");
                    if let Err(e) = btsp::perform_handshake(&hello_line, &mut reader, &mut writer) {
                        warn!(error = %e, "BTSP handshake failed");
                        return Ok(());
                    }
                }
                Ok(btsp::FirstLineResult::PlainJsonRpc(first_line)) => {
                    debug!("plain JSON-RPC on BTSP-required socket");
                    Self::dispatch_line(&first_line, &mut writer)?;
                }
                Err(e) => {
                    warn!(error = %e, "BTSP first-line classification failed");
                    return Ok(());
                }
            }
        }

        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }
            Self::dispatch_line(&line, &mut writer)?;
        }
        Ok(())
    }

    fn dispatch_line<W: Write>(line: &str, writer: &mut W) -> std::io::Result<()> {
        let response = match serde_json::from_str::<JsonRpcRequest>(line) {
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
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::thread::JoinHandle;
    use std::time::Duration;

    const REQ_FLOW: &str = r#"{"jsonrpc":"2.0","method":"game.evaluate_flow","params":{"challenge":0.5,"skill":0.5},"id":42}"#;

    static TEST_SOCKET_SEQ: AtomicU64 = AtomicU64::new(0);

    fn test_socket_dir() -> PathBuf {
        let n = TEST_SOCKET_SEQ.fetch_add(1, Ordering::SeqCst);
        std::env::temp_dir().join(format!("ludospring_test_{}_{}", std::process::id(), n))
    }

    fn spawn_test_server(sock: PathBuf) -> JoinHandle<()> {
        std::thread::spawn(move || {
            let server = IpcServer::with_path(&sock);
            let _ = server.run();
        })
    }

    fn wait_for_socket(sock: &Path) -> bool {
        for _ in 0..50 {
            if sock.exists() {
                return true;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        false
    }

    fn cleanup_test_socket(sock: &Path, dir: &Path) {
        std::fs::remove_file(sock).ok();
        std::fs::remove_dir(dir).ok();
    }

    #[test]
    fn server_round_trip() {
        let dir = test_socket_dir();
        std::fs::create_dir_all(&dir).ok();
        let sock = dir.join("test.sock");

        let sock_clone = sock.clone();
        let handle = spawn_test_server(sock_clone);

        if wait_for_socket(&sock) {
            let mut stream = UnixStream::connect(&sock)
                .unwrap_or_else(|e| panic!("connect to test socket: {e}"));
            stream.write_all(REQ_FLOW.as_bytes()).ok();
            stream.write_all(b"\n").ok();
            stream.flush().ok();

            stream.shutdown(std::net::Shutdown::Write).ok();

            let mut buf = String::new();
            stream.read_to_string(&mut buf).ok();
            assert!(buf.contains("flow"), "expected flow in response: {buf}");
        }

        cleanup_test_socket(&sock, &dir);
        drop(handle);
    }

    #[test]
    fn malformed_json_returns_parse_error() {
        let dir = test_socket_dir();
        std::fs::create_dir_all(&dir).ok();
        let sock = dir.join("test.sock");

        let sock_clone = sock.clone();
        let handle = spawn_test_server(sock_clone);

        assert!(wait_for_socket(&sock), "socket did not appear");

        let mut stream =
            UnixStream::connect(&sock).unwrap_or_else(|e| panic!("connect to test socket: {e}"));
        stream
            .write_all(b"{not valid json}\n")
            .unwrap_or_else(|e| panic!("write: {e}"));
        stream.flush().ok();
        stream.shutdown(std::net::Shutdown::Write).ok();

        let mut buf = String::new();
        stream
            .read_to_string(&mut buf)
            .unwrap_or_else(|e| panic!("read: {e}"));

        let line = buf.lines().next().unwrap_or("");
        let v: serde_json::Value =
            serde_json::from_str(line).unwrap_or_else(|e| panic!("response JSON: {e} {line}"));
        assert_eq!(v["error"]["code"], -32700);

        cleanup_test_socket(&sock, &dir);
        drop(handle);
    }

    #[test]
    fn empty_line_is_ignored() {
        let dir = test_socket_dir();
        std::fs::create_dir_all(&dir).ok();
        let sock = dir.join("test.sock");

        let sock_clone = sock.clone();
        let handle = spawn_test_server(sock_clone);

        assert!(wait_for_socket(&sock), "socket did not appear");

        let mut stream =
            UnixStream::connect(&sock).unwrap_or_else(|e| panic!("connect to test socket: {e}"));
        stream.write_all(b"\n\n").ok();
        stream.write_all(REQ_FLOW.as_bytes()).ok();
        stream.write_all(b"\n").ok();
        stream.flush().ok();
        stream.shutdown(std::net::Shutdown::Write).ok();

        let mut buf = String::new();
        stream.read_to_string(&mut buf).ok();
        assert!(buf.contains("flow"), "expected flow in response: {buf}");

        cleanup_test_socket(&sock, &dir);
        drop(handle);
    }

    #[test]
    fn oversized_request_does_not_crash() {
        let dir = test_socket_dir();
        std::fs::create_dir_all(&dir).ok();
        let sock = dir.join("test.sock");

        let sock_clone = sock.clone();
        let handle = spawn_test_server(sock_clone);

        assert!(wait_for_socket(&sock), "socket did not appear");

        let pad = "x".repeat(100_000);
        let req = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "game.evaluate_flow",
            "params": {"challenge": 0.5, "skill": 0.5, "pad": pad},
            "id": 42
        });
        let line = req.to_string();

        let mut stream =
            UnixStream::connect(&sock).unwrap_or_else(|e| panic!("connect to test socket: {e}"));
        stream.write_all(line.as_bytes()).ok();
        stream.write_all(b"\n").ok();
        stream.flush().ok();
        stream.shutdown(std::net::Shutdown::Write).ok();

        let mut buf = String::new();
        stream.read_to_string(&mut buf).ok();
        assert!(
            buf.contains("flow") || buf.contains("error"),
            "expected response body: len={}",
            buf.len()
        );

        let mut stream2 =
            UnixStream::connect(&sock).unwrap_or_else(|e| panic!("second connect: {e}"));
        stream2.write_all(REQ_FLOW.as_bytes()).ok();
        stream2.write_all(b"\n").ok();
        stream2.flush().ok();
        stream2.shutdown(std::net::Shutdown::Write).ok();
        let mut buf2 = String::new();
        stream2.read_to_string(&mut buf2).ok();
        assert!(buf2.contains("flow"), "server should still respond: {buf2}");

        cleanup_test_socket(&sock, &dir);
        drop(handle);
    }

    #[test]
    fn concurrent_connections() {
        let dir = test_socket_dir();
        std::fs::create_dir_all(&dir).ok();
        let sock = dir.join("test.sock");

        let sock_clone = sock.clone();
        let handle = spawn_test_server(sock_clone);

        assert!(wait_for_socket(&sock), "socket did not appear");

        let sock_path = sock.clone();
        let clients: Vec<_> = (0..3)
            .map(|_| {
                let p = sock_path.clone();
                std::thread::spawn(move || {
                    let mut stream =
                        UnixStream::connect(&p).unwrap_or_else(|e| panic!("connect: {e}"));
                    stream.write_all(REQ_FLOW.as_bytes()).ok();
                    stream.write_all(b"\n").ok();
                    stream.flush().ok();
                    stream.shutdown(std::net::Shutdown::Write).ok();
                    let mut buf = String::new();
                    stream.read_to_string(&mut buf).ok();
                    buf
                })
            })
            .collect();

        for c in clients {
            let buf = c.join().unwrap_or_else(|e| panic!("client thread: {e:?}"));
            assert!(buf.contains("flow"), "expected flow in response: {buf}");
        }

        cleanup_test_socket(&sock, &dir);
        drop(handle);
    }

    #[test]
    fn partial_write_then_close() {
        let dir = test_socket_dir();
        std::fs::create_dir_all(&dir).ok();
        let sock = dir.join("test.sock");

        let sock_clone = sock.clone();
        let handle = spawn_test_server(sock_clone);

        assert!(wait_for_socket(&sock), "socket did not appear");

        {
            let mut stream = UnixStream::connect(&sock)
                .unwrap_or_else(|e| panic!("connect to test socket: {e}"));
            stream
                .write_all(br#"{"jsonrpc":"2.0","meth"#)
                .unwrap_or_else(|e| panic!("write: {e}"));
        }

        let mut stream2 =
            UnixStream::connect(&sock).unwrap_or_else(|e| panic!("second connect: {e}"));
        stream2.write_all(REQ_FLOW.as_bytes()).ok();
        stream2.write_all(b"\n").ok();
        stream2.flush().ok();
        stream2.shutdown(std::net::Shutdown::Write).ok();
        let mut buf = String::new();
        stream2.read_to_string(&mut buf).ok();
        assert!(buf.contains("flow"), "server should still respond: {buf}");

        cleanup_test_socket(&sock, &dir);
        drop(handle);
    }
}

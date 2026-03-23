// SPDX-License-Identifier: AGPL-3.0-or-later
//! Shared helpers for IPC integration tests: temp Unix sockets and `IpcServer` lifecycle.

#![cfg(feature = "ipc")]

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;

static DIR_SEQ: AtomicU64 = AtomicU64::new(0);

/// Unique per-process temp directory for sockets (avoids collisions under parallel tests).
pub fn ipc_test_socket_dir() -> PathBuf {
    let n = DIR_SEQ.fetch_add(1, Ordering::SeqCst);
    std::env::temp_dir().join(format!("ludospring_ipc_util_{}_{n}", std::process::id()))
}

/// Running [`ludospring_barracuda::ipc::IpcServer`] on a Unix socket, stopped via [`IpcTestServer::shutdown`].
pub struct IpcTestServer {
    /// Path to the bound Unix socket.
    pub socket_path: PathBuf,
    shutdown: Arc<AtomicBool>,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl IpcTestServer {
    /// Binds [`IpcServer`](ludospring_barracuda::ipc::IpcServer) on `ipc_test_socket_dir()/server.sock`.
    pub fn start() -> Self {
        let dir = ipc_test_socket_dir();
        std::fs::create_dir_all(&dir).expect("ipc test socket dir");
        let sock = dir.join("server.sock");
        let shutdown = Arc::new(AtomicBool::new(false));
        let shutdown_clone = Arc::clone(&shutdown);
        let sock_clone = sock.clone();
        let handle = std::thread::spawn(move || {
            let server = ludospring_barracuda::ipc::IpcServer::with_path(&sock_clone);
            let _ = server.run_until(&shutdown_clone);
        });
        for _ in 0..100 {
            if sock.exists() {
                return Self {
                    socket_path: sock,
                    shutdown,
                    handle: Some(handle),
                };
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        shutdown.store(true, Ordering::Relaxed);
        let _ = handle.join();
        panic!("IPC test server did not bind socket in time");
    }

    /// Remove socket file, stop server thread, and drop temp dir.
    pub fn shutdown(mut self) {
        self.shutdown.store(true, Ordering::Relaxed);
        let sock = &self.socket_path;
        std::fs::remove_file(sock).ok();
        if let Some(parent) = sock.parent() {
            std::fs::remove_dir(parent).ok();
        }
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}

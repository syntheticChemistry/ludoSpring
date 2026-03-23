// SPDX-License-Identifier: AGPL-3.0-or-later
//! biomeOS niche deployment integration for ludoSpring.
//!
//! Per the Spring-as-Niche Deployment Standard, this module handles:
//! - **Registration**: `lifecycle.register` / `capability.deregister` via `NeuralBridge`
//! - **Neural API communication**: typed client from `crate::ipc::NeuralBridge`
//!
//! All identity, capabilities, and semantic mappings come from [`crate::niche`]
//! (single source of truth). This module never hardcodes peer primal names.

use tracing::{info, warn};

use crate::ipc::NeuralBridge;

pub use crate::niche::{
    CAPABILITIES as GAME_CAPABILITIES, NICHE_DOMAIN as GAME_DOMAIN,
    SEMANTIC_MAPPINGS as GAME_SEMANTIC_MAPPINGS,
};

fn register_domain_inner(bridge: &NeuralBridge, socket_path: &std::path::Path) {
    match bridge.register(socket_path) {
        Ok(_) => {
            info!(
                domain = crate::niche::NICHE_DOMAIN,
                capabilities = crate::niche::CAPABILITIES.len(),
                "registered domain via NeuralBridge"
            );
        }
        Err(e) => {
            warn!("capability.register failed (non-fatal): {e}");
        }
    }
}

fn deregister_domain_inner(bridge: &NeuralBridge) {
    let _ = bridge.deregister();
    info!(domain = crate::niche::NICHE_DOMAIN, "deregistered domain");
}

/// Register the `game` domain and all capabilities with biomeOS Neural API.
///
/// Uses [`NeuralBridge`] for typed communication with the Neural API.
/// Non-fatal if Neural API is unavailable — ludoSpring runs standalone.
pub fn register_domain(socket_path: &std::path::Path) {
    let Ok(bridge) = NeuralBridge::discover() else {
        info!("Neural API not found — running standalone (domain registration skipped)");
        return;
    };

    register_domain_inner(&bridge, socket_path);
}

/// Deregister the `game` domain from biomeOS Neural API.
///
/// Called on SIGTERM shutdown for clean niche teardown.
pub fn deregister_domain() {
    let Ok(bridge) = NeuralBridge::discover() else {
        return;
    };

    deregister_domain_inner(&bridge);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn re_exports_match_niche() {
        assert_eq!(GAME_DOMAIN, crate::niche::NICHE_DOMAIN);
        assert_eq!(GAME_CAPABILITIES.len(), crate::niche::CAPABILITIES.len());
        assert_eq!(
            GAME_SEMANTIC_MAPPINGS.len(),
            crate::niche::SEMANTIC_MAPPINGS.len()
        );
    }

    #[test]
    fn register_domain_no_panic_when_neural_api_unavailable() {
        let path = std::env::temp_dir().join(format!(
            "ludospring-biomeos-register-test-{}.sock",
            std::process::id()
        ));
        register_domain(&path);
    }

    #[test]
    fn deregister_domain_no_panic_when_neural_api_unavailable() {
        deregister_domain();
    }

    #[cfg(all(unix, feature = "ipc"))]
    #[test]
    fn register_and_deregister_inner_with_live_ipc_server() {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::time::Duration;

        let dir = std::env::temp_dir().join(format!(
            "ludospring-biomeos-ipc-{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        std::fs::create_dir_all(&dir).expect("temp dir");
        let sock = dir.join("neural.sock");
        let shutdown = Arc::new(AtomicBool::new(false));
        let shutdown_clone = Arc::clone(&shutdown);
        let sock_clone = sock.clone();
        let handle = std::thread::spawn(move || {
            let server = crate::ipc::IpcServer::with_path(&sock_clone);
            let _ = server.run_until(&shutdown_clone);
        });
        for _ in 0..100 {
            if sock.exists() {
                break;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        assert!(sock.exists(), "test server socket");

        let bridge = NeuralBridge::with_socket_and_timeout(sock.clone(), Duration::from_secs(5));
        let our_sock = std::env::temp_dir().join(format!(
            "ludospring-biomeos-oursock-{}.sock",
            std::process::id()
        ));
        register_domain_inner(&bridge, &our_sock);
        deregister_domain_inner(&bridge);

        shutdown.store(true, Ordering::Relaxed);
        std::fs::remove_file(&sock).ok();
        std::fs::remove_dir(&dir).ok();
        let _ = handle.join();
    }
}

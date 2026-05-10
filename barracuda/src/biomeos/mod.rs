// SPDX-License-Identifier: AGPL-3.0-or-later
//! biomeOS niche deployment integration for ludoSpring.
//!
//! Per the Spring-as-Niche Deployment Standard, this module handles:
//! - **Registration**: `lifecycle.register` / `capability.deregister` via `NeuralBridge`
//! - **Method registration**: `method.register` (biomeOS v3.51 dynamic registration)
//! - **Composition status**: `composition.status` health/resource monitoring
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

/// Composition status response from biomeOS v3.51 `composition.status`.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct CompositionStatus {
    /// Count of primals in Active state (proxy for load).
    pub active_users: u32,
    /// Per-primal state, latency, failure count, resurrection count.
    pub primal_health: Vec<PrimalHealth>,
    /// Host CPU/memory/disk from `/proc` via `biomeos-system`.
    pub resource_pressure: ResourcePressure,
    /// Total primals known to biomeOS topology.
    pub total_primals: u32,
    /// Monotonic counter from `LifecycleHandler`.
    pub topology_version: u64,
}

/// Per-primal health entry from `composition.status`.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct PrimalHealth {
    /// Primal name (e.g. "bearDog", "toadStool").
    pub name: String,
    /// Current state: "active", "degraded", "stopped".
    pub state: String,
    /// Last-measured IPC round-trip latency in milliseconds.
    pub latency_ms: u64,
    /// Cumulative failure count since last resurrection.
    pub failures: u32,
    /// How many times biomeOS has restarted this primal.
    pub resurrection_count: u32,
}

/// Host resource pressure from `composition.status`.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ResourcePressure {
    /// CPU utilization (0.0–1.0).
    pub cpu: f64,
    /// Memory utilization (0.0–1.0).
    pub memory: f64,
    /// Disk utilization (0.0–1.0).
    pub disk: f64,
}

/// Response from biomeOS v3.51 `method.register`.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct MethodRegisterResponse {
    /// Number of methods successfully registered.
    pub registered: u32,
    /// Domain prefixes extracted from method names.
    pub domains: Vec<String>,
    /// Primal name as registered.
    pub primal: String,
    /// Socket endpoint registered for routing.
    pub endpoint: String,
}

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

/// Register spring-specific methods with biomeOS v3.51 `method.register`.
///
/// Sends all `game.*` capabilities from the niche for dynamic semantic routing.
/// Non-fatal if biomeOS is unavailable.
pub fn register_methods(socket_path: &std::path::Path) -> Option<MethodRegisterResponse> {
    let bridge = NeuralBridge::discover().ok()?;

    let game_methods: Vec<&str> = crate::niche::CAPABILITIES
        .iter()
        .filter(|c| c.starts_with("game."))
        .copied()
        .collect();

    let params = serde_json::json!({
        "primal": crate::niche::NICHE_NAME,
        "transport": socket_path.to_string_lossy(),
        "methods": game_methods,
    });

    match bridge.call_raw("method.register", &params) {
        Ok(resp) => {
            let parsed: Option<MethodRegisterResponse> = serde_json::from_value(resp).ok();
            if let Some(ref r) = parsed {
                info!(
                    registered = r.registered,
                    domains = ?r.domains,
                    "methods registered with biomeOS v3.51"
                );
            }
            parsed
        }
        Err(e) => {
            warn!("method.register failed (non-fatal): {e}");
            None
        }
    }
}

/// Query biomeOS v3.51 `composition.status` for health and resource pressure.
///
/// Returns `None` if biomeOS is unavailable or the response is malformed.
pub fn composition_status() -> Option<CompositionStatus> {
    let bridge = NeuralBridge::discover().ok()?;

    match bridge.call_raw("composition.status", &serde_json::Value::Null) {
        Ok(resp) => {
            let parsed: Option<CompositionStatus> = serde_json::from_value(resp).ok();
            if let Some(ref s) = parsed {
                info!(
                    active = s.active_users,
                    primals = s.total_primals,
                    cpu = s.resource_pressure.cpu,
                    "composition.status polled"
                );
            }
            parsed
        }
        Err(e) => {
            warn!("composition.status failed (non-fatal): {e}");
            None
        }
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
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

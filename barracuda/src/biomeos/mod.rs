// SPDX-License-Identifier: AGPL-3.0-or-later
//! biomeOS niche deployment integration for ludoSpring.
//!
//! Per the Spring-as-Niche Deployment Standard, this module handles:
//! - **Registration**: `lifecycle.register` / `capability.deregister` via [`NeuralBridge`]
//! - **Neural API communication**: typed client from [`crate::ipc::NeuralBridge`]
//!
//! All identity, capabilities, and semantic mappings come from [`crate::niche`]
//! (single source of truth). This module never hardcodes peer primal names.

use tracing::{info, warn};

use crate::ipc::NeuralBridge;

pub use crate::niche::{
    CAPABILITIES as GAME_CAPABILITIES, NICHE_DOMAIN as GAME_DOMAIN,
    SEMANTIC_MAPPINGS as GAME_SEMANTIC_MAPPINGS,
};

/// Register the `game` domain and all capabilities with biomeOS Neural API.
///
/// Uses [`NeuralBridge`] for typed communication with the Neural API.
/// Non-fatal if Neural API is unavailable — ludoSpring runs standalone.
pub fn register_domain(socket_path: &std::path::Path) {
    let Ok(bridge) = NeuralBridge::discover() else {
        info!("Neural API not found — running standalone (domain registration skipped)");
        return;
    };

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

/// Deregister the `game` domain from biomeOS Neural API.
///
/// Called on SIGTERM shutdown for clean niche teardown.
pub fn deregister_domain() {
    let Ok(bridge) = NeuralBridge::discover() else {
        return;
    };

    let _ = bridge.deregister();
    info!(domain = crate::niche::NICHE_DOMAIN, "deregistered domain");
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
}

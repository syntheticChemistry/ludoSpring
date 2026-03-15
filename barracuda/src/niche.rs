// SPDX-License-Identifier: AGPL-3.0-or-later
//! Niche self-knowledge for ludoSpring.
//!
//! Single source of truth for primal identity, capabilities, semantic mappings,
//! operation dependencies, and cost estimates. Follows the airSpring `niche.rs`
//! pattern: a primal has complete self-knowledge and discovers others at runtime.
//!
//! This module has **no IPC dependencies** — it is pure constants and metadata
//! that any part of the crate (or external consumers) can reference without
//! pulling in the `ipc` feature.

/// Primal identity name (used for socket naming, registration, and logging).
pub const NICHE_NAME: &str = "ludospring";

/// Capability domain — all capabilities are prefixed with this.
pub const NICHE_DOMAIN: &str = "game";

/// All capabilities this primal exposes (game science + provenance + telemetry).
///
/// Each entry follows `domain.operation` naming per the Universal IPC Standard.
pub const CAPABILITIES: &[&str] = &[
    "game.evaluate_flow",
    "game.fitts_cost",
    "game.engagement",
    "game.analyze_ui",
    "game.accessibility",
    "game.wfc_step",
    "game.difficulty_adjustment",
    "game.generate_noise",
    "game.begin_session",
    "game.record_action",
    "game.complete_session",
    "game.poll_telemetry",
];

/// Semantic mappings: short name → fully qualified capability.
///
/// Used by biomeOS `CapabilityTaxonomy` for cross-primal routing.
/// Short names allow ecosystem consumers to reference operations without
/// knowing the fully qualified domain prefix.
pub const SEMANTIC_MAPPINGS: &[(&str, &str)] = &[
    ("evaluate_flow", "game.evaluate_flow"),
    ("fitts_cost", "game.fitts_cost"),
    ("engagement", "game.engagement"),
    ("analyze_ui", "game.analyze_ui"),
    ("accessibility", "game.accessibility"),
    ("wfc_step", "game.wfc_step"),
    ("difficulty_adjustment", "game.difficulty_adjustment"),
    ("generate_noise", "game.generate_noise"),
    ("begin_session", "game.begin_session"),
    ("record_action", "game.record_action"),
    ("complete_session", "game.complete_session"),
    ("poll_telemetry", "game.poll_telemetry"),
];

/// Neural API Enhancement 2: dependency hints for the Pathway Learner.
///
/// Maps each capability to its required input parameters and any
/// operational dependencies. biomeOS uses this for parallelization
/// planning and data-flow optimization.
#[must_use]
pub fn operation_dependencies() -> serde_json::Value {
    serde_json::json!({
        "game.evaluate_flow": { "requires": ["challenge", "skill"] },
        "game.fitts_cost": { "requires": ["distance", "target_width"] },
        "game.engagement": { "requires": ["session_duration_s", "action_count"] },
        "game.analyze_ui": { "requires": ["elements"] },
        "game.accessibility": { "requires": ["feature_flags"] },
        "game.wfc_step": { "requires": ["grid_dimensions", "n_tiles"] },
        "game.difficulty_adjustment": { "requires": ["outcomes"] },
        "game.generate_noise": { "requires": ["coordinates"] },
        "game.begin_session": { "requires": ["session_name"] },
        "game.record_action": { "requires": ["session_id", "action"] },
        "game.complete_session": { "requires": ["session_id"], "depends_on": ["game.begin_session"] },
        "game.poll_telemetry": { "requires": [] },
    })
}

/// Neural API Enhancement 3: cost estimates for scheduling.
///
/// Provides latency, CPU intensity, and memory estimates so biomeOS
/// can make informed scheduling and resource allocation decisions.
#[must_use]
pub fn cost_estimates() -> serde_json::Value {
    serde_json::json!({
        "game.evaluate_flow": { "typical_latency_us": 5, "cpu_intensity": "low", "memory_bytes": 128 },
        "game.fitts_cost": { "typical_latency_us": 3, "cpu_intensity": "low", "memory_bytes": 64 },
        "game.engagement": { "typical_latency_us": 10, "cpu_intensity": "low", "memory_bytes": 256 },
        "game.analyze_ui": { "typical_latency_us": 50, "cpu_intensity": "medium", "memory_bytes": 4096 },
        "game.accessibility": { "typical_latency_us": 8, "cpu_intensity": "low", "memory_bytes": 128 },
        "game.wfc_step": { "typical_latency_us": 200, "cpu_intensity": "medium", "memory_bytes": 16384 },
        "game.difficulty_adjustment": { "typical_latency_us": 15, "cpu_intensity": "low", "memory_bytes": 512 },
        "game.generate_noise": { "typical_latency_us": 100, "cpu_intensity": "medium", "memory_bytes": 1024 },
        "game.begin_session": { "typical_latency_us": 500, "cpu_intensity": "low", "memory_bytes": 1024 },
        "game.record_action": { "typical_latency_us": 200, "cpu_intensity": "low", "memory_bytes": 512 },
        "game.complete_session": { "typical_latency_us": 1000, "cpu_intensity": "low", "memory_bytes": 2048 },
        "game.poll_telemetry": { "typical_latency_us": 10, "cpu_intensity": "low", "memory_bytes": 256 },
    })
}

/// Resolve the biomeOS family ID from environment.
///
/// Priority: `FAMILY_ID` → `BIOMEOS_FAMILY_ID` → `"default"`.
#[must_use]
pub fn family_id() -> String {
    std::env::var("FAMILY_ID")
        .or_else(|_| std::env::var("BIOMEOS_FAMILY_ID"))
        .unwrap_or_else(|_| "default".to_string())
}

/// Socket directories in XDG-compliant priority order.
///
/// 1. `BIOMEOS_SOCKET_DIR` — explicit ecosystem override
/// 2. `$XDG_RUNTIME_DIR/biomeos/` — standard runtime location
/// 3. `$TMPDIR/biomeos-$USER` — user-scoped temp fallback
/// 4. `temp_dir()` — platform-agnostic last resort
#[must_use]
pub fn socket_dirs() -> Vec<std::path::PathBuf> {
    use std::path::PathBuf;

    let mut dirs = Vec::new();

    if let Ok(d) = std::env::var("BIOMEOS_SOCKET_DIR") {
        dirs.push(PathBuf::from(d));
    }

    if let Ok(xdg) = std::env::var("XDG_RUNTIME_DIR") {
        dirs.push(PathBuf::from(xdg).join("biomeos"));
    }

    let user = std::env::var("USER").unwrap_or_else(|_| "unknown".to_string());
    dirs.push(std::env::temp_dir().join(format!("biomeos-{user}")));
    dirs.push(std::env::temp_dir());

    dirs
}

/// Resolve the socket path for this primal's IPC server.
///
/// Explicit overrides checked first, then XDG-compliant directory chain.
#[must_use]
pub fn resolve_server_socket() -> std::path::PathBuf {
    use std::path::PathBuf;

    if let Ok(explicit) = std::env::var("LUDOSPRING_SOCK") {
        return PathBuf::from(explicit);
    }
    if let Ok(explicit) = std::env::var("LUDOSPRING_SOCKET") {
        return PathBuf::from(explicit);
    }

    let fid = family_id();
    let sock_name = format!("{NICHE_NAME}-{fid}.sock");

    for dir in socket_dirs() {
        if dir.is_dir() || std::fs::create_dir_all(&dir).is_ok() {
            return dir.join(&sock_name);
        }
    }

    std::env::temp_dir().join(sock_name)
}

/// Resolve the Neural API socket path (discovered by convention, not name).
///
/// The Neural API socket name follows `neural-api-{family_id}.sock`.
#[must_use]
pub fn resolve_neural_api_socket() -> Option<std::path::PathBuf> {
    if let Ok(explicit) = std::env::var("NEURAL_API_SOCKET") {
        let p = std::path::PathBuf::from(&explicit);
        if p.exists() {
            return Some(p);
        }
    }

    let fid = family_id();
    let sock_name = format!("neural-api-{fid}.sock");

    for dir in socket_dirs() {
        let p = dir.join(&sock_name);
        if p.exists() {
            return Some(p);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_constants() {
        assert_eq!(NICHE_NAME, "ludospring");
        assert_eq!(NICHE_DOMAIN, "game");
    }

    #[test]
    fn capabilities_consistent() {
        assert_eq!(CAPABILITIES.len(), 12);
        assert_eq!(SEMANTIC_MAPPINGS.len(), 12);

        for (short, full) in SEMANTIC_MAPPINGS {
            assert!(
                CAPABILITIES.contains(full),
                "mapping {short} -> {full} not in CAPABILITIES"
            );
        }
    }

    #[test]
    fn all_capabilities_prefixed_with_domain() {
        for cap in CAPABILITIES {
            assert!(
                cap.starts_with("game."),
                "capability '{cap}' does not start with '{NICHE_DOMAIN}.'"
            );
        }
    }

    #[test]
    fn operation_dependencies_covers_all_capabilities() {
        let deps = operation_dependencies();
        for cap in CAPABILITIES {
            assert!(
                deps.get(cap).is_some(),
                "missing dependency entry for {cap}"
            );
        }
    }

    #[test]
    fn cost_estimates_covers_all_capabilities() {
        let costs = cost_estimates();
        for cap in CAPABILITIES {
            assert!(costs.get(cap).is_some(), "missing cost entry for {cap}");
        }
    }

    #[test]
    fn socket_dirs_never_empty() {
        let dirs = socket_dirs();
        assert!(!dirs.is_empty(), "should always resolve at least one dir");
    }

    #[test]
    fn family_id_has_default() {
        let fid = family_id();
        assert!(!fid.is_empty());
    }
}

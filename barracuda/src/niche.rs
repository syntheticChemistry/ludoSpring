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

/// Conventional directory name for ecosystem IPC sockets.
///
/// Used under `$XDG_RUNTIME_DIR/` and `$TMPDIR/` to locate peer primals.
/// Not a dependency on biomeOS — just the conventional socket namespace.
pub const ECOSYSTEM_SOCKET_DIR: &str = "biomeos";

/// Default family ID when none is set in the environment.
const DEFAULT_FAMILY_ID: &str = "default";

/// All capabilities this primal exposes.
///
/// Organized by domain:
/// - Game science (flow, engagement, accessibility, procedural)
/// - Provenance (session lifecycle)
/// - AI narration (Squirrel-backed NPC dialogue and voices)
/// - Visualization (petalTongue scene push)
/// - Storage (NestGate-backed persistence)
/// - Certificates (loamSpine-backed permanent records)
/// - DAG queries (rhizoCrypt-backed NPC memory)
/// - GPU compute (toadStool-backed shader dispatch)
///
/// Each entry follows `domain.operation` naming per the Universal IPC Standard.
pub const CAPABILITIES: &[&str] = &[
    // ── Game science ────────────────────────────────────────────────────────
    "game.evaluate_flow",
    "game.fitts_cost",
    "game.engagement",
    "game.analyze_ui",
    "game.accessibility",
    "game.wfc_step",
    "game.difficulty_adjustment",
    "game.generate_noise",
    // ── Provenance lifecycle ────────────────────────────────────────────────
    "game.begin_session",
    "game.record_action",
    "game.complete_session",
    "game.poll_telemetry",
    // ── AI narration (Squirrel) ─────────────────────────────────────────────
    "game.npc_dialogue",
    "game.narrate_action",
    "game.voice_check",
    // ── Visualization & interaction (petalTongue) ──────────────────────────
    "game.push_scene",
    "game.tick",
    "game.subscribe_interaction",
    "game.poll_interaction",
    // ── DAG queries (rhizoCrypt) ────────────────────────────────────────────
    "game.query_vertices",
    // ── Certificates (loamSpine) ────────────────────────────────────────────
    "game.mint_certificate",
    // ── Storage (NestGate) ──────────────────────────────────────────────────
    "game.storage_put",
    "game.storage_get",
    // ── GPU compute (toadStool + barraCuda) ─────────────────────────────────
    "game.gpu.fog_of_war",
    "game.gpu.tile_lighting",
    "game.gpu.pathfind",
    "game.gpu.perlin_terrain",
    "game.gpu.batch_raycast",
    // ── Health probes (coralReef Iter 51 / healthSpring V32 pattern) ──────
    "health.liveness",
    "health.readiness",
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
    ("npc_dialogue", "game.npc_dialogue"),
    ("narrate_action", "game.narrate_action"),
    ("voice_check", "game.voice_check"),
    ("push_scene", "game.push_scene"),
    ("game_tick", "game.tick"),
    ("subscribe_interaction", "game.subscribe_interaction"),
    ("poll_interaction", "game.poll_interaction"),
    ("query_vertices", "game.query_vertices"),
    ("mint_certificate", "game.mint_certificate"),
    ("storage_put", "game.storage_put"),
    ("storage_get", "game.storage_get"),
    ("gpu_fog_of_war", "game.gpu.fog_of_war"),
    ("gpu_tile_lighting", "game.gpu.tile_lighting"),
    ("gpu_pathfind", "game.gpu.pathfind"),
    ("gpu_perlin_terrain", "game.gpu.perlin_terrain"),
    ("gpu_batch_raycast", "game.gpu.batch_raycast"),
    ("liveness", "health.liveness"),
    ("readiness", "health.readiness"),
];

// ── Niche Dependencies (SPRING_COMPOSITION_PATTERNS §11 — MUST) ─────

/// A primal dependency declared at the niche level.
///
/// Aligns with the proto-nucleate graph's node list and makes
/// capability-based discovery self-documenting. Per
/// `SPRING_COMPOSITION_PATTERNS` §11, every spring MUST declare its
/// dependencies as a typed table.
#[derive(Debug, Clone, Copy)]
pub struct NicheDependency {
    /// Primal name (e.g. `"toadstool"`).
    pub name: &'static str,
    /// Primal's role in this composition (e.g. `"compute"`).
    pub role: &'static str,
    /// Whether the primal is required for core operation.
    pub required: bool,
    /// Primary capability domain for discovery.
    pub capability: &'static str,
}

/// Primal dependencies for this niche — mirrors proto-nucleate graph nodes.
pub const DEPENDENCIES: &[NicheDependency] = &[
    NicheDependency {
        name: "beardog",
        role: "security",
        required: true,
        capability: "crypto",
    },
    NicheDependency {
        name: "songbird",
        role: "discovery",
        required: true,
        capability: "discovery",
    },
    NicheDependency {
        name: "toadstool",
        role: "compute",
        required: true,
        capability: "compute",
    },
    NicheDependency {
        name: "coralreef",
        role: "shader",
        required: true,
        capability: "shader",
    },
    NicheDependency {
        name: "barracuda",
        role: "tensor",
        required: true,
        capability: "tensor",
    },
    NicheDependency {
        name: "squirrel",
        role: "ai",
        required: true,
        capability: "ai",
    },
    NicheDependency {
        name: "petaltongue",
        role: "visualization",
        required: false,
        capability: "visualization",
    },
    NicheDependency {
        name: "nestgate",
        role: "storage",
        required: true,
        capability: "storage",
    },
    NicheDependency {
        name: "rhizocrypt",
        role: "provenance_dag",
        required: false,
        capability: "dag",
    },
    NicheDependency {
        name: "loamspine",
        role: "permanence",
        required: false,
        capability: "certificate",
    },
    NicheDependency {
        name: "sweetgrass",
        role: "attribution",
        required: false,
        capability: "braid",
    },
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
        "game.npc_dialogue": { "requires": ["npc_name", "personality_prompt", "player_input"], "external": ["ai.query"] },
        "game.narrate_action": { "requires": ["action", "context"], "external": ["ai.suggest"] },
        "game.voice_check": { "requires": ["voice_name", "voice_personality", "game_state"], "external": ["ai.analyze"] },
        "game.push_scene": { "requires": ["session_id", "channel", "scene"], "external": ["visualization.render.scene"] },
        "game.tick": { "requires": ["session_id", "scene"], "external": ["visualization.render.scene", "interaction.poll"] },
        "game.subscribe_interaction": { "requires": ["session_id"], "external": ["interaction.subscribe"] },
        "game.poll_interaction": { "requires": ["session_id"], "external": ["interaction.poll"] },
        "game.query_vertices": { "requires": ["session_id"], "external": ["dag.vertex.query"] },
        "game.mint_certificate": { "requires": ["cert_type", "owner", "payload"], "external": ["certificate.mint"] },
        "game.storage_put": { "requires": ["key", "data"], "external": ["storage.store"] },
        "game.storage_get": { "requires": ["key"], "external": ["storage.retrieve"] },
        "game.gpu.fog_of_war": { "requires": ["grid_w", "grid_h", "viewer_x", "viewer_y", "sight_radius"], "external": ["compute.submit"] },
        "game.gpu.tile_lighting": { "requires": ["grid_w", "grid_h", "lights"], "external": ["compute.submit"] },
        "game.gpu.pathfind": { "requires": ["grid_w", "grid_h", "start_x", "start_y"], "external": ["compute.submit"] },
        "game.gpu.perlin_terrain": { "requires": ["grid_w", "grid_h"], "external": ["compute.submit"] },
        "game.gpu.batch_raycast": { "requires": ["map_w", "map_h", "player_x", "player_y", "n_rays"], "external": ["compute.dispatch"] },
        "health.liveness": { "requires": [] },
        "health.readiness": { "requires": [] },
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
        "game.npc_dialogue": { "typical_latency_us": 500_000, "cpu_intensity": "external", "memory_bytes": 4096 },
        "game.narrate_action": { "typical_latency_us": 300_000, "cpu_intensity": "external", "memory_bytes": 2048 },
        "game.voice_check": { "typical_latency_us": 200_000, "cpu_intensity": "external", "memory_bytes": 2048 },
        "game.push_scene": { "typical_latency_us": 1000, "cpu_intensity": "low", "memory_bytes": 8192 },
        "game.tick": { "typical_latency_us": 5000, "cpu_intensity": "medium", "memory_bytes": 32768 },
        "game.subscribe_interaction": { "typical_latency_us": 1000, "cpu_intensity": "low", "memory_bytes": 4096 },
        "game.poll_interaction": { "typical_latency_us": 500, "cpu_intensity": "low", "memory_bytes": 8192 },
        "game.query_vertices": { "typical_latency_us": 5000, "cpu_intensity": "external", "memory_bytes": 16384 },
        "game.mint_certificate": { "typical_latency_us": 10_000, "cpu_intensity": "external", "memory_bytes": 4096 },
        "game.storage_put": { "typical_latency_us": 5000, "cpu_intensity": "external", "memory_bytes": 65536 },
        "game.storage_get": { "typical_latency_us": 3000, "cpu_intensity": "external", "memory_bytes": 65536 },
        "game.gpu.fog_of_war": { "typical_latency_us": 500, "cpu_intensity": "gpu", "memory_bytes": 262_144 },
        "game.gpu.tile_lighting": { "typical_latency_us": 500, "cpu_intensity": "gpu", "memory_bytes": 262_144 },
        "game.gpu.pathfind": { "typical_latency_us": 2000, "cpu_intensity": "gpu", "memory_bytes": 262_144 },
        "game.gpu.perlin_terrain": { "typical_latency_us": 1000, "cpu_intensity": "gpu", "memory_bytes": 524_288 },
        "game.gpu.batch_raycast": { "typical_latency_us": 800, "cpu_intensity": "gpu", "memory_bytes": 131_072 },
        "health.liveness": { "typical_latency_us": 1, "cpu_intensity": "none", "memory_bytes": 32 },
        "health.readiness": { "typical_latency_us": 5, "cpu_intensity": "low", "memory_bytes": 128 },
    })
}

/// Resolve the biomeOS family ID from environment.
///
/// Priority: `FAMILY_ID` → `BIOMEOS_FAMILY_ID` → `"default"`.
#[must_use]
pub fn family_id() -> String {
    std::env::var("FAMILY_ID")
        .or_else(|_| std::env::var("BIOMEOS_FAMILY_ID"))
        .unwrap_or_else(|_| DEFAULT_FAMILY_ID.to_string())
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
        dirs.push(PathBuf::from(xdg).join(ECOSYSTEM_SOCKET_DIR));
    }

    let user = std::env::var("USER").unwrap_or_else(|_| "unknown".to_string());
    dirs.push(std::env::temp_dir().join(format!("{ECOSYSTEM_SOCKET_DIR}-{user}")));
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
        assert_eq!(CAPABILITIES.len(), 30);
        assert_eq!(SEMANTIC_MAPPINGS.len(), 30);

        for (short, full) in SEMANTIC_MAPPINGS {
            assert!(
                CAPABILITIES.contains(full),
                "mapping {short} -> {full} not in CAPABILITIES"
            );
        }
    }

    #[test]
    fn all_capabilities_are_namespaced() {
        let allowed_prefixes = ["game.", "health."];
        for cap in CAPABILITIES {
            assert!(
                allowed_prefixes.iter().any(|p| cap.starts_with(p)),
                "capability '{cap}' has no recognized namespace prefix"
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
    fn dependencies_table_complete() {
        assert_eq!(DEPENDENCIES.len(), 11, "11 proto-nucleate primals");
        let names: Vec<&str> = DEPENDENCIES.iter().map(|d| d.name).collect();
        for expected in [
            "beardog",
            "songbird",
            "toadstool",
            "coralreef",
            "barracuda",
            "squirrel",
            "petaltongue",
            "nestgate",
            "rhizocrypt",
            "loamspine",
            "sweetgrass",
        ] {
            assert!(names.contains(&expected), "missing dependency: {expected}");
        }
    }

    #[test]
    fn dependencies_have_capabilities() {
        for dep in DEPENDENCIES {
            assert!(!dep.capability.is_empty(), "{} needs capability", dep.name);
            assert!(!dep.role.is_empty(), "{} needs role", dep.name);
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

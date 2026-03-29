// SPDX-License-Identifier: AGPL-3.0-or-later
//! Capability domain registry — introspectable method catalog.
//!
//! Follows the wetSpring `capability_domains.rs` pattern: a central registry
//! of all domain prefixes and their methods, queryable at runtime for
//! `capability.list` responses and biomeOS Pathway Learner scheduling.
//!
//! This module complements [`crate::niche`] — niche provides identity and
//! flat capability lists; this module provides structured domain metadata.

/// A capability domain with its methods and metadata.
#[derive(Debug)]
pub struct Domain {
    /// Domain prefix (e.g. `"game"`, `"dag"`, `"storage"`).
    pub prefix: &'static str,
    /// Human-readable description.
    pub description: &'static str,
    /// Methods within this domain.
    pub methods: &'static [Method],
}

/// A single capability method within a domain.
#[derive(Debug)]
pub struct Method {
    /// Method name (e.g. `"evaluate_flow"`, `"session.create"`).
    pub name: &'static str,
    /// Fully qualified name (e.g. `"game.evaluate_flow"`).
    pub fqn: &'static str,
    /// Whether this method delegates to an external primal.
    pub external: bool,
}

/// All domains exposed by ludoSpring.
pub const DOMAINS: &[Domain] = &[
    Domain {
        prefix: "game",
        description: "Game science — HCI models, flow, engagement, procedural generation",
        methods: &[
            Method {
                name: "evaluate_flow",
                fqn: "game.evaluate_flow",
                external: false,
            },
            Method {
                name: "fitts_cost",
                fqn: "game.fitts_cost",
                external: false,
            },
            Method {
                name: "engagement",
                fqn: "game.engagement",
                external: false,
            },
            Method {
                name: "analyze_ui",
                fqn: "game.analyze_ui",
                external: false,
            },
            Method {
                name: "accessibility",
                fqn: "game.accessibility",
                external: false,
            },
            Method {
                name: "wfc_step",
                fqn: "game.wfc_step",
                external: false,
            },
            Method {
                name: "difficulty_adjustment",
                fqn: "game.difficulty_adjustment",
                external: false,
            },
            Method {
                name: "generate_noise",
                fqn: "game.generate_noise",
                external: false,
            },
            Method {
                name: "begin_session",
                fqn: "game.begin_session",
                external: true,
            },
            Method {
                name: "record_action",
                fqn: "game.record_action",
                external: true,
            },
            Method {
                name: "complete_session",
                fqn: "game.complete_session",
                external: true,
            },
            Method {
                name: "poll_telemetry",
                fqn: "game.poll_telemetry",
                external: false,
            },
            Method {
                name: "npc_dialogue",
                fqn: "game.npc_dialogue",
                external: true,
            },
            Method {
                name: "narrate_action",
                fqn: "game.narrate_action",
                external: true,
            },
            Method {
                name: "voice_check",
                fqn: "game.voice_check",
                external: true,
            },
            Method {
                name: "push_scene",
                fqn: "game.push_scene",
                external: true,
            },
            Method {
                name: "query_vertices",
                fqn: "game.query_vertices",
                external: true,
            },
            Method {
                name: "mint_certificate",
                fqn: "game.mint_certificate",
                external: true,
            },
            Method {
                name: "storage_put",
                fqn: "game.storage_put",
                external: true,
            },
            Method {
                name: "storage_get",
                fqn: "game.storage_get",
                external: true,
            },
            Method {
                name: "gpu.fog_of_war",
                fqn: "game.gpu.fog_of_war",
                external: true,
            },
            Method {
                name: "gpu.tile_lighting",
                fqn: "game.gpu.tile_lighting",
                external: true,
            },
            Method {
                name: "gpu.pathfind",
                fqn: "game.gpu.pathfind",
                external: true,
            },
            Method {
                name: "gpu.perlin_terrain",
                fqn: "game.gpu.perlin_terrain",
                external: true,
            },
            Method {
                name: "gpu.batch_raycast",
                fqn: "game.gpu.batch_raycast",
                external: true,
            },
        ],
    },
    Domain {
        prefix: "health",
        description: "Health probes — Kubernetes-style liveness and readiness checks",
        methods: &[
            Method {
                name: "liveness",
                fqn: "health.liveness",
                external: false,
            },
            Method {
                name: "readiness",
                fqn: "health.readiness",
                external: false,
            },
        ],
    },
];

/// Valid domain prefixes for method validation.
pub const VALID_DOMAIN_PREFIXES: &[&str] = &["game", "health"];

/// All fully qualified method names across all domains.
#[must_use]
pub fn all_methods() -> Vec<&'static str> {
    DOMAINS
        .iter()
        .flat_map(|d| d.methods.iter().map(|m| m.fqn))
        .collect()
}

/// Methods that delegate to external primals.
#[must_use]
pub fn external_methods() -> Vec<&'static str> {
    DOMAINS
        .iter()
        .flat_map(|d| d.methods.iter().filter(|m| m.external).map(|m| m.fqn))
        .collect()
}

/// Methods handled locally without external primal calls.
#[must_use]
pub fn local_methods() -> Vec<&'static str> {
    DOMAINS
        .iter()
        .flat_map(|d| d.methods.iter().filter(|m| !m.external).map(|m| m.fqn))
        .collect()
}

/// Build a structured `capability.list` response with domain metadata.
#[must_use]
pub fn capability_list_response() -> serde_json::Value {
    let domains: Vec<serde_json::Value> = DOMAINS
        .iter()
        .map(|d| {
            let methods: Vec<serde_json::Value> = d
                .methods
                .iter()
                .map(|m| {
                    serde_json::json!({
                        "name": m.name,
                        "fqn": m.fqn,
                        "external": m.external,
                    })
                })
                .collect();
            serde_json::json!({
                "prefix": d.prefix,
                "description": d.description,
                "methods": methods,
                "count": d.methods.len(),
            })
        })
        .collect();

    serde_json::json!({
        "primal": crate::niche::NICHE_NAME,
        "domain": crate::niche::NICHE_DOMAIN,
        "domains": domains,
        "total_capabilities": all_methods().len(),
        "external_count": external_methods().len(),
        "local_count": local_methods().len(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_methods_matches_niche_capabilities() {
        let methods = all_methods();
        assert_eq!(
            methods.len(),
            crate::niche::CAPABILITIES.len(),
            "capability_domains and niche.rs must stay in sync"
        );
        for fqn in &methods {
            assert!(
                crate::niche::CAPABILITIES.contains(fqn),
                "domain method {fqn} not in niche::CAPABILITIES"
            );
        }
    }

    #[test]
    fn valid_prefixes_cover_all_domains() {
        for domain in DOMAINS {
            assert!(
                VALID_DOMAIN_PREFIXES.contains(&domain.prefix),
                "domain {} not in VALID_DOMAIN_PREFIXES",
                domain.prefix
            );
        }
    }

    #[test]
    fn external_and_local_partition() {
        let ext = external_methods();
        let loc = local_methods();
        assert_eq!(ext.len() + loc.len(), all_methods().len());
        for m in &ext {
            assert!(!loc.contains(m), "{m} in both external and local");
        }
    }

    #[test]
    fn capability_list_response_structure() {
        let resp = capability_list_response();
        assert_eq!(resp["primal"], "ludospring");
        assert_eq!(resp["domain"], "game");
        assert_eq!(resp["total_capabilities"], 27);
    }
}

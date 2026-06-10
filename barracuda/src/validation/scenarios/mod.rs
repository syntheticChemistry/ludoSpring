// SPDX-License-Identifier: AGPL-3.0-or-later
//! Validation scenarios — eukaryotic absorbed experiments with ScenarioMeta.
//!
//! Each scenario is a representative experiment from ludoSpring's 100 prokaryotic
//! crates, absorbed into the UniBin as a callable validation unit with provenance.

pub mod registry;

pub use registry::{Scenario, ScenarioMeta, ScenarioRegistry, Tier, Track};

#[cfg(feature = "ipc")]
mod s_audit_integration;
#[cfg(feature = "ipc")]
mod s_composition_gaps;
mod s_composition_parity;
mod s_engagement_metrics;
mod s_interaction_laws;
mod s_niche_depth;
mod s_procedural_gen;
mod s_raycaster_budget;
#[cfg(feature = "ipc")]
mod s_tier2_convergence;
mod s_tier4_math_parity;
#[cfg(feature = "ipc")]
mod s_tower_atomic;

/// Build the full scenario registry for ludoSpring.
pub fn build_registry() -> ScenarioRegistry {
    let mut r = ScenarioRegistry::new();
    r.register(s_interaction_laws::SCENARIO);
    r.register(s_procedural_gen::SCENARIO);
    r.register(s_engagement_metrics::SCENARIO);
    r.register(s_composition_parity::SCENARIO);
    r.register(s_raycaster_budget::SCENARIO);
    r.register(s_tier4_math_parity::SCENARIO);
    r.register(s_niche_depth::SCENARIO);
    #[cfg(feature = "ipc")]
    r.register(s_audit_integration::SCENARIO);
    #[cfg(feature = "ipc")]
    r.register(s_composition_gaps::SCENARIO);
    #[cfg(feature = "ipc")]
    r.register(s_tier2_convergence::SCENARIO);
    #[cfg(feature = "ipc")]
    r.register(s_tower_atomic::SCENARIO);
    r
}

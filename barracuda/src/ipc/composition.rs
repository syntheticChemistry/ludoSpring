// SPDX-License-Identifier: AGPL-3.0-or-later
//! Deploy graph composition validation at runtime.
//!
//! Per `SPRING_COMPOSITION_PATTERNS` §5, a spring MUST be able to probe
//! its proto-nucleate at runtime and report which dependencies are live,
//! degraded, or absent. This module implements the `CompositionReport`
//! returned by `lifecycle.composition` (dispatched in `handlers/lifecycle.rs`).

use super::discovery::{DiscoveryResult, discover_by_capability, discover_primal_tiered};
use crate::niche::{DEPENDENCIES, NicheDependency};

/// Per-dependency liveness status.
#[derive(Debug, Clone, serde::Serialize)]
pub struct DependencyStatus {
    /// Primal name.
    pub name: &'static str,
    /// Role in this composition.
    pub role: &'static str,
    /// Whether the primal is required.
    pub required: bool,
    /// Discovery result: `"live"`, `"absent"`, or `"degraded"`.
    pub status: &'static str,
    /// Discovery tier if found, or paths searched if absent.
    pub detail: String,
}

/// Full composition report for the current deployment.
#[derive(Debug, Clone, serde::Serialize)]
pub struct CompositionReport {
    /// Spring identity.
    pub spring: &'static str,
    /// Proto-nucleate composition model.
    pub composition_model: &'static str,
    /// Fragments metadata.
    pub fragments: &'static [&'static str],
    /// Per-dependency statuses.
    pub dependencies: Vec<DependencyStatus>,
    /// Number of dependencies that are live.
    pub live_count: usize,
    /// Number of required dependencies that are absent.
    pub missing_required: usize,
    /// Whether the composition is fully satisfied.
    pub complete: bool,
}

/// Declared fragment set for ludoSpring's proto-nucleate.
///
/// `nest_atomic` included because ludoSpring wires NestGate + provenance
/// trio (rhizoCrypt, loamSpine, sweetGrass) as niche dependencies. Trio
/// primals are `required: false` — the Nest fragment is aspirational
/// until upstream blockers (GAP-06 rhizoCrypt UDS, GAP-07 loamSpine
/// startup) are resolved.
pub const FRAGMENTS: &[&str] = &["tower_atomic", "node_atomic", "nest_atomic", "meta_tier"];

/// Probe all niche dependencies and build a composition report.
#[must_use]
pub fn validate_composition() -> CompositionReport {
    let mut deps = Vec::with_capacity(DEPENDENCIES.len());
    let mut live_count = 0usize;
    let mut missing_required = 0usize;

    for dep in DEPENDENCIES {
        let status = probe_dependency(dep);
        if status.status == "live" {
            live_count += 1;
        } else if dep.required {
            missing_required += 1;
        }
        deps.push(status);
    }

    CompositionReport {
        spring: crate::niche::NICHE_NAME,
        composition_model: "pure",
        fragments: FRAGMENTS,
        dependencies: deps,
        live_count,
        missing_required,
        complete: missing_required == 0,
    }
}

/// Probe a dependency by capability first (by_capability), then fall
/// back to name-based tiered discovery. Per `SPRING_COMPOSITION_PATTERNS`
/// §3, capability-based is the canonical resolution path; name-based is
/// the operational fallback for primals that may not yet advertise
/// capabilities in their `lifecycle.status` response.
fn probe_dependency(dep: &NicheDependency) -> DependencyStatus {
    let display_name = dep.hint_name.unwrap_or(dep.capability);

    if let Some(ep) = discover_by_capability(dep.capability) {
        return DependencyStatus {
            name: display_name,
            role: dep.role,
            required: dep.required,
            status: "live",
            detail: format!("by_capability({}) → {}", dep.capability, ep.name),
        };
    }

    if let Some(hint) = dep.hint_name {
        let result = discover_primal_tiered(hint);
        match result {
            DiscoveryResult::Found { tier, .. } => {
                return DependencyStatus {
                    name: display_name,
                    role: dep.role,
                    required: dep.required,
                    status: "live",
                    detail: format!("by_hint_name via {tier}"),
                };
            }
            DiscoveryResult::NotFound { .. } => {}
        }
    }

    DependencyStatus {
        name: display_name,
        role: dep.role,
        required: dep.required,
        status: "absent",
        detail: format!("no provider for capability '{}'", dep.capability),
    }
}

/// Serialize the composition report as JSON for IPC responses.
///
/// # Errors
///
/// Returns an [`super::envelope::IpcError::Serialization`] on failure (should
/// never happen for this struct).
pub fn composition_json() -> Result<serde_json::Value, super::envelope::IpcError> {
    let report = validate_composition();
    Ok(serde_json::to_value(report)?)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn validate_composition_runs_without_panic() {
        let report = validate_composition();
        assert_eq!(report.spring, "ludospring");
        assert_eq!(report.composition_model, "pure");
        assert_eq!(report.dependencies.len(), DEPENDENCIES.len());
    }

    #[test]
    fn composition_json_serializes() {
        let val = composition_json().expect("serialization");
        assert_eq!(val["spring"], "ludospring");
        assert!(val["dependencies"].is_array());
    }

    #[test]
    fn fragments_match_proto_nucleate() {
        assert!(FRAGMENTS.contains(&"tower_atomic"));
        assert!(FRAGMENTS.contains(&"node_atomic"));
        assert!(FRAGMENTS.contains(&"nest_atomic"));
        assert!(FRAGMENTS.contains(&"meta_tier"));
    }
}

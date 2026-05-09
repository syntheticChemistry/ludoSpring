// SPDX-License-Identifier: AGPL-3.0-or-later
//! Certification organelle — absorbed guidestone layers for eukaryotic UniBin.
//!
//! Three-tier domain science certification:
//! - **Tier 1 (bare)**: deterministic output, reference-traceable, self-verifying,
//!   environment-agnostic, tolerance-documented. No IPC needed.
//! - **Tier 2 (IPC)**: domain science via composition IPC to barraCuda.
//!   Uses `check_skip()` when primals are absent.
//! - **Tier 3 (NUCLEUS)**: cross-atomic validation (BearDog crypto,
//!   NestGate storage, cross-atomic pipeline).
//!
//! Conforms to guideStone Composition Standard v1.2.0.

pub mod constants;
pub mod tier1;
pub mod tier2;
pub mod tier3;

use primalspring::composition::{CompositionContext, validate_liveness};
use primalspring::validation::ValidationResult;

/// Maximum certification tier (inclusive).
pub const MAX_TIER: u8 = 3;

/// Run certification up to the specified tier.
///
/// Returns a `ValidationResult` with all checks executed. Caller decides
/// exit code semantics.
pub fn certify(max_tier: u8) -> ValidationResult {
    let mut v = ValidationResult::new("ludoSpring guideStone — Game Science Certification");
    ValidationResult::print_banner("ludoSpring guideStone — Three-Tier Domain Science");

    // Tier 1: LOCAL_CAPABILITIES (bare, no primals needed)
    v.section("Tier 1: Deterministic Output");
    tier1::validate_determinism(&mut v);

    v.section("Tier 1: Reference-Traceable");
    tier1::validate_traceability(&mut v);

    v.section("Tier 1: Self-Verifying");
    tier1::validate_self_verification(&mut v);

    v.section("Tier 1: Environment-Agnostic");
    tier1::validate_environment_agnostic(&mut v);

    v.section("Tier 1: Tolerance-Documented");
    tier1::validate_tolerance_documentation(&mut v);

    if max_tier < 2 {
        v.finish();
        return v;
    }

    // Tier 2: IPC-WIRED (domain science, skip if primals absent)
    v.section("Tier 2: Discovery");
    let mut ctx = CompositionContext::from_live_discovery_with_fallback();

    let required = &["tensor", "compute"];
    let alive = validate_liveness(&mut ctx, &mut v, required);

    if alive == 0 {
        eprintln!("[certify] No NUCLEUS primals discovered — Tier 1 (bare) only.");
        v.finish();
        return v;
    }

    v.section("Tier 2: Interaction Laws");
    tier2::validate_interaction_laws(&mut ctx, &mut v);

    v.section("Tier 2: Math Primitives");
    tier2::validate_math_primitives(&mut ctx, &mut v);

    v.section("Tier 2: Statistics");
    tier2::validate_statistics(&mut ctx, &mut v);

    v.section("Tier 2: Procedural Generation");
    tier2::validate_procedural(&mut ctx, &mut v);

    v.section("Tier 2: Tensor & Compute");
    tier2::validate_tensor_and_compute(&mut ctx, &mut v);

    if max_tier < 3 {
        v.finish();
        return v;
    }

    // Tier 3: FULL NUCLEUS (cross-atomic validation)
    v.section("Tier 3: Security (BearDog)");
    tier3::validate_security(&mut ctx, &mut v);

    v.section("Tier 3: Storage (NestGate)");
    tier3::validate_storage(&mut ctx, &mut v);

    v.section("Tier 3: Cross-Atomic Pipeline");
    tier3::validate_cross_atomic(&mut ctx, &mut v);

    v.finish();
    v
}

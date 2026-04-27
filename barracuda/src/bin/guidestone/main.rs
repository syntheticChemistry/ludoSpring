// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![allow(missing_docs)] // guideStone binary — no public API

//! ludoSpring guideStone — self-validating NUCLEUS node for game science.
//!
//! Inherits primalSpring base composition certification (6 layers).
//! Validates game science (interaction laws, procedural generation,
//! engagement metrics) through primal IPC against Python golden values.
//!
//! Conforms to guideStone Composition Standard v1.2.0 (primalSpring v0.9.17).
//!
//! # Three-Tier Validation
//!
//! - **Tier 1 — LOCAL_CAPABILITIES** (bare): five certified properties
//!   validated from first principles, no primals needed.
//! - **Tier 2 — IPC-WIRED**: domain science via composition IPC to
//!   barraCuda. Uses `check_skip()` when primals are absent.
//! - **Tier 3 — FULL NUCLEUS**: cross-atomic validation (BearDog crypto,
//!   NestGate storage roundtrip, cross-atomic pipeline).
//!
//! # Five Certified Properties (Tier 1)
//!
//! 1. **Deterministic Output** — recompute every golden value locally
//! 2. **Reference-Traceable** — every constant sourced to a paper
//! 3. **Self-Verifying** — tamper detection + BLAKE3 checksum manifest
//! 4. **Environment-Agnostic** — pure Rust, no network, no filesystem
//! 5. **Tolerance-Documented** — every tolerance named and ordered
//!
//! # Exit Codes
//!
//! - `0` — all checks passed (NUCLEUS certified)
//! - `1` — one or more checks failed
//! - `2` — bare properties passed, no NUCLEUS deployed
//!
//! # NUCLEUS Deployment Requirements (v0.9.17)
//!
//! Tier 3 validation requires these env vars when deploying primals:
//! - `BEARDOG_FAMILY_SEED` — required for BearDog crypto operations
//! - `SONGBIRD_SECURITY_PROVIDER=beardog` — Songbird federation
//! - `NESTGATE_JWT_SECRET` — NestGate storage authentication

mod constants;
mod tier1;
mod tier2;
mod tier3;

use primalspring::composition::{CompositionContext, validate_liveness};
use primalspring::validation::ValidationResult;

fn main() {
    let mut v = ValidationResult::new("ludoSpring guideStone — Game Science Certification");
    ValidationResult::print_banner("ludoSpring guideStone — Three-Tier Domain Science");

    // ════════════════════════════════════════════════════════════════════
    // Tier 1: LOCAL_CAPABILITIES (bare, no primals needed)
    // ════════════════════════════════════════════════════════════════════
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

    // ════════════════════════════════════════════════════════════════════
    // Tier 2: IPC-WIRED (domain science, skip if primals absent)
    // ════════════════════════════════════════════════════════════════════
    v.section("Tier 2: Discovery");
    let mut ctx = CompositionContext::from_live_discovery_with_fallback();

    let required = &["tensor", "compute"];
    let alive = validate_liveness(&mut ctx, &mut v, required);

    if alive == 0 {
        eprintln!("[guideStone] No NUCLEUS primals discovered — Tier 1 (bare) only.");
        v.finish();
        std::process::exit(v.exit_code_skip_aware());
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

    // ════════════════════════════════════════════════════════════════════
    // Tier 3: FULL NUCLEUS (cross-atomic validation)
    // ════════════════════════════════════════════════════════════════════
    v.section("Tier 3: Security (BearDog)");
    tier3::validate_security(&mut ctx, &mut v);

    v.section("Tier 3: Storage (NestGate)");
    tier3::validate_storage(&mut ctx, &mut v);

    v.section("Tier 3: Cross-Atomic Pipeline");
    tier3::validate_cross_atomic(&mut ctx, &mut v);

    v.finish();
    std::process::exit(v.exit_code());
}

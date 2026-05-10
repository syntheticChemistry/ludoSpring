// SPDX-License-Identifier: AGPL-3.0-or-later
//! Scenario: Tier 4 Math Parity — verify `crate::math` dual-path correctness.
//!
//! Confirms the math module (sigmoid, dot, lcg_step, state_to_f64) produces
//! identical results regardless of whether the `local` feature links barraCuda
//! or uses inline fallbacks.

use super::registry::{Scenario, ScenarioMeta, Tier, Track};
use crate::validation::{BaselineProvenance, ValidationHarness};

pub const SCENARIO: Scenario = Scenario {
    meta: ScenarioMeta {
        id: "tier4_math_parity",
        track: Track::CompositionParity,
        tier: Tier::Rust,
        provenance_crate: "tier4_rewiring",
        provenance_date: "2026-05-10",
        description: "Verify crate::math dual-path produces identical results",
    },
    run: run_tier4_math_parity,
};

fn run_tier4_math_parity(h: &mut ValidationHarness) {
    let prov = BaselineProvenance {
        script: "analytical (1/(1+e^-x), dot product, Knuth MMIX LCG)",
        commit: "tier4_v59",
        date: "2026-05-10",
        command: "analytical derivation — no external script",
    };
    h.print_provenance(&[&prov]);

    // Sigmoid known values (1 / (1 + e^(-x)))
    let sigmoid_cases: &[(f64, f64)] = &[
        (0.0, 0.5),
        (1.0, 0.731_058_578_630_005),
        (-1.0, 0.268_941_421_369_995),
        (5.0, 0.993_307_149_075_715_3),
        (-5.0, 0.006_692_850_924_284_856),
    ];
    for (x, expected) in sigmoid_cases {
        let result = crate::math::sigmoid(*x);
        h.check_abs(&format!("sigmoid({x})"), result, *expected, 1e-14);
    }

    // Dot product
    let a = [1.0, 2.0, 3.0, 4.0, 5.0];
    let b = [5.0, 4.0, 3.0, 2.0, 1.0];
    h.check_abs("dot([1..5], [5..1])", crate::math::dot(&a, &b), 35.0, 1e-14);

    // LCG determinism (Knuth MMIX)
    let seed: u64 = 42;
    let step1 = crate::math::lcg_step(seed);
    let step2 = crate::math::lcg_step(step1);
    h.check_bool("lcg_step deterministic", step1 != seed && step2 != step1);
    h.check_bool(
        "lcg_step(42) stable",
        step1
            == 42_u64
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407),
    );

    // state_to_f64 range [0, 1)
    for i in 0..10 {
        let state = crate::math::lcg_step(seed.wrapping_add(i * 997));
        let f = crate::math::state_to_f64(state);
        h.check_bool(
            &format!("state_to_f64 in [0,1) @ {i}"),
            (0.0..1.0).contains(&f),
        );
    }
}

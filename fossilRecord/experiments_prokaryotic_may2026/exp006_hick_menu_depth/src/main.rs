// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
//! Exp006: Hick's law menu depth — validation binary.
//!
//! Validates Hick's law across different menu architectures:
//! flat menus, hierarchical menus, and combined Fitts+Hick interaction cost.
//!
//! # Provenance
//!
//! Hick (1952), Hyman (1953): RT = a + b * log2(N+1).
//! Hierarchical menus: Cockburn et al. (2007) "A Predictive Model of
//! Menu Performance." CHI '07 — total time = sum of Hick RT per level.
//! Python baseline: `baselines/python/interaction_laws.py` (2026-03-11).

use ludospring_barracuda::interaction::input_laws::{hick_reaction_time, interaction_cost};
use ludospring_barracuda::tolerances;
use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "baselines/python/interaction_laws.py",
    commit: "19e402c0",
    date: "2026-03-11",
    command: "python3 baselines/python/run_all_baselines.py",
};

fn validate_flat_menu(h: &mut ValidationHarness) {
    let a = tolerances::HICK_A_MS;
    let b = tolerances::HICK_B_MS;

    let rt_2 = hick_reaction_time(2, a, b);
    let rt_4 = hick_reaction_time(4, a, b);
    let rt_8 = hick_reaction_time(8, a, b);
    let rt_16 = hick_reaction_time(16, a, b);

    h.check_upper(
        "Hick RT grows logarithmically: ratio RT(16)/RT(2) < 2",
        rt_16 / rt_2,
        2.0,
    );
    h.check_bool(
        "RT(2) < RT(4) < RT(8) < RT(16)",
        rt_2 < rt_4 && rt_4 < rt_8 && rt_8 < rt_16,
    );
    h.check_abs(
        "Hick RT for N=7 = 650ms (Python baseline)",
        hick_reaction_time(7, a, b),
        650.0,
        tolerances::ANALYTICAL_TOL,
    );
}

fn validate_hierarchical_menu(h: &mut ValidationHarness) {
    let a = tolerances::HICK_A_MS;
    let b = tolerances::HICK_B_MS;

    let flat_64 = hick_reaction_time(64, a, b);
    let hier_8x8 = hick_reaction_time(8, a, b) + hick_reaction_time(8, a, b);
    let hier_4x4x4 =
        hick_reaction_time(4, a, b) + hick_reaction_time(4, a, b) + hick_reaction_time(4, a, b);

    h.check_bool(
        "hierarchy trades breadth for depth (8x8 > flat-64 when a is high)",
        hier_8x8 > flat_64,
    );
    h.check_bool(
        "4x4x4 hierarchy slower than 8x8 (depth overhead)",
        hier_4x4x4 > hier_8x8,
    );
}

fn validate_combined_cost(h: &mut ValidationHarness) {
    let cost = interaction_cost(
        100.0,
        20.0,
        5,
        tolerances::FITTS_A_MOUSE_MS,
        tolerances::FITTS_B_MOUSE_MS,
        tolerances::HICK_A_MS,
        tolerances::HICK_B_MS,
    );

    let fitts = tolerances::FITTS_B_MOUSE_MS.mul_add(
        (2.0 * 100.0 / 20.0 + 1.0_f64).log2(),
        tolerances::FITTS_A_MOUSE_MS,
    );
    let hick = tolerances::HICK_B_MS.mul_add(6.0_f64.log2(), tolerances::HICK_A_MS);
    let expected = fitts + hick;

    h.check_abs(
        "interaction_cost = Fitts + Hick",
        cost,
        expected,
        tolerances::ANALYTICAL_TOL,
    );
}

fn main() {
    let mut h = ValidationHarness::new("exp006_hick_menu_depth");
    h.print_provenance(&[&PROVENANCE]);

    validate_flat_menu(&mut h);
    validate_hierarchical_menu(&mut h);
    validate_combined_cost(&mut h);

    h.finish();
}

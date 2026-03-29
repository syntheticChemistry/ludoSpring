// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
//! Exp013: L-system protein backbone growth — validation binary.
//!
//! Validates Lindenmayer system implementations: algae growth (Fibonacci
//! sequence), Koch curve (self-similar fractal), and protein backbone
//! generation (helix/sheet/linker/turn elements).
//!
//! # Provenance
//!
//! Lindenmayer (1968). "Mathematical models for cellular interactions."
//! Prusinkiewicz & Lindenmayer (1990). "The Algorithmic Beauty of Plants."

use ludospring_barracuda::procedural::lsystem::{presets, turtle_interpret};
use ludospring_barracuda::tolerances;
use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "N/A (analytical — Lindenmayer 1968, Prusinkiewicz 1990)",
    commit: "4b683e3e",
    date: "2026-03-15",
    command: "N/A (analytical)",
};

#[expect(
    clippy::cast_precision_loss,
    reason = "L-system lengths < 10000; fits in f64"
)]
fn validate_algae_fibonacci(h: &mut ValidationHarness) {
    let sys = presets::algae();
    let fibonacci = [1, 2, 3, 5, 8, 13, 21, 34];

    for (generation, &expected) in fibonacci.iter().enumerate() {
        #[expect(
            clippy::cast_possible_truncation,
            reason = "generation index < 8; fits in u32"
        )]
        let gen_u32 = generation as u32;
        let actual = sys.symbol_count(gen_u32);
        h.check_abs(
            &format!("gen {generation}: length = {expected} (Fibonacci)"),
            actual as f64,
            f64::from(expected),
            tolerances::ANALYTICAL_TOL,
        );
    }
}

#[expect(
    clippy::cast_precision_loss,
    reason = "symbol counts < 10000; fits in f64"
)]
fn validate_koch_growth(h: &mut ValidationHarness) {
    let sys = presets::koch_curve();

    let g0 = sys.symbol_count(0);
    let g1 = sys.symbol_count(1);

    h.check_abs(
        "Koch gen 0: 1 symbol",
        g0 as f64,
        1.0,
        tolerances::ANALYTICAL_TOL,
    );

    h.check_abs(
        "Koch gen 1: 9 symbols (F+F-F-F+F)",
        g1 as f64,
        9.0,
        tolerances::ANALYTICAL_TOL,
    );
}

fn validate_protein_backbone(h: &mut ValidationHarness) {
    let sys = presets::protein_backbone();
    let gen3 = sys.generate(3);

    let has_helix = gen3.contains('H');
    let has_sheet = gen3.contains('S');
    let has_linker = gen3.contains('L');
    let has_turn = gen3.contains('T');

    h.check_bool(
        "gen 3 contains all structural elements (H, S, L, T)",
        has_helix && has_sheet && has_linker && has_turn,
    );

    let g1 = sys.symbol_count(1);
    let g2 = sys.symbol_count(2);
    let g3 = sys.symbol_count(3);
    h.check_bool("protein grows: gen1 < gen2 < gen3", g1 < g2 && g2 < g3);
}

fn validate_turtle_geometry(h: &mut ValidationHarness) {
    let points = turtle_interpret("FF", 1.0, 90.0);
    h.check_abs(
        "two forward steps → x=2.0",
        points.last().map_or(0.0, |p| p.0),
        2.0,
        tolerances::ANALYTICAL_TOL,
    );

    let square = turtle_interpret("F+F+F+F", 1.0, 90.0);
    let end = square.last().copied().unwrap_or((0.0, 0.0));
    h.check_abs(
        "F+F+F+F with 90° returns near origin",
        end.0.hypot(end.1),
        0.0,
        1e-8,
    );
}

fn validate_determinism(h: &mut ValidationHarness) {
    let sys = presets::dragon_curve();
    let a = sys.generate(6);
    let b = sys.generate(6);
    h.check_bool("dragon curve gen 6 is deterministic", a == b);
}

fn main() {
    let mut h = ValidationHarness::new("exp013_lsystem_protein_backbone");
    h.print_provenance(&[&PROVENANCE]);

    validate_algae_fibonacci(&mut h);
    validate_koch_growth(&mut h);
    validate_protein_backbone(&mut h);
    validate_turtle_geometry(&mut h);
    validate_determinism(&mut h);

    h.finish();
}

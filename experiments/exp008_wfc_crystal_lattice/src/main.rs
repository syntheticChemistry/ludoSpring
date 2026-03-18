// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
//! Exp008: WFC crystal lattice generation — validation binary.
//!
//! Uses Wave Function Collapse to generate chemically valid crystal lattice
//! structures under adjacency constraints. Validates constraint propagation,
//! determinism, and structural validity.
//!
//! # Provenance
//!
//! Gumin, M. (2016). "WFC" — procedural bitmap generation.
//! Karth, I. & Smith, A.M. (2017). "WFC is constraint solving." FDG '17.
//! Crystal lattice rules: `NaCl` (rock salt) structure — Na must neighbor Cl.

use ludospring_barracuda::procedural::wfc::{AdjacencyRules, WfcCell, WfcGrid};
use ludospring_barracuda::tolerances;
use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "N/A (analytical — Gumin 2016, Karth & Smith 2017)",
    commit: "74cf9488",
    date: "2026-03-15",
    command: "N/A (deterministic WFC + NaCl adjacency rules)",
};

fn nacl_rules() -> AdjacencyRules {
    use std::collections::BTreeSet;
    let na_neighbors = BTreeSet::from([1]);
    let cl_neighbors = BTreeSet::from([0]);

    AdjacencyRules {
        right: vec![na_neighbors.clone(), cl_neighbors.clone()],
        up: vec![na_neighbors, cl_neighbors],
    }
}

#[expect(
    clippy::cast_precision_loss,
    reason = "grid entropy ≤ 65535; fits in f64 mantissa"
)]
fn validate_unconstrained(h: &mut ValidationHarness) {
    let grid = WfcGrid::new(4, 4, 3);
    h.check_abs(
        "new 4x4 grid with 3 tiles: each cell has entropy 3",
        grid.get(0, 0).map_or(0.0, |c| c.entropy() as f64),
        3.0,
        tolerances::ANALYTICAL_TOL,
    );
    h.check_bool(
        "uncollapsed grid is not fully collapsed",
        !grid.is_fully_collapsed(),
    );
}

#[expect(
    clippy::cast_precision_loss,
    reason = "grid cell counts ≤ 1024; fits in f64"
)]
fn validate_nacl_propagation(h: &mut ValidationHarness) {
    let rules = nacl_rules();
    let mut grid = WfcGrid::new(4, 4, 2);

    grid.collapse(0, 0, 0);
    let removed = grid.propagate(&rules);

    h.check_lower(
        "NaCl rules propagate (options removed > 0)",
        removed as f64,
        1.0,
    );

    if let Some(cell) = grid.get(1, 0) {
        h.check_bool(
            "cell (1,0) forced to Cl after propagating Na at (0,0)",
            cell.is_collapsed() && cell.collapsed_tile() == Some(1),
        );
    }

    if let Some(cell) = grid.get(0, 1) {
        h.check_bool(
            "cell (0,1) forced to Cl above Na at (0,0)",
            cell.is_collapsed() && cell.collapsed_tile() == Some(1),
        );
    }
}

#[expect(
    clippy::cast_precision_loss,
    reason = "propagation counts ≤ grid size; fits in f64"
)]
fn validate_determinism(h: &mut ValidationHarness) {
    let rules = nacl_rules();

    let mut g1 = WfcGrid::new(4, 4, 2);
    g1.collapse(0, 0, 0);
    let r1 = g1.propagate(&rules);

    let mut g2 = WfcGrid::new(4, 4, 2);
    g2.collapse(0, 0, 0);
    let r2 = g2.propagate(&rules);

    h.check_abs(
        "same seed produces same propagation count",
        r1 as f64,
        r2 as f64,
        tolerances::ANALYTICAL_TOL,
    );

    let all_match = (0..4).all(|y| {
        (0..4).all(|x| g1.get(x, y).map(WfcCell::entropy) == g2.get(x, y).map(WfcCell::entropy))
    });
    h.check_bool(
        "cell-by-cell entropy matches between identical runs",
        all_match,
    );
}

fn main() {
    let mut h = ValidationHarness::new("exp008_wfc_crystal_lattice");
    h.print_provenance(&[&PROVENANCE]);

    validate_unconstrained(&mut h);
    validate_nacl_propagation(&mut h);
    validate_determinism(&mut h);

    h.finish();
}

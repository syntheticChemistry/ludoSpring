// SPDX-License-Identifier: AGPL-3.0-or-later
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
use ludospring_barracuda::validation::ValidationResult;

fn report(r: &ValidationResult) {
    if r.passed {
        println!("  PASS  {}: {}", r.experiment, r.description);
    } else {
        println!(
            "  FAIL  {}: {} (got={:.4}, want={:.4}, tol={:.4})",
            r.experiment, r.description, r.measured, r.expected, r.tolerance
        );
    }
}

fn nacl_rules() -> AdjacencyRules {
    use std::collections::BTreeSet;
    // Tile 0 = Na, Tile 1 = Cl
    // NaCl rule: Na neighbors Cl, Cl neighbors Na (alternating)
    let na_neighbors = BTreeSet::from([1]); // Na can only be next to Cl
    let cl_neighbors = BTreeSet::from([0]); // Cl can only be next to Na

    AdjacencyRules {
        right: vec![na_neighbors.clone(), cl_neighbors.clone()],
        up: vec![na_neighbors, cl_neighbors],
    }
}

#[expect(
    clippy::cast_precision_loss,
    reason = "grid entropy ≤ 65535; fits in f64 mantissa"
)]
fn validate_unconstrained(results: &mut Vec<ValidationResult>) {
    println!("Part 1: Unconstrained WFC");
    let grid = WfcGrid::new(4, 4, 3);
    let r = ValidationResult::check(
        "exp008_initial_entropy",
        "new 4x4 grid with 3 tiles: each cell has entropy 3",
        grid.get(0, 0).map_or(0.0, |c| c.entropy() as f64),
        3.0,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);

    let r = ValidationResult::check(
        "exp008_not_collapsed",
        "uncollapsed grid is not fully collapsed",
        if grid.is_fully_collapsed() { 0.0 } else { 1.0 },
        1.0,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);
}

#[expect(
    clippy::cast_precision_loss,
    reason = "grid cell counts ≤ 1024; fits in f64"
)]
fn validate_nacl_propagation(results: &mut Vec<ValidationResult>) {
    println!("\nPart 2: NaCl crystal lattice propagation");
    let rules = nacl_rules();
    let mut grid = WfcGrid::new(4, 4, 2);

    // Place Na at (0,0)
    grid.collapse(0, 0, 0);
    let removed = grid.propagate(&rules);

    // Full constraint propagation on a 4x4 binary-tile grid resolves many
    // cells in a single pass — count depends on grid topology.
    let r = ValidationResult::check(
        "exp008_propagation",
        "NaCl rules propagate (options removed > 0)",
        removed as f64,
        15.0,
        1.0,
    );
    report(&r);
    results.push(r);

    // Cell (1,0) should be Cl (only valid neighbor of Na to the right)
    if let Some(cell) = grid.get(1, 0) {
        let r = ValidationResult::check(
            "exp008_nacl_neighbor",
            "cell (1,0) forced to Cl after propagating Na at (0,0)",
            if cell.is_collapsed() && cell.collapsed_tile() == Some(1) {
                1.0
            } else {
                0.0
            },
            1.0,
            tolerances::ANALYTICAL_TOL,
        );
        report(&r);
        results.push(r);
    }

    // Cell (0,1) should be Cl (above Na)
    if let Some(cell) = grid.get(0, 1) {
        let r = ValidationResult::check(
            "exp008_nacl_above",
            "cell (0,1) forced to Cl above Na at (0,0)",
            if cell.is_collapsed() && cell.collapsed_tile() == Some(1) {
                1.0
            } else {
                0.0
            },
            1.0,
            tolerances::ANALYTICAL_TOL,
        );
        report(&r);
        results.push(r);
    }
}

#[expect(
    clippy::cast_precision_loss,
    reason = "propagation counts ≤ grid size; fits in f64"
)]
fn validate_determinism(results: &mut Vec<ValidationResult>) {
    println!("\nPart 3: WFC determinism");
    let rules = nacl_rules();

    let mut g1 = WfcGrid::new(4, 4, 2);
    g1.collapse(0, 0, 0);
    let r1 = g1.propagate(&rules);

    let mut g2 = WfcGrid::new(4, 4, 2);
    g2.collapse(0, 0, 0);
    let r2 = g2.propagate(&rules);

    let r = ValidationResult::check(
        "exp008_deterministic",
        "same seed produces same propagation count",
        r1 as f64,
        r2 as f64,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);

    // Verify cell-by-cell match
    let mut all_match = true;
    for y in 0..4 {
        for x in 0..4 {
            let e1 = g1.get(x, y).map(WfcCell::entropy);
            let e2 = g2.get(x, y).map(WfcCell::entropy);
            if e1 != e2 {
                all_match = false;
            }
        }
    }
    let r = ValidationResult::check(
        "exp008_cell_match",
        "cell-by-cell entropy matches between identical runs",
        if all_match { 1.0 } else { 0.0 },
        1.0,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);
}

fn main() {
    println!("=== Exp008: WFC Crystal Lattice (Validation) ===\n");
    let mut results = Vec::new();

    validate_unconstrained(&mut results);
    validate_nacl_propagation(&mut results);
    validate_determinism(&mut results);

    let passed = results.iter().filter(|r| r.passed).count();
    let failed = results.len() - passed;
    println!("\n{passed} passed, {failed} failed");
    if failed > 0 {
        std::process::exit(1);
    }
}

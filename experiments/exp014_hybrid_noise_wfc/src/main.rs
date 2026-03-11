// SPDX-License-Identifier: AGPL-3.0-or-later
//! Exp014: Hybrid noise+WFC voxel world — validation binary.
//!
//! Combines Perlin noise density fields with Wave Function Collapse
//! constraint propagation to generate chemically valid voxel worlds
//! where atom placement follows both spatial coherence (noise) and
//! adjacency rules (WFC).
//!
//! # Provenance
//!
//! Perlin (1985, 2002): noise for density distribution.
//! Gumin (2016), Karth & Smith (2017): WFC for constraint satisfaction.

use ludospring_barracuda::procedural::noise::{fbm_2d, perlin_2d};
use ludospring_barracuda::procedural::wfc::{AdjacencyRules, WfcGrid};
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

#[expect(
    clippy::cast_precision_loss,
    reason = "grid counts ≤ 1024; fits in f64"
)]
fn validate_noise_seeded_wfc(results: &mut Vec<ValidationResult>) {
    println!("Part 1: Noise-seeded WFC grid");
    let grid_size = 8;
    let n_tiles = 3; // 0=void, 1=rock, 2=crystal

    let mut grid = WfcGrid::new(grid_size, grid_size, n_tiles);

    // Use noise to seed initial collapses: high density → rock
    let mut seeded = 0;
    for y in 0..grid_size {
        for x in 0..grid_size {
            let density = perlin_2d(x as f64 * 0.5, y as f64 * 0.5);
            if density > 0.3 {
                grid.collapse(x, y, 1); // rock
                seeded += 1;
            } else if density < -0.3 {
                grid.collapse(x, y, 2); // crystal
                seeded += 1;
            }
        }
    }

    let r = ValidationResult::check(
        "exp014_seeds",
        "noise seeding places some tiles (not zero, not all)",
        if seeded > 0 && seeded < grid_size * grid_size {
            1.0
        } else {
            0.0
        },
        1.0,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);

    // Propagate unconstrained rules (any tile can neighbor any tile)
    let rules = AdjacencyRules::unconstrained(n_tiles);
    let removed = grid.propagate(&rules);
    let r = ValidationResult::check(
        "exp014_propagation",
        "propagation runs without contradiction",
        if grid.has_contradiction() { 0.0 } else { 1.0 },
        1.0,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);

    println!(
        "  (seeded {seeded} of {} cells, removed {removed} options)",
        grid_size * grid_size
    );
}

#[expect(
    clippy::cast_precision_loss,
    reason = "grid counts ≤ 1024; fits in f64"
)]
fn validate_density_coherence(results: &mut Vec<ValidationResult>) {
    println!("\nPart 2: Noise density coherence in generated world");
    // Generate a density field and verify spatial coherence
    let size = 16;
    let mut field = Vec::with_capacity(size * size);
    for y in 0..size {
        for x in 0..size {
            field.push(fbm_2d(x as f64 * 0.2, y as f64 * 0.2, 4, 2.0, 0.5));
        }
    }

    // Adjacent cells should be correlated
    let mut total_diff = 0.0;
    let mut count = 0;
    for y in 0..size {
        for x in 0..size - 1 {
            total_diff += (field[y * size + x] - field[y * size + x + 1]).abs();
            count += 1;
        }
    }
    let avg_diff = total_diff / f64::from(count);

    let r = ValidationResult::check(
        "exp014_coherence",
        "average adjacent cell difference < 0.3 (spatial coherence)",
        avg_diff,
        0.0,
        0.3,
    );
    report(&r);
    results.push(r);
}

fn validate_hybrid_determinism(results: &mut Vec<ValidationResult>) {
    println!("\nPart 3: Hybrid pipeline determinism");
    // Same noise seed + same WFC rules → same result
    let density_a = perlin_2d(1.5, 2.5);
    let density_b = perlin_2d(1.5, 2.5);

    let r = ValidationResult::check(
        "exp014_noise_det",
        "noise is deterministic",
        (density_a - density_b).abs(),
        0.0,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);

    let rules = AdjacencyRules::unconstrained(2);
    let mut g1 = WfcGrid::new(4, 4, 2);
    g1.collapse(0, 0, 0);
    let r1 = g1.propagate(&rules);

    let mut g2 = WfcGrid::new(4, 4, 2);
    g2.collapse(0, 0, 0);
    let r2 = g2.propagate(&rules);

    let r = ValidationResult::check(
        "exp014_wfc_det",
        "WFC propagation is deterministic",
        if r1 == r2 { 1.0 } else { 0.0 },
        1.0,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);
}

fn main() {
    println!("=== Exp014: Hybrid Noise+WFC Voxel World (Validation) ===\n");
    let mut results = Vec::new();

    validate_noise_seeded_wfc(&mut results);
    validate_density_coherence(&mut results);
    validate_hybrid_determinism(&mut results);

    let passed = results.iter().filter(|r| r.passed).count();
    let failed = results.len() - passed;
    println!("\n{passed} passed, {failed} failed");
    if failed > 0 {
        std::process::exit(1);
    }
}

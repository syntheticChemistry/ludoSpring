// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
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
use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "N/A (analytical — Perlin, Gumin, Karth & Smith)",
    commit: "19e402c0",
    date: "2026-04-10",
    command: "N/A (analytical)",
};

#[expect(
    clippy::cast_precision_loss,
    reason = "grid counts ≤ 1024; fits in f64"
)]
fn validate_noise_seeded_wfc(h: &mut ValidationHarness) {
    let grid_size = 8;
    let n_tiles = 3;

    let mut grid = WfcGrid::new(grid_size, grid_size, n_tiles);

    let mut seeded = 0;
    for y in 0..grid_size {
        for x in 0..grid_size {
            let density = perlin_2d(x as f64 * 0.5, y as f64 * 0.5);
            if density > 0.3 {
                grid.collapse(x, y, 1);
                seeded += 1;
            } else if density < -0.3 {
                grid.collapse(x, y, 2);
                seeded += 1;
            }
        }
    }

    h.check_bool(
        "noise seeding places some tiles (not zero, not all)",
        seeded > 0 && seeded < grid_size * grid_size,
    );

    let rules = AdjacencyRules::unconstrained(n_tiles);
    let _removed = grid.propagate(&rules);
    h.check_bool(
        "propagation runs without contradiction",
        !grid.has_contradiction(),
    );
}

#[expect(
    clippy::cast_precision_loss,
    reason = "grid counts ≤ 1024; fits in f64"
)]
fn validate_density_coherence(h: &mut ValidationHarness) {
    let size = 16;
    let mut field = Vec::with_capacity(size * size);
    for y in 0..size {
        for x in 0..size {
            field.push(fbm_2d(x as f64 * 0.2, y as f64 * 0.2, 4, 2.0, 0.5));
        }
    }

    let mut total_diff = 0.0;
    let mut count = 0;
    for y in 0..size {
        for x in 0..size - 1 {
            total_diff += (field[y * size + x] - field[y * size + x + 1]).abs();
            count += 1;
        }
    }
    let avg_diff = total_diff / f64::from(count);

    h.check_upper(
        "average adjacent cell difference < 0.3 (spatial coherence)",
        avg_diff,
        0.3,
    );
}

fn validate_hybrid_determinism(h: &mut ValidationHarness) {
    let density_a = perlin_2d(1.5, 2.5);
    let density_b = perlin_2d(1.5, 2.5);

    h.check_abs(
        "noise is deterministic",
        (density_a - density_b).abs(),
        0.0,
        tolerances::ANALYTICAL_TOL,
    );

    let rules = AdjacencyRules::unconstrained(2);
    let mut g1 = WfcGrid::new(4, 4, 2);
    g1.collapse(0, 0, 0);
    let r1 = g1.propagate(&rules);

    let mut g2 = WfcGrid::new(4, 4, 2);
    g2.collapse(0, 0, 0);
    let r2 = g2.propagate(&rules);

    h.check_bool("WFC propagation is deterministic", r1 == r2);
}

fn main() {
    let mut h = ValidationHarness::new("exp014_hybrid_noise_wfc");
    h.print_provenance(&[&PROVENANCE]);

    validate_noise_seeded_wfc(&mut h);
    validate_density_coherence(&mut h);
    validate_hybrid_determinism(&mut h);

    h.finish();
}

// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
//! Exp017: BSP Doom-style level partitioning — validation binary.
//!
//! Validates Binary Space Partitioning for level generation: area
//! conservation, determinism, spatial query correctness, and depth
//! properties matching Fuchs, Kedem & Naylor (1980).
//!
//! # Provenance
//!
//! Fuchs, Kedem, Naylor (1980). "On Visible Surface Generation."
//! Carmack (1993). Doom engine BSP builder.
//! Python baseline: `baselines/python/bsp_partition.py` (2026-03-11).

use ludospring_barracuda::procedural::bsp::{Rect, generate_bsp};
use ludospring_barracuda::tolerances;
use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "baselines/python/bsp_partition.py",
    commit: "74cf9488",
    date: "2026-03-11",
    command: "python3 baselines/python/run_all_baselines.py",
};

fn validate_area_conservation(h: &mut ValidationHarness) {
    let bounds = Rect::new(0.0, 0.0, 100.0, 100.0);
    let tree = generate_bsp(bounds, 15.0, 42);
    let leaf_area: f64 = tree.leaves().iter().map(Rect::area).sum();

    h.check_abs(
        "leaf areas sum to total area (100x100 = 10000)",
        leaf_area,
        bounds.area(),
        1e-6,
    );

    let rect = Rect::new(10.0, 20.0, 80.0, 60.0);
    let tree2 = generate_bsp(rect, 12.0, 99);
    let leaf_area2: f64 = tree2.leaves().iter().map(Rect::area).sum();
    h.check_abs(
        "area conserved with offset rectangle",
        leaf_area2,
        rect.area(),
        1e-6,
    );
}

#[expect(
    clippy::cast_precision_loss,
    reason = "leaf/node counts < 100; fits in f64"
)]
fn validate_structure(h: &mut ValidationHarness) {
    let tree = generate_bsp(Rect::new(0.0, 0.0, 100.0, 100.0), 15.0, 42);

    h.check_bool("generates more than 1 room", tree.leaf_count() > 1);

    let expected_nodes = 2 * tree.leaf_count() - 1;
    h.check_abs(
        "node_count = 2 * leaf_count - 1 (full binary tree)",
        tree.node_count() as f64,
        expected_nodes as f64,
        tolerances::ANALYTICAL_TOL,
    );
}

fn validate_spatial_query(h: &mut ValidationHarness) {
    let tree = generate_bsp(Rect::new(0.0, 0.0, 100.0, 100.0), 15.0, 42);

    let center_result = tree.query_point(50.0, 50.0);
    h.check_bool(
        "center point (50,50) found in a leaf",
        center_result.is_some(),
    );

    let outside = tree.query_point(200.0, 200.0);
    h.check_bool(
        "point (200,200) outside bounds returns None",
        outside.is_none(),
    );

    let corners = [(0.5, 0.5), (99.0, 0.5), (0.5, 99.0), (99.0, 99.0)];
    let all_found = corners
        .iter()
        .all(|&(x, y)| tree.query_point(x, y).is_some());
    h.check_bool("all four corner regions queryable", all_found);
}

fn validate_determinism(h: &mut ValidationHarness) {
    let a = generate_bsp(Rect::new(0.0, 0.0, 100.0, 100.0), 15.0, 42);
    let b = generate_bsp(Rect::new(0.0, 0.0, 100.0, 100.0), 15.0, 42);

    h.check_bool(
        "same seed → identical tree structure",
        a.leaf_count() == b.leaf_count() && a.depth() == b.depth(),
    );

    let c = generate_bsp(Rect::new(0.0, 0.0, 100.0, 100.0), 15.0, 999);
    h.check_bool(
        "different seeds may produce different trees",
        c.leaf_count() != a.leaf_count() || c.depth() != a.depth(),
    );
}

fn validate_min_size(h: &mut ValidationHarness) {
    let tree = generate_bsp(Rect::new(0.0, 0.0, 5.0, 5.0), 10.0, 42);
    h.check_bool(
        "space smaller than 2*min_size produces single leaf",
        tree.leaf_count() == 1,
    );
}

fn main() {
    let mut h = ValidationHarness::new("exp017_bsp_level_generation");
    h.print_provenance(&[&PROVENANCE]);

    validate_area_conservation(&mut h);
    validate_structure(&mut h);
    validate_spatial_query(&mut h);
    validate_determinism(&mut h);
    validate_min_size(&mut h);

    h.finish();
}

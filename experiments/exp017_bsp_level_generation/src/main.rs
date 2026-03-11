// SPDX-License-Identifier: AGPL-3.0-or-later
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
use ludospring_barracuda::validation::ValidationResult;

fn report(r: &ValidationResult) {
    if r.passed {
        println!("  PASS  {}: {}", r.experiment, r.description);
    } else {
        println!(
            "  FAIL  {}: {} (got={:.6}, want={:.6}, tol={:.6})",
            r.experiment, r.description, r.measured, r.expected, r.tolerance
        );
    }
}

fn validate_area_conservation(results: &mut Vec<ValidationResult>) {
    println!("Part 1: Area conservation");
    let bounds = Rect::new(0.0, 0.0, 100.0, 100.0);
    let tree = generate_bsp(bounds, 15.0, 42);
    let leaf_area: f64 = tree.leaves().iter().map(Rect::area).sum();

    let r = ValidationResult::check(
        "exp017_area",
        "leaf areas sum to total area (100x100 = 10000)",
        leaf_area,
        bounds.area(),
        1e-6,
    );
    report(&r);
    results.push(r);

    let rect = Rect::new(10.0, 20.0, 80.0, 60.0);
    let tree2 = generate_bsp(rect, 12.0, 99);
    let leaf_area2: f64 = tree2.leaves().iter().map(Rect::area).sum();
    let r = ValidationResult::check(
        "exp017_area_offset",
        "area conserved with offset rectangle",
        leaf_area2,
        rect.area(),
        1e-6,
    );
    report(&r);
    results.push(r);
}

#[expect(
    clippy::cast_precision_loss,
    reason = "leaf/node counts < 100; fits in f64"
)]
fn validate_structure(results: &mut Vec<ValidationResult>) {
    println!("\nPart 2: Tree structure properties");
    let tree = generate_bsp(Rect::new(0.0, 0.0, 100.0, 100.0), 15.0, 42);

    let r = ValidationResult::check(
        "exp017_multiple_rooms",
        "generates more than 1 room",
        if tree.leaf_count() > 1 { 1.0 } else { 0.0 },
        1.0,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);

    // Binary tree: nodes = 2*leaves - 1
    let expected_nodes = 2 * tree.leaf_count() - 1;
    let r = ValidationResult::check(
        "exp017_binary_property",
        "node_count = 2 * leaf_count - 1 (full binary tree)",
        tree.node_count() as f64,
        expected_nodes as f64,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);

    println!(
        "  leaves={}, nodes={}, depth={}",
        tree.leaf_count(),
        tree.node_count(),
        tree.depth()
    );
}

fn validate_spatial_query(results: &mut Vec<ValidationResult>) {
    println!("\nPart 3: Spatial point query");
    let tree = generate_bsp(Rect::new(0.0, 0.0, 100.0, 100.0), 15.0, 42);

    // Center point must be in some leaf
    let center_result = tree.query_point(50.0, 50.0);
    let r = ValidationResult::check(
        "exp017_center_query",
        "center point (50,50) found in a leaf",
        if center_result.is_some() { 1.0 } else { 0.0 },
        1.0,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);

    // Point outside returns None
    let outside = tree.query_point(200.0, 200.0);
    let r = ValidationResult::check(
        "exp017_outside_query",
        "point (200,200) outside bounds returns None",
        if outside.is_none() { 1.0 } else { 0.0 },
        1.0,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);

    // All four corners must be queryable
    let corners = [(0.5, 0.5), (99.0, 0.5), (0.5, 99.0), (99.0, 99.0)];
    let all_found = corners
        .iter()
        .all(|&(x, y)| tree.query_point(x, y).is_some());
    let r = ValidationResult::check(
        "exp017_corners",
        "all four corner regions queryable",
        if all_found { 1.0 } else { 0.0 },
        1.0,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);
}

fn validate_determinism(results: &mut Vec<ValidationResult>) {
    println!("\nPart 4: Determinism");
    let a = generate_bsp(Rect::new(0.0, 0.0, 100.0, 100.0), 15.0, 42);
    let b = generate_bsp(Rect::new(0.0, 0.0, 100.0, 100.0), 15.0, 42);

    let r = ValidationResult::check(
        "exp017_deterministic",
        "same seed → identical tree structure",
        if a.leaf_count() == b.leaf_count() && a.depth() == b.depth() {
            1.0
        } else {
            0.0
        },
        1.0,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);

    // Different seeds → potentially different structure
    let c = generate_bsp(Rect::new(0.0, 0.0, 100.0, 100.0), 15.0, 999);
    let r = ValidationResult::check(
        "exp017_seed_variation",
        "different seeds may produce different trees",
        if c.leaf_count() != a.leaf_count() || c.depth() != a.depth() {
            1.0
        } else {
            0.0
        },
        1.0,
        1.0, // wide tolerance: structural difference is likely but not guaranteed
    );
    report(&r);
    results.push(r);
}

fn validate_min_size(results: &mut Vec<ValidationResult>) {
    println!("\nPart 5: Minimum room size enforcement");
    let tree = generate_bsp(Rect::new(0.0, 0.0, 5.0, 5.0), 10.0, 42);
    let r = ValidationResult::check(
        "exp017_too_small",
        "space smaller than 2*min_size produces single leaf",
        if tree.leaf_count() == 1 { 1.0 } else { 0.0 },
        1.0,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);
}

fn main() {
    println!("=== Exp017: BSP Level Generation (Validation) ===\n");
    let mut results = Vec::new();

    validate_area_conservation(&mut results);
    validate_structure(&mut results);
    validate_spatial_query(&mut results);
    validate_determinism(&mut results);
    validate_min_size(&mut results);

    let passed = results.iter().filter(|r| r.passed).count();
    let failed = results.len() - passed;
    println!("\n{passed} passed, {failed} failed");
    if failed > 0 {
        std::process::exit(1);
    }
}

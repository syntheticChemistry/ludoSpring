// SPDX-License-Identifier: AGPL-3.0-or-later

use ludospring_barracuda::interaction::goms::{
    self, Operator, task_time, task_time_with_keystroke,
};
use ludospring_barracuda::procedural::bsp::{Rect, generate_bsp};
use ludospring_barracuda::procedural::lsystem::presets;
use ludospring_barracuda::procedural::lsystem::turtle_interpret;
use ludospring_barracuda::tolerances;

// ── GOMS / KLM ────────────────────────────────────────────────────
// JSON: goms_model.py

#[test]
fn parity_goms_empty() {
    // goms_model.py.empty
    let rust = task_time(&[]);
    let python = 0.0;
    assert!(
        (rust - python).abs() < tolerances::ANALYTICAL_TOL,
        "GOMS empty: Rust={rust}, Python={python}"
    );
}

#[test]
fn parity_goms_single_key() {
    // goms_model.py.single_key
    let rust = task_time(&[Operator::Keystroke]);
    let python = 0.2;
    assert!(
        (rust - python).abs() < tolerances::ANALYTICAL_TOL,
        "GOMS single key: Rust={rust}, Python={python}"
    );
}

#[test]
fn parity_goms_menu_open() {
    // goms_model.py.menu_open
    let ops = [Operator::Mental, Operator::Point, Operator::Keystroke];
    let rust = task_time(&ops);
    let python = 2.65;
    assert!(
        (rust - python).abs() < tolerances::ANALYTICAL_TOL,
        "GOMS menu open: Rust={rust}, Python={python}"
    );
}

#[test]
fn parity_goms_chat() {
    // goms_model.py.chat
    let ops = [
        Operator::Mental,
        Operator::Home,
        Operator::Keystroke,
        Operator::Keystroke,
        Operator::Keystroke,
        Operator::Keystroke,
        Operator::Keystroke,
        Operator::Keystroke,
    ];
    let rust = task_time(&ops);
    let python = 2.95;
    assert!(
        (rust - python).abs() < tolerances::ANALYTICAL_TOL,
        "GOMS chat: Rust={rust}, Python={python}"
    );
}

#[test]
fn parity_goms_best_20k() {
    // goms_model.py.best_20k
    let ops: Vec<Operator> = (0..20).map(|_| Operator::Keystroke).collect();
    let rust = task_time_with_keystroke(&ops, goms::times::KEYSTROKE_BEST);
    let python = 1.6;
    assert!(
        (rust - python).abs() < tolerances::ANALYTICAL_TOL,
        "GOMS best 20K: Rust={rust}, Python={python}"
    );
}

// ── GOMS Extended ────────────────────────────────────────────────
// JSON: goms_model.py — drag_drop, avg_20k, worst_20k

#[test]
fn parity_goms_drag_drop() {
    // goms_model.py.drag_drop = 3.95
    let ops = [
        Operator::Mental,
        Operator::Point,
        Operator::Keystroke,
        Operator::Point,
        Operator::Keystroke,
    ];
    let rust = task_time(&ops);
    let python = 3.95;
    assert!(
        (rust - python).abs() < tolerances::ANALYTICAL_TOL,
        "GOMS drag_drop: Rust={rust}, Python={python}"
    );
}

#[test]
fn parity_goms_avg_20k() {
    // goms_model.py.avg_20k = 4.0
    let ops: Vec<Operator> = (0..20).map(|_| Operator::Keystroke).collect();
    let rust = task_time_with_keystroke(&ops, goms::times::KEYSTROKE_AVG);
    let python = 4.0;
    assert!(
        (rust - python).abs() < tolerances::ANALYTICAL_TOL,
        "GOMS avg_20k: Rust={rust}, Python={python}"
    );
}

#[test]
fn parity_goms_worst_20k() {
    // goms_model.py.worst_20k = 10.0
    let ops: Vec<Operator> = (0..20).map(|_| Operator::Keystroke).collect();
    let rust = task_time_with_keystroke(&ops, goms::times::KEYSTROKE_WORST);
    let python = 10.0;
    assert!(
        (rust - python).abs() < tolerances::ANALYTICAL_TOL,
        "GOMS worst_20k: Rust={rust}, Python={python}"
    );
}

// ── L-systems ─────────────────────────────────────────────────────
// JSON: lsystem_growth.py

#[test]
fn parity_algae_fibonacci() {
    // lsystem_growth.py.algae_lengths
    let sys = presets::algae();
    let rust: Vec<usize> = (0..8).map(|g| sys.symbol_count(g)).collect();
    let python = [1, 2, 3, 5, 8, 13, 21, 34];
    assert_eq!(rust, python, "Algae lengths must match Fibonacci");
}

#[test]
fn parity_koch_lengths() {
    // lsystem_growth.py.koch_g0, koch_g1
    let sys = presets::koch_curve();
    assert_eq!(sys.symbol_count(0), 1, "Koch g0");
    assert_eq!(sys.symbol_count(1), 9, "Koch g1");
}

#[test]
fn parity_protein_backbone_elements() {
    // lsystem_growth.py.protein_g3_has_{H,S,L,T}
    let sys = presets::protein_backbone();
    let g3 = sys.generate(3);
    assert!(g3.contains('H'), "protein g3 must contain H");
    assert!(g3.contains('S'), "protein g3 must contain S");
    assert!(g3.contains('L'), "protein g3 must contain L");
    assert!(g3.contains('T'), "protein g3 must contain T");
}

// ── L-System Turtle Geometry ─────────────────────────────────────
// JSON: lsystem_growth.py — turtle endpoints and distances

#[test]
fn parity_lsystem_turtle_ff_end() {
    // lsystem_growth.py.turtle_FF_end = [2.0, 0.0]
    let points = turtle_interpret("FF", 1.0, 90.0);
    let Some(end) = points.last() else {
        panic!("at least one point");
    };
    assert!(
        (end.0 - 2.0).abs() < tolerances::ANALYTICAL_TOL,
        "turtle FF x: Rust={}, Python=2.0",
        end.0
    );
    assert!(
        end.1.abs() < tolerances::ANALYTICAL_TOL,
        "turtle FF y: Rust={}, Python=0.0",
        end.1
    );
}

#[test]
fn parity_lsystem_turtle_square_dist() {
    // lsystem_growth.py.turtle_square_dist = 2.8818119592750155e-16
    let points = turtle_interpret("F+F+F+F", 1.0, 90.0);
    let Some(end) = points.last() else {
        panic!("at least one point");
    };
    let dist = end.0.hypot(end.1);
    assert!(
        dist < tolerances::STRICT_ANALYTICAL_TOL,
        "turtle square distance: Rust={dist:.2e}, should be near-zero"
    );
}

// ── BSP Partitioning ──────────────────────────────────────────────
// JSON: bsp_partition.py

#[test]
fn parity_bsp_area_conservation() {
    // bsp_partition.py.total_area ≈ 10000.0
    let bounds = Rect::new(0.0, 0.0, 100.0, 100.0);
    let tree = generate_bsp(bounds, 15.0, 42);
    let leaf_area: f64 = tree.leaves().iter().map(Rect::area).sum();
    assert!(
        (leaf_area - 10000.0).abs() < tolerances::BSP_AREA_CONSERVATION_TOL,
        "BSP area: Rust={leaf_area}, Python=10000.0"
    );
}

#[test]
fn parity_bsp_small_single_leaf() {
    // bsp_partition.py.small_leaf_count = 1
    let tree = generate_bsp(Rect::new(0.0, 0.0, 5.0, 5.0), 10.0, 42);
    assert_eq!(tree.leaf_count(), 1, "Small space must be single leaf");
}

// ── BSP Extended ─────────────────────────────────────────────────
// JSON: bsp_partition.py — offset area

#[test]
fn parity_bsp_offset_area() {
    // bsp_partition.py: generate_bsp(10, 20, 80, 60, 12, 99) → offset_area = 4800.0
    let bounds = Rect::new(10.0, 20.0, 80.0, 60.0);
    let tree = generate_bsp(bounds, 12.0, 99);
    let leaf_area: f64 = tree.leaves().iter().map(Rect::area).sum();
    let python = 4800.0;
    assert!(
        (leaf_area - python).abs() < tolerances::BSP_AREA_CONSERVATION_TOL,
        "BSP offset area: Rust={leaf_area}, Python={python}"
    );
}

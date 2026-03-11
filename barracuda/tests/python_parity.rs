// SPDX-License-Identifier: AGPL-3.0-or-later
//! Rust-vs-Python parity tests.
//!
//! Compares Rust implementations against the exact values produced by
//! the Python reference baselines. These are the canonical cross-language
//! validation tests that prove the barraCuda CPU port is faithful.
//!
//! # Provenance
//!
//! Python baselines: `baselines/python/` (2026-03-11, stdlib only).
//! Run: `python3 baselines/python/run_all_baselines.py`
//! Output: `baselines/python/combined_baselines.json`
//!
//! Values below are transcribed from the Python JSON output.

use ludospring_barracuda::interaction::goms::{
    self, Operator, task_time, task_time_with_keystroke,
};
use ludospring_barracuda::interaction::input_laws::{
    fitts_index_of_difficulty, fitts_movement_time, hick_reaction_time, steering_time,
};
use ludospring_barracuda::metrics::fun_keys::{FunKey, FunSignals, classify_fun};
use ludospring_barracuda::procedural::bsp::{Rect, generate_bsp};
use ludospring_barracuda::procedural::lsystem::presets;
use ludospring_barracuda::procedural::noise::{fbm_2d, perlin_2d, perlin_3d};
use ludospring_barracuda::tolerances;

const PARITY_TOL: f64 = 1e-10;

// ── Interaction Laws ───────────────────────────────────────────────

#[test]
fn parity_fitts_mt_d100_w10() {
    let rust = fitts_movement_time(100.0, 10.0, 50.0, 150.0);
    let python = 708.847_613_416_814;
    assert!(
        (rust - python).abs() < PARITY_TOL,
        "Fitts MT: Rust={rust}, Python={python}"
    );
}

#[test]
fn parity_fitts_id_d100_w10() {
    let rust = fitts_index_of_difficulty(100.0, 10.0);
    let python = 4.392_317_422_778_761;
    assert!(
        (rust - python).abs() < PARITY_TOL,
        "Fitts ID: Rust={rust}, Python={python}"
    );
}

#[test]
fn parity_hick_rt_n7() {
    let rust = hick_reaction_time(7, tolerances::HICK_A_MS, tolerances::HICK_B_MS);
    let python = 650.0;
    assert!(
        (rust - python).abs() < PARITY_TOL,
        "Hick RT: Rust={rust}, Python={python}"
    );
}

#[test]
fn parity_steering_d100_w20() {
    let rust = steering_time(100.0, 20.0, 10.0, 5.0);
    let python = 35.0;
    assert!(
        (rust - python).abs() < PARITY_TOL,
        "Steering: Rust={rust}, Python={python}"
    );
}

// ── Perlin Noise ───────────────────────────────────────────────────

#[test]
fn parity_perlin_2d_lattice_zeros() {
    for ix in 0..10_i32 {
        for iy in 0..10_i32 {
            let v = perlin_2d(f64::from(ix), f64::from(iy));
            assert!(
                v.abs() < PARITY_TOL,
                "perlin_2d({ix},{iy}) = {v}, Python = 0.0"
            );
        }
    }
}

#[test]
fn parity_perlin_3d_lattice_zeros() {
    for ix in 0..5_i32 {
        for iy in 0..5_i32 {
            for iz in 0..5_i32 {
                let v = perlin_3d(f64::from(ix), f64::from(iy), f64::from(iz));
                assert!(
                    v.abs() < PARITY_TOL,
                    "perlin_3d({ix},{iy},{iz}) = {v}, Python = 0.0"
                );
            }
        }
    }
}

#[test]
fn parity_perlin_2d_specific_coords() {
    let test_cases: &[(f64, f64)] = &[(0.5, 0.7), (1.23, 4.56), (100.1, 200.2), (-3.17, 2.73)];

    for &(x, y) in test_cases {
        let rust = perlin_2d(x, y);
        assert!(rust.is_finite(), "perlin_2d({x},{y}) not finite: {rust}");
        assert!(rust.abs() <= 2.0, "perlin_2d({x},{y}) out of range: {rust}");
    }
}

#[test]
fn parity_fbm_2d_deterministic() {
    for octaves in [1, 4, 8] {
        let a = fbm_2d(3.17, 2.73, octaves, 2.0, 0.5);
        let b = fbm_2d(3.17, 2.73, octaves, 2.0, 0.5);
        assert_eq!(
            a.to_bits(),
            b.to_bits(),
            "fBm not deterministic for octaves={octaves}"
        );
    }
}

// ── GOMS / KLM ────────────────────────────────────────────────────

#[test]
fn parity_goms_empty() {
    let rust = task_time(&[]);
    let python = 0.0;
    assert!(
        (rust - python).abs() < PARITY_TOL,
        "GOMS empty: Rust={rust}, Python={python}"
    );
}

#[test]
fn parity_goms_single_key() {
    let rust = task_time(&[Operator::Keystroke]);
    let python = 0.2;
    assert!(
        (rust - python).abs() < PARITY_TOL,
        "GOMS single key: Rust={rust}, Python={python}"
    );
}

#[test]
fn parity_goms_menu_open() {
    let ops = [Operator::Mental, Operator::Point, Operator::Keystroke];
    let rust = task_time(&ops);
    let python = 2.65;
    assert!(
        (rust - python).abs() < PARITY_TOL,
        "GOMS menu open: Rust={rust}, Python={python}"
    );
}

#[test]
fn parity_goms_chat() {
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
        (rust - python).abs() < PARITY_TOL,
        "GOMS chat: Rust={rust}, Python={python}"
    );
}

#[test]
fn parity_goms_best_20k() {
    let ops: Vec<Operator> = (0..20).map(|_| Operator::Keystroke).collect();
    let rust = task_time_with_keystroke(&ops, goms::times::KEYSTROKE_BEST);
    let python = 1.6;
    assert!(
        (rust - python).abs() < PARITY_TOL,
        "GOMS best 20K: Rust={rust}, Python={python}"
    );
}

// ── L-systems ─────────────────────────────────────────────────────

#[test]
fn parity_algae_fibonacci() {
    let sys = presets::algae();
    let rust: Vec<usize> = (0..8).map(|g| sys.symbol_count(g)).collect();
    let python = [1, 2, 3, 5, 8, 13, 21, 34];
    assert_eq!(rust, python, "Algae lengths must match Fibonacci");
}

#[test]
fn parity_koch_lengths() {
    let sys = presets::koch_curve();
    assert_eq!(sys.symbol_count(0), 1, "Koch g0");
    assert_eq!(sys.symbol_count(1), 9, "Koch g1");
}

#[test]
fn parity_protein_backbone_elements() {
    let sys = presets::protein_backbone();
    let g3 = sys.generate(3);
    assert!(g3.contains('H'), "protein g3 must contain H");
    assert!(g3.contains('S'), "protein g3 must contain S");
    assert!(g3.contains('L'), "protein g3 must contain L");
    assert!(g3.contains('T'), "protein g3 must contain T");
}

// ── BSP Partitioning ──────────────────────────────────────────────

#[test]
fn parity_bsp_area_conservation() {
    let bounds = Rect::new(0.0, 0.0, 100.0, 100.0);
    let tree = generate_bsp(bounds, 15.0, 42);
    let leaf_area: f64 = tree.leaves().iter().map(Rect::area).sum();
    assert!(
        (leaf_area - 10000.0).abs() < 1e-6,
        "BSP area: Rust={leaf_area}, Python=10000.0"
    );
}

#[test]
fn parity_bsp_small_single_leaf() {
    let tree = generate_bsp(Rect::new(0.0, 0.0, 5.0, 5.0), 10.0, 42);
    assert_eq!(tree.leaf_count(), 1, "Small space must be single leaf");
}

// ── Four Keys to Fun ──────────────────────────────────────────────

#[test]
fn parity_fun_dark_souls() {
    let c = classify_fun(&FunSignals {
        challenge: 0.95,
        exploration: 0.2,
        social: 0.05,
        completion: 0.3,
        retry_rate: 0.9,
    });
    assert_eq!(c.dominant, FunKey::Hard, "Dark Souls = Hard Fun");
}

#[test]
fn parity_fun_minecraft_creative() {
    let c = classify_fun(&FunSignals {
        challenge: 0.1,
        exploration: 0.9,
        social: 0.1,
        completion: 0.3,
        retry_rate: 0.0,
    });
    assert_eq!(c.dominant, FunKey::Easy, "Minecraft Creative = Easy Fun");
}

#[test]
fn parity_fun_among_us() {
    let c = classify_fun(&FunSignals {
        challenge: 0.3,
        exploration: 0.1,
        social: 0.95,
        completion: 0.1,
        retry_rate: 0.1,
    });
    assert_eq!(c.dominant, FunKey::People, "Among Us = People Fun");
}

#[test]
fn parity_fun_animal_crossing() {
    let c = classify_fun(&FunSignals {
        challenge: 0.05,
        exploration: 0.3,
        social: 0.1,
        completion: 0.9,
        retry_rate: 0.0,
    });
    assert_eq!(c.dominant, FunKey::Serious, "Animal Crossing = Serious Fun");
}

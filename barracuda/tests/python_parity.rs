// SPDX-License-Identifier: AGPL-3.0-or-later
//! Rust-vs-Python parity tests.
//!
//! Compares Rust implementations against the exact values produced by
//! the Python reference baselines. These are the canonical cross-language
//! validation tests that prove the barraCuda CPU port is faithful.
//!
//! # Provenance
//!
//! - **Baselines**: `baselines/python/` (stdlib only, no numpy/scipy)
//! - **Generated**: 2026-04-10
//! - **Python**: CPython 3.10.12 (math module only)
//! - **Command**: `python3 baselines/python/run_all_baselines.py`
//! - **Output**: `baselines/python/combined_baselines.json`
//! - **Commit**: `19e402c0b5b023db6e8df53dc4494b572a3ecd4b`
//! - **Note**: Baseline JSON was generated under Python 3.10.12.
//!   `run_all_baselines.py` requires >= 3.10 (`REQUIRED_PYTHON_MIN`); outputs use
//!   stdlib `math` only with no version-dependent behavior in the parity range.
//!
//! Every expected value below is transcribed from the Python JSON output.
//! The comment after each value cites the exact JSON key path.
//! Tolerance uses `tolerances::ANALYTICAL_TOL` (1e-10) — the only error
//! source is IEEE 754 reassociation between Python and Rust f64.

use ludospring_barracuda::interaction::flow::{FlowState, evaluate_flow};
use ludospring_barracuda::interaction::goms::{
    self, Operator, task_time, task_time_with_keystroke,
};
use ludospring_barracuda::interaction::input_laws::{
    fitts_index_of_difficulty, fitts_movement_time, hick_reaction_time, steering_time,
};
use ludospring_barracuda::metrics::engagement::{EngagementSnapshot, compute_engagement};
use ludospring_barracuda::metrics::fun_keys::{FunKey, FunSignals, classify_fun};
use ludospring_barracuda::procedural::bsp::{Rect, generate_bsp};
use ludospring_barracuda::procedural::lsystem::presets;
use ludospring_barracuda::procedural::noise::{fbm_2d, perlin_2d, perlin_3d};
use ludospring_barracuda::tolerances;

// ── Interaction Laws ───────────────────────────────────────────────
// JSON: interaction_laws.py

#[test]
fn parity_fitts_mt_d100_w10() {
    // interaction_laws.py.fitts_mt_D100_W10
    let rust = fitts_movement_time(100.0, 10.0, 50.0, 150.0);
    let python = 708.847_613_416_814;
    assert!(
        (rust - python).abs() < tolerances::ANALYTICAL_TOL,
        "Fitts MT: Rust={rust}, Python={python}"
    );
}

#[test]
fn parity_fitts_id_d100_w10() {
    // interaction_laws.py.fitts_id_D100_W10
    let rust = fitts_index_of_difficulty(100.0, 10.0);
    let python = 4.392_317_422_778_761;
    assert!(
        (rust - python).abs() < tolerances::ANALYTICAL_TOL,
        "Fitts ID: Rust={rust}, Python={python}"
    );
}

#[test]
fn parity_hick_rt_n7() {
    // interaction_laws.py.hick_rt_N7
    let rust = hick_reaction_time(7, tolerances::HICK_A_MS, tolerances::HICK_B_MS);
    let python = 650.0;
    assert!(
        (rust - python).abs() < tolerances::ANALYTICAL_TOL,
        "Hick RT: Rust={rust}, Python={python}"
    );
}

#[test]
fn parity_steering_d100_w20() {
    // interaction_laws.py.steering_D100_W20
    let rust = steering_time(100.0, 20.0, 10.0, 5.0);
    let python = 35.0;
    assert!(
        (rust - python).abs() < tolerances::ANALYTICAL_TOL,
        "Steering: Rust={rust}, Python={python}"
    );
}

// ── Perlin Noise ───────────────────────────────────────────────────
// JSON: perlin_noise.py

#[test]
fn parity_perlin_2d_lattice_zeros() {
    // perlin_noise.py.perlin_2d_lattice — all integer coords are 0.0
    for ix in 0..10_i32 {
        for iy in 0..10_i32 {
            let v = perlin_2d(f64::from(ix), f64::from(iy));
            assert!(
                v.abs() < tolerances::ANALYTICAL_TOL,
                "perlin_2d({ix},{iy}) = {v}, Python = 0.0"
            );
        }
    }
}

#[test]
fn parity_perlin_3d_lattice_zeros() {
    // perlin_noise.py.perlin_3d_lattice — all integer coords are 0.0
    for ix in 0..5_i32 {
        for iy in 0..5_i32 {
            for iz in 0..5_i32 {
                let v = perlin_3d(f64::from(ix), f64::from(iy), f64::from(iz));
                assert!(
                    v.abs() < tolerances::ANALYTICAL_TOL,
                    "perlin_3d({ix},{iy},{iz}) = {v}, Python = 0.0"
                );
            }
        }
    }
}

#[test]
fn parity_perlin_2d_specific_coords() {
    // perlin_noise.py.perlin_2d_samples — exact Python reference values
    let cases: &[(f64, f64, f64)] = &[
        // (x, y, expected)  — key: "x,y"
        (0.5, 0.7, 0.0),                           // "0.5,0.7"
        (1.23, 4.56, 0.630_427_670_085_576_7),     // "1.23,4.56"
        (100.1, 200.2, -0.128_796_431_359_991_14), // "100.1,200.2"
        (-3.17, 2.73, 0.059_750_319_722_442_49),   // "-3.17,2.73"
    ];

    for &(x, y, expected) in cases {
        let rust = perlin_2d(x, y);
        assert!(
            (rust - expected).abs() < tolerances::ANALYTICAL_TOL,
            "perlin_2d({x},{y}): Rust={rust}, Python={expected}"
        );
    }
}

#[test]
fn parity_fbm_2d_exact_values() {
    // perlin_noise.py.fbm_2d_samples — exact Python reference values at (3.17, 2.73)
    let cases: &[(u32, f64)] = &[
        (1, -0.002_422_928_849_557_970_4), // "octaves=1"
        (4, -0.050_648_294_213_875_43),    // "octaves=4"
        (8, -0.069_506_437_975_332_79),    // "octaves=8"
    ];

    for &(octaves, expected) in cases {
        let rust = fbm_2d(3.17, 2.73, octaves, 2.0, 0.5);
        assert!(
            (rust - expected).abs() < tolerances::ANALYTICAL_TOL,
            "fbm_2d(3.17,2.73,octaves={octaves}): Rust={rust}, Python={expected}"
        );
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

// ── Four Keys to Fun ──────────────────────────────────────────────
// JSON: fun_keys_model.py

#[test]
fn parity_fun_dark_souls() {
    // fun_keys_model.py.dark_souls_boss.dominant = "hard"
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
    // fun_keys_model.py.minecraft_creative.dominant = "easy"
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
    // fun_keys_model.py.among_us.dominant = "people"
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
    // fun_keys_model.py.animal_crossing.dominant = "serious"
    let c = classify_fun(&FunSignals {
        challenge: 0.05,
        exploration: 0.3,
        social: 0.1,
        completion: 0.9,
        retry_rate: 0.0,
    });
    assert_eq!(c.dominant, FunKey::Serious, "Animal Crossing = Serious Fun");
}

// ── Four Keys: Numeric Scores ────────────────────────────────────
// JSON: fun_keys_model.py — extended: exact score parity per scenario

#[test]
fn parity_fun_dark_souls_scores() {
    // fun_keys_model.py.dark_souls_boss.scores
    let c = classify_fun(&FunSignals {
        challenge: 0.95,
        exploration: 0.2,
        social: 0.05,
        completion: 0.3,
        retry_rate: 0.9,
    });
    assert!((c.scores.hard - 0.93).abs() < tolerances::ANALYTICAL_TOL);
    assert!((c.scores.easy - 0.17).abs() < tolerances::ANALYTICAL_TOL);
    assert!((c.scores.people - 0.05).abs() < tolerances::ANALYTICAL_TOL);
    assert!((c.scores.serious - 0.36).abs() < tolerances::ANALYTICAL_TOL);
}

#[test]
fn parity_fun_minecraft_scores() {
    // fun_keys_model.py.minecraft_creative.scores
    let c = classify_fun(&FunSignals {
        challenge: 0.1,
        exploration: 0.9,
        social: 0.1,
        completion: 0.3,
        retry_rate: 0.0,
    });
    assert!((c.scores.hard - 0.06).abs() < tolerances::ANALYTICAL_TOL);
    assert!((c.scores.easy - 0.9).abs() < tolerances::ANALYTICAL_TOL);
    assert!((c.scores.people - 0.1).abs() < tolerances::ANALYTICAL_TOL);
    assert!((c.scores.serious - 0.48).abs() < tolerances::ANALYTICAL_TOL);
}

#[test]
fn parity_fun_among_us_scores() {
    // fun_keys_model.py.among_us.scores
    let c = classify_fun(&FunSignals {
        challenge: 0.3,
        exploration: 0.1,
        social: 0.95,
        completion: 0.1,
        retry_rate: 0.1,
    });
    assert!((c.scores.hard - 0.22).abs() < tolerances::ANALYTICAL_TOL);
    assert!((c.scores.easy - 0.22).abs() < tolerances::ANALYTICAL_TOL);
    assert!((c.scores.people - 0.95).abs() < tolerances::ANALYTICAL_TOL);
    assert!((c.scores.serious - 0.1825).abs() < tolerances::ANALYTICAL_TOL);
}

#[test]
fn parity_fun_animal_crossing_scores() {
    // fun_keys_model.py.animal_crossing.scores
    let c = classify_fun(&FunSignals {
        challenge: 0.05,
        exploration: 0.3,
        social: 0.1,
        completion: 0.9,
        retry_rate: 0.0,
    });
    assert!((c.scores.hard - 0.03).abs() < tolerances::ANALYTICAL_TOL);
    assert!((c.scores.easy - 0.43).abs() < tolerances::ANALYTICAL_TOL);
    assert!((c.scores.people - 0.1).abs() < tolerances::ANALYTICAL_TOL);
    assert!((c.scores.serious - 0.9075).abs() < tolerances::ANALYTICAL_TOL);
}

#[test]
fn parity_fun_celeste() {
    // fun_keys_model.py.celeste.dominant = "hard", scores
    let c = classify_fun(&FunSignals {
        challenge: 0.9,
        exploration: 0.3,
        social: 0.0,
        completion: 0.4,
        retry_rate: 0.85,
    });
    assert_eq!(c.dominant, FunKey::Hard, "Celeste = Hard Fun");
    assert!((c.scores.hard - 0.88).abs() < tolerances::ANALYTICAL_TOL);
    assert!((c.scores.easy - 0.26).abs() < tolerances::ANALYTICAL_TOL);
    assert!((c.scores.people - 0.0).abs() < tolerances::ANALYTICAL_TOL);
    assert!((c.scores.serious - 0.445).abs() < tolerances::ANALYTICAL_TOL);
}

#[test]
fn parity_fun_no_mans_sky() {
    // fun_keys_model.py.no_mans_sky.dominant = "easy", scores
    let c = classify_fun(&FunSignals {
        challenge: 0.15,
        exploration: 0.85,
        social: 0.15,
        completion: 0.2,
        retry_rate: 0.05,
    });
    assert_eq!(c.dominant, FunKey::Easy, "No Man's Sky = Easy Fun");
    assert!((c.scores.hard - 0.11).abs() < tolerances::ANALYTICAL_TOL);
    assert!((c.scores.easy - 0.85).abs() < tolerances::ANALYTICAL_TOL);
    assert!((c.scores.people - 0.15).abs() < tolerances::ANALYTICAL_TOL);
    assert!((c.scores.serious - 0.395).abs() < tolerances::ANALYTICAL_TOL);
}

// ── Interaction Laws: Doom Scenarios ─────────────────────────────
// JSON: interaction_laws.py.fitts_doom_scenarios

#[test]
fn parity_fitts_doom_close_barrel() {
    // fitts_doom_scenarios.close_barrel.mt_ms
    let rust = fitts_movement_time(
        50.0,
        30.0,
        tolerances::FITTS_A_MOUSE_MS,
        tolerances::FITTS_B_MOUSE_MS,
    );
    let python = 367.321_582_612_990_4;
    assert!(
        (rust - python).abs() < tolerances::ANALYTICAL_TOL,
        "close_barrel: Rust={rust}, Python={python}"
    );
}

#[test]
fn parity_fitts_doom_medium_imp() {
    // fitts_doom_scenarios.medium_imp.mt_ms
    let rust = fitts_movement_time(
        150.0,
        20.0,
        tolerances::FITTS_A_MOUSE_MS,
        tolerances::FITTS_B_MOUSE_MS,
    );
    let python = 650.0;
    assert!(
        (rust - python).abs() < tolerances::ANALYTICAL_TOL,
        "medium_imp: Rust={rust}, Python={python}"
    );
}

#[test]
fn parity_fitts_doom_far_cacodemon() {
    // fitts_doom_scenarios.far_cacodemon.mt_ms
    let rust = fitts_movement_time(
        300.0,
        15.0,
        tolerances::FITTS_A_MOUSE_MS,
        tolerances::FITTS_B_MOUSE_MS,
    );
    let python = 853.632_800_692_712_5;
    assert!(
        (rust - python).abs() < tolerances::ANALYTICAL_TOL,
        "far_cacodemon: Rust={rust}, Python={python}"
    );
}

#[test]
fn parity_fitts_doom_sniper_far_tiny() {
    // fitts_doom_scenarios.sniper_far_tiny.mt_ms
    let rust = fitts_movement_time(
        400.0,
        5.0,
        tolerances::FITTS_A_MOUSE_MS,
        tolerances::FITTS_B_MOUSE_MS,
    );
    let python = 1_149.637_531_717_192_6;
    assert!(
        (rust - python).abs() < tolerances::ANALYTICAL_TOL,
        "sniper_far_tiny: Rust={rust}, Python={python}"
    );
}

// ── Hick's Law: Choice Sweep ─────────────────────────────────────
// JSON: interaction_laws.py.hick_choice_sweep

#[test]
fn parity_hick_choice_sweep() {
    let cases: &[(usize, f64)] = &[
        (2, 437.744_375_108_173_4),
        (4, 548.289_214_233_104_3),
        (7, 650.0),
        (10, 718.914_742_795_594_6),
        (16, 813.119_426_187_550_9),
    ];
    for &(n, python) in cases {
        let rust = hick_reaction_time(n, tolerances::HICK_A_MS, tolerances::HICK_B_MS);
        assert!(
            (rust - python).abs() < tolerances::ANALYTICAL_TOL,
            "Hick N={n}: Rust={rust}, Python={python}"
        );
    }
}

// ── Flow / Engagement / DDA ──────────────────────────────────────
// JSON: flow_engagement.py

#[test]
fn parity_flow_states() {
    // flow_engagement.py.flow_states — matching Python's evaluate_flow exactly
    let w = tolerances::FLOW_CHANNEL_WIDTH;
    assert_eq!(
        evaluate_flow(0.5, 0.5, w),
        FlowState::Flow,
        "exact_diagonal"
    );
    assert_eq!(
        evaluate_flow(0.5, w.mul_add(-0.9, 0.5), w),
        FlowState::Flow,
        "inside_channel_low"
    );
    assert_eq!(
        evaluate_flow(0.5, w.mul_add(0.9, 0.5), w),
        FlowState::Flow,
        "inside_channel_high"
    );
    assert_eq!(
        evaluate_flow(0.9, 0.1, w),
        FlowState::Anxiety,
        "high_challenge_low_skill"
    );
    assert_eq!(
        evaluate_flow(0.1, 0.9, w),
        FlowState::Boredom,
        "low_challenge_high_skill"
    );
}

#[test]
fn parity_engagement_active() {
    // flow_engagement.py.engagement_active — 300s session, 200 actions, 15 explore, 10 challenge, 20 retry, 15 pauses
    let snap = EngagementSnapshot {
        session_duration_s: 300.0,
        action_count: 200,
        exploration_breadth: 15,
        challenge_seeking: 10,
        retry_count: 20,
        deliberate_pauses: 15,
    };
    let m = compute_engagement(&snap);
    let python_composite = 0.298_333_333_333_333_34;
    assert!(
        (m.composite - python_composite).abs() < tolerances::ANALYTICAL_TOL,
        "active composite: Rust={}, Python={python_composite}",
        m.composite
    );
    assert!((m.actions_per_minute - 40.0).abs() < tolerances::ANALYTICAL_TOL);
    assert!((m.exploration_rate - 3.0).abs() < tolerances::ANALYTICAL_TOL);
    assert!((m.challenge_appetite - 0.05).abs() < tolerances::ANALYTICAL_TOL);
    assert!((m.persistence - 0.1).abs() < tolerances::ANALYTICAL_TOL);
    assert!((m.deliberation - 0.075).abs() < tolerances::ANALYTICAL_TOL);
}

#[test]
fn parity_engagement_idle() {
    // flow_engagement.py.engagement_idle — 300s session, 2 actions
    let snap = EngagementSnapshot {
        session_duration_s: 300.0,
        action_count: 2,
        exploration_breadth: 1,
        challenge_seeking: 0,
        retry_count: 0,
        deliberate_pauses: 0,
    };
    let m = compute_engagement(&snap);
    let python_composite = 0.009_333_333_333_333_334;
    assert!(
        (m.composite - python_composite).abs() < tolerances::ANALYTICAL_TOL,
        "idle composite: Rust={}, Python={python_composite}",
        m.composite
    );
}

#[test]
fn parity_engagement_zero() {
    // flow_engagement.py.engagement_zero — 0s session, 0 actions → 0.0
    let snap = EngagementSnapshot::default();
    let m = compute_engagement(&snap);
    assert!(
        m.composite.abs() < tolerances::ANALYTICAL_TOL,
        "zero composite: Rust={}",
        m.composite
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

// ── Fun Keys: Edge Cases ─────────────────────────────────────────
// JSON: fun_keys_model.py — zero_scores, max_scores

#[test]
fn parity_fun_keys_zero_scores() {
    // fun_keys_model.py.zero_scores: all signals zero
    // Expected: hard=0.0, easy=0.2, people=0, serious=0.3
    let c = classify_fun(&FunSignals {
        challenge: 0.0,
        exploration: 0.0,
        social: 0.0,
        completion: 0.0,
        retry_rate: 0.0,
    });
    assert!(
        c.scores.hard.abs() < tolerances::ANALYTICAL_TOL,
        "zero hard: Rust={}, Python=0.0",
        c.scores.hard
    );
    assert!(
        (c.scores.easy - 0.2).abs() < tolerances::ANALYTICAL_TOL,
        "zero easy: Rust={}, Python=0.2",
        c.scores.easy
    );
    assert!(
        c.scores.people.abs() < tolerances::ANALYTICAL_TOL,
        "zero people: Rust={}, Python=0.0",
        c.scores.people
    );
    assert!(
        (c.scores.serious - 0.3).abs() < tolerances::ANALYTICAL_TOL,
        "zero serious: Rust={}, Python=0.3",
        c.scores.serious
    );
}

#[test]
fn parity_fun_keys_max_scores() {
    // fun_keys_model.py.max_scores: all signals at 1.0
    // Expected: hard=1.0, easy=0.8, people=1, serious=0.7
    let c = classify_fun(&FunSignals {
        challenge: 1.0,
        exploration: 1.0,
        social: 1.0,
        completion: 1.0,
        retry_rate: 1.0,
    });
    assert!(
        (c.scores.hard - 1.0).abs() < tolerances::ANALYTICAL_TOL,
        "max hard: Rust={}, Python=1.0",
        c.scores.hard
    );
    assert!(
        (c.scores.easy - 0.8).abs() < tolerances::ANALYTICAL_TOL,
        "max easy: Rust={}, Python=0.8",
        c.scores.easy
    );
    assert!(
        (c.scores.people - 1.0).abs() < tolerances::ANALYTICAL_TOL,
        "max people: Rust={}, Python=1.0",
        c.scores.people
    );
    assert!(
        (c.scores.serious - 0.7).abs() < tolerances::ANALYTICAL_TOL,
        "max serious: Rust={}, Python=0.7",
        c.scores.serious
    );
}

// ── Perlin fBm 3D Lattice Check ──────────────────────────────────
// JSON: perlin_noise.py — fbm_3d_sample

#[test]
fn parity_fbm_3d_lattice_zero() {
    // perlin_noise.py.fbm_3d_sample = 0.0 (integer lattice point)
    use ludospring_barracuda::procedural::noise::fbm_3d;
    let rust = fbm_3d(0.0, 0.0, 0.0, 4, 2.0, 0.5);
    let python = 0.0;
    assert!(
        (rust - python).abs() < tolerances::ANALYTICAL_TOL,
        "fBm 3D lattice origin: Rust={rust}, Python={python}"
    );
}

// ── L-System Turtle Geometry ─────────────────────────────────────
// JSON: lsystem_growth.py — turtle endpoints and distances

#[test]
fn parity_lsystem_turtle_ff_end() {
    // lsystem_growth.py.turtle_FF_end = [2.0, 0.0]
    use ludospring_barracuda::procedural::lsystem::turtle_interpret;
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
    use ludospring_barracuda::procedural::lsystem::turtle_interpret;
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

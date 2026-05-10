// SPDX-License-Identifier: AGPL-3.0-or-later

use ludospring_barracuda::interaction::input_laws::{
    fitts_index_of_difficulty, fitts_movement_time, hick_reaction_time, steering_time,
};
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

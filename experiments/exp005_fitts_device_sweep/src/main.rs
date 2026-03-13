// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
//! Exp005: Fitts's law device sweep — validation binary.
//!
//! Validates Fitts's law predictions across multiple input devices using
//! empirically derived (a,b) constants from published studies.
//!
//! # Provenance
//!
//! Device constants from:
//! - Mouse: `MacKenzie` (1992), Table 2. a=50ms, b=150ms.
//! - Gamepad thumbstick: Looser et al. (2005). a=100ms, b=200ms.
//! - Touchscreen: Bi et al. (2013), "`FFitts` Law." a=70ms, b=120ms.
//! - Head-gaze: Qian & Teather (2017). a=200ms, b=280ms.
//!
//! Python baseline: `baselines/python/interaction_laws.py` (2026-03-11).

use ludospring_barracuda::interaction::input_laws::{
    fitts_index_of_difficulty, fitts_movement_time,
};
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

struct DeviceProfile {
    name: &'static str,
    a: f64,
    b: f64,
}

const DEVICES: &[DeviceProfile] = &[
    DeviceProfile {
        name: "mouse",
        a: 50.0,
        b: 150.0,
    },
    DeviceProfile {
        name: "gamepad",
        a: 100.0,
        b: 200.0,
    },
    DeviceProfile {
        name: "touchscreen",
        a: 70.0,
        b: 120.0,
    },
    DeviceProfile {
        name: "head_gaze",
        a: 200.0,
        b: 280.0,
    },
];

fn validate_monotonicity(results: &mut Vec<ValidationResult>) {
    println!("Part 1: Fitts monotonicity across devices");
    for device in DEVICES {
        let close = fitts_movement_time(50.0, 30.0, device.a, device.b);
        let mid = fitts_movement_time(150.0, 20.0, device.a, device.b);
        let far = fitts_movement_time(300.0, 10.0, device.a, device.b);

        let r = ValidationResult::check(
            &format!("exp005_{}_mono", device.name),
            &format!("{}: close < mid < far", device.name),
            if close < mid && mid < far { 1.0 } else { 0.0 },
            1.0,
            tolerances::ANALYTICAL_TOL,
        );
        report(&r);
        results.push(r);
    }
}

fn validate_device_ordering(results: &mut Vec<ValidationResult>) {
    println!("\nPart 2: Device speed ordering (same target)");
    let d = 100.0;
    let w = 20.0;

    let mouse = fitts_movement_time(d, w, DEVICES[0].a, DEVICES[0].b);
    let touch = fitts_movement_time(d, w, DEVICES[2].a, DEVICES[2].b);
    let gaze = fitts_movement_time(d, w, DEVICES[3].a, DEVICES[3].b);

    let r = ValidationResult::check(
        "exp005_mouse_fast",
        "mouse faster than head-gaze for same target",
        if mouse < gaze { 1.0 } else { 0.0 },
        1.0,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);

    let r = ValidationResult::check(
        "exp005_touch_fast",
        "touchscreen faster than head-gaze",
        if touch < gaze { 1.0 } else { 0.0 },
        1.0,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);

    let r = ValidationResult::check(
        "exp005_touch_vs_mouse",
        "touchscreen faster than mouse (lower b coefficient)",
        if touch < mouse { 1.0 } else { 0.0 },
        1.0,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);
}

fn validate_analytical_values(results: &mut Vec<ValidationResult>) {
    println!("\nPart 3: Analytical values (Python baseline match)");
    let mt = fitts_movement_time(
        100.0,
        10.0,
        tolerances::FITTS_A_MOUSE_MS,
        tolerances::FITTS_B_MOUSE_MS,
    );
    let r = ValidationResult::check(
        "exp005_fitts_exact",
        "Fitts MT D=100 W=10 matches Python baseline",
        mt,
        708.847_613_416_814,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);

    let id = fitts_index_of_difficulty(100.0, 10.0);
    let r = ValidationResult::check(
        "exp005_id_exact",
        "Fitts ID D=100 W=10 = log2(21)",
        id,
        4.392_317_422_778_761,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);
}

fn main() {
    println!("=== Exp005: Fitts's Law Device Sweep (Validation) ===\n");
    let mut results = Vec::new();

    validate_monotonicity(&mut results);
    validate_device_ordering(&mut results);
    validate_analytical_values(&mut results);

    let passed = results.iter().filter(|r| r.passed).count();
    let failed = results.len() - passed;
    println!("\n{passed} passed, {failed} failed");
    if failed > 0 {
        std::process::exit(1);
    }
}

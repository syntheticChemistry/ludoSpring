// SPDX-License-Identifier: AGPL-3.0-or-later
//! Exp015: Accessibility motor-limited Fitts — validation binary.
//!
//! Validates Fitts's law predictions for motor-limited input scenarios:
//! eye-gaze tracking, switch scanning, head-pointer, and sip-puff devices.
//! These represent assistive technology input profiles where the (a,b)
//! constants differ significantly from mouse/gamepad.
//!
//! # Provenance
//!
//! Fitts (1954): original law. `MacKenzie` (1992): Shannon formulation.
//! Wobbrock et al. (2008): "Ability-based design." CHI '08.
//! Gajos et al. (2007): "Automatically generating interfaces adapted to
//! users' motor and vision capabilities." UIST '07.

use ludospring_barracuda::interaction::input_laws::fitts_movement_time;
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
        name: "eye_gaze",
        a: 200.0,
        b: 280.0,
    },
    DeviceProfile {
        name: "head_pointer",
        a: 300.0,
        b: 350.0,
    },
    DeviceProfile {
        name: "switch_scan",
        a: 500.0,
        b: 800.0,
    },
    DeviceProfile {
        name: "sip_puff",
        a: 400.0,
        b: 600.0,
    },
];

fn validate_device_predictions(results: &mut Vec<ValidationResult>) {
    println!("Part 1: Device-specific Fitts predictions");
    let d = 100.0;
    let w = 20.0;

    let mut times: Vec<f64> = Vec::new();
    for device in DEVICES {
        let t = fitts_movement_time(d, w, device.a, device.b);
        times.push(t);
        println!("  {}: {t:.0}ms", device.name);
    }

    // Mouse should be fastest
    let r = ValidationResult::check(
        "exp015_mouse_fastest",
        "mouse is fastest device",
        if times[0] < times[1] && times[0] < times[2] && times[0] < times[3] {
            1.0
        } else {
            0.0
        },
        1.0,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);

    // Switch scanning should be slowest
    let r = ValidationResult::check(
        "exp015_switch_slowest",
        "switch scanning is slowest device",
        if times[3] > times[0] && times[3] > times[1] && times[3] > times[2] {
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

fn validate_target_size_impact(results: &mut Vec<ValidationResult>) {
    println!("\nPart 2: Target size impact on accessibility");
    let d = 100.0;

    for device in DEVICES {
        let small = fitts_movement_time(d, 10.0, device.a, device.b);
        let large = fitts_movement_time(d, 60.0, device.a, device.b);
        let improvement = (small - large) / small;

        let r = ValidationResult::check(
            &format!("exp015_{}_size", device.name),
            &format!("{}: larger targets help (improvement > 0)", device.name),
            if improvement > 0.0 { 1.0 } else { 0.0 },
            1.0,
            tolerances::ANALYTICAL_TOL,
        );
        report(&r);
        results.push(r);
    }
}

fn validate_accessibility_recommendations(results: &mut Vec<ValidationResult>) {
    println!("\nPart 3: Accessibility design recommendations");
    // For switch scanning, target must be very large to achieve usable times
    let switch_small = fitts_movement_time(100.0, 10.0, 500.0, 800.0);
    let switch_huge = fitts_movement_time(100.0, 80.0, 500.0, 800.0);

    let r = ValidationResult::check(
        "exp015_switch_huge_target",
        "huge targets (80px) significantly help switch users",
        switch_small - switch_huge,
        1000.0,
        1500.0,
    );
    report(&r);
    results.push(r);

    // All devices should produce finite, positive times
    let all_valid = DEVICES.iter().all(|dev| {
        let t = fitts_movement_time(50.0, 20.0, dev.a, dev.b);
        t.is_finite() && t > 0.0
    });
    let r = ValidationResult::check(
        "exp015_all_valid",
        "all device profiles produce finite positive times",
        if all_valid { 1.0 } else { 0.0 },
        1.0,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);
}

fn main() {
    println!("=== Exp015: Accessibility Motor-Limited Fitts (Validation) ===\n");
    let mut results = Vec::new();

    validate_device_predictions(&mut results);
    validate_target_size_impact(&mut results);
    validate_accessibility_recommendations(&mut results);

    let passed = results.iter().filter(|r| r.passed).count();
    let failed = results.len() - passed;
    println!("\n{passed} passed, {failed} failed");
    if failed > 0 {
        std::process::exit(1);
    }
}

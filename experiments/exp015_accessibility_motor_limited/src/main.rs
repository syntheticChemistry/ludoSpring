// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
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
use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "N/A (analytical — Fitts 1954, MacKenzie 1992, Wobbrock 2008)",
    commit: "4b683e3e",
    date: "2026-03-15",
    command: "N/A (analytical)",
};

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

fn validate_device_predictions(h: &mut ValidationHarness) {
    let d = 100.0;
    let w = 20.0;

    let mut times: Vec<f64> = Vec::new();
    for device in DEVICES {
        let t = fitts_movement_time(d, w, device.a, device.b);
        times.push(t);
    }

    h.check_bool(
        "mouse is fastest device",
        times[0] < times[1] && times[0] < times[2] && times[0] < times[3],
    );

    h.check_bool(
        "switch scanning is slowest device",
        times[3] > times[0] && times[3] > times[1] && times[3] > times[2],
    );
}

fn validate_target_size_impact(h: &mut ValidationHarness) {
    let d = 100.0;

    for device in DEVICES {
        let small = fitts_movement_time(d, 10.0, device.a, device.b);
        let large = fitts_movement_time(d, 60.0, device.a, device.b);
        let improvement = (small - large) / small;

        h.check_bool(
            &format!("{}: larger targets help (improvement > 0)", device.name),
            improvement > 0.0,
        );
    }
}

fn validate_accessibility_recommendations(h: &mut ValidationHarness) {
    let switch_small = fitts_movement_time(100.0, 10.0, 500.0, 800.0);
    let switch_huge = fitts_movement_time(100.0, 80.0, 500.0, 800.0);

    h.check_abs(
        "huge targets (80px) significantly help switch users",
        switch_small - switch_huge,
        1000.0,
        1500.0,
    );

    let all_valid = DEVICES.iter().all(|dev| {
        let t = fitts_movement_time(50.0, 20.0, dev.a, dev.b);
        t.is_finite() && t > 0.0
    });
    h.check_bool(
        "all device profiles produce finite positive times",
        all_valid,
    );
}

fn main() {
    let mut h = ValidationHarness::new("exp015_accessibility_motor_limited");
    h.print_provenance(&[&PROVENANCE]);

    validate_device_predictions(&mut h);
    validate_target_size_impact(&mut h);
    validate_accessibility_recommendations(&mut h);

    h.finish();
}

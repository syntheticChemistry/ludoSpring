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
use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "baselines/python/interaction_laws.py",
    commit: "74cf9488",
    date: "2026-03-11",
    command: "python3 baselines/python/run_all_baselines.py",
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

fn validate_monotonicity(h: &mut ValidationHarness) {
    for device in DEVICES {
        let close = fitts_movement_time(50.0, 30.0, device.a, device.b);
        let mid = fitts_movement_time(150.0, 20.0, device.a, device.b);
        let far = fitts_movement_time(300.0, 10.0, device.a, device.b);

        h.check_bool(
            &format!("{}: close < mid < far", device.name),
            close < mid && mid < far,
        );
    }
}

fn validate_device_ordering(h: &mut ValidationHarness) {
    let d = 100.0;
    let w = 20.0;

    let mouse = fitts_movement_time(d, w, DEVICES[0].a, DEVICES[0].b);
    let touch = fitts_movement_time(d, w, DEVICES[2].a, DEVICES[2].b);
    let gaze = fitts_movement_time(d, w, DEVICES[3].a, DEVICES[3].b);

    h.check_bool("mouse faster than head-gaze for same target", mouse < gaze);
    h.check_bool("touchscreen faster than head-gaze", touch < gaze);
    h.check_bool(
        "touchscreen faster than mouse (lower b coefficient)",
        touch < mouse,
    );
}

fn validate_analytical_values(h: &mut ValidationHarness) {
    let mt = fitts_movement_time(
        100.0,
        10.0,
        tolerances::FITTS_A_MOUSE_MS,
        tolerances::FITTS_B_MOUSE_MS,
    );
    h.check_abs(
        "Fitts MT D=100 W=10 matches Python baseline",
        mt,
        708.847_613_416_814,
        tolerances::ANALYTICAL_TOL,
    );

    let id = fitts_index_of_difficulty(100.0, 10.0);
    h.check_abs(
        "Fitts ID D=100 W=10 = log2(21)",
        id,
        4.392_317_422_778_761,
        tolerances::ANALYTICAL_TOL,
    );
}

fn main() {
    let mut h = ValidationHarness::new("exp005_fitts_device_sweep");
    h.print_provenance(&[&PROVENANCE]);

    validate_monotonicity(&mut h);
    validate_device_ordering(&mut h);
    validate_analytical_values(&mut h);

    h.finish();
}

// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
//! Exp007: Steering law tunnel sweep — validation binary.
//!
//! Validates the steering law (Accot & Zhai 1997) across varying tunnel
//! widths and lengths, modeling corridor navigation in FPS games and
//! menu ribbon traversal in RTS interfaces.
//!
//! # Provenance
//!
//! Accot, J. & Zhai, S. (1997). "Beyond Fitts' law: models for trajectory-
//! based HCI tasks." CHI '97. T = a + b * (D/W).
//! Python baseline: `baselines/python/interaction_laws.py` (2026-03-11).

use ludospring_barracuda::interaction::input_laws::steering_time;
use ludospring_barracuda::tolerances;
use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "baselines/python/interaction_laws.py",
    commit: "4b683e3e",
    date: "2026-03-11",
    command: "python3 baselines/python/run_all_baselines.py",
};

fn validate_known_values(h: &mut ValidationHarness) {
    let t = steering_time(
        100.0,
        20.0,
        tolerances::STEERING_A_MS,
        tolerances::STEERING_B_MS,
    );
    let expected = tolerances::STEERING_A_MS + tolerances::STEERING_B_MS * (100.0 / 20.0);
    h.check_abs(
        "steering T = a + b*(D/W) (Python baseline)",
        t,
        expected,
        tolerances::ANALYTICAL_TOL,
    );
}

fn validate_width_sweep(h: &mut ValidationHarness) {
    let a = tolerances::STEERING_A_MS;
    let b = tolerances::STEERING_B_MS;
    let d = 200.0;

    let widths = [5.0, 10.0, 20.0, 40.0, 80.0];
    let times: Vec<f64> = widths.iter().map(|&w| steering_time(d, w, a, b)).collect();

    h.check_bool(
        "wider tunnels are faster (monotonically decreasing)",
        times.windows(2).all(|pair| pair[0] > pair[1]),
    );

    let narrow = steering_time(d, 10.0, a, b);
    let wide = steering_time(d, 20.0, a, b);
    let ratio = (narrow - a) / (wide - a);
    h.check_abs(
        "halving width doubles D/W term (ratio ~2.0)",
        ratio,
        2.0,
        tolerances::ANALYTICAL_TOL,
    );
}

fn validate_game_scenarios(h: &mut ValidationHarness) {
    let a = tolerances::FITTS_A_MOUSE_MS;
    let b_steering = 8.0;

    let corridor = steering_time(500.0, 30.0, a, b_steering);
    let ribbon = steering_time(200.0, 8.0, a, b_steering);

    h.check_bool(
        "narrow menu ribbon harder than wide corridor",
        ribbon > corridor,
    );

    let inf = steering_time(100.0, 0.0, a, b_steering);
    h.check_bool("zero-width tunnel returns infinity", inf.is_infinite());
}

fn main() {
    let mut h = ValidationHarness::new("exp007_steering_tunnel");
    h.print_provenance(&[&PROVENANCE]);

    validate_known_values(&mut h);
    validate_width_sweep(&mut h);
    validate_game_scenarios(&mut h);

    h.finish();
}

// SPDX-License-Identifier: AGPL-3.0-or-later
//! Interaction-law validation binary.
//!
//! Compares Rust implementations of Fitts's law, Hick's law, the steering law,
//! KLM/GOMS task times, and flow-state evaluation against the Python reference
//! outputs in `baselines/python/combined_baselines.json` (from
//! `baselines/python/run_all_baselines.py`).
#![forbid(unsafe_code)]

use ludospring_barracuda::interaction::flow::{FlowState, evaluate_flow};
use ludospring_barracuda::interaction::goms::{Operator, task_time};
use ludospring_barracuda::interaction::input_laws::{
    fitts_movement_time, hick_reaction_time, steering_time,
};
use ludospring_barracuda::tolerances::{
    ANALYTICAL_TOL, FITTS_A_MOUSE_MS, FITTS_B_MOUSE_MS, FLOW_CHANNEL_WIDTH, HICK_A_MS, HICK_B_MS,
    STEERING_A_MS, STEERING_B_MS,
};
use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};

fn main() {
    let provenance = BaselineProvenance {
        script: "baselines/python/run_all_baselines.py",
        commit: "19e402c0",
        date: "2026-04-10",
        command: "python3 baselines/python/run_all_baselines.py",
    };

    let mut h = ValidationHarness::new("Interaction laws (Python parity)");
    h.print_provenance(&[&provenance]);

    let fitts_mt = fitts_movement_time(100.0, 10.0, FITTS_A_MOUSE_MS, FITTS_B_MOUSE_MS);
    h.check_abs(
        "Fitts MT D=100 W=10",
        fitts_mt,
        708.847_613_416_814,
        ANALYTICAL_TOL,
    );

    let hick_rt = hick_reaction_time(7, HICK_A_MS, HICK_B_MS);
    h.check_abs("Hick RT N=7", hick_rt, 650.0, ANALYTICAL_TOL);

    let steer = steering_time(100.0, 20.0, STEERING_A_MS, STEERING_B_MS);
    h.check_abs("Steering D=100 W=20", steer, 35.0, ANALYTICAL_TOL);

    h.check_bool(
        "Flow balanced (0.5, 0.5)",
        evaluate_flow(0.5, 0.5, FLOW_CHANNEL_WIDTH) == FlowState::Flow,
    );

    let goms_empty = task_time(&[]);
    h.check_abs("GOMS empty task", goms_empty, 0.0, ANALYTICAL_TOL);

    let menu_open = task_time(&[Operator::Mental, Operator::Point, Operator::Keystroke]);
    h.check_abs("GOMS menu open", menu_open, 2.65, ANALYTICAL_TOL);

    h.finish();
}

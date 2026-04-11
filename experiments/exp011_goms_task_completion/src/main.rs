// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
//! Exp011: GOMS task completion time prediction — validation binary.
//!
//! Validates the Keystroke-Level Model (KLM) from Card, Moran & Newell
//! (1983) across common game interaction sequences: menu navigation,
//! inventory management, and chat input.
//!
//! # Provenance
//!
//! Card, S.K., Moran, T.P., & Newell, A. (1983). "The Psychology of
//! Human-Computer Interaction." Table 2: operator times.
//! Python baseline: `baselines/python/goms_model.py` (2026-03-11).

use ludospring_barracuda::interaction::goms::{
    self, Operator, operator_counts, task_time, task_time_with_keystroke,
};
use ludospring_barracuda::tolerances;
use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "baselines/python/goms_model.py",
    commit: "19e402c0",
    date: "2026-03-11",
    command: "python3 baselines/python/run_all_baselines.py",
};

fn validate_known_values(h: &mut ValidationHarness) {
    h.check_abs(
        "empty task = 0 seconds",
        task_time(&[]),
        0.0,
        tolerances::ANALYTICAL_TOL,
    );

    let single_key = task_time(&[Operator::Keystroke]);
    h.check_abs(
        "single keystroke = 0.20s (average typist)",
        single_key,
        goms::times::KEYSTROKE_AVG,
        tolerances::ANALYTICAL_TOL,
    );
}

fn validate_menu_navigation(h: &mut ValidationHarness) {
    let open_inventory = [Operator::Mental, Operator::Point, Operator::Keystroke];
    let expected = goms::times::MENTAL + goms::times::POINT + goms::times::KEYSTROKE_AVG;
    let t = task_time(&open_inventory);

    h.check_abs(
        "M+P+K = 1.35 + 1.10 + 0.20 = 2.65s",
        t,
        expected,
        tolerances::ANALYTICAL_TOL,
    );

    let drag_drop = [
        Operator::Mental,
        Operator::Point,
        Operator::Keystroke,
        Operator::Point,
        Operator::Keystroke,
    ];
    let t2 = task_time(&drag_drop);
    h.check_abs(
        "drag-drop = M + 2P + 2K",
        t2,
        2.0f64.mul_add(
            goms::times::KEYSTROKE_AVG,
            2.0f64.mul_add(goms::times::POINT, goms::times::MENTAL),
        ),
        tolerances::ANALYTICAL_TOL,
    );
}

fn validate_chat_input(h: &mut ValidationHarness) {
    let chat = [
        Operator::Mental,
        Operator::Home,
        Operator::Keystroke,
        Operator::Keystroke,
        Operator::Keystroke,
        Operator::Keystroke,
        Operator::Keystroke,
        Operator::Keystroke,
    ];
    let t = task_time(&chat);
    let expected = 6.0f64.mul_add(
        goms::times::KEYSTROKE_AVG,
        goms::times::MENTAL + goms::times::HOME,
    );
    h.check_abs(
        "typing 'gg wp' + enter = M + H + 6K",
        t,
        expected,
        tolerances::ANALYTICAL_TOL,
    );

    let counts = operator_counts(&chat);
    h.check_bool(
        "correct operator counts for chat sequence",
        counts.mentals == 1 && counts.homes == 1 && counts.keystrokes == 6,
    );
}

fn validate_skill_levels(h: &mut ValidationHarness) {
    let typing_task: Vec<Operator> = (0..20).map(|_| Operator::Keystroke).collect();

    let best = task_time_with_keystroke(&typing_task, goms::times::KEYSTROKE_BEST);
    let avg = task_time_with_keystroke(&typing_task, goms::times::KEYSTROKE_AVG);
    let worst = task_time_with_keystroke(&typing_task, goms::times::KEYSTROKE_WORST);

    h.check_bool(
        "best < avg < worst typist for 20 keystrokes",
        best < avg && avg < worst,
    );

    h.check_abs(
        "best typist 20 keys = 1.6s",
        best,
        20.0 * goms::times::KEYSTROKE_BEST,
        tolerances::ANALYTICAL_TOL,
    );
}

fn main() {
    let mut h = ValidationHarness::new("exp011_goms_task_completion");
    h.print_provenance(&[&PROVENANCE]);

    validate_known_values(&mut h);
    validate_menu_navigation(&mut h);
    validate_chat_input(&mut h);
    validate_skill_levels(&mut h);

    h.finish();
}

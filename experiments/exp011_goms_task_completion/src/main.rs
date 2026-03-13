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

fn validate_known_values(results: &mut Vec<ValidationResult>) {
    println!("Part 1: Known analytical values");

    let r = ValidationResult::check(
        "exp011_empty",
        "empty task = 0 seconds",
        task_time(&[]),
        0.0,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);

    let single_key = task_time(&[Operator::Keystroke]);
    let r = ValidationResult::check(
        "exp011_single_key",
        "single keystroke = 0.20s (average typist)",
        single_key,
        goms::times::KEYSTROKE_AVG,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);
}

fn validate_menu_navigation(results: &mut Vec<ValidationResult>) {
    println!("\nPart 2: Menu navigation (M P K sequence)");
    // Open inventory: think → point to menu → click
    let open_inventory = [Operator::Mental, Operator::Point, Operator::Keystroke];
    let expected = goms::times::MENTAL + goms::times::POINT + goms::times::KEYSTROKE_AVG;
    let t = task_time(&open_inventory);

    let r = ValidationResult::check(
        "exp011_menu_open",
        "M+P+K = 1.35 + 1.10 + 0.20 = 2.65s",
        t,
        expected,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);

    // Select item: think → point → click → point → click (drag-drop)
    let drag_drop = [
        Operator::Mental,
        Operator::Point,
        Operator::Keystroke,
        Operator::Point,
        Operator::Keystroke,
    ];
    let t2 = task_time(&drag_drop);
    let r = ValidationResult::check(
        "exp011_drag_drop",
        "drag-drop = M + 2P + 2K",
        t2,
        2.0f64.mul_add(
            goms::times::KEYSTROKE_AVG,
            2.0f64.mul_add(goms::times::POINT, goms::times::MENTAL),
        ),
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);
}

fn validate_chat_input(results: &mut Vec<ValidationResult>) {
    println!("\nPart 3: Chat input (type a message)");
    // Type "gg wp" = M + H (to keyboard) + 5K + K(enter)
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
    let r = ValidationResult::check(
        "exp011_chat",
        "typing 'gg wp' + enter = M + H + 6K",
        t,
        expected,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);

    let counts = operator_counts(&chat);
    let r = ValidationResult::check(
        "exp011_counts",
        "correct operator counts for chat sequence",
        if counts.mentals == 1 && counts.homes == 1 && counts.keystrokes == 6 {
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

fn validate_skill_levels(results: &mut Vec<ValidationResult>) {
    println!("\nPart 4: Skill level comparison");
    let typing_task: Vec<Operator> = (0..20).map(|_| Operator::Keystroke).collect();

    let best = task_time_with_keystroke(&typing_task, goms::times::KEYSTROKE_BEST);
    let avg = task_time_with_keystroke(&typing_task, goms::times::KEYSTROKE_AVG);
    let worst = task_time_with_keystroke(&typing_task, goms::times::KEYSTROKE_WORST);

    let r = ValidationResult::check(
        "exp011_skill_order",
        "best < avg < worst typist for 20 keystrokes",
        if best < avg && avg < worst { 1.0 } else { 0.0 },
        1.0,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);

    let r = ValidationResult::check(
        "exp011_best_time",
        "best typist 20 keys = 1.6s",
        best,
        20.0 * goms::times::KEYSTROKE_BEST,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);
}

fn main() {
    println!("=== Exp011: GOMS Task Completion (Validation) ===\n");
    let mut results = Vec::new();

    validate_known_values(&mut results);
    validate_menu_navigation(&mut results);
    validate_chat_input(&mut results);
    validate_skill_levels(&mut results);

    let passed = results.iter().filter(|r| r.passed).count();
    let failed = results.len() - passed;
    println!("\n{passed} passed, {failed} failed");
    if failed > 0 {
        std::process::exit(1);
    }
}

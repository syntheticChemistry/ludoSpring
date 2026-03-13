// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
//! Exp018: Four Keys to Fun classification sweep — validation binary.
//!
//! Validates Lazzaro's (2004) fun taxonomy across game scenario archetypes.
//! Each archetype should classify to the expected dominant fun type.
//!
//! # Provenance
//!
//! Lazzaro, N. (2004). "Why We Play Games: Four Keys to More Emotion
//! Without Story." GDC '04.
//! Python baseline: `baselines/python/fun_keys_model.py` (2026-03-11).

use ludospring_barracuda::metrics::fun_keys::{FunKey, FunSignals, classify_fun};
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

struct Scenario {
    name: &'static str,
    signals: FunSignals,
    expected: FunKey,
}

fn validate_archetypes(results: &mut Vec<ValidationResult>) {
    println!("Part 1: Game scenario archetypes");
    let scenarios = [
        Scenario {
            name: "Dark Souls boss",
            signals: FunSignals {
                challenge: 0.95,
                exploration: 0.2,
                social: 0.05,
                completion: 0.3,
                retry_rate: 0.9,
            },
            expected: FunKey::Hard,
        },
        Scenario {
            name: "Minecraft creative",
            signals: FunSignals {
                challenge: 0.1,
                exploration: 0.9,
                social: 0.1,
                completion: 0.3,
                retry_rate: 0.0,
            },
            expected: FunKey::Easy,
        },
        Scenario {
            name: "Among Us social deduction",
            signals: FunSignals {
                challenge: 0.3,
                exploration: 0.1,
                social: 0.95,
                completion: 0.1,
                retry_rate: 0.1,
            },
            expected: FunKey::People,
        },
        Scenario {
            name: "Animal Crossing collection",
            signals: FunSignals {
                challenge: 0.05,
                exploration: 0.3,
                social: 0.1,
                completion: 0.9,
                retry_rate: 0.0,
            },
            expected: FunKey::Serious,
        },
        Scenario {
            name: "Celeste precision platformer",
            signals: FunSignals {
                challenge: 0.9,
                exploration: 0.3,
                social: 0.0,
                completion: 0.4,
                retry_rate: 0.85,
            },
            expected: FunKey::Hard,
        },
        Scenario {
            name: "No Man's Sky exploration",
            signals: FunSignals {
                challenge: 0.15,
                exploration: 0.85,
                social: 0.15,
                completion: 0.2,
                retry_rate: 0.05,
            },
            expected: FunKey::Easy,
        },
    ];

    for scenario in &scenarios {
        let result = classify_fun(&scenario.signals);
        let r = ValidationResult::check(
            "exp018_archetype",
            &format!("{} → {}", scenario.name, scenario.expected),
            if result.dominant == scenario.expected {
                1.0
            } else {
                0.0
            },
            1.0,
            tolerances::ANALYTICAL_TOL,
        );
        report(&r);
        if !r.passed {
            println!(
                "    got {} (H={:.2}, E={:.2}, P={:.2}, S={:.2})",
                result.dominant,
                result.scores.hard,
                result.scores.easy,
                result.scores.people,
                result.scores.serious
            );
        }
        results.push(r);
    }
}

fn validate_score_properties(results: &mut Vec<ValidationResult>) {
    println!("\nPart 2: Score properties");

    // Zero signals should produce non-negative scores
    let zero = classify_fun(&FunSignals::default());
    let all_nonneg = zero.scores.hard >= 0.0
        && zero.scores.easy >= 0.0
        && zero.scores.people >= 0.0
        && zero.scores.serious >= 0.0;
    let r = ValidationResult::check(
        "exp018_zero_nonneg",
        "zero signals → non-negative scores",
        if all_nonneg { 1.0 } else { 0.0 },
        1.0,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);

    // Max signals should produce bounded scores (≤ 1.0)
    let max = classify_fun(&FunSignals {
        challenge: 1.0,
        exploration: 1.0,
        social: 1.0,
        completion: 1.0,
        retry_rate: 1.0,
    });
    let all_bounded = max.scores.hard <= 1.0
        && max.scores.easy <= 1.0
        && max.scores.people <= 1.0
        && max.scores.serious <= 1.0;
    let r = ValidationResult::check(
        "exp018_max_bounded",
        "max signals → all scores ≤ 1.0",
        if all_bounded { 1.0 } else { 0.0 },
        1.0,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);
}

fn validate_sensitivity(results: &mut Vec<ValidationResult>) {
    println!("\nPart 3: Sensitivity — dominant changes with signal shift");
    let base = FunSignals {
        challenge: 0.5,
        exploration: 0.5,
        social: 0.5,
        completion: 0.5,
        retry_rate: 0.5,
    };

    // Boosting challenge should push toward Hard
    let boosted = FunSignals {
        challenge: 1.0,
        retry_rate: 1.0,
        ..base
    };
    let result = classify_fun(&boosted);
    let r = ValidationResult::check(
        "exp018_boost_hard",
        "boosting challenge+retry → Hard dominates",
        if result.dominant == FunKey::Hard {
            1.0
        } else {
            0.0
        },
        1.0,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);

    // Boosting social should push toward People
    let social_boost = FunSignals {
        social: 1.0,
        challenge: 0.0,
        exploration: 0.0,
        completion: 0.0,
        retry_rate: 0.0,
    };
    let result = classify_fun(&social_boost);
    let r = ValidationResult::check(
        "exp018_boost_people",
        "isolated social signal → People dominates",
        if result.dominant == FunKey::People {
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

fn main() {
    println!("=== Exp018: Four Keys to Fun Classification (Validation) ===\n");
    let mut results = Vec::new();

    validate_archetypes(&mut results);
    validate_score_properties(&mut results);
    validate_sensitivity(&mut results);

    let passed = results.iter().filter(|r| r.passed).count();
    let failed = results.len() - passed;
    println!("\n{passed} passed, {failed} failed");
    if failed > 0 {
        std::process::exit(1);
    }
}

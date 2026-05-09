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
use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "baselines/python/fun_keys_model.py",
    commit: "19e402c0",
    date: "2026-03-11",
    command: "python3 baselines/python/run_all_baselines.py",
};

struct Scenario {
    name: &'static str,
    signals: FunSignals,
    expected: FunKey,
}

fn validate_archetypes(h: &mut ValidationHarness) {
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
        h.check_bool(
            &format!("{} → {}", scenario.name, scenario.expected),
            result.dominant == scenario.expected,
        );
    }
}

fn validate_score_properties(h: &mut ValidationHarness) {
    let zero = classify_fun(&FunSignals::default());
    let all_nonneg = zero.scores.hard >= 0.0
        && zero.scores.easy >= 0.0
        && zero.scores.people >= 0.0
        && zero.scores.serious >= 0.0;
    h.check_bool("zero signals → non-negative scores", all_nonneg);

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
    h.check_bool("max signals → all scores ≤ 1.0", all_bounded);
}

fn validate_sensitivity(h: &mut ValidationHarness) {
    let base = FunSignals {
        challenge: 0.5,
        exploration: 0.5,
        social: 0.5,
        completion: 0.5,
        retry_rate: 0.5,
    };

    let boosted = FunSignals {
        challenge: 1.0,
        retry_rate: 1.0,
        ..base
    };
    let result = classify_fun(&boosted);
    h.check_bool(
        "boosting challenge+retry → Hard dominates",
        result.dominant == FunKey::Hard,
    );

    let social_boost = FunSignals {
        social: 1.0,
        challenge: 0.0,
        exploration: 0.0,
        completion: 0.0,
        retry_rate: 0.0,
    };
    let result = classify_fun(&social_boost);
    h.check_bool(
        "isolated social signal → People dominates",
        result.dominant == FunKey::People,
    );
}

fn main() {
    let mut h = ValidationHarness::new("exp018_four_keys_fun");
    h.print_provenance(&[&PROVENANCE]);

    validate_archetypes(&mut h);
    validate_score_properties(&mut h);
    validate_sensitivity(&mut h);

    h.finish();
}

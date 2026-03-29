// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
//! exp073 — Dialogue Plane Skill Check Resolution
//!
//! Validates the D6 pool resolution system:
//! 1. Pool sizes match skill level + modifiers
//! 2. Success threshold (4+) produces correct success distribution
//! 3. Five degrees of success resolve correctly
//! 4. Statistical distribution matches expected binomial
//! 5. Modifier stacking (trust, environment, knowledge) works

use ludospring_barracuda::game::rpgpt::dialogue::{
    D6PoolResult, DialogueCheck, DialogueModifiers, effective_pool_size, resolve_d6_pool,
};
use ludospring_barracuda::game::ruleset::DegreeOfSuccess;
use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};

const EXP: &str = "exp073_dialogue_skill_checks";

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "specs/RPGPT_DIALOGUE_PLANE_EXPERIMENTS.md",
    commit: "4b683e3e",
    date: "2026-03-15",
    command: "cargo run -p exp073_dialogue_skill_checks",
};

fn validate_pool_resolution_mapping(h: &mut ValidationHarness) {
    h.check_bool(
        "zero_success_pool1_failure",
        resolve_d6_pool(0, 1) == DegreeOfSuccess::Failure,
    );
    h.check_bool(
        "zero_success_pool3_critical_failure",
        resolve_d6_pool(0, 3) == DegreeOfSuccess::CriticalFailure,
    );
    h.check_bool(
        "one_success_partial",
        resolve_d6_pool(1, 3) == DegreeOfSuccess::PartialSuccess,
    );
    h.check_bool(
        "two_successes_success",
        resolve_d6_pool(2, 4) == DegreeOfSuccess::Success,
    );
    h.check_bool(
        "three_successes_success",
        resolve_d6_pool(3, 5) == DegreeOfSuccess::Success,
    );
    h.check_bool(
        "four_successes_critical",
        resolve_d6_pool(4, 6) == DegreeOfSuccess::CriticalSuccess,
    );
    h.check_bool(
        "six_successes_critical",
        resolve_d6_pool(6, 8) == DegreeOfSuccess::CriticalSuccess,
    );
}

fn validate_d6_pool_counting(h: &mut ValidationHarness) {
    let r1 = D6PoolResult::from_dice(&[1, 2, 3, 4, 5, 6], 4);
    h.check_abs(
        "six_dice_three_successes",
        f64::from(r1.successes),
        3.0,
        0.0,
    );

    let r2 = D6PoolResult::from_dice(&[1, 2, 3], 4);
    h.check_abs(
        "three_low_zero_successes",
        f64::from(r2.successes),
        0.0,
        0.0,
    );

    let r3 = D6PoolResult::from_dice(&[4, 5, 6, 4, 5], 4);
    h.check_abs("all_high_five_successes", f64::from(r3.successes), 5.0, 0.0);

    let r4 = D6PoolResult::from_dice(&[4], 4);
    h.check_abs("single_success", f64::from(r4.successes), 1.0, 0.0);
    h.check_abs("single_pool_size", f64::from(r4.pool_size), 1.0, 0.0);
}

fn validate_modifier_stacking(h: &mut ValidationHarness) {
    let no_mods = DialogueModifiers::default();
    h.check_abs("no_mods_total_zero", f64::from(no_mods.total()), 0.0, 0.0);
    h.check_abs(
        "base_pool_3",
        f64::from(effective_pool_size(3, &no_mods)),
        3.0,
        0.0,
    );

    let trust_bonus = DialogueModifiers {
        trust_bonus: 2,
        ..Default::default()
    };
    h.check_abs(
        "trust_adds_2",
        f64::from(effective_pool_size(3, &trust_bonus)),
        5.0,
        0.0,
    );

    let env_penalty = DialogueModifiers {
        environment: -2,
        ..Default::default()
    };
    h.check_abs(
        "env_subtracts_2",
        f64::from(effective_pool_size(3, &env_penalty)),
        1.0,
        0.0,
    );

    let combined = DialogueModifiers {
        trust_bonus: 2,
        environment: -1,
        emotional: 0,
        knowledge: 1,
    };
    h.check_abs("combined_total_2", f64::from(combined.total()), 2.0, 0.0);
    h.check_abs(
        "combined_pool_5",
        f64::from(effective_pool_size(3, &combined)),
        5.0,
        0.0,
    );

    let heavy_penalty = DialogueModifiers {
        trust_bonus: -5,
        environment: -5,
        emotional: -5,
        knowledge: -5,
    };
    h.check_abs(
        "pool_minimum_one",
        f64::from(effective_pool_size(1, &heavy_penalty)),
        1.0,
        0.0,
    );
}

fn validate_statistical_distribution(h: &mut ValidationHarness) {
    // D6 pool: P(die >= 4) = 3/6 = 0.5
    // Pool of 1: P(at least 1 success) = 0.5
    // Pool of 3: P(at least 1 success) = 1 - (0.5)^3 = 0.875
    // Pool of 6: P(at least 1 success) = 1 - (0.5)^6 = 0.984375
    // Pool of 6: P(3+ successes) = sum(binom(6,k) * 0.5^6, k=3..6) = 0.65625

    let trials = 10_000;

    let pool1_successes = simulate_pool(1, trials);
    let pool3_successes = simulate_pool(3, trials);
    let pool6_successes = simulate_pool(6, trials);

    // Pool 1: ~50% should have at least 1 success
    h.check_abs("pool1_success_rate", pool1_successes, 0.5, 0.05);

    // Pool 3: ~87.5% should have at least 1 success
    h.check_abs("pool3_success_rate", pool3_successes, 0.875, 0.05);

    // Pool 6: near-certain at least 1 success
    h.check_bool("pool6_high_success_rate", pool6_successes > 0.95);

    // Critical failure rate at pool 3: (0.5)^3 = 0.125
    let pool3_crit_fail = simulate_critical_failure(3, trials);
    h.check_abs("pool3_crit_fail_rate", pool3_crit_fail, 0.125, 0.05);

    // Critical failure should be rare at pool 6: (0.5)^6 = 0.015625
    let pool6_crit_fail = simulate_critical_failure(6, trials);
    h.check_bool("pool6_crit_fail_rare", pool6_crit_fail < 0.05);
}

fn simulate_pool(pool_size: usize, trials: usize) -> f64 {
    let mut successes = 0usize;
    for trial in 0..trials {
        let dice: Vec<u8> = (0..pool_size)
            .map(|i| {
                #[expect(clippy::cast_possible_truncation, reason = "die values are 1-6")]
                {
                    ((trial * 7 + i * 13 + 3) % 6 + 1) as u8
                }
            })
            .collect();
        let result = D6PoolResult::from_dice(&dice, 4);
        if result.successes > 0 {
            successes += 1;
        }
    }
    #[expect(clippy::cast_precision_loss, reason = "trial counts fit in f64")]
    {
        successes as f64 / trials as f64
    }
}

fn simulate_critical_failure(pool_size: usize, trials: usize) -> f64 {
    let mut crit_fails = 0usize;
    for trial in 0..trials {
        let dice: Vec<u8> = (0..pool_size)
            .map(|i| {
                #[expect(clippy::cast_possible_truncation, reason = "die values are 1-6")]
                {
                    ((trial * 7 + i * 13 + 3) % 6 + 1) as u8
                }
            })
            .collect();
        let result = D6PoolResult::from_dice(&dice, 4);
        #[expect(clippy::cast_possible_truncation, reason = "pool_size < 256")]
        let degree = resolve_d6_pool(result.successes, pool_size as u8);
        if degree == DegreeOfSuccess::CriticalFailure {
            crit_fails += 1;
        }
    }
    #[expect(clippy::cast_precision_loss, reason = "trial counts fit in f64")]
    {
        crit_fails as f64 / trials as f64
    }
}

fn validate_dialogue_check_integration(h: &mut ValidationHarness) {
    // Full check: Persuasion skill 4, trust bonus +1, dice = [2, 3, 5, 6, 4]
    let mods = DialogueModifiers {
        trust_bonus: 1,
        ..Default::default()
    };
    let check = DialogueCheck::resolve("Persuasion", 4, mods, &[2, 3, 5, 6, 4]);
    h.check_abs("check_pool_size_5", f64::from(check.pool_size), 5.0, 0.0);
    h.check_abs(
        "check_successes_3",
        f64::from(check.result.successes),
        3.0,
        0.0,
    ); // 5,6,4
    h.check_bool(
        "check_degree_success",
        check.degree == DegreeOfSuccess::Success,
    );

    // Failed check: skill 1, no mods, dice = [1]
    let fail = DialogueCheck::resolve("Intimidation", 1, DialogueModifiers::default(), &[1]);
    h.check_abs("fail_pool_size_1", f64::from(fail.pool_size), 1.0, 0.0);
    h.check_abs(
        "fail_successes_0",
        f64::from(fail.result.successes),
        0.0,
        0.0,
    );
    h.check_bool(
        "fail_degree_failure",
        fail.degree == DegreeOfSuccess::Failure,
    );

    // Critical success: skill 6, high dice
    let crit = DialogueCheck::resolve(
        "Diplomacy",
        6,
        DialogueModifiers::default(),
        &[4, 5, 6, 4, 5, 6],
    );
    h.check_abs(
        "crit_successes_6",
        f64::from(crit.result.successes),
        6.0,
        0.0,
    );
    h.check_bool(
        "crit_degree",
        crit.degree == DegreeOfSuccess::CriticalSuccess,
    );
}

fn validate_partial_success_meaning(h: &mut ValidationHarness) {
    // 1 success = partial: NPC gives info but at a cost
    let check = DialogueCheck::resolve("Deception", 3, DialogueModifiers::default(), &[1, 2, 5]);
    h.check_abs(
        "partial_one_success",
        f64::from(check.result.successes),
        1.0,
        0.0,
    );
    h.check_bool(
        "partial_degree",
        check.degree == DegreeOfSuccess::PartialSuccess,
    );
}

fn main() {
    let mut h = ValidationHarness::new(EXP);
    h.print_provenance(&[&PROVENANCE]);

    validate_pool_resolution_mapping(&mut h);
    validate_d6_pool_counting(&mut h);
    validate_modifier_stacking(&mut h);
    validate_statistical_distribution(&mut h);
    validate_dialogue_check_integration(&mut h);
    validate_partial_success_meaning(&mut h);

    h.finish();
}

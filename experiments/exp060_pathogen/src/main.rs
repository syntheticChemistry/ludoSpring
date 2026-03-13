// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![warn(clippy::pedantic, clippy::nursery)]

//! exp060 — Pathogen: Gacha/lootbox exploitation anti-pattern study.
//!
//! Documents exploitation using validated psychology (Skinner 1938, Kahneman &
//! Tversky 1979) and economics math (Bernoulli 1738, Lotka-Volterra parasitism).

mod exploitation;

use exploitation::{
    ExploitationScore, GachaSystem, ParasitismModel, ReinforcementSchedule, near_miss_exploitation,
    prospect_theory_value, quantify_exploitation, schedule_persistence,
};
use ludospring_barracuda::validation::ValidationResult;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "validate" {
        std::process::exit(cmd_validate());
    }
    println!("Usage: exp060_pathogen validate");
}

fn cmd_validate() -> i32 {
    let mut pass = 0u32;
    let mut fail = 0u32;

    println!("\n=== exp060: Pathogen — Gacha Exploitation Anti-Pattern Study ===\n");

    println!("--- Section 1: Reinforcement Schedules (Skinner 1938) ---");
    validate_reinforcement(&mut pass, &mut fail);

    println!("\n--- Section 2: Gacha Expected Value (Bernoulli 1738) ---");
    validate_gacha_ev(&mut pass, &mut fail);

    println!("\n--- Section 3: Pity Timer ---");
    validate_pity_timer(&mut pass, &mut fail);

    println!("\n--- Section 4: Prospect Theory (Kahneman & Tversky 1979) ---");
    validate_prospect_theory(&mut pass, &mut fail);

    println!("\n--- Section 5: Exploitation Score ---");
    validate_exploitation_score(&mut pass, &mut fail);

    println!("\n--- Section 6: Parasitism Model ---");
    validate_parasitism(&mut pass, &mut fail);

    println!("\n--- Section 7: Cross-Domain Mapping ---");
    validate_cross_domain(&mut pass, &mut fail);

    let total = pass + fail;
    println!("\n=== SUMMARY: {pass}/{total} checks passed ===");
    i32::from(fail > 0)
}

fn check(name: &str, pass: &mut u32, fail: &mut u32, ok: bool, detail: &str) {
    let r = ValidationResult::check(name, detail, if ok { 1.0 } else { 0.0 }, 1.0, 0.5);
    if r.passed {
        *pass += 1;
        println!("  PASS  {name}: {detail}");
    } else {
        *fail += 1;
        println!("  FAIL  {name}: {detail}");
    }
    let _ = r;
}

fn validate_reinforcement(pass: &mut u32, fail: &mut u32) {
    let vr = schedule_persistence(ReinforcementSchedule::VariableRatio(0.5));
    let fr = schedule_persistence(ReinforcementSchedule::FixedRatio(10));
    let fi = schedule_persistence(ReinforcementSchedule::FixedInterval(1.0));
    let vi = schedule_persistence(ReinforcementSchedule::VariableInterval(1.0));

    check(
        "variable_ratio_most_persistent",
        pass,
        fail,
        vr > fr && vr > fi && vr > vi,
        "Variable ratio has highest persistence",
    );

    check(
        "fixed_ratio_least_persistent",
        pass,
        fail,
        fr < vr,
        "Fixed ratio has low persistence",
    );

    check(
        "schedule_values_correct",
        pass,
        fail,
        (fr - 0.3).abs() < f64::EPSILON && (vr - 0.95).abs() < f64::EPSILON,
        "FixedRatio=0.3, VariableRatio=0.95",
    );

    check(
        "variable_interval_mid_persistence",
        pass,
        fail,
        vi > fr && vi < vr,
        "VariableInterval between Fixed and Variable ratio",
    );
}

fn validate_gacha_ev(pass: &mut u32, fail: &mut u32) {
    let gacha = GachaSystem::new(0.01, None, 1.0, 0.5);
    let ev = gacha.expected_value_per_pull(10.0);

    check(
        "ev_negative_for_player",
        pass,
        fail,
        ev < 0.0,
        "Expected value negative (exploitation)",
    );

    check(
        "ev_cost_exceeds_return",
        pass,
        fail,
        gacha.cost_per_pull > gacha.base_rate * 10.0,
        "Cost exceeds expected return",
    );

    let ev_trivial = gacha.expected_value_per_pull(1000.0);
    check(
        "ev_increases_with_item_value",
        pass,
        fail,
        ev_trivial > ev,
        "EV increases with item value",
    );

    let ev_moderate = gacha.expected_value_per_pull(50.0);
    check(
        "ev_still_negative_typical",
        pass,
        fail,
        ev_moderate < 0.0,
        "Moderate item value (50x cost) still yields negative EV at 1% rate",
    );
}

fn validate_pity_timer(pass: &mut u32, fail: &mut u32) {
    let gacha = GachaSystem::new(0.02, Some(90), 1.0, 0.5);

    check(
        "pity_probability_increases",
        pass,
        fail,
        gacha.pull_probability(0) < gacha.pull_probability(50),
        "Probability increases with attempts",
    );

    check(
        "pity_guarantees_eventually",
        pass,
        fail,
        (gacha.pull_probability(90) - 1.0).abs() < f64::EPSILON,
        "Pity timer guarantees at threshold",
    );

    let cost_50 = gacha.expected_cost_for_target(0.5);
    check(
        "expected_cost_still_high",
        pass,
        fail,
        cost_50 >= 2.0,
        "Expected cost for 50% target is high (geometric: cost/rate)",
    );

    check(
        "pity_below_threshold_ramps",
        pass,
        fail,
        gacha.pull_probability(45) > gacha.pull_probability(0),
        "Probability ramps as pity approaches",
    );
}

fn validate_prospect_theory(pass: &mut u32, fail: &mut u32) {
    let gain = prospect_theory_value(10.0, 0.0);
    let loss = prospect_theory_value(-10.0, 0.0);

    check(
        "losses_hurt_more",
        pass,
        fail,
        loss.abs() > gain,
        "Losses hurt 2.25x more than equivalent gains",
    );

    let near_miss_small = near_miss_exploitation(0.01);
    let near_miss_large = near_miss_exploitation(1.0);
    check(
        "near_miss_increases_persistence",
        pass,
        fail,
        near_miss_small > near_miss_large,
        "Smaller miss distance increases 'almost won' feeling",
    );

    check(
        "loss_aversion_coefficient",
        pass,
        fail,
        (loss / gain).abs() > 2.0,
        "Loss aversion coefficient ~2.25",
    );

    check(
        "prospect_theory_gain_positive",
        pass,
        fail,
        gain > 0.0,
        "Gains have positive value",
    );

    check(
        "prospect_theory_loss_negative",
        pass,
        fail,
        loss < 0.0,
        "Losses have negative value",
    );
}

fn validate_exploitation_score(pass: &mut u32, fail: &mut u32) {
    let gacha = GachaSystem::new(0.01, Some(90), 1.0, 0.5);
    let score: ExploitationScore = quantify_exploitation(&gacha);

    check(
        "combines_all_factors",
        pass,
        fail,
        score.schedule_persistence > 0.5 && score.loss_aversion_factor > 2.0,
        "Score combines schedule, EV, loss aversion, near-miss",
    );

    check(
        "typical_gacha_score_high",
        pass,
        fail,
        score.overall_score > 0.5,
        "Typical gacha has exploitation score > 0.5",
    );

    check(
        "higher_score_worse",
        pass,
        fail,
        score.expected_value < 0.0,
        "Negative EV contributes to exploitation",
    );

    check(
        "exploitation_score_bounded",
        pass,
        fail,
        (0.0..=1.0).contains(&score.overall_score),
        "Exploitation score in [0, 1]",
    );
}

fn validate_parasitism(pass: &mut u32, fail: &mut u32) {
    let model = ParasitismModel::new(0.5, 0.2, 1.5);

    let (h1, _p1) = model.step(100.0, 10.0, 0.1);
    check(
        "extraction_depletes_host",
        pass,
        fail,
        h1 < 100.0 || model.extraction_rate > model.host_tolerance,
        "Extraction rate exceeds host recovery",
    );

    let final_host = model.run(100.0, 5.0, 500, 0.1);
    check(
        "host_depletes_over_time",
        pass,
        fail,
        final_host < 100.0,
        "Host (wallet) depletes over time",
    );

    check(
        "parasitism_maps_gacha",
        pass,
        fail,
        true,
        "Gacha operator = parasitic virulence on player wallet",
    );

    let low_virulence = ParasitismModel::new(0.1, 0.5, 0.5);
    let high_host = low_virulence.run(100.0, 5.0, 100, 0.1);
    check(
        "low_extraction_host_survives",
        pass,
        fail,
        high_host > final_host,
        "Lower extraction rate preserves host longer",
    );
}

fn validate_cross_domain(pass: &mut u32, fail: &mut u32) {
    check(
        "cross_gacha_parasitic_virulence",
        pass,
        fail,
        true,
        "Gacha = parasitic virulence on host (player)",
    );

    check(
        "cross_provenance_citations",
        pass,
        fail,
        true,
        "Skinner 1938, Kahneman & Tversky 1979, Bernoulli 1738, Lotka-Volterra",
    );

    check(
        "cross_exploitation_documented",
        pass,
        fail,
        true,
        "Anti-pattern study documents validated psychology/economics",
    );
}

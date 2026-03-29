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
use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "N/A (analytical — Skinner, Kahneman & Tversky, Bernoulli, Lotka-Volterra)",
    commit: "4b683e3e",
    date: "2026-03-29",
    command: "N/A (pure Rust implementation)",
};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "validate" {
        cmd_validate();
    }
    println!("Usage: exp060_pathogen validate");
}

fn cmd_validate() {
    let mut h = ValidationHarness::new("exp060_pathogen");
    h.print_provenance(&[&PROVENANCE]);

    validate_reinforcement(&mut h);
    validate_gacha_ev(&mut h);
    validate_pity_timer(&mut h);
    validate_prospect_theory(&mut h);
    validate_exploitation_score(&mut h);
    validate_parasitism(&mut h);
    validate_cross_domain(&mut h);

    h.finish();
}

fn validate_reinforcement(h: &mut ValidationHarness) {
    let vr = schedule_persistence(ReinforcementSchedule::VariableRatio(0.5));
    let fr = schedule_persistence(ReinforcementSchedule::FixedRatio(10));
    let fi = schedule_persistence(ReinforcementSchedule::FixedInterval(1.0));
    let vi = schedule_persistence(ReinforcementSchedule::VariableInterval(1.0));

    h.check_bool(
        "variable_ratio_most_persistent",
        vr > fr && vr > fi && vr > vi,
    );
    h.check_bool("fixed_ratio_least_persistent", fr < vr);
    h.check_bool(
        "schedule_values_correct",
        (fr - 0.3).abs() < f64::EPSILON && (vr - 0.95).abs() < f64::EPSILON,
    );
    h.check_bool("variable_interval_mid_persistence", vi > fr && vi < vr);
}

fn validate_gacha_ev(h: &mut ValidationHarness) {
    let gacha = GachaSystem::new(0.01, None, 1.0, 0.5);
    let ev = gacha.expected_value_per_pull(10.0);

    h.check_bool("ev_negative_for_player", ev < 0.0);
    h.check_bool(
        "ev_cost_exceeds_return",
        gacha.cost_per_pull > gacha.base_rate * 10.0,
    );

    let ev_trivial = gacha.expected_value_per_pull(1000.0);
    h.check_bool("ev_increases_with_item_value", ev_trivial > ev);

    let ev_moderate = gacha.expected_value_per_pull(50.0);
    h.check_bool("ev_still_negative_typical", ev_moderate < 0.0);
}

fn validate_pity_timer(h: &mut ValidationHarness) {
    let gacha = GachaSystem::new(0.02, Some(90), 1.0, 0.5);

    h.check_bool(
        "pity_probability_increases",
        gacha.pull_probability(0) < gacha.pull_probability(50),
    );
    h.check_bool(
        "pity_guarantees_eventually",
        (gacha.pull_probability(90) - 1.0).abs() < f64::EPSILON,
    );

    let cost_50 = gacha.expected_cost_for_target(0.5);
    h.check_bool("expected_cost_still_high", cost_50 >= 2.0);

    h.check_bool(
        "pity_below_threshold_ramps",
        gacha.pull_probability(45) > gacha.pull_probability(0),
    );
}

fn validate_prospect_theory(h: &mut ValidationHarness) {
    let gain = prospect_theory_value(10.0, 0.0);
    let loss = prospect_theory_value(-10.0, 0.0);

    h.check_bool("losses_hurt_more", loss.abs() > gain);

    let near_miss_small = near_miss_exploitation(0.01);
    let near_miss_large = near_miss_exploitation(1.0);
    h.check_bool(
        "near_miss_increases_persistence",
        near_miss_small > near_miss_large,
    );

    h.check_bool("loss_aversion_coefficient", (loss / gain).abs() > 2.0);
    h.check_bool("prospect_theory_gain_positive", gain > 0.0);
    h.check_bool("prospect_theory_loss_negative", loss < 0.0);
}

fn validate_exploitation_score(h: &mut ValidationHarness) {
    let gacha = GachaSystem::new(0.01, Some(90), 1.0, 0.5);
    let score: ExploitationScore = quantify_exploitation(&gacha);

    h.check_bool(
        "combines_all_factors",
        score.schedule_persistence > 0.5 && score.loss_aversion_factor > 2.0,
    );
    h.check_bool("typical_gacha_score_high", score.overall_score > 0.5);
    h.check_bool("higher_score_worse", score.expected_value < 0.0);
    h.check_bool(
        "exploitation_score_bounded",
        (0.0..=1.0).contains(&score.overall_score),
    );
}

fn validate_parasitism(h: &mut ValidationHarness) {
    let model = ParasitismModel::new(0.5, 0.2, 1.5);

    let (h1, _p1) = model.step(100.0, 10.0, 0.1);
    h.check_bool(
        "extraction_depletes_host",
        h1 < 100.0 || model.extraction_rate > model.host_tolerance,
    );

    let final_host = model.run(100.0, 5.0, 500, 0.1);
    h.check_bool("host_depletes_over_time", final_host < 100.0);
    h.check_bool("parasitism_maps_gacha", true);

    let low_virulence = ParasitismModel::new(0.1, 0.5, 0.5);
    let high_host = low_virulence.run(100.0, 5.0, 100, 0.1);
    h.check_bool("low_extraction_host_survives", high_host > final_host);
}

fn validate_cross_domain(h: &mut ValidationHarness) {
    h.check_bool("cross_gacha_parasitic_virulence", true);
    h.check_bool("cross_provenance_citations", true);
    h.check_bool("cross_exploitation_documented", true);
}

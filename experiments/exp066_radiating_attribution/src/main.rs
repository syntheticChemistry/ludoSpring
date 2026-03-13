// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
//! exp066 — Radiating Attribution
//!
//! sunCloud radiating attribution mechanism: when a Novel Ferment Transcript
//! generates value, walk the sweetGrass attribution chain and compute
//! proportional credit for every contributor.

mod attribution;

use std::collections::HashMap;

use attribution::{
    AgentRole, AttributionChain, DecayModel, RoleWeighting, ValueEvent,
    ValueEventType, compute_distribution, simulate_cascade, verify_distribution,
};
use ludospring_barracuda::validation::ValidationResult;

const EXP: &str = "exp066_radiating_attribution";

const fn bool_f64(b: bool) -> f64 {
    if b { 1.0 } else { 0.0 }
}

// ===========================================================================
// 1. Role Weighting
// ===========================================================================

fn validate_role_weighting() -> Vec<ValidationResult> {
    let mut results = Vec::new();
    let w = RoleWeighting::default();

    results.push(ValidationResult::check(
        EXP, "creator_weight_1",
        w.weight(AgentRole::Creator), 1.0, 0.0,
    ));
    results.push(ValidationResult::check(
        EXP, "contributor_weight_0_7",
        w.weight(AgentRole::Contributor), 0.7, 0.0,
    ));
    results.push(ValidationResult::check(
        EXP, "validator_weight_0_5",
        w.weight(AgentRole::Validator), 0.5, 0.0,
    ));
    results.push(ValidationResult::check(
        EXP, "observer_weight_0_2",
        w.weight(AgentRole::Observer), 0.2, 0.0,
    ));

    let creator_gt_contributor = w.weight(AgentRole::Creator) > w.weight(AgentRole::Contributor);
    results.push(ValidationResult::check(
        EXP, "creator_gt_contributor",
        bool_f64(creator_gt_contributor), 1.0, 0.0,
    ));
    let contributor_gt_validator = w.weight(AgentRole::Contributor) > w.weight(AgentRole::Validator);
    results.push(ValidationResult::check(
        EXP, "contributor_gt_validator",
        bool_f64(contributor_gt_validator), 1.0, 0.0,
    ));
    let validator_gt_observer = w.weight(AgentRole::Validator) > w.weight(AgentRole::Observer);
    results.push(ValidationResult::check(
        EXP, "validator_gt_observer",
        bool_f64(validator_gt_observer), 1.0, 0.0,
    ));

    let mut custom = HashMap::new();
    custom.insert(AgentRole::Creator, 0.9);
    custom.insert(AgentRole::Contributor, 0.6);
    let custom_w = RoleWeighting { weights: custom };
    results.push(ValidationResult::check(
        EXP, "configurable_weights_creator",
        custom_w.weight(AgentRole::Creator), 0.9, 0.0,
    ));
    results.push(ValidationResult::check(
        EXP, "configurable_weights_contributor",
        custom_w.weight(AgentRole::Contributor), 0.6, 0.0,
    ));
    results.push(ValidationResult::check(
        EXP, "unknown_role_zero",
        custom_w.weight(AgentRole::Validator), 0.0, 0.0,
    ));

    results
}

// ===========================================================================
// 2. Decay Models
// ===========================================================================

fn validate_decay_models() -> Vec<ValidationResult> {
    let mut results = Vec::new();

    let none = DecayModel::None;
    results.push(ValidationResult::check(
        EXP, "no_decay_returns_base",
        none.apply(1.0, 100), 1.0, 0.0,
    ));

    let linear = DecayModel::Linear { half_life_ticks: 50 };
    results.push(ValidationResult::check(
        EXP, "linear_zero_ticks_full_weight",
        linear.apply(1.0, 0), 1.0, 1e-10,
    ));
    results.push(ValidationResult::check(
        EXP, "linear_half_life_halves",
        linear.apply(1.0, 50), 0.5, 1e-10,
    ));
    results.push(ValidationResult::check(
        EXP, "linear_double_half_life_zero",
        linear.apply(1.0, 100), 0.0, 1e-10,
    ));
    results.push(ValidationResult::check(
        EXP, "linear_never_negative",
        bool_f64(linear.apply(1.0, 200) >= 0.0), 1.0, 0.0,
    ));

    let exp_decay = DecayModel::Exponential { decay_rate: 0.01 };
    results.push(ValidationResult::check(
        EXP, "exponential_zero_ticks_full",
        exp_decay.apply(1.0, 0), 1.0, 1e-10,
    ));
    let large_ticks = exp_decay.apply(1.0, 1000);
    results.push(ValidationResult::check(
        EXP, "exponential_approaches_zero",
        bool_f64(large_ticks < 0.001), 1.0, 0.0,
    ));

    results
}

// ===========================================================================
// 3. Single Creator
// ===========================================================================

fn validate_single_creator() -> Vec<ValidationResult> {
    let mut results = Vec::new();

    let mut chain = AttributionChain::new();
    chain.add("did:key:alice", AgentRole::Creator, 0, "created artwork");

    let event = ValueEvent {
        event_type: ValueEventType::Sale,
        amount: 100.0,
        tick: 10,
    };
    let dist = compute_distribution(&chain, &event, &DecayModel::None, &RoleWeighting::default());

    results.push(ValidationResult::check(
        EXP, "single_creator_one_share",
        dist.shares.len() as f64, 1.0, 0.0,
    ));
    results.push(ValidationResult::check(
        EXP, "single_creator_100_percent",
        dist.shares[0].share, 1.0, 1e-10,
    ));
    results.push(ValidationResult::check(
        EXP, "single_creator_verify_sum",
        bool_f64(verify_distribution(&dist)), 1.0, 0.0,
    ));

    results
}

// ===========================================================================
// 4. Multi Contributor
// ===========================================================================

fn validate_multi_contributor() -> Vec<ValidationResult> {
    let mut results = Vec::new();

    let mut chain = AttributionChain::new();
    chain.add("did:key:alice", AgentRole::Creator, 0, "created");
    chain.add("did:key:bob", AgentRole::Contributor, 5, "contributed");
    chain.add("did:key:carol", AgentRole::Validator, 8, "validated");

    let event = ValueEvent {
        event_type: ValueEventType::Sale,
        amount: 100.0,
        tick: 10,
    };
    let dist = compute_distribution(&chain, &event, &DecayModel::None, &RoleWeighting::default());

    results.push(ValidationResult::check(
        EXP, "multi_three_shares",
        dist.shares.len() as f64, 3.0, 0.0,
    ));

    let creator_share = dist.shares.iter().find(|s| s.agent_did == "did:key:alice").map(|s| s.share).unwrap_or(0.0);
    let contributor_share = dist.shares.iter().find(|s| s.agent_did == "did:key:bob").map(|s| s.share).unwrap_or(0.0);
    let validator_share = dist.shares.iter().find(|s| s.agent_did == "did:key:carol").map(|s| s.share).unwrap_or(0.0);

    let sum = creator_share + contributor_share + validator_share;
    results.push(ValidationResult::check(
        EXP, "multi_sum_one",
        sum, 1.0, 1e-10,
    ));
    results.push(ValidationResult::check(
        EXP, "multi_creator_largest",
        bool_f64(creator_share > contributor_share && creator_share > validator_share), 1.0, 0.0,
    ));

    let expected_creator = 1.0 / (1.0 + 0.7 + 0.5);
    results.push(ValidationResult::check(
        EXP, "multi_proportional_to_weights",
        creator_share, expected_creator, 1e-10,
    ));
    results.push(ValidationResult::check(
        EXP, "multi_verify_distribution",
        bool_f64(verify_distribution(&dist)), 1.0, 0.0,
    ));

    results
}

// ===========================================================================
// 5. Time Decay
// ===========================================================================

fn validate_time_decay() -> Vec<ValidationResult> {
    let mut results = Vec::new();

    let mut chain = AttributionChain::new();
    chain.add("did:key:early", AgentRole::Contributor, 0, "early");
    chain.add("did:key:late", AgentRole::Contributor, 90, "late");

    let event = ValueEvent {
        event_type: ValueEventType::Sale,
        amount: 100.0,
        tick: 100,
    };

    let no_decay = compute_distribution(&chain, &event, &DecayModel::None, &RoleWeighting::default());
    let early_no = no_decay.shares.iter().find(|s| s.agent_did == "did:key:early").map(|s| s.share).unwrap_or(0.0);
    let late_no = no_decay.shares.iter().find(|s| s.agent_did == "did:key:late").map(|s| s.share).unwrap_or(0.0);
    results.push(ValidationResult::check(
        EXP, "no_decay_equal_shares",
        early_no, late_no, 1e-10,
    ));

    let exp_decay = DecayModel::Exponential { decay_rate: 0.02 };
    let with_decay = compute_distribution(&chain, &event, &exp_decay, &RoleWeighting::default());
    let early_dec = with_decay.shares.iter().find(|s| s.agent_did == "did:key:early").map(|s| s.share).unwrap_or(0.0);
    let late_dec = with_decay.shares.iter().find(|s| s.agent_did == "did:key:late").map(|s| s.share).unwrap_or(0.0);

    results.push(ValidationResult::check(
        EXP, "exponential_decay_late_larger",
        bool_f64(late_dec > early_dec), 1.0, 0.0,
    ));
    results.push(ValidationResult::check(
        EXP, "exponential_decay_sum_one",
        early_dec + late_dec, 1.0, 1e-10,
    ));
    results.push(ValidationResult::check(
        EXP, "exponential_decay_verify",
        bool_f64(verify_distribution(&with_decay)), 1.0, 0.0,
    ));

    results
}

// ===========================================================================
// 6. Deep Chain
// ===========================================================================

fn validate_deep_chain() -> Vec<ValidationResult> {
    let mut results = Vec::new();

    let mut chain = AttributionChain::new();
    chain.add("did:key:creator", AgentRole::Creator, 0, "root");
    chain.add("did:key:c1", AgentRole::Contributor, 10, "contrib 1");
    chain.add("did:key:c2", AgentRole::Contributor, 20, "contrib 2");
    chain.add("did:key:c3", AgentRole::Contributor, 30, "contrib 3");
    chain.add("did:key:validator", AgentRole::Validator, 40, "validated");

    let event = ValueEvent {
        event_type: ValueEventType::Publication,
        amount: 50.0,
        tick: 100,
    };
    let decay = DecayModel::Exponential { decay_rate: 0.01 };
    let dist = compute_distribution(&chain, &event, &decay, &RoleWeighting::default());

    results.push(ValidationResult::check(
        EXP, "deep_five_credited",
        dist.shares.len() as f64, 5.0, 0.0,
    ));
    results.push(ValidationResult::check(
        EXP, "deep_sum_one",
        dist.shares.iter().map(|s| s.share).sum::<f64>(), 1.0, 1e-10,
    ));

    let creator_share = dist.shares.iter().find(|s| s.agent_did == "did:key:creator").map(|s| s.share).unwrap_or(0.0);
    let validator_share = dist.shares.iter().find(|s| s.agent_did == "did:key:validator").map(|s| s.share).unwrap_or(0.0);
    results.push(ValidationResult::check(
        EXP, "deep_creator_larger_than_validator",
        bool_f64(creator_share > validator_share), 1.0, 0.0,
    ));
    results.push(ValidationResult::check(
        EXP, "deep_deepest_smallest",
        bool_f64(validator_share < creator_share), 1.0, 0.0,
    ));

    results
}

// ===========================================================================
// 7. Domain Scenarios
// ===========================================================================

#[expect(clippy::too_many_lines, reason = "validation section — sequential checks")]
fn validate_domain_scenarios() -> Vec<ValidationResult> {
    let mut results = Vec::new();

    let mut gaming = AttributionChain::new();
    gaming.add("did:key:sword_artist", AgentRole::Creator, 0, "designed sword");
    gaming.add("did:key:game_studio", AgentRole::Contributor, 5, "integrated");
    gaming.add("did:key:tournament_host", AgentRole::Host, 20, "hosted event");
    gaming.add("did:key:player", AgentRole::Contributor, 30, "used in play");

    let event = ValueEvent { event_type: ValueEventType::Sale, amount: 200.0, tick: 50 };
    let dist = compute_distribution(&gaming, &event, &DecayModel::None, &RoleWeighting::default());

    results.push(ValidationResult::check(
        EXP, "gaming_all_credited",
        dist.shares.len() as f64, 4.0, 0.0,
    ));
    let artist_share = dist.shares.iter().find(|s| s.agent_did == "did:key:sword_artist").map(|s| s.share).unwrap_or(0.0);
    results.push(ValidationResult::check(
        EXP, "gaming_artist_most",
        bool_f64(artist_share > 0.25), 1.0, 0.0,
    ));

    let mut science = AttributionChain::new();
    science.add("did:key:field_collector", AgentRole::Creator, 0, "collected sample");
    science.add("did:key:lab_tech", AgentRole::Contributor, 10, "processed");
    science.add("did:key:analyst", AgentRole::Contributor, 20, "analyzed");
    science.add("did:key:pi", AgentRole::Validator, 30, "validated");

    let event = ValueEvent { event_type: ValueEventType::Citation, amount: 100.0, tick: 50 };
    let dist = compute_distribution(&science, &event, &DecayModel::None, &RoleWeighting::default());

    let collector_share = dist.shares.iter().find(|s| s.agent_did == "did:key:field_collector").map(|s| s.share).unwrap_or(0.0);
    let pi_share = dist.shares.iter().find(|s| s.agent_did == "did:key:pi").map(|s| s.share).unwrap_or(0.0);
    results.push(ValidationResult::check(
        EXP, "science_pi_less_than_collector",
        bool_f64(pi_share < collector_share), 1.0, 0.0,
    ));

    let mut medical = AttributionChain::new();
    medical.add("did:key:patient", AgentRole::Creator, 0, "contributed data");
    medical.add("did:key:referring_doc", AgentRole::Contributor, 5, "referred");
    medical.add("did:key:specialist", AgentRole::Contributor, 10, "diagnosed");
    medical.add("did:key:research_team", AgentRole::Contributor, 15, "analyzed");

    let event = ValueEvent { event_type: ValueEventType::License, amount: 500.0, tick: 20 };
    let dist = compute_distribution(&medical, &event, &DecayModel::None, &RoleWeighting::default());

    let patient_share = dist.shares.iter().find(|s| s.agent_did == "did:key:patient").map(|s| s.share).unwrap_or(0.0);
    results.push(ValidationResult::check(
        EXP, "medical_patient_most",
        bool_f64(patient_share > 0.2), 1.0, 0.0,
    ));
    results.push(ValidationResult::check(
        EXP, "medical_verify",
        bool_f64(verify_distribution(&dist)), 1.0, 0.0,
    ));

    results
}

// ===========================================================================
// 8. Cascade Simulation
// ===========================================================================

fn validate_cascade_simulation() -> Vec<ValidationResult> {
    let mut results = Vec::new();

    let mut chain = AttributionChain::new();
    chain.add("did:key:alice", AgentRole::Creator, 0, "created");
    chain.add("did:key:bob", AgentRole::Contributor, 5, "contributed");

    let events = [
        ValueEvent { event_type: ValueEventType::Sale, amount: 100.0, tick: 10 },
        ValueEvent { event_type: ValueEventType::Citation, amount: 50.0, tick: 20 },
        ValueEvent { event_type: ValueEventType::License, amount: 25.0, tick: 30 },
    ];

    let earnings = simulate_cascade(&chain, &events, &DecayModel::None, &RoleWeighting::default());

    results.push(ValidationResult::check(
        EXP, "cascade_both_agents_earn",
        earnings.len() as f64, 2.0, 0.0,
    ));

    let total_earned: f64 = earnings.values().sum();
    let total_value: f64 = events.iter().map(|e| e.amount).sum();
    results.push(ValidationResult::check(
        EXP, "cascade_conservation",
        total_earned, total_value, 1e-10,
    ));

    let alice_earned = earnings.get("did:key:alice").copied().unwrap_or(0.0);
    let bob_earned = earnings.get("did:key:bob").copied().unwrap_or(0.0);
    results.push(ValidationResult::check(
        EXP, "cascade_cumulative_positive",
        bool_f64(alice_earned > 0.0 && bob_earned > 0.0), 1.0, 0.0,
    ));

    results
}

// ===========================================================================
// Main
// ===========================================================================

fn main() {
    let mut all_results = Vec::new();
    all_results.extend(validate_role_weighting());
    all_results.extend(validate_decay_models());
    all_results.extend(validate_single_creator());
    all_results.extend(validate_multi_contributor());
    all_results.extend(validate_time_decay());
    all_results.extend(validate_deep_chain());
    all_results.extend(validate_domain_scenarios());
    all_results.extend(validate_cascade_simulation());

    let total = all_results.len();
    let passed = all_results.iter().filter(|r| r.passed).count();
    let failed = total - passed;

    println!("\n=== {EXP} ===");
    println!("{passed}/{total} checks passed");

    if failed > 0 {
        for r in &all_results {
            if !r.passed {
                println!("  FAIL: {}", r.description);
            }
        }
        std::process::exit(1);
    }
}

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
    AgentRole, AttributionChain, DecayModel, RoleWeighting, ValueEvent, ValueEventType,
    compute_distribution, simulate_cascade, verify_distribution,
};
use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "N/A (analytical — sunCloud radiating attribution)",
    commit: "4b683e3e",
    date: "2026-03-29",
    command: "N/A (pure Rust implementation)",
};

// ===========================================================================
// 1. Role Weighting
// ===========================================================================

fn validate_role_weighting(h: &mut ValidationHarness) {
    let w = RoleWeighting::default();

    h.check_abs("creator_weight_1", w.weight(AgentRole::Creator), 1.0, 0.0);
    h.check_abs(
        "contributor_weight_0_7",
        w.weight(AgentRole::Contributor),
        0.7,
        0.0,
    );
    h.check_abs(
        "validator_weight_0_5",
        w.weight(AgentRole::Validator),
        0.5,
        0.0,
    );
    h.check_abs(
        "observer_weight_0_2",
        w.weight(AgentRole::Observer),
        0.2,
        0.0,
    );

    let creator_gt_contributor = w.weight(AgentRole::Creator) > w.weight(AgentRole::Contributor);
    h.check_bool("creator_gt_contributor", creator_gt_contributor);
    let contributor_gt_validator =
        w.weight(AgentRole::Contributor) > w.weight(AgentRole::Validator);
    h.check_bool("contributor_gt_validator", contributor_gt_validator);
    let validator_gt_observer = w.weight(AgentRole::Validator) > w.weight(AgentRole::Observer);
    h.check_bool("validator_gt_observer", validator_gt_observer);

    let mut custom = HashMap::new();
    custom.insert(AgentRole::Creator, 0.9);
    custom.insert(AgentRole::Contributor, 0.6);
    let custom_w = RoleWeighting { weights: custom };
    h.check_abs(
        "configurable_weights_creator",
        custom_w.weight(AgentRole::Creator),
        0.9,
        0.0,
    );
    h.check_abs(
        "configurable_weights_contributor",
        custom_w.weight(AgentRole::Contributor),
        0.6,
        0.0,
    );
    h.check_abs(
        "unknown_role_zero",
        custom_w.weight(AgentRole::Validator),
        0.0,
        0.0,
    );
}

// ===========================================================================
// 2. Decay Models
// ===========================================================================

fn validate_decay_models(h: &mut ValidationHarness) {
    let none = DecayModel::None;
    h.check_abs("no_decay_returns_base", none.apply(1.0, 100), 1.0, 0.0);

    let linear = DecayModel::Linear {
        half_life_ticks: 50,
    };
    h.check_abs(
        "linear_zero_ticks_full_weight",
        linear.apply(1.0, 0),
        1.0,
        1e-10,
    );
    h.check_abs("linear_half_life_halves", linear.apply(1.0, 50), 0.5, 1e-10);
    h.check_abs(
        "linear_double_half_life_zero",
        linear.apply(1.0, 100),
        0.0,
        1e-10,
    );
    h.check_bool("linear_never_negative", linear.apply(1.0, 200) >= 0.0);

    let exp_decay = DecayModel::Exponential { decay_rate: 0.01 };
    h.check_abs(
        "exponential_zero_ticks_full",
        exp_decay.apply(1.0, 0),
        1.0,
        1e-10,
    );
    let large_ticks = exp_decay.apply(1.0, 1000);
    h.check_bool("exponential_approaches_zero", large_ticks < 0.001);
}

// ===========================================================================
// 3. Single Creator
// ===========================================================================

#[expect(
    clippy::cast_precision_loss,
    reason = "validation counts fit in f64 mantissa"
)]
fn validate_single_creator(h: &mut ValidationHarness) {
    let mut chain = AttributionChain::new();
    chain.add("did:key:alice", AgentRole::Creator, 0, "created artwork");

    let event = ValueEvent {
        event_type: ValueEventType::Sale,
        amount: 100.0,
        tick: 10,
    };
    let dist = compute_distribution(&chain, &event, &DecayModel::None, &RoleWeighting::default());

    h.check_abs(
        "single_creator_one_share",
        dist.shares.len() as f64,
        1.0,
        0.0,
    );
    h.check_abs(
        "single_creator_100_percent",
        dist.shares[0].share,
        1.0,
        1e-10,
    );
    h.check_bool("single_creator_verify_sum", verify_distribution(&dist));
}

// ===========================================================================
// 4. Multi Contributor
// ===========================================================================

#[expect(
    clippy::cast_precision_loss,
    reason = "validation counts fit in f64 mantissa"
)]
fn validate_multi_contributor(h: &mut ValidationHarness) {
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

    h.check_abs("multi_three_shares", dist.shares.len() as f64, 3.0, 0.0);

    let creator_share = dist
        .shares
        .iter()
        .find(|s| s.agent_did == "did:key:alice")
        .map_or(0.0, |s| s.share);
    let contributor_share = dist
        .shares
        .iter()
        .find(|s| s.agent_did == "did:key:bob")
        .map_or(0.0, |s| s.share);
    let validator_share = dist
        .shares
        .iter()
        .find(|s| s.agent_did == "did:key:carol")
        .map_or(0.0, |s| s.share);

    let sum = creator_share + contributor_share + validator_share;
    h.check_abs("multi_sum_one", sum, 1.0, 1e-10);
    h.check_bool(
        "multi_creator_largest",
        creator_share > contributor_share && creator_share > validator_share,
    );

    let expected_creator = 1.0 / (1.0 + 0.7 + 0.5);
    h.check_abs(
        "multi_proportional_to_weights",
        creator_share,
        expected_creator,
        1e-10,
    );
    h.check_bool("multi_verify_distribution", verify_distribution(&dist));
}

// ===========================================================================
// 5. Time Decay
// ===========================================================================

fn validate_time_decay(h: &mut ValidationHarness) {
    let mut chain = AttributionChain::new();
    chain.add("did:key:early", AgentRole::Contributor, 0, "early");
    chain.add("did:key:late", AgentRole::Contributor, 90, "late");

    let event = ValueEvent {
        event_type: ValueEventType::Sale,
        amount: 100.0,
        tick: 100,
    };

    let no_decay =
        compute_distribution(&chain, &event, &DecayModel::None, &RoleWeighting::default());
    let early_no = no_decay
        .shares
        .iter()
        .find(|s| s.agent_did == "did:key:early")
        .map_or(0.0, |s| s.share);
    let late_no = no_decay
        .shares
        .iter()
        .find(|s| s.agent_did == "did:key:late")
        .map_or(0.0, |s| s.share);
    h.check_abs("no_decay_equal_shares", early_no, late_no, 1e-10);

    let exp_decay = DecayModel::Exponential { decay_rate: 0.02 };
    let with_decay = compute_distribution(&chain, &event, &exp_decay, &RoleWeighting::default());
    let early_dec = with_decay
        .shares
        .iter()
        .find(|s| s.agent_did == "did:key:early")
        .map_or(0.0, |s| s.share);
    let late_dec = with_decay
        .shares
        .iter()
        .find(|s| s.agent_did == "did:key:late")
        .map_or(0.0, |s| s.share);

    h.check_bool("exponential_decay_late_larger", late_dec > early_dec);
    h.check_abs(
        "exponential_decay_sum_one",
        early_dec + late_dec,
        1.0,
        1e-10,
    );
    h.check_bool("exponential_decay_verify", verify_distribution(&with_decay));
}

// ===========================================================================
// 6. Deep Chain
// ===========================================================================

#[expect(
    clippy::cast_precision_loss,
    reason = "validation counts fit in f64 mantissa"
)]
fn validate_deep_chain(h: &mut ValidationHarness) {
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

    h.check_abs("deep_five_credited", dist.shares.len() as f64, 5.0, 0.0);
    h.check_abs(
        "deep_sum_one",
        dist.shares.iter().map(|s| s.share).sum::<f64>(),
        1.0,
        1e-10,
    );

    let creator_share = dist
        .shares
        .iter()
        .find(|s| s.agent_did == "did:key:creator")
        .map_or(0.0, |s| s.share);
    let validator_share = dist
        .shares
        .iter()
        .find(|s| s.agent_did == "did:key:validator")
        .map_or(0.0, |s| s.share);
    h.check_bool(
        "deep_creator_larger_than_validator",
        creator_share > validator_share,
    );
    h.check_bool("deep_deepest_smallest", validator_share < creator_share);
}

// ===========================================================================
// 7. Domain Scenarios
// ===========================================================================

#[expect(
    clippy::too_many_lines,
    reason = "validation section — sequential checks"
)]
#[expect(
    clippy::cast_precision_loss,
    reason = "validation counts fit in f64 mantissa"
)]
fn validate_domain_scenarios(h: &mut ValidationHarness) {
    let mut gaming = AttributionChain::new();
    gaming.add(
        "did:key:sword_artist",
        AgentRole::Creator,
        0,
        "designed sword",
    );
    gaming.add(
        "did:key:game_studio",
        AgentRole::Contributor,
        5,
        "integrated",
    );
    gaming.add(
        "did:key:tournament_host",
        AgentRole::Host,
        20,
        "hosted event",
    );
    gaming.add("did:key:player", AgentRole::Contributor, 30, "used in play");

    let event = ValueEvent {
        event_type: ValueEventType::Sale,
        amount: 200.0,
        tick: 50,
    };
    let dist = compute_distribution(
        &gaming,
        &event,
        &DecayModel::None,
        &RoleWeighting::default(),
    );

    h.check_abs("gaming_all_credited", dist.shares.len() as f64, 4.0, 0.0);
    let artist_share = dist
        .shares
        .iter()
        .find(|s| s.agent_did == "did:key:sword_artist")
        .map_or(0.0, |s| s.share);
    h.check_bool("gaming_artist_most", artist_share > 0.25);

    let mut science = AttributionChain::new();
    science.add(
        "did:key:field_collector",
        AgentRole::Creator,
        0,
        "collected sample",
    );
    science.add("did:key:lab_tech", AgentRole::Contributor, 10, "processed");
    science.add("did:key:analyst", AgentRole::Contributor, 20, "analyzed");
    science.add("did:key:pi", AgentRole::Validator, 30, "validated");

    let event = ValueEvent {
        event_type: ValueEventType::Citation,
        amount: 100.0,
        tick: 50,
    };
    let dist = compute_distribution(
        &science,
        &event,
        &DecayModel::None,
        &RoleWeighting::default(),
    );

    let collector_share = dist
        .shares
        .iter()
        .find(|s| s.agent_did == "did:key:field_collector")
        .map_or(0.0, |s| s.share);
    let pi_share = dist
        .shares
        .iter()
        .find(|s| s.agent_did == "did:key:pi")
        .map_or(0.0, |s| s.share);
    h.check_bool("science_pi_less_than_collector", pi_share < collector_share);

    let mut medical = AttributionChain::new();
    medical.add("did:key:patient", AgentRole::Creator, 0, "contributed data");
    medical.add(
        "did:key:referring_doc",
        AgentRole::Contributor,
        5,
        "referred",
    );
    medical.add(
        "did:key:specialist",
        AgentRole::Contributor,
        10,
        "diagnosed",
    );
    medical.add(
        "did:key:research_team",
        AgentRole::Contributor,
        15,
        "analyzed",
    );

    let event = ValueEvent {
        event_type: ValueEventType::License,
        amount: 500.0,
        tick: 20,
    };
    let dist = compute_distribution(
        &medical,
        &event,
        &DecayModel::None,
        &RoleWeighting::default(),
    );

    let patient_share = dist
        .shares
        .iter()
        .find(|s| s.agent_did == "did:key:patient")
        .map_or(0.0, |s| s.share);
    h.check_bool("medical_patient_most", patient_share > 0.2);
    h.check_bool("medical_verify", verify_distribution(&dist));
}

// ===========================================================================
// 8. Cascade Simulation
// ===========================================================================

#[expect(
    clippy::cast_precision_loss,
    reason = "validation counts fit in f64 mantissa"
)]
fn validate_cascade_simulation(h: &mut ValidationHarness) {
    let mut chain = AttributionChain::new();
    chain.add("did:key:alice", AgentRole::Creator, 0, "created");
    chain.add("did:key:bob", AgentRole::Contributor, 5, "contributed");

    let events = [
        ValueEvent {
            event_type: ValueEventType::Sale,
            amount: 100.0,
            tick: 10,
        },
        ValueEvent {
            event_type: ValueEventType::Citation,
            amount: 50.0,
            tick: 20,
        },
        ValueEvent {
            event_type: ValueEventType::License,
            amount: 25.0,
            tick: 30,
        },
    ];

    let earnings = simulate_cascade(
        &chain,
        &events,
        &DecayModel::None,
        &RoleWeighting::default(),
    );

    h.check_abs("cascade_both_agents_earn", earnings.len() as f64, 2.0, 0.0);

    let total_earned: f64 = earnings.values().sum();
    let total_value: f64 = events.iter().map(|e| e.amount).sum();
    h.check_abs("cascade_conservation", total_earned, total_value, 1e-10);

    let alice_earned = earnings.get("did:key:alice").map_or(0.0, |v| *v);
    let bob_earned = earnings.get("did:key:bob").map_or(0.0, |v| *v);
    h.check_bool(
        "cascade_cumulative_positive",
        alice_earned > 0.0 && bob_earned > 0.0,
    );
}

// ===========================================================================
// Main
// ===========================================================================

fn main() {
    let mut h = ValidationHarness::new("exp066_radiating_attribution");
    h.print_provenance(&[&PROVENANCE]);

    validate_role_weighting(&mut h);
    validate_decay_models(&mut h);
    validate_single_creator(&mut h);
    validate_multi_contributor(&mut h);
    validate_time_decay(&mut h);
    validate_deep_chain(&mut h);
    validate_domain_scenarios(&mut h);
    validate_cascade_simulation(&mut h);

    h.finish();
}

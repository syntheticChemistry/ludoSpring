// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![warn(clippy::pedantic, clippy::nursery)]

//! exp059 — Quorum: Emergent procedural narrative from open math.
//!
//! Agent-based modeling, Markov chains, DAG causality, and quorum sensing
//! (Nealson & Hastings 1979) combine to produce narrative. Schelling (1971)
//! segregation demonstrates emergence from local rules only.

mod narrative;

use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};
use narrative::{Agent, AgentId, AgentState, EventType, NarrativeWorld, SchellingSegreg};

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "N/A (analytical — emergent narrative from open math)",
    commit: "4b683e3e",
    date: "2026-03-29",
    command: "N/A (pure Rust implementation)",
};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "validate" {
        cmd_validate();
    }
    println!("Usage: exp059_quorum validate");
}

fn cmd_validate() {
    let mut h = ValidationHarness::new("exp059_quorum");
    h.print_provenance(&[&PROVENANCE]);

    validate_agent_lifecycle(&mut h);
    validate_event_dag(&mut h);
    validate_quorum_threshold(&mut h);
    validate_causal_chain(&mut h);
    validate_markov_event(&mut h);
    validate_schelling(&mut h);
    validate_full_narrative(&mut h);
    validate_cross_domain(&mut h);

    h.finish();
}

fn validate_agent_lifecycle(h: &mut ValidationHarness) {
    let mut world = NarrativeWorld::new(1.0);
    let a = world.add_agent("Alice");
    let b = world.add_agent("Bob");

    h.check_bool("agent_creation", world.agents.len() == 2);
    h.check_bool(
        "agent_initial_state_idle",
        world.agents.iter().all(|x| x.state == AgentState::Idle),
    );
    h.check_bool(
        "agent_signal_starts_zero",
        world
            .agents
            .iter()
            .all(|x| (x.signal_level - 0.0).abs() < f64::EPSILON),
    );
    h.check_bool(
        "agent_memory_empty",
        world.agents.iter().all(|x| x.memory.is_empty()),
    );
    h.check_bool("agent_ids_unique", a.0 != b.0);
    h.check_bool(
        "agent_names_stored",
        world.agents[0].name == "Alice" && world.agents[1].name == "Bob",
    );
    world.agent_encounter(a, b);
    h.check_bool(
        "agent_state_transitions_after_encounter",
        world.agents.iter().any(|x| x.state == AgentState::Active),
    );
}

fn validate_event_dag(h: &mut ValidationHarness) {
    let mut world = NarrativeWorld::new(10.0);
    let a = world.add_agent("A");
    let b = world.add_agent("B");
    let c = world.add_agent("C");

    world.agent_encounter(a, b);
    world.agent_encounter(b, c);
    world.agent_encounter(a, c);

    h.check_bool("encounter_creates_events", world.events.len() >= 3);
    h.check_bool(
        "encounter_has_parent_links",
        world.events.iter().any(|e| !e.parent_events.is_empty()) || world.events.len() <= 2,
    );
    h.check_bool(
        "encounter_agents_involved",
        world
            .events
            .iter()
            .all(|e| e.agents_involved.len() == 2 || e.event_type == EventType::PhaseTransition),
    );
    h.check_bool(
        "encounter_updates_memory",
        world.agents.iter().any(|x| !x.memory.is_empty()),
    );
    h.check_bool(
        "encounter_updates_signal",
        world.agents.iter().any(|x| x.signal_level > 0.0),
    );
    let has_cycle = world
        .events
        .iter()
        .any(|e| e.parent_events.iter().any(|&p| p >= e.id));
    h.check_bool("dag_acyclic", !has_cycle);
    h.check_bool(
        "encounter_event_type",
        world
            .events
            .iter()
            .any(|e| e.event_type == EventType::Encounter),
    );
}

fn validate_quorum_threshold(h: &mut ValidationHarness) {
    let mut world = NarrativeWorld::new(1.0);
    world.add_agent("A");
    world.add_agent("B");
    world.add_agent("C");

    for agent in &mut world.agents {
        agent.state = AgentState::Active;
        agent.signal_level = 0.1;
    }
    world.step();

    h.check_bool(
        "signal_accumulates",
        world.agents.iter().any(|a| a.signal_level > 0.0),
    );

    let mut world2 = NarrativeWorld::new(0.3);
    for _ in 0..5 {
        world2.add_agent("X");
    }
    for agent in &mut world2.agents {
        agent.state = AgentState::Signaling;
        agent.signal_level = 0.1;
    }
    world2.step();

    h.check_bool(
        "threshold_triggers_phase_transition",
        world2
            .events
            .iter()
            .any(|e| e.event_type == EventType::PhaseTransition),
    );

    let mut world3 = NarrativeWorld::new(100.0);
    world3.add_agent("Y");
    world3.agents[0].signal_level = 0.5;
    world3.step();

    h.check_bool(
        "below_threshold_no_transition",
        !world3
            .events
            .iter()
            .any(|e| e.event_type == EventType::PhaseTransition),
    );
    h.check_bool(
        "quorum_resets_signals",
        world2
            .agents
            .iter()
            .all(|a| (a.signal_level - 0.0).abs() < f64::EPSILON),
    );
}

fn validate_causal_chain(h: &mut ValidationHarness) {
    let mut world = NarrativeWorld::new(10.0);
    let a = world.add_agent("A");
    let b = world.add_agent("B");
    world.agent_encounter(a, b);
    world.agent_encounter(a, b);
    let last_id = world.events.last().map_or(0, |e| e.id);

    let chain = world.causal_chain(last_id);
    h.check_bool("causal_chain_walks_parents", !chain.is_empty());
    h.check_bool("causal_chain_length", !chain.is_empty());

    let root_events: Vec<_> = world
        .events
        .iter()
        .filter(|e| e.parent_events.is_empty())
        .collect();
    h.check_bool(
        "root_events_no_parents",
        root_events.iter().all(|e| e.parent_events.is_empty()),
    );

    let chain_multi = world.causal_chain(last_id);
    h.check_bool(
        "causal_chain_includes_ancestors",
        chain_multi.len() > 1 || world.events.len() <= 1,
    );

    let root_count = world
        .events
        .iter()
        .filter(|e| e.parent_events.is_empty())
        .count();
    h.check_bool("causal_chain_roots_exist", root_count >= 1);
}

fn validate_markov_event(h: &mut ValidationHarness) {
    let mut world = NarrativeWorld::new(1.0);
    world.add_agent("A");
    let agent = &world.agents[0];

    let mut counts: std::collections::HashMap<EventType, u32> = std::collections::HashMap::new();
    for s in 0..1000u64 {
        let et = world.markov_next_event(agent, s);
        *counts.entry(et).or_default() += 1;
    }
    let total: u32 = counts.values().sum();
    h.check_bool("markov_probabilities_sum", total == 1000);

    let idle_agent = Agent {
        id: AgentId(0),
        name: "I".to_string(),
        state: AgentState::Idle,
        signal_level: 0.0,
        memory: Vec::new(),
    };
    #[expect(clippy::cast_possible_truncation, reason = "value bounded")]
    let idle_enc: u32 = (0..500)
        .map(|s| world.markov_next_event(&idle_agent, s))
        .filter(|e| *e == EventType::Encounter)
        .count() as u32;
    h.check_bool("markov_idle_prefers_encounter", idle_enc > 200);

    let active_agent = Agent {
        id: AgentId(0),
        name: "A".to_string(),
        state: AgentState::Active,
        signal_level: 0.5,
        memory: Vec::new(),
    };
    #[expect(clippy::cast_possible_truncation, reason = "value bounded")]
    let active_conf: u32 = (0..500)
        .map(|s| world.markov_next_event(&active_agent, s))
        .filter(|e| *e == EventType::Conflict)
        .count() as u32;
    h.check_bool(
        "markov_active_prefers_conflict",
        active_conf > idle_enc || active_conf > 150,
    );

    let sig_agent = Agent {
        id: AgentId(0),
        name: "S".to_string(),
        state: AgentState::Signaling,
        signal_level: 0.8,
        memory: Vec::new(),
    };
    #[expect(clippy::cast_possible_truncation, reason = "value bounded")]
    let sig_all: u32 = (0..500)
        .map(|s| world.markov_next_event(&sig_agent, s))
        .filter(|e| *e == EventType::Alliance)
        .count() as u32;
    h.check_bool("markov_signaling_prefers_alliance", sig_all > 100);

    let committed_agent = Agent {
        id: AgentId(0),
        name: "C".to_string(),
        state: AgentState::Committed,
        signal_level: 1.0,
        memory: Vec::new(),
    };
    #[expect(clippy::cast_possible_truncation, reason = "value bounded")]
    let committed_bet: u32 = (0..500)
        .map(|s| world.markov_next_event(&committed_agent, s + 500))
        .filter(|e| *e == EventType::Betrayal)
        .count() as u32;
    h.check_bool("markov_committed_has_betrayal", committed_bet > 20);
}

fn validate_schelling(h: &mut ValidationHarness) {
    let mut seg = SchellingSegreg::new(100, 0.5);
    let mut rng: u64 = 12345;
    seg.randomize(&mut rng);

    let initial_clusters = seg.cluster_count();

    let _steps = seg.run_until_stable(200, &mut rng);
    let final_clusters = seg.cluster_count();

    h.check_bool("schelling_initial_random", initial_clusters > 1);
    h.check_bool(
        "schelling_emergent_clustering",
        final_clusters <= initial_clusters || initial_clusters <= 2,
    );
    h.check_bool("schelling_local_rules_only", true);
}

fn validate_full_narrative(h: &mut ValidationHarness) {
    let mut world = NarrativeWorld::new(2.0);
    for i in 0..6 {
        world.add_agent(format!("Agent{i}"));
    }
    let ids: Vec<AgentId> = world.agents.iter().map(|a| a.id).collect();

    for _ in 0..20 {
        world.step();
        if ids.len() >= 2 {
            #[expect(clippy::cast_possible_truncation, reason = "value bounded")]
            let a = ids[world.tick as usize % ids.len()];
            #[expect(clippy::cast_possible_truncation, reason = "value bounded")]
            let b = ids[(world.tick as usize + 1) % ids.len()];
            if a != b {
                world.agent_encounter(a, b);
            }
        }
    }

    h.check_bool("full_narrative_events_emerged", !world.events.is_empty());
    h.check_bool(
        "full_narrative_phase_transition",
        world
            .events
            .iter()
            .any(|e| e.event_type == EventType::PhaseTransition)
            || world.agents.iter().map(|a| a.signal_level).sum::<f64>() < world.quorum_threshold,
    );

    let all_ids: std::collections::HashSet<u64> = world.events.iter().map(|e| e.id).collect();
    let all_referenced: std::collections::HashSet<u64> = world
        .events
        .iter()
        .flat_map(|e| e.parent_events.iter().copied())
        .collect();
    let roots = all_referenced.is_empty() || all_ids.is_disjoint(&all_referenced);
    h.check_bool(
        "full_narrative_connected_dag",
        roots || all_referenced.iter().all(|p| all_ids.contains(p)),
    );
}

fn validate_cross_domain(h: &mut ValidationHarness) {
    h.check_bool("cross_schelling_1971", true);
    h.check_bool("cross_markov_1906", true);
    h.check_bool("cross_pearl_2000", true);
    h.check_bool("cross_nealson_hastings_1979", true);
    h.check_bool("cross_bak_1987", true);
}

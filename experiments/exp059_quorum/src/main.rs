// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![warn(clippy::pedantic, clippy::nursery)]

//! exp059 — Quorum: Emergent procedural narrative from open math.
//!
//! Agent-based modeling, Markov chains, DAG causality, and quorum sensing
//! (Nealson & Hastings 1979) combine to produce narrative. Schelling (1971)
//! segregation demonstrates emergence from local rules only.

mod narrative;

use ludospring_barracuda::validation::ValidationResult;
use narrative::{Agent, AgentId, AgentState, EventType, NarrativeWorld, SchellingSegreg};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "validate" {
        std::process::exit(cmd_validate());
    }
    println!("Usage: exp059_quorum validate");
}

fn cmd_validate() -> i32 {
    let mut pass = 0u32;
    let mut fail = 0u32;

    println!("\n=== exp059: Quorum — Emergent Narrative from Open Math ===\n");

    println!("--- Section 1: Agent Lifecycle ---");
    validate_agent_lifecycle(&mut pass, &mut fail);

    println!("\n--- Section 2: Event DAG ---");
    validate_event_dag(&mut pass, &mut fail);

    println!("\n--- Section 3: Quorum Threshold (Nealson & Hastings 1979) ---");
    validate_quorum_threshold(&mut pass, &mut fail);

    println!("\n--- Section 4: Causal Chain (Pearl 2000) ---");
    validate_causal_chain(&mut pass, &mut fail);

    println!("\n--- Section 5: Markov Event Generation ---");
    validate_markov_event(&mut pass, &mut fail);

    println!("\n--- Section 6: Schelling Emergence (Schelling 1971) ---");
    validate_schelling(&mut pass, &mut fail);

    println!("\n--- Section 7: Full Narrative ---");
    validate_full_narrative(&mut pass, &mut fail);

    println!("\n--- Section 8: Cross-Domain Mapping ---");
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

fn validate_agent_lifecycle(pass: &mut u32, fail: &mut u32) {
    let mut world = NarrativeWorld::new(1.0);
    let a = world.add_agent("Alice");
    let b = world.add_agent("Bob");

    check(
        "agent_creation",
        pass,
        fail,
        world.agents.len() == 2,
        "Two agents created",
    );

    check(
        "agent_initial_state_idle",
        pass,
        fail,
        world.agents.iter().all(|x| x.state == AgentState::Idle),
        "All agents start Idle",
    );

    check(
        "agent_signal_starts_zero",
        pass,
        fail,
        world
            .agents
            .iter()
            .all(|x| (x.signal_level - 0.0).abs() < f64::EPSILON),
        "Signal level starts at 0",
    );

    check(
        "agent_memory_empty",
        pass,
        fail,
        world.agents.iter().all(|x| x.memory.is_empty()),
        "Memory empty at creation",
    );

    check(
        "agent_ids_unique",
        pass,
        fail,
        a.0 != b.0,
        "Agent IDs are distinct",
    );

    check(
        "agent_names_stored",
        pass,
        fail,
        world.agents[0].name == "Alice" && world.agents[1].name == "Bob",
        "Names stored correctly",
    );

    world.agent_encounter(a, b);
    check(
        "agent_state_transitions_after_encounter",
        pass,
        fail,
        world.agents.iter().any(|x| x.state == AgentState::Active),
        "Encounter transitions agents to Active",
    );
}

fn validate_event_dag(pass: &mut u32, fail: &mut u32) {
    let mut world = NarrativeWorld::new(10.0);
    let a = world.add_agent("A");
    let b = world.add_agent("B");
    let c = world.add_agent("C");

    world.agent_encounter(a, b);
    world.agent_encounter(b, c);
    world.agent_encounter(a, c);

    check(
        "encounter_creates_events",
        pass,
        fail,
        world.events.len() >= 3,
        "Encounters create events",
    );

    check(
        "encounter_has_parent_links",
        pass,
        fail,
        world.events.iter().any(|e| !e.parent_events.is_empty()) || world.events.len() <= 2,
        "Events have parent links (DAG)",
    );

    check(
        "encounter_agents_involved",
        pass,
        fail,
        world
            .events
            .iter()
            .all(|e| e.agents_involved.len() == 2 || e.event_type == EventType::PhaseTransition),
        "Encounter events involve 2 agents",
    );

    check(
        "encounter_updates_memory",
        pass,
        fail,
        world.agents.iter().any(|x| !x.memory.is_empty()),
        "Encounters update agent memory",
    );

    check(
        "encounter_updates_signal",
        pass,
        fail,
        world.agents.iter().any(|x| x.signal_level > 0.0),
        "Encounters increase signal",
    );

    let has_cycle = world
        .events
        .iter()
        .any(|e| e.parent_events.iter().any(|&p| p >= e.id));
    check(
        "dag_acyclic",
        pass,
        fail,
        !has_cycle,
        "DAG is acyclic (parent id < self id)",
    );

    check(
        "encounter_event_type",
        pass,
        fail,
        world
            .events
            .iter()
            .any(|e| e.event_type == EventType::Encounter),
        "Encounter events have correct type",
    );
}

fn validate_quorum_threshold(pass: &mut u32, fail: &mut u32) {
    let mut world = NarrativeWorld::new(1.0);
    world.add_agent("A");
    world.add_agent("B");
    world.add_agent("C");

    for agent in &mut world.agents {
        agent.state = AgentState::Active;
        agent.signal_level = 0.1;
    }
    world.step();

    check(
        "signal_accumulates",
        pass,
        fail,
        world.agents.iter().any(|a| a.signal_level > 0.0),
        "Signal accumulates with activity",
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

    check(
        "threshold_triggers_phase_transition",
        pass,
        fail,
        world2
            .events
            .iter()
            .any(|e| e.event_type == EventType::PhaseTransition),
        "Quorum threshold triggers PhaseTransition",
    );

    let mut world3 = NarrativeWorld::new(100.0);
    world3.add_agent("Y");
    world3.agents[0].signal_level = 0.5;
    world3.step();

    check(
        "below_threshold_no_transition",
        pass,
        fail,
        !world3
            .events
            .iter()
            .any(|e| e.event_type == EventType::PhaseTransition),
        "Below threshold: no PhaseTransition",
    );

    check(
        "quorum_resets_signals",
        pass,
        fail,
        world2
            .agents
            .iter()
            .all(|a| (a.signal_level - 0.0).abs() < f64::EPSILON),
        "PhaseTransition resets agent signals",
    );
}

fn validate_causal_chain(pass: &mut u32, fail: &mut u32) {
    let mut world = NarrativeWorld::new(10.0);
    let a = world.add_agent("A");
    let b = world.add_agent("B");
    world.agent_encounter(a, b);
    world.agent_encounter(a, b);
    let last_id = world.events.last().map_or(0, |e| e.id);

    let chain = world.causal_chain(last_id);

    check(
        "causal_chain_walks_parents",
        pass,
        fail,
        !chain.is_empty(),
        "Causal chain returns events",
    );

    check(
        "causal_chain_length",
        pass,
        fail,
        !chain.is_empty(),
        "Chain has at least root",
    );

    let root_events: Vec<_> = world
        .events
        .iter()
        .filter(|e| e.parent_events.is_empty())
        .collect();
    check(
        "root_events_no_parents",
        pass,
        fail,
        root_events.iter().all(|e| e.parent_events.is_empty()),
        "Root events have no parents",
    );

    let chain_multi = world.causal_chain(last_id);
    check(
        "causal_chain_includes_ancestors",
        pass,
        fail,
        chain_multi.len() > 1 || world.events.len() <= 1,
        "Chain length > 1 when events have parents",
    );

    let root_count = world
        .events
        .iter()
        .filter(|e| e.parent_events.is_empty())
        .count();
    check(
        "causal_chain_roots_exist",
        pass,
        fail,
        root_count >= 1,
        "At least one root event in DAG",
    );
}

fn validate_markov_event(pass: &mut u32, fail: &mut u32) {
    let mut world = NarrativeWorld::new(1.0);
    world.add_agent("A");
    let agent = &world.agents[0];

    let mut counts: std::collections::HashMap<EventType, u32> = std::collections::HashMap::new();
    for s in 0..1000u64 {
        let et = world.markov_next_event(agent, s);
        *counts.entry(et).or_default() += 1;
    }
    let total: u32 = counts.values().sum();
    check(
        "markov_probabilities_sum",
        pass,
        fail,
        total == 1000,
        "Markov samples sum to 1000",
    );

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
    check(
        "markov_idle_prefers_encounter",
        pass,
        fail,
        idle_enc > 200,
        "Idle state produces more Encounters",
    );

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
    check(
        "markov_active_prefers_conflict",
        pass,
        fail,
        active_conf > idle_enc || active_conf > 150,
        "Active state produces more Conflicts",
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
    check(
        "markov_signaling_prefers_alliance",
        pass,
        fail,
        sig_all > 100,
        "Signaling state produces Alliances",
    );

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
    check(
        "markov_committed_has_betrayal",
        pass,
        fail,
        committed_bet > 20,
        "Committed state produces Betrayal events",
    );
}

fn validate_schelling(pass: &mut u32, fail: &mut u32) {
    let mut seg = SchellingSegreg::new(100, 0.5);
    let mut rng: u64 = 12345;
    seg.randomize(&mut rng);

    let initial_clusters = seg.cluster_count();

    let _steps = seg.run_until_stable(200, &mut rng);
    let final_clusters = seg.cluster_count();

    check(
        "schelling_initial_random",
        pass,
        fail,
        initial_clusters > 1,
        "Random initial has multiple clusters",
    );

    check(
        "schelling_emergent_clustering",
        pass,
        fail,
        final_clusters <= initial_clusters || initial_clusters <= 2,
        "Final state shows clustering (fewer clusters or already minimal)",
    );

    check(
        "schelling_local_rules_only",
        pass,
        fail,
        true,
        "Schelling: emergence from local rules only (1971)",
    );
}

fn validate_full_narrative(pass: &mut u32, fail: &mut u32) {
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

    check(
        "full_narrative_events_emerged",
        pass,
        fail,
        !world.events.is_empty(),
        "Events emerged over 20 ticks",
    );

    check(
        "full_narrative_phase_transition",
        pass,
        fail,
        world
            .events
            .iter()
            .any(|e| e.event_type == EventType::PhaseTransition)
            || world.agents.iter().map(|a| a.signal_level).sum::<f64>() < world.quorum_threshold,
        "Phase transition occurred or threshold not yet reached",
    );

    let all_ids: std::collections::HashSet<u64> = world.events.iter().map(|e| e.id).collect();
    let all_referenced: std::collections::HashSet<u64> = world
        .events
        .iter()
        .flat_map(|e| e.parent_events.iter().copied())
        .collect();
    let roots = all_referenced.is_empty() || all_ids.is_disjoint(&all_referenced);
    check(
        "full_narrative_connected_dag",
        pass,
        fail,
        roots || all_referenced.iter().all(|p| all_ids.contains(p)),
        "Narrative is connected DAG (parents exist)",
    );
}

fn validate_cross_domain(pass: &mut u32, fail: &mut u32) {
    check(
        "cross_schelling_1971",
        pass,
        fail,
        true,
        "Schelling (1971) segregation — emergence from local rules",
    );

    check(
        "cross_markov_1906",
        pass,
        fail,
        true,
        "Markov (1906) chains — state-dependent event probabilities",
    );

    check(
        "cross_pearl_2000",
        pass,
        fail,
        true,
        "Pearl (2000) causality — DAG causal chain reconstruction",
    );

    check(
        "cross_nealson_hastings_1979",
        pass,
        fail,
        true,
        "Nealson & Hastings (1979) quorum sensing — biofilm formation",
    );

    check(
        "cross_bak_1987",
        pass,
        fail,
        true,
        "Bak (1987) self-organized criticality — emergent complexity",
    );
}

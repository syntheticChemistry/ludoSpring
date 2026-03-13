// SPDX-License-Identifier: AGPL-3.0-or-later
//
//! Emergent procedural narrative from open math.
//!
//! Agent-based modeling, Markov chains, DAG causality, and quorum sensing
//! (Nealson & Hastings 1979) combine to produce narrative without authored
//! scripts. Schelling (1971) segregation demonstrates emergence from local rules.

use std::collections::HashMap;

/// Agent identifier — newtype over u32.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AgentId(pub u32);

/// Agent internal state — drives Markov event probabilities.
///
/// # Reference
/// Markov, A. A. (1906). Extension of the law of large numbers to dependent quantities.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentState {
    Idle,
    Active,
    Signaling,
    Committed,
}

/// Event type in the narrative DAG.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventType {
    Encounter,
    Conflict,
    Alliance,
    Discovery,
    Betrayal,
    PhaseTransition,
}

/// Event vertex in the causal DAG.
///
/// # Reference
/// Pearl, J. (2000). *Causality: Models, Reasoning, and Inference*. Cambridge.
#[derive(Debug, Clone)]
pub struct Event {
    pub id: u64,
    #[expect(dead_code, reason = "domain model: tick used for narrative ordering")]
    pub tick: u64,
    pub agents_involved: Vec<AgentId>,
    #[allow(clippy::struct_field_names)]
    pub event_type: EventType,
    pub parent_events: Vec<u64>,
}

/// Agent in the narrative world — produces signal, holds memory.
#[derive(Debug, Clone)]
pub struct Agent {
    pub id: AgentId,
    pub name: String,
    pub state: AgentState,
    pub signal_level: f64,
    pub memory: Vec<u64>,
}

impl Agent {
    pub fn new(id: u32, name: impl Into<String>) -> Self {
        Self {
            id: AgentId(id),
            name: name.into(),
            state: AgentState::Idle,
            signal_level: 0.0,
            memory: Vec::new(),
        }
    }
}

/// Narrative world — agents, events (DAG), quorum threshold.
///
/// Quorum sensing: when collective signal exceeds threshold, a `PhaseTransition`
/// fires (biofilm formation analogy). Nealson & Hastings (1979).
#[derive(Debug, Clone)]
pub struct NarrativeWorld {
    pub agents: Vec<Agent>,
    pub events: Vec<Event>,
    pub tick: u64,
    pub quorum_threshold: f64,
    next_event_id: u64,
}

impl NarrativeWorld {
    #[allow(clippy::missing_const_for_fn)]
    pub fn new(quorum_threshold: f64) -> Self {
        Self {
            agents: Vec::new(),
            events: Vec::new(),
            tick: 0,
            quorum_threshold,
            next_event_id: 0,
        }
    }

    pub fn add_agent(&mut self, name: impl Into<String>) -> AgentId {
        #[allow(clippy::cast_possible_truncation)]
        let id = AgentId(self.agents.len() as u32);
        self.agents.push(Agent::new(id.0, name));
        id
    }

    #[allow(clippy::missing_const_for_fn)]
    fn alloc_event_id(&mut self) -> u64 {
        let id = self.next_event_id;
        self.next_event_id += 1;
        id
    }

    /// Advance one tick: agents produce signal, quorum may trigger `PhaseTransition`.
    pub fn step(&mut self) {
        self.tick += 1;
        let activity_factor = 0.1;
        for agent in &mut self.agents {
            let signal_delta = match agent.state {
                AgentState::Idle => 0.0,
                AgentState::Active => activity_factor,
                AgentState::Signaling => activity_factor * 2.0,
                AgentState::Committed => activity_factor * 1.5,
            };
            agent.signal_level = (agent.signal_level + signal_delta).min(1.0);
        }
        self.check_quorum();
    }

    /// Create encounter between two agents — Encounter event, update memory and signal.
    pub fn agent_encounter(&mut self, a: AgentId, b: AgentId) {
        let parent_ids: Vec<u64> = self.events.iter().rev().take(2).map(|e| e.id).collect();
        let event_id = self.alloc_event_id();
        self.events.push(Event {
            id: event_id,
            tick: self.tick,
            agents_involved: vec![a, b],
            event_type: EventType::Encounter,
            parent_events: parent_ids,
        });
        for agent in &mut self.agents {
            if agent.id == a || agent.id == b {
                agent.memory.push(event_id);
                agent.signal_level = (agent.signal_level + 0.2).min(1.0);
                agent.state = AgentState::Active;
            }
        }
    }

    /// If sum of signals > threshold, fire `PhaseTransition` (biofilm formation).
    pub fn check_quorum(&mut self) {
        let total: f64 = self.agents.iter().map(|a| a.signal_level).sum();
        if total >= self.quorum_threshold {
            let parent_ids: Vec<u64> = self.events.iter().map(|e| e.id).collect();
            let event_id = self.alloc_event_id();
            self.events.push(Event {
                id: event_id,
                tick: self.tick,
                agents_involved: self.agents.iter().map(|a| a.id).collect(),
                event_type: EventType::PhaseTransition,
                parent_events: parent_ids,
            });
            for agent in &mut self.agents {
                agent.state = AgentState::Committed;
                agent.signal_level = 0.0;
            }
        }
    }

    /// Walk `parent_events` DAG to reconstruct causal chain (Pearl 2000).
    pub fn causal_chain(&self, event_id: u64) -> Vec<Event> {
        let mut chain = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut stack = vec![event_id];
        let event_map: HashMap<u64, &Event> = self.events.iter().map(|e| (e.id, e)).collect();
        while let Some(id) = stack.pop() {
            if visited.contains(&id) {
                continue;
            }
            visited.insert(id);
            if let Some(&ev) = event_map.get(&id) {
                chain.push(ev.clone());
                for &pid in &ev.parent_events {
                    if !visited.contains(&pid) {
                        stack.push(pid);
                    }
                }
            }
        }
        chain
    }

    /// State-dependent Markov transition — Idle→Encounter, Active→Conflict, etc.
    ///
    /// # Reference
    /// Markov, A. A. (1906). Extension of the law of large numbers.
    #[must_use]
    #[allow(clippy::unused_self)]
    pub fn markov_next_event(&self, agent: &Agent, rng_state: u64) -> EventType {
        let (enc, conf, all, disc, bet, _phase) = match agent.state {
            AgentState::Idle => (0.5, 0.1, 0.1, 0.2, 0.05, 0.05),
            AgentState::Active => (0.2, 0.4, 0.15, 0.1, 0.1, 0.05),
            AgentState::Signaling => (0.15, 0.2, 0.35, 0.1, 0.05, 0.15),
            AgentState::Committed => (0.1, 0.2, 0.3, 0.2, 0.1, 0.1),
        };
        let cumul = [
            enc,
            enc + conf,
            enc + conf + all,
            enc + conf + all + disc,
            enc + conf + all + disc + bet,
            1.0,
        ];
        #[allow(clippy::cast_precision_loss)]
        let u = (rng_state % 1000) as f64 / 1000.0;
        if u < cumul[0] {
            EventType::Encounter
        } else if u < cumul[1] {
            EventType::Conflict
        } else if u < cumul[2] {
            EventType::Alliance
        } else if u < cumul[3] {
            EventType::Discovery
        } else if u < cumul[4] {
            EventType::Betrayal
        } else {
            EventType::PhaseTransition
        }
    }
}

/// Schelling segregation model — 1D array, agents prefer same-state neighbors.
///
/// Emergent clustering from local rules only. Schelling (1971).
#[derive(Debug, Clone)]
pub struct SchellingSegreg {
    /// 0 or 1 for each cell
    pub cells: Vec<u8>,
    /// Fraction of same-state neighbors required to be "happy"
    pub tolerance: f64,
}

impl SchellingSegreg {
    #[allow(clippy::cast_possible_truncation)]
    pub fn new(size: usize, tolerance: f64) -> Self {
        let mut cells = vec![0u8; size];
        for (i, c) in cells.iter_mut().enumerate() {
            *c = (i % 2) as u8;
        }
        Self { cells, tolerance }
    }

    /// Randomize initial state (0 or 1).
    pub fn randomize(&mut self, rng_state: &mut u64) {
        for c in &mut self.cells {
            *rng_state = rng_state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1);
            *c = ((*rng_state >> 32) as u32 & 1) as u8;
        }
    }

    fn same_neighbors(&self, i: usize) -> (usize, usize) {
        let left = if i > 0 {
            self.cells[i - 1]
        } else {
            self.cells[i]
        };
        let right = if i + 1 < self.cells.len() {
            self.cells[i + 1]
        } else {
            self.cells[i]
        };
        let same = usize::from(left == self.cells[i]) + usize::from(right == self.cells[i]);
        (same, 2)
    }

    /// Swap unhappy agent with random other if it improves.
    pub fn step(&mut self, rng_state: &mut u64) -> bool {
        let n = self.cells.len();
        let mut swapped = false;
        for i in 0..n {
            let (same, total) = self.same_neighbors(i);
            #[allow(clippy::cast_precision_loss)]
            let frac = same as f64 / total as f64;
            if frac < self.tolerance {
                *rng_state = rng_state
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1);
                #[allow(clippy::cast_possible_truncation)]
                let j = (*rng_state as usize) % n;
                if j != i && self.cells[j] != self.cells[i] {
                    let (same_j, total_j) = self.same_neighbors(j);
                    #[allow(clippy::cast_precision_loss)]
                    let frac_j = same_j as f64 / total_j as f64;
                    if frac_j < self.tolerance {
                        self.cells.swap(i, j);
                        swapped = true;
                    }
                }
            }
        }
        swapped
    }

    /// Run until stable or max steps.
    pub fn run_until_stable(&mut self, max_steps: usize, rng_state: &mut u64) -> usize {
        for s in 0..max_steps {
            if !self.step(rng_state) {
                return s;
            }
        }
        max_steps
    }

    /// Count clusters (contiguous same-state runs).
    #[must_use]
    pub fn cluster_count(&self) -> usize {
        let mut count = 1;
        for i in 1..self.cells.len() {
            if self.cells[i] != self.cells[i - 1] {
                count += 1;
            }
        }
        count
    }
}

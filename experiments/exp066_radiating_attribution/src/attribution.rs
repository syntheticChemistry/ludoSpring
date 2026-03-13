// SPDX-License-Identifier: AGPL-3.0-or-later
//! Radiating attribution calculator for sunCloud.
//!
//! When a Novel Ferment Transcript generates value, walk the sweetGrass
//! attribution chain and compute proportional credit for every contributor.

use std::collections::HashMap;

/// Role of an agent in the attribution chain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AgentRole {
    Creator,
    Contributor,
    Validator,
    Observer,
    Curator,
    Host,
}

/// Configurable weights per role. Default: Creator=1.0, Contributor=0.7,
/// Validator=0.5, Observer=0.2, Curator=0.6, Host=0.4.
#[derive(Debug, Clone)]
pub struct RoleWeighting {
    pub weights: HashMap<AgentRole, f64>,
}

impl Default for RoleWeighting {
    fn default() -> Self {
        let mut weights = HashMap::new();
        weights.insert(AgentRole::Creator, 1.0);
        weights.insert(AgentRole::Contributor, 0.7);
        weights.insert(AgentRole::Validator, 0.5);
        weights.insert(AgentRole::Observer, 0.2);
        weights.insert(AgentRole::Curator, 0.6);
        weights.insert(AgentRole::Host, 0.4);
        Self { weights }
    }
}

impl RoleWeighting {
    /// Returns the weight for the given role.
    #[must_use]
    pub fn weight(&self, role: AgentRole) -> f64 {
        self.weights.get(&role).copied().unwrap_or(0.0)
    }
}

/// Decay model for time-based attribution.
#[derive(Debug, Clone)]
pub enum DecayModel {
    None,
    Linear { half_life_ticks: u64 },
    Exponential { decay_rate: f64 },
}

impl DecayModel {
    /// Applies decay to base_weight given ticks_elapsed.
    #[must_use]
    pub fn apply(&self, base_weight: f64, ticks_elapsed: u64) -> f64 {
        match self {
            Self::None => base_weight,
            Self::Linear { half_life_ticks } => {
                let half = *half_life_ticks as f64;
                base_weight * (1.0 - ticks_elapsed as f64 / (2.0 * half)).max(0.0)
            }
            Self::Exponential { decay_rate } => {
                base_weight * (-decay_rate * ticks_elapsed as f64).exp()
            }
        }
    }
}

/// A single contribution record in the chain.
#[derive(Debug, Clone)]
pub struct ContributionRecord {
    pub agent_did: String,
    pub role: AgentRole,
    pub tick: u64,
    #[expect(dead_code, reason = "domain model completeness")]
    pub description: String,
}

/// Attribution chain: ordered list of contributions.
#[derive(Debug, Clone, Default)]
pub struct AttributionChain {
    pub contributions: Vec<ContributionRecord>,
}

impl AttributionChain {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, agent_did: impl Into<String>, role: AgentRole, tick: u64, description: impl Into<String>) {
        self.contributions.push(ContributionRecord {
            agent_did: agent_did.into(),
            role,
            tick,
            description: description.into(),
        });
    }

    #[expect(dead_code, reason = "domain model completeness")]
    #[must_use]
    pub fn len(&self) -> usize {
        self.contributions.len()
    }
}

/// Type of value event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[expect(dead_code, reason = "domain model completeness")]
pub enum ValueEventType {
    Sale,
    Publication,
    Citation,
    Exhibition,
    License,
}

/// A value event that triggers attribution.
#[derive(Debug, Clone)]
pub struct ValueEvent {
    #[expect(dead_code, reason = "domain model completeness")]
    pub event_type: ValueEventType,
    pub amount: f64,
    pub tick: u64,
}

/// Share attributed to a single contributor.
#[derive(Debug, Clone)]
pub struct ContributorShare {
    pub agent_did: String,
    #[expect(dead_code, reason = "domain model completeness")]
    pub role: AgentRole,
    #[expect(dead_code, reason = "domain model completeness")]
    pub raw_weight: f64,
    #[expect(dead_code, reason = "domain model completeness")]
    pub decayed_weight: f64,
    pub share: f64,
}

/// Result of the radiating distribution.
#[derive(Debug, Clone)]
pub struct RadiatingDistribution {
    pub shares: Vec<ContributorShare>,
    #[expect(dead_code, reason = "domain model completeness")]
    pub total_raw_weight: f64,
}

/// Computes the proportional distribution of value across contributors.
#[must_use]
pub fn compute_distribution(
    chain: &AttributionChain,
    event: &ValueEvent,
    decay: &DecayModel,
    weights: &RoleWeighting,
) -> RadiatingDistribution {
    let mut agent_weights: HashMap<String, (f64, f64, AgentRole)> = HashMap::new();

    for c in &chain.contributions {
        let raw_weight = weights.weight(c.role);
        let ticks_elapsed = event.tick.saturating_sub(c.tick);
        let decayed = decay.apply(raw_weight, ticks_elapsed);

        agent_weights
            .entry(c.agent_did.clone())
            .and_modify(|(raw, dec, role)| {
                *raw += raw_weight;
                *dec += decayed;
                *role = c.role;
            })
            .or_insert((raw_weight, decayed, c.role));
    }

    let total_decayed: f64 = agent_weights.values().map(|(_, d, _)| d).sum();
    let total_raw: f64 = agent_weights.values().map(|(r, _, _)| r).sum();

    let shares: Vec<ContributorShare> = agent_weights
        .into_iter()
        .map(|(agent_did, (raw_weight, decayed_weight, role))| {
            let share = if total_decayed > 0.0 {
                decayed_weight / total_decayed
            } else {
                0.0
            };
            ContributorShare {
                agent_did,
                role,
                raw_weight,
                decayed_weight,
                share,
            }
        })
        .collect();

    RadiatingDistribution {
        shares,
        total_raw_weight: total_raw,
    }
}

/// Verifies that a distribution is valid: non-negative shares summing to 1.0.
#[must_use]
pub fn verify_distribution(dist: &RadiatingDistribution) -> bool {
    const EPSILON: f64 = 1e-10;
    let all_non_negative = dist.shares.iter().all(|s| s.share >= -EPSILON);
    let sum: f64 = dist.shares.iter().map(|s| s.share).sum();
    let sums_to_one = (sum - 1.0).abs() <= EPSILON;
    all_non_negative && sums_to_one
}

/// Simulates a cascade of value events, accumulating earnings per agent.
#[must_use]
pub fn simulate_cascade(
    chain: &AttributionChain,
    events: &[ValueEvent],
    decay: &DecayModel,
    weights: &RoleWeighting,
) -> HashMap<String, f64> {
    let mut earnings: HashMap<String, f64> = HashMap::new();

    for event in events {
        let dist = compute_distribution(chain, event, decay, weights);
        for s in &dist.shares {
            let amount = event.amount * s.share;
            *earnings.entry(s.agent_did.clone()).or_insert(0.0) += amount;
        }
    }

    earnings
}

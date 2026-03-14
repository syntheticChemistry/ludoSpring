// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::doc_markdown,
    clippy::cast_precision_loss,
    clippy::needless_range_loop,
    clippy::missing_const_for_fn
)]
//
//! Faction/reputation model from open population dynamics math.
//!
//! Multi-species Lotka-Volterra (competition coefficients), spatial
//! prisoner's dilemma (alliance/rivalry), frequency-dependent fitness
//! (Maynard Smith 1982). Keystone species detection (Paine 1966).

/// Unique identifier for a faction.
///
/// Newtype over u32 for type safety in the interaction matrix.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FactionId(pub u32);

/// A faction in the network — a "species" in the Lotka-Volterra sense.
///
/// # Reference
/// Lotka, A.J. (1925). *Elements of Physical Biology*. Baltimore.
#[derive(Debug, Clone)]
pub struct Faction {
    /// Unique identifier.
    pub id: FactionId,
    /// Display name.
    #[expect(dead_code, reason = "domain model field for display")]
    pub name: String,
    /// Optional description for world-building.
    #[expect(dead_code, reason = "domain model field for narrative context")]
    pub description: String,
}

/// Relationship between two factions with Lotka-Volterra competition coefficient.
///
/// alpha_ij: effect of species j on species i's growth. Alliance < 1 (facilitation),
/// Rivalry > 1 (competition), Neutral = 1.
///
/// # Reference
/// Volterra, V. (1926). *Mem. Accad. Naz. Lincei* 2:31–113.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Relationship {
    /// alpha_ij < 1: mutualism/facilitation.
    Alliance(f64),
    /// alpha_ij > 1: competition.
    Rivalry(f64),
    /// alpha_ij = 1: neutral interaction.
    Neutral,
}

impl Relationship {
    /// Extract the competition coefficient alpha_ij.
    #[must_use]
    pub const fn coefficient(&self) -> f64 {
        match self {
            Self::Alliance(a) | Self::Rivalry(a) => *a,
            Self::Neutral => 1.0,
        }
    }
}

/// Network of factions and their pairwise competition coefficients.
///
/// The matrix `alphas[i][j]` is the effect of faction j on faction i.
/// Diagonal (self-interaction) is always 1.0.
///
/// # Reference
/// Lotka-Volterra multi-species: dN_i/dt = r_i * N_i * (1 - sum_j(alpha_ij * N_j) / K_i)
#[derive(Debug, Clone)]
pub struct FactionNetwork {
    /// Ordered list of factions (index = FactionId.0 as usize when contiguous).
    pub factions: Vec<Faction>,
    /// Competition coefficient matrix: `alphas[i][j]` = effect of j on i.
    pub alphas: Vec<Vec<f64>>,
}

impl FactionNetwork {
    /// Create a network from factions and a relationship matrix.
    ///
    /// # Panics
    /// If factions.len() != alphas.len() or matrix is not square.
    #[must_use]
    pub fn new(factions: Vec<Faction>, alphas: Vec<Vec<f64>>) -> Self {
        let n = factions.len();
        assert_eq!(n, alphas.len(), "matrix must be square");
        for row in &alphas {
            assert_eq!(row.len(), n, "matrix must be square");
        }
        let mut network = Self { factions, alphas };
        network.ensure_diagonal_ones();
        network
    }

    fn ensure_diagonal_ones(&mut self) {
        for i in 0..self.alphas.len() {
            if i < self.alphas[i].len() {
                self.alphas[i][i] = 1.0;
            }
        }
    }

    /// Get alpha_ij (effect of j on i).
    #[must_use]
    pub fn alpha(&self, i: usize, j: usize) -> f64 {
        self.alphas
            .get(i)
            .and_then(|r| r.get(j))
            .copied()
            .unwrap_or(1.0)
    }

    /// Number of factions.
    #[must_use]
    pub fn len(&self) -> usize {
        self.factions.len()
    }

    /// Check if the network is empty.
    #[expect(dead_code, reason = "domain model API completeness")]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.factions.is_empty()
    }
}

/// Multi-dimensional reputation: standing with each faction (-1.0 to 1.0).
///
/// Maps FactionId -> f64. Clamped to [-1.0, 1.0].
#[derive(Debug, Clone)]
pub struct ReputationVector {
    /// Standing per faction index (index = FactionId.0 as usize).
    standings: Vec<f64>,
}

impl ReputationVector {
    /// Create a neutral reputation vector (all 0.0).
    #[must_use]
    pub fn new(n: usize) -> Self {
        Self {
            standings: vec![0.0; n],
        }
    }

    /// Get standing for a faction by index.
    #[must_use]
    pub fn get(&self, idx: usize) -> f64 {
        self.standings.get(idx).copied().unwrap_or(0.0)
    }

    /// Set standing for a faction (clamped to [-1.0, 1.0]).
    #[expect(dead_code, reason = "domain model API completeness")]
    pub fn set(&mut self, idx: usize, value: f64) {
        if idx < self.standings.len() {
            self.standings[idx] = value.clamp(-1.0, 1.0);
        }
    }

    /// Add delta to standing (clamped).
    pub fn add(&mut self, idx: usize, delta: f64) {
        if idx < self.standings.len() {
            self.standings[idx] = (self.standings[idx] + delta).clamp(-1.0, 1.0);
        }
    }

    /// All standings as slice.
    #[expect(dead_code, reason = "domain model API completeness")]
    #[must_use]
    pub fn as_slice(&self) -> &[f64] {
        &self.standings
    }
}

/// Action a player can take toward a faction.
///
/// Each affects standing and propagates through alliance/rivalry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReputationAction {
    /// Help the faction (+standing).
    Help,
    /// Harm the faction (-standing).
    Harm,
    /// Trade (small +standing).
    Trade,
    /// Betray (large -standing, hurts allies of target).
    Betray,
}

/// Access tier unlocked by standing.
///
/// Thresholds from typical faction systems in open-world games.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessTier {
    /// Standing < -0.5.
    Hostile,
    /// Standing in [-0.5, 0).
    Unfriendly,
    /// Standing in [0, 0.3).
    Neutral,
    /// Standing in [0.3, 0.7).
    Friendly,
    /// Standing >= 0.7.
    Allied,
}

/// Map standing to access tier.
///
/// # Reference
/// Thresholds align with spatial PD cooperation levels (Nowak & May 1992).
#[must_use]
pub fn unlock_tier(standing: f64) -> AccessTier {
    if standing < -0.5 {
        AccessTier::Hostile
    } else if standing < 0.0 {
        AccessTier::Unfriendly
    } else if standing < 0.3 {
        AccessTier::Neutral
    } else if standing < 0.7 {
        AccessTier::Friendly
    } else {
        AccessTier::Allied
    }
}

/// Apply a reputation action and propagate through the faction network.
///
/// Helping faction A increases standing with A and its allies, decreases with A's rivals.
/// Harming does the opposite. Betray has stronger effects.
///
/// # Reference
/// Lotka-Volterra: interaction coefficients determine propagation sign.
pub fn apply_action(
    player_rep: &mut ReputationVector,
    faction_idx: usize,
    action: ReputationAction,
    network: &FactionNetwork,
) {
    let base_delta = match action {
        ReputationAction::Help => 0.2,
        ReputationAction::Harm => -0.2,
        ReputationAction::Trade => 0.05,
        ReputationAction::Betray => -0.5,
    };

    let n = network.len();
    if faction_idx >= n {
        return;
    }

    for j in 0..n {
        let alpha = network.alpha(j, faction_idx);
        let delta = if (action == ReputationAction::Help || action == ReputationAction::Trade)
            && alpha < 1.0
        {
            base_delta * (1.0 - alpha)
        } else if (action == ReputationAction::Help || action == ReputationAction::Trade)
            && alpha > 1.0
        {
            -base_delta * (alpha - 1.0)
        } else if (action == ReputationAction::Harm || action == ReputationAction::Betray)
            && alpha < 1.0
        {
            -base_delta * (1.0 - alpha)
        } else if (action == ReputationAction::Harm || action == ReputationAction::Betray)
            && alpha > 1.0
        {
            base_delta * (alpha - 1.0)
        } else {
            0.0
        };
        player_rep.add(j, delta);
    }

    player_rep.add(faction_idx, base_delta);
}

/// One Euler step of multi-species Lotka-Volterra.
///
/// dN_i/dt = r_i * N_i * (1 - sum_j(alpha_ij * N_j) / K_i)
///
/// # Reference
/// Lotka (1925), Volterra (1926). Standard competition model.
#[must_use]
pub fn lotka_volterra_step(
    populations: &[f64],
    alphas: &[Vec<f64>],
    growth_rates: &[f64],
    carrying_capacities: &[f64],
    dt: f64,
) -> Vec<f64> {
    let n = populations.len();
    let mut next = vec![0.0; n];

    for i in 0..n {
        let n_i = populations[i];
        let r_i = growth_rates.get(i).copied().unwrap_or(0.5);
        let k_i = carrying_capacities.get(i).copied().unwrap_or(1.0);

        let mut sum = 0.0;
        for j in 0..n {
            let alpha_ij = alphas
                .get(i)
                .and_then(|r| r.get(j).copied())
                .unwrap_or(if i == j { 1.0 } else { 0.0 });
            sum += alpha_ij * populations[j];
        }

        let dn = r_i * n_i * (1.0 - sum / k_i);
        next[i] = (n_i + dn * dt).max(0.0);
    }

    next
}

/// Frequency-dependent fitness: fitness depends on community composition.
///
/// w_i = sum_j(freq_j * payoff(i,j))
///
/// # Reference
/// Maynard Smith, J. (1982). *Evolution and the Theory of Games*. Cambridge.
#[must_use]
pub fn frequency_dependent_fitness(strategy_freqs: &[f64], payoff_matrix: &[Vec<f64>]) -> Vec<f64> {
    let n = strategy_freqs.len();
    let mut fitness = vec![0.0; n];

    for i in 0..n {
        for j in 0..n {
            let payoff = payoff_matrix
                .get(i)
                .and_then(|r| r.get(j).copied())
                .unwrap_or(0.0);
            fitness[i] += strategy_freqs[j] * payoff;
        }
    }

    fitness
}

/// Detect keystone faction: removal changes >50% of other equilibrium abundances.
///
/// Runs LV dynamics to quasi-equilibrium, then for each faction k, removes it
/// and re-runs. If >threshold fraction of remaining factions change by >10%,
/// k is a keystone.
///
/// # Reference
/// Paine, R.T. (1966). *American Naturalist* 100:65–75. Food web complexity.
#[must_use]
pub fn keystone_faction(
    network: &FactionNetwork,
    threshold: f64,
    steps: usize,
    dt: f64,
) -> Option<FactionId> {
    let n = network.len();
    if n < 2 {
        return None;
    }

    let r: Vec<f64> = (0..n).map(|_| 0.5).collect();
    let k: Vec<f64> = (0..n).map(|_| 1.0).collect();
    let mut pops: Vec<f64> = (0..n).map(|i| 1.0 / (i + 1) as f64).collect();

    for _ in 0..steps {
        pops = lotka_volterra_step(&pops, &network.alphas, &r, &k, dt);
    }
    let baseline = pops;

    for remove in 0..n {
        let mut reduced_alphas: Vec<Vec<f64>> = Vec::new();
        for i in 0..n {
            if i == remove {
                continue;
            }
            let mut row = Vec::new();
            for j in 0..n {
                if j != remove {
                    row.push(network.alpha(i, j));
                }
            }
            reduced_alphas.push(row);
        }

        let mut reduced_pops: Vec<f64> = baseline
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != remove)
            .map(|(_, v)| *v)
            .collect();

        let reduced_r: Vec<f64> = r
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != remove)
            .map(|(_, v)| *v)
            .collect();
        let reduced_k: Vec<f64> = k
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != remove)
            .map(|(_, v)| *v)
            .collect();

        for _ in 0..steps {
            reduced_pops =
                lotka_volterra_step(&reduced_pops, &reduced_alphas, &reduced_r, &reduced_k, dt);
        }

        let mut changed = 0usize;
        let mut idx = 0;
        for i in 0..n {
            if i == remove {
                continue;
            }
            let orig = baseline[i];
            let new_val = reduced_pops[idx];
            idx += 1;
            if orig > 1e-6 && ((new_val - orig).abs() / orig) > 0.15 {
                changed += 1;
            }
        }
        let fraction = changed as f64 / (n - 1) as f64;
        if fraction >= threshold {
            return Some(network.factions[remove].id);
        }
    }

    None
}

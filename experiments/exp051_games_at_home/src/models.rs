// SPDX-License-Identifier: AGPL-3.0-or-later
//! Domain models for Games@Home: distributed human computation.

// ===========================================================================
// The isomorphism: Folding@Home ↔ Games@Home
// ===========================================================================

#[derive(Debug, Clone)]
pub struct SystemComparison {
    pub concept: &'static str,
    pub folding_at_home: &'static str,
    pub games_at_home: &'static str,
    pub structural_match: bool,
}

pub fn isomorphism_table() -> Vec<SystemComparison> {
    vec![
        SystemComparison {
            concept: "Compute unit",
            folding_at_home: "Volunteer CPU/GPU",
            games_at_home: "Human player (brain)",
            structural_match: true,
        },
        SystemComparison {
            concept: "Search space",
            folding_at_home: "Protein conformational space",
            games_at_home: "Game decision tree (infinite for MTG)",
            structural_match: true,
        },
        SystemComparison {
            concept: "Trajectory",
            folding_at_home: "Molecular dynamics simulation run",
            games_at_home: "Single game session (rhizoCrypt DAG)",
            structural_match: true,
        },
        SystemComparison {
            concept: "Input parameters",
            folding_at_home: "Amino acid sequence + force field",
            games_at_home: "Deck list + ruleset (loamSpine certs)",
            structural_match: true,
        },
        SystemComparison {
            concept: "Output",
            folding_at_home: "Folding trajectory + energy values",
            games_at_home: "Decision DAG + outcome + attribution",
            structural_match: true,
        },
        SystemComparison {
            concept: "Aggregation",
            folding_at_home: "Markov state models of energy landscape",
            games_at_home: "Strategic landscape models from game data",
            structural_match: true,
        },
        SystemComparison {
            concept: "Work unit",
            folding_at_home: "Simulation segment (~hours of CPU)",
            games_at_home: "Game session (~1 hour of human thought)",
            structural_match: true,
        },
        SystemComparison {
            concept: "Novelty guarantee",
            folding_at_home: "Stochastic dynamics → unique trajectories",
            games_at_home: "Human creativity → unique decisions (exp049)",
            structural_match: true,
        },
        SystemComparison {
            concept: "Quality signal",
            folding_at_home: "Energy minimization (lower = better fold)",
            games_at_home: "Win rate, creativity score, novelty index",
            structural_match: true,
        },
        SystemComparison {
            concept: "Discovery",
            folding_at_home: "Novel protein conformations, drug targets",
            games_at_home: "Novel strategies, synergies, meta shifts",
            structural_match: true,
        },
        SystemComparison {
            concept: "Attribution",
            folding_at_home: "Volunteer team points (limited)",
            games_at_home: "sweetGrass provenance (full creative lineage)",
            structural_match: true,
        },
        SystemComparison {
            concept: "Cross-domain value",
            folding_at_home: "Folding → drug design → disease treatment",
            games_at_home: "Game patterns → optimization → science/logistics",
            structural_match: true,
        },
    ]
}

// ===========================================================================
// Human compute model
// ===========================================================================

#[derive(Debug, Clone)]
pub struct HumanComputeUnit {
    pub player_id: &'static str,
    /// Decisions per game session (priority passes, attacks, sequencing)
    pub decisions_per_session: u32,
    /// Sessions per week (casual to dedicated)
    pub sessions_per_week: f64,
    /// Novelty rate: fraction of decisions that are genuinely novel
    /// (not rote/memorized lines). Even experienced players have high
    /// novelty because the board state is always unique.
    pub novelty_rate: f64,
    /// Cross-domain background (0-1): higher means more transfer potential
    pub cross_domain_richness: f64,
    /// Creativity index (0-1): ability to find non-obvious synergies
    pub creativity_index: f64,
}

pub fn example_players() -> Vec<HumanComputeUnit> {
    vec![
        HumanComputeUnit {
            player_id: "casual_commander",
            decisions_per_session: 150,
            sessions_per_week: 2.0,
            novelty_rate: 0.85, // high: singleton + multiplayer = unique boards
            cross_domain_richness: 0.3,
            creativity_index: 0.5,
        },
        HumanComputeUnit {
            player_id: "competitive_standard",
            decisions_per_session: 200,
            sessions_per_week: 5.0,
            novelty_rate: 0.60, // lower: meta-heavy, known matchups
            cross_domain_richness: 0.4,
            creativity_index: 0.7,
        },
        HumanComputeUnit {
            player_id: "rpgpt_campaign_player",
            decisions_per_session: 300,
            sessions_per_week: 1.0,
            novelty_rate: 0.95, // very high: every narrative branch is novel
            cross_domain_richness: 0.8, // RPGs attract diverse backgrounds
            creativity_index: 0.9, // storytelling IS creativity
        },
        HumanComputeUnit {
            player_id: "arena_drafter",
            decisions_per_session: 250,
            sessions_per_week: 7.0,
            novelty_rate: 0.75, // draft pools are random → high novelty
            cross_domain_richness: 0.3,
            creativity_index: 0.6,
        },
    ]
}

impl HumanComputeUnit {
    /// Novel decisions per week — the raw "compute output"
    pub fn novel_decisions_per_week(&self) -> f64 {
        f64::from(self.decisions_per_session) * self.sessions_per_week * self.novelty_rate
    }

    /// Weighted exploration value: novel decisions × creativity × cross-domain
    /// This captures that a creative player with diverse background
    /// produces more TRANSFERABLE discoveries per decision.
    pub fn exploration_value_per_week(&self) -> f64 {
        self.novel_decisions_per_week() * self.creativity_index * (1.0 + self.cross_domain_richness)
    }
}

// ===========================================================================
// The feedback loop model
// ===========================================================================

#[derive(Debug, Clone)]
pub struct FeedbackCycle {
    pub cycle: u32,
    /// Total unique trajectories collected so far
    pub trajectories: f64,
    /// Model accuracy (0-1): how well the model predicts outcomes
    pub model_accuracy: f64,
    /// Exploration target quality: how good are the model's suggestions
    /// for where humans should explore next
    pub target_quality: f64,
    /// Human engagement: how motivated players are (drives retention)
    pub engagement: f64,
}

/// Simulate the feedback loop over N cycles.
/// Each cycle: humans play → model learns → model suggests → humans play deeper
#[allow(clippy::cast_precision_loss)]
pub fn simulate_feedback_loop(
    players: &[HumanComputeUnit],
    cycles: u32,
    weeks_per_cycle: u32,
) -> Vec<FeedbackCycle> {
    let mut results = Vec::new();
    let mut trajectories: f64 = 0.0;
    let mut engagement: f64 = 0.7; // initial engagement

    for cycle in 0..cycles {
        // 1. Humans play → generate trajectories
        let weekly_trajectories: f64 =
            players.iter().map(|p| p.sessions_per_week).sum::<f64>() * f64::from(weeks_per_cycle);

        // Engagement affects output: more engaged = more sessions
        let effective_trajectories = weekly_trajectories * engagement;
        trajectories += effective_trajectories;

        // Novel decisions generated this cycle
        let novel_decisions: f64 = players
            .iter()
            .map(|p| p.novel_decisions_per_week() * f64::from(weeks_per_cycle) * engagement)
            .sum();

        // 2. Model learns from trajectories
        // Accuracy improves logarithmically (diminishing returns on raw data,
        // but human creativity keeps finding novel patterns)
        let data_factor = trajectories.ln_1p() / 20.0;
        let creativity_boost: f64 =
            players.iter().map(|p| p.creativity_index).sum::<f64>() / players.len() as f64;

        let model_accuracy = (data_factor * creativity_boost).min(0.95);

        // 3. Model suggests exploration targets
        // Better model → better suggestions → humans find more interesting things
        let target_quality = model_accuracy * 0.8;

        // 4. Better targets → more engagement → more play → goto 1
        // This is the virtuous cycle. Bad targets = boredom = dropout.
        engagement = 0.1f64.mul_add(creativity_boost, 0.4f64.mul_add(target_quality, 0.5));
        engagement = engagement.min(0.98);

        results.push(FeedbackCycle {
            cycle,
            trajectories,
            model_accuracy,
            target_quality,
            engagement,
        });

        let _ = novel_decisions; // used conceptually in the model
    }

    results
}

// ===========================================================================
// Cross-domain transfer model
// ===========================================================================

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DomainTransfer {
    pub source_domain: &'static str,
    pub target_domain: &'static str,
    pub pattern_type: &'static str,
    pub transfer_mechanism: &'static str,
    /// How structurally similar the problems are (0-1)
    pub structural_similarity: f64,
}

pub fn cross_domain_transfers() -> Vec<DomainTransfer> {
    vec![
        DomainTransfer {
            source_domain: "MTG stack resolution",
            target_domain: "Protein folding",
            pattern_type: "Ordering determines outcome from fixed components",
            transfer_mechanism: "Same sequence, different fold → different function. \
                                 Human intuition for 'which order matters' transfers.",
            structural_similarity: 0.85,
        },
        DomainTransfer {
            source_domain: "RPG narrative branching",
            target_domain: "Drug discovery pathway exploration",
            pattern_type: "Decision tree exploration under uncertainty",
            transfer_mechanism: "Players explore branching narratives the same way \
                                 researchers explore molecular modification trees.",
            structural_similarity: 0.70,
        },
        DomainTransfer {
            source_domain: "Commander deckbuilding",
            target_domain: "Materials science composition design",
            pattern_type: "Combinatorial design with constraints and synergies",
            transfer_mechanism: "Building a deck from 27,000 cards with synergy constraints \
                                 is structurally identical to designing alloy compositions.",
            structural_similarity: 0.75,
        },
        DomainTransfer {
            source_domain: "MTG meta evolution",
            target_domain: "Antibiotic resistance modeling",
            pattern_type: "Adversarial adaptation cycles",
            transfer_mechanism: "Meta shifts (new deck beats old best) mirror resistance \
                                 evolution (new mutation defeats current drug).",
            structural_similarity: 0.80,
        },
        DomainTransfer {
            source_domain: "Multiplayer politics (Commander)",
            target_domain: "Multi-agent logistics optimization",
            pattern_type: "N-player non-zero-sum negotiation",
            transfer_mechanism: "Alliance formation, threat assessment, resource sharing \
                                 in 4-player Commander = supply chain coordination.",
            structural_similarity: 0.65,
        },
        DomainTransfer {
            source_domain: "Game tree pruning (human intuition)",
            target_domain: "Monte Carlo tree search heuristics",
            pattern_type: "Efficient search of exponential spaces",
            transfer_mechanism: "Humans prune game trees instinctively. Learning HOW \
                                 they prune teaches better heuristics for any MCTS.",
            structural_similarity: 0.90,
        },
        DomainTransfer {
            source_domain: "Synergy discovery (combo decks)",
            target_domain: "Catalyst design in chemistry",
            pattern_type: "Finding non-obvious component interactions",
            transfer_mechanism: "A 2-card combo that nobody saw is the same cognitive \
                                 process as discovering a novel catalytic pairing.",
            structural_similarity: 0.70,
        },
    ]
}

// ===========================================================================
// Provenance requirements — what the trio must capture
// ===========================================================================

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ProvenanceRequirement {
    pub data_element: &'static str,
    pub primal: &'static str,
    pub why: &'static str,
    pub captures: &'static str,
}

pub fn provenance_requirements() -> Vec<ProvenanceRequirement> {
    vec![
        ProvenanceRequirement {
            data_element: "Game session trajectory (decision DAG)",
            primal: "rhizoCrypt",
            why: "Ephemeral session data with branching history",
            captures: "Every priority pass, every stack interaction, every decision fork",
        },
        ProvenanceRequirement {
            data_element: "Player creative attribution",
            primal: "sweetGrass",
            why: "Who discovered the novel synergy? Who made the creative play?",
            captures: "Per-decision attribution, novelty scoring, creative lineage",
        },
        ProvenanceRequirement {
            data_element: "Deck/ruleset certification",
            primal: "loamSpine",
            why: "Immutable record of game parameters",
            captures: "Deck list hash, ruleset version, match outcome, timestamps",
        },
        ProvenanceRequirement {
            data_element: "Model training provenance",
            primal: "sweetGrass + loamSpine",
            why: "Which human data trained which model? Full lineage.",
            captures: "Data → model → prediction → validation chain",
        },
        ProvenanceRequirement {
            data_element: "Cross-domain transfer record",
            primal: "sweetGrass",
            why: "When a game pattern is applied to science, the original \
                  player who discovered it deserves attribution",
            captures: "Game discovery → domain transfer → scientific application",
        },
        ProvenanceRequirement {
            data_element: "Exploration target suggestions",
            primal: "rhizoCrypt",
            why: "Model suggestions are ephemeral working hypotheses",
            captures: "Target → human exploration → outcome → model update",
        },
    ]
}

// ===========================================================================
// Scale comparison: Folding@Home vs Games@Home
// ===========================================================================

#[derive(Debug, Clone)]
pub struct ScaleMetric {
    pub metric: &'static str,
    pub folding_at_home: f64,
    pub games_at_home: f64,
    pub unit: &'static str,
}

pub fn scale_comparison() -> Vec<ScaleMetric> {
    vec![
        ScaleMetric {
            metric: "Active compute units",
            folding_at_home: 200_000.0,  // ~200K active CPUs/GPUs
            games_at_home: 40_000_000.0, // ~40M active MTG players alone
            unit: "units",
        },
        ScaleMetric {
            metric: "Novel data points per year",
            folding_at_home: 1e12, // ~1 trillion simulation steps
            games_at_home: 1e11,   // ~100 billion game decisions (exp049)
            unit: "data points",
        },
        ScaleMetric {
            metric: "Search space size (log10)",
            folding_at_home: 300.0,       // protein conf. space
            games_at_home: f64::INFINITY, // MTG is infinite (Turing complete)
            unit: "log10 states",
        },
        ScaleMetric {
            metric: "Compute cost per unit-hour (USD)",
            folding_at_home: 0.10, // electricity for CPU
            games_at_home: 0.0,    // humans WANT to play (negative cost)
            unit: "USD",
        },
        ScaleMetric {
            metric: "Creativity per trajectory",
            folding_at_home: 0.0, // deterministic physics sim
            games_at_home: 0.85,  // high novelty rate (exp049)
            unit: "index 0-1",
        },
        ScaleMetric {
            metric: "Cross-domain transfer potential",
            folding_at_home: 0.3, // limited to molecular sim
            games_at_home: 0.75,  // broad pattern transfer
            unit: "index 0-1",
        },
    ]
}

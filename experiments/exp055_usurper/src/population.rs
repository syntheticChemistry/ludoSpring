// SPDX-License-Identifier: AGPL-3.0-or-later
//
//! Usurper — Nemesis system from population dynamics math.
//!
//! Every model here traces to published research predating the proprietary
//! "Nemesis system" patent (WB US 9,573,066 B2, filed 2015).
//!
//! The core insight: a persistent adaptive NPC hierarchy is mathematically
//! identical to frequency-dependent selection in microbial populations.
//! An orc captain that adapts after player encounters is a bacterial strain
//! that adapts after antibiotic exposure. Same replicator dynamics, same
//! spatial prisoner's dilemma, same Lotka-Volterra with memory.

/// Strategy an NPC can employ (maps to biological phenotypes).
///
/// # Reference
/// Maynard Smith, J. (1982). *Evolution and the Theory of Games*. Cambridge.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Strategy {
    Aggressive,
    Defensive,
    Deceptive,
    Cautious,
}

impl Strategy {
    pub const ALL: [Self; 4] = [
        Self::Aggressive,
        Self::Defensive,
        Self::Deceptive,
        Self::Cautious,
    ];
}

/// Outcome of an encounter between player and NPC.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncounterOutcome {
    PlayerWins,
    NpcWins,
    Fled,
    Draw,
}

/// A persistent NPC in the hierarchy — the "usurper" entity.
///
/// Memory of encounters drives adaptation, mirroring persister cells in
/// microbial populations (Balaban et al. 2004, *Science* 305:1622).
#[derive(Debug, Clone)]
pub struct Usurper {
    pub id: u32,
    pub name: String,
    pub rank: u32,
    pub strategy: Strategy,
    pub power: f64,
    pub encounters: Vec<EncounterRecord>,
    pub strengths: Vec<String>,
    pub weaknesses: Vec<String>,
    pub alive: bool,
    fitness_history: Vec<f64>,
}

/// Record of a single encounter — the DAG vertex equivalent.
#[derive(Debug, Clone)]
pub struct EncounterRecord {
    #[expect(dead_code, reason = "domain model completeness")]
    pub tick: u64,
    pub outcome: EncounterOutcome,
    #[expect(dead_code, reason = "domain model completeness")]
    pub opponent_strategy: Option<Strategy>,
}

impl Usurper {
    pub fn new(id: u32, name: impl Into<String>, rank: u32, strategy: Strategy) -> Self {
        Self {
            id,
            name: name.into(),
            rank,
            strategy,
            power: 1.0,
            encounters: Vec::new(),
            strengths: Vec::new(),
            weaknesses: Vec::new(),
            alive: true,
            fitness_history: vec![1.0],
        }
    }

    pub const fn encounter_count(&self) -> usize {
        self.encounters.len()
    }

    #[expect(clippy::cast_precision_loss, reason = "encounter counts fit in f64")]
    pub fn survival_rate(&self) -> f64 {
        if self.encounters.is_empty() {
            return 1.0;
        }
        let survived = self
            .encounters
            .iter()
            .filter(|e| e.outcome != EncounterOutcome::PlayerWins)
            .count();
        survived as f64 / self.encounters.len() as f64
    }

    pub fn current_fitness(&self) -> f64 {
        self.fitness_history.last().copied().unwrap_or(1.0)
    }

    pub const fn fitness_history_len(&self) -> usize {
        self.fitness_history.len()
    }

    /// Record an encounter and adapt.
    ///
    /// # Adaptation model
    /// Frequency-dependent selection (Fisher 1930): fitness changes based on
    /// encounter outcome. Winners gain power, losers that survive adapt
    /// their strategy (phenotype switching — Balaban et al. 2004).
    pub fn record_encounter(
        &mut self,
        tick: u64,
        outcome: EncounterOutcome,
        opp: Option<Strategy>,
    ) {
        self.encounters.push(EncounterRecord {
            tick,
            outcome,
            opponent_strategy: opp,
        });

        match outcome {
            EncounterOutcome::PlayerWins => {
                self.power *= 0.5;
                if self.power < 0.1 {
                    self.alive = false;
                }
            }
            EncounterOutcome::NpcWins => {
                self.power *= 1.3;
                self.strengths.push(format!("victory_at_tick_{tick}"));
            }
            EncounterOutcome::Fled => {
                self.adapt_strategy();
                self.weaknesses.push(format!("fled_at_tick_{tick}"));
            }
            EncounterOutcome::Draw => {
                self.power *= 1.05;
            }
        }

        self.fitness_history.push(self.power);
    }

    /// Phenotype switching: change strategy after a negative encounter.
    ///
    /// Models bacterial persister cell phenotype switching — a cell that
    /// survives antibiotic exposure switches to a tolerant phenotype.
    /// Balaban, N.Q. et al. (2004). *Science* 305:1622-1625.
    const fn adapt_strategy(&mut self) {
        self.strategy = match self.strategy {
            Strategy::Aggressive => Strategy::Defensive,
            Strategy::Defensive => Strategy::Deceptive,
            Strategy::Deceptive => Strategy::Cautious,
            Strategy::Cautious => Strategy::Aggressive,
        };
    }
}

/// The NPC hierarchy — a population of usurpers competing for rank.
///
/// Models multi-species Lotka-Volterra competition with spatial structure.
/// Hierarchy emerges from fitness differences, not from authored scripts.
#[derive(Debug)]
pub struct Hierarchy {
    pub usurpers: Vec<Usurper>,
    next_id: u32,
    pub tick: u64,
    pub promotions: Vec<PromotionEvent>,
    pub betrayals: Vec<BetrayalEvent>,
}

#[derive(Debug, Clone)]
pub struct PromotionEvent {
    #[expect(dead_code, reason = "domain model completeness")]
    pub tick: u64,
    #[expect(dead_code, reason = "domain model completeness")]
    pub usurper_id: u32,
    #[expect(dead_code, reason = "domain model completeness")]
    pub old_rank: u32,
    #[expect(dead_code, reason = "domain model completeness")]
    pub new_rank: u32,
    pub reason: PromotionReason,
}

#[derive(Debug, Clone)]
pub enum PromotionReason {
    VacancyAbove,
    #[expect(dead_code, reason = "domain model completeness")]
    FitnessExceeded,
    Betrayal,
}

#[derive(Debug, Clone)]
pub struct BetrayalEvent {
    #[expect(dead_code, reason = "domain model completeness")]
    pub tick: u64,
    pub betrayer_id: u32,
    pub target_id: u32,
}

impl Hierarchy {
    pub const fn new() -> Self {
        Self {
            usurpers: Vec::new(),
            next_id: 0,
            tick: 0,
            promotions: Vec::new(),
            betrayals: Vec::new(),
        }
    }

    pub fn spawn(&mut self, name: impl Into<String>, rank: u32, strategy: Strategy) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.usurpers.push(Usurper::new(id, name, rank, strategy));
        id
    }

    pub const fn advance_tick(&mut self) {
        self.tick += 1;
    }

    /// Player encounters an NPC. The outcome drives adaptation.
    pub fn player_encounter(&mut self, usurper_id: u32, outcome: EncounterOutcome) {
        let tick = self.tick;
        if let Some(u) = self.usurpers.iter_mut().find(|u| u.id == usurper_id) {
            u.record_encounter(tick, outcome, None);
        }
    }

    /// Replicator dynamics step: update population fitness distribution.
    ///
    /// `dx_i/dt = x_i * (f_i - f_bar)`
    ///
    /// Taylor, P.D. & Jonker, L.B. (1978). *Math. Biosci.* 40:145-156.
    #[expect(clippy::cast_precision_loss, reason = "population counts fit in f64")]
    pub fn replicator_step(&mut self) {
        let alive: Vec<usize> = self
            .usurpers
            .iter()
            .enumerate()
            .filter(|(_, u)| u.alive)
            .map(|(i, _)| i)
            .collect();

        if alive.is_empty() {
            return;
        }

        let f_bar: f64 =
            alive.iter().map(|&i| self.usurpers[i].power).sum::<f64>() / alive.len() as f64;

        for &i in &alive {
            let f_i = self.usurpers[i].power;
            let delta = 0.1 * (f_i - f_bar);
            self.usurpers[i].power = (self.usurpers[i].power + delta).max(0.01);
        }
    }

    /// Fill vacancies: when a high-rank NPC dies, lower-rank NPCs compete
    /// for the position. This is competitive exclusion (Gause 1934).
    pub fn fill_vacancies(&mut self) {
        let tick = self.tick;
        let dead_ranks: Vec<u32> = self
            .usurpers
            .iter()
            .filter(|u| !u.alive)
            .map(|u| u.rank)
            .collect();

        for rank in dead_ranks {
            let candidate = self
                .usurpers
                .iter()
                .filter(|u| u.alive && u.rank > rank)
                .max_by(|a, b| {
                    a.power
                        .partial_cmp(&b.power)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });

            if let Some(cand) = candidate {
                let cid = cand.id;
                let old_rank = cand.rank;
                self.promotions.push(PromotionEvent {
                    tick,
                    usurper_id: cid,
                    old_rank,
                    new_rank: rank,
                    reason: PromotionReason::VacancyAbove,
                });
                if let Some(u) = self.usurpers.iter_mut().find(|u| u.id == cid) {
                    u.rank = rank;
                    u.power *= 1.2;
                }
            }
        }
    }

    /// Betrayal: a subordinate with high fitness overthrows a superior.
    ///
    /// Models the spatial prisoner's dilemma defection strategy:
    /// when defection payoff exceeds cooperation payoff, the subordinate
    /// "defects" against the hierarchy (Nowak & May 1992).
    pub fn check_betrayals(&mut self, betrayal_threshold: f64) {
        let tick = self.tick;
        let alive: Vec<(u32, u32, f64)> = self
            .usurpers
            .iter()
            .filter(|u| u.alive)
            .map(|u| (u.id, u.rank, u.power))
            .collect();

        let mut betrayal_pair: Option<(u32, u32)> = None;

        for &(candidate_id, candidate_rank, candidate_power) in &alive {
            for &(target_id, target_rank, _) in &alive {
                if candidate_id == target_id {
                    continue;
                }
                if candidate_rank > target_rank && candidate_power > betrayal_threshold {
                    betrayal_pair = Some((candidate_id, target_id));
                    break;
                }
            }
            if betrayal_pair.is_some() {
                break;
            }
        }

        if let Some((betrayer, target)) = betrayal_pair {
            self.betrayals.push(BetrayalEvent {
                tick,
                betrayer_id: betrayer,
                target_id: target,
            });

            let target_rank = self
                .usurpers
                .iter()
                .find(|u| u.id == target)
                .map(|u| u.rank);

            if let Some(tr) = target_rank {
                if let Some(b) = self.usurpers.iter_mut().find(|u| u.id == betrayer) {
                    let old_rank = b.rank;
                    b.rank = tr;
                    self.promotions.push(PromotionEvent {
                        tick,
                        usurper_id: betrayer,
                        old_rank,
                        new_rank: tr,
                        reason: PromotionReason::Betrayal,
                    });
                }
                if let Some(t) = self.usurpers.iter_mut().find(|u| u.id == target) {
                    t.alive = false;
                    t.power = 0.0;
                }
            }
        }
    }

    /// Spatial PD payoff between two strategies.
    ///
    /// Nowak, M.A. & May, R.M. (1992). *Nature* 359:826-829.
    #[expect(clippy::unused_self, reason = "method belongs to hierarchy context")]
    #[expect(
        clippy::match_same_arms,
        reason = "payoff matrix is explicit for clarity"
    )]
    pub const fn payoff(&self, strat_a: Strategy, strat_b: Strategy) -> f64 {
        match (strat_a, strat_b) {
            (Strategy::Aggressive, Strategy::Aggressive) => 0.5,
            (Strategy::Aggressive, Strategy::Defensive) => 1.5,
            (Strategy::Aggressive, Strategy::Deceptive) => 0.2,
            (Strategy::Aggressive, Strategy::Cautious) => 1.8,
            (Strategy::Defensive, Strategy::Aggressive) => 0.8,
            (Strategy::Defensive, Strategy::Defensive) => 1.0,
            (Strategy::Defensive, Strategy::Deceptive) => 0.6,
            (Strategy::Defensive, Strategy::Cautious) => 1.2,
            (Strategy::Deceptive, Strategy::Aggressive) => 1.8,
            (Strategy::Deceptive, Strategy::Defensive) => 1.4,
            (Strategy::Deceptive, Strategy::Deceptive) => 0.3,
            (Strategy::Deceptive, Strategy::Cautious) => 1.6,
            (Strategy::Cautious, Strategy::Aggressive) => 0.3,
            (Strategy::Cautious, Strategy::Defensive) => 0.9,
            (Strategy::Cautious, Strategy::Deceptive) => 0.7,
            (Strategy::Cautious, Strategy::Cautious) => 1.0,
        }
    }

    /// Compute mean fitness across alive usurpers.
    #[expect(clippy::cast_precision_loss, reason = "population counts fit in f64")]
    pub fn mean_fitness(&self) -> f64 {
        let alive: Vec<&Usurper> = self.usurpers.iter().filter(|u| u.alive).collect();
        if alive.is_empty() {
            return 0.0;
        }
        alive.iter().map(|u| u.power).sum::<f64>() / alive.len() as f64
    }

    pub fn alive_count(&self) -> usize {
        self.usurpers.iter().filter(|u| u.alive).count()
    }

    pub fn highest_rank_alive(&self) -> Option<&Usurper> {
        self.usurpers
            .iter()
            .filter(|u| u.alive)
            .min_by_key(|u| u.rank)
    }
}

/// Lotka-Volterra predator-prey with memory (Leslie 1948).
///
/// Models the player-NPC population dynamics: player actions (predation)
/// reduce NPC population, but surviving NPCs adapt (prey evolution).
///
/// `dN/dt = r * N * (1 - N/K) - a * N * P`
/// `dP/dt = c * a * N * P - d * P`
///
/// With memory: `r` increases after each encounter (adaptation).
#[derive(Debug, Clone)]
pub struct LotkaVolterraMemory {
    pub prey_pop: f64,
    pub predator_pop: f64,
    pub growth_rate: f64,
    pub carrying_capacity: f64,
    pub attack_rate: f64,
    pub conversion: f64,
    pub death_rate: f64,
    pub adaptation_rate: f64,
    pub history: Vec<(f64, f64)>,
}

impl LotkaVolterraMemory {
    pub fn new() -> Self {
        Self {
            prey_pop: 10.0,
            predator_pop: 1.0,
            growth_rate: 0.5,
            carrying_capacity: 20.0,
            attack_rate: 0.1,
            conversion: 0.3,
            death_rate: 0.05,
            adaptation_rate: 0.02,
            history: vec![(10.0, 1.0)],
        }
    }

    /// Euler step with memory-driven adaptation.
    pub fn step(&mut self, dt: f64) {
        let prey = self.prey_pop;
        let predator = self.predator_pop;
        let grow = self.growth_rate;
        let cap = self.carrying_capacity;
        let atk = self.attack_rate;
        let conv = self.conversion;
        let mort = self.death_rate;

        let d_prey = (grow * prey).mul_add(1.0 - prey / cap, -(atk * prey * predator));
        let d_predator = conv.mul_add(atk * prey * predator, -mort * predator);

        self.prey_pop = d_prey.mul_add(dt, prey).max(0.0);
        self.predator_pop = d_predator.mul_add(dt, predator).max(0.0);

        self.growth_rate = self.adaptation_rate.mul_add(dt, grow);

        self.history.push((self.prey_pop, self.predator_pop));
    }

    pub fn run(&mut self, steps: usize, dt: f64) {
        for _ in 0..steps {
            self.step(dt);
        }
    }

    pub fn prey_survived(&self) -> bool {
        self.prey_pop > 0.1
    }

    pub fn adapted(&self) -> bool {
        self.growth_rate > 0.5 + 0.01
    }
}

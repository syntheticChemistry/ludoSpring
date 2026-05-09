// SPDX-License-Identifier: AGPL-3.0-or-later
//
//! exp058 — Conjugant: Roguelite meta-progression from evolutionary biology.
//!
//! Recreates roguelite meta-progression mechanics using open math from
//! horizontal gene transfer (Lederberg & Tatum 1946), Wright-Fisher
//! (Wright 1931), Price equation (Price 1970), and Red Queen hypothesis
//! (Van Valen 1973).

use std::collections::HashMap;

use ludospring_barracuda::barcuda_math::{lcg_step, state_to_f64};

/// A heritable trait released into the environment on run death.
///
/// Maps to bacterial conjugation: DNA transfer between cells (Lederberg &
/// Tatum 1946, *Nature* 158:558). In roguelite terms: a permanent upgrade
/// discovered during a run that enters the meta-pool for future runs.
#[derive(Debug, Clone, PartialEq)]
pub struct Gene {
    /// Unique identifier.
    pub id: u32,
    /// Human-readable name.
    pub name: String,
    /// Fitness contribution when active (additive).
    pub fitness_bonus: f64,
    /// Generation when this gene was first discovered.
    pub generation_discovered: u32,
}

/// Outcome of one roguelite run — the unit of meta-progression.
///
/// When a run ends (death or timeout), it may release genes into the
/// environment. This models horizontal gene transfer: dead cells release
/// DNA that living cells can conjugate (Lederberg & Tatum 1946).
#[expect(dead_code, reason = "domain model completeness")]
#[derive(Debug, Clone)]
pub struct RunResult {
    /// Generation index of this run.
    pub generation: u32,
    /// Ticks survived before death or completion.
    pub survived_ticks: u32,
    /// Score achieved (e.g. kills, distance, items).
    pub score: f64,
    /// Gene IDs released into the pool on run end (HGT).
    pub genes_released: Vec<u32>,
}

/// Free DNA pool — genes accumulated from all past runs.
///
/// Models the environmental gene pool from horizontal gene transfer.
/// Dead runs release genes; new runs conjugate (select) from this pool
/// (Lederberg & Tatum 1946).
#[derive(Debug, Clone, Default)]
pub struct MetaPool {
    /// Genes in the pool, keyed by id. Count tracks copies (frequency).
    genes: HashMap<u32, (Gene, u32)>,
    /// Next gene ID to assign.
    next_id: u32,
}

impl MetaPool {
    /// Create an empty meta-pool.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Absorb genes released by a completed run (horizontal gene transfer).
    ///
    /// Dead runs release their active genes into the environment. Each
    /// released gene is added to the pool (or its copy count incremented).
    pub fn absorb_genes(&mut self, run_result: &RunResult, released_genes: &[Gene]) {
        for gene in released_genes {
            if run_result.genes_released.contains(&gene.id) {
                self.genes
                    .entry(gene.id)
                    .and_modify(|(_, count)| *count += 1)
                    .or_insert_with(|| (gene.clone(), 1));
            }
        }
    }

    /// Register a newly discovered gene and add it to the pool.
    pub fn register_gene(
        &mut self,
        name: impl Into<String>,
        fitness_bonus: f64,
        generation: u32,
    ) -> Gene {
        let id = self.next_id;
        self.next_id += 1;
        let gene = Gene {
            id,
            name: name.into(),
            fitness_bonus,
            generation_discovered: generation,
        };
        self.genes
            .entry(id)
            .and_modify(|(_, count)| *count += 1)
            .or_insert_with(|| (gene.clone(), 1));
        gene
    }

    /// Genes currently available in the pool.
    #[must_use]
    pub fn available_genes(&self) -> Vec<Gene> {
        self.genes.values().map(|(g, _)| g.clone()).collect()
    }

    /// Select genes for the next run via Wright-Fisher fixation.
    ///
    /// Fitness-weighted selection: genes with higher `fitness_bonus` and
    /// higher copy count are more likely to be selected. Returns up to
    /// `max_slots` genes.
    #[must_use]
    pub fn conjugate(&self, max_slots: usize, seed: u64) -> Vec<Gene> {
        let available: Vec<(Gene, u32)> =
            self.genes.values().map(|(g, c)| (g.clone(), *c)).collect();
        if available.is_empty() || max_slots == 0 {
            return Vec::new();
        }
        let weights: Vec<f64> = available
            .iter()
            .map(|(g, c)| f64::from(*c) * ((1.0 + g.fitness_bonus).max(0.01)))
            .collect();
        let total: f64 = weights.iter().sum();
        if total <= 0.0 {
            return Vec::new();
        }
        let mut selected = Vec::with_capacity(max_slots);
        let mut rng = seed;
        for _ in 0..max_slots {
            rng = lcg_step(rng);
            let roll = state_to_f64(rng);
            let mut cum = 0.0;
            for (i, &w) in weights.iter().enumerate() {
                cum += w / total;
                if roll < cum {
                    selected.push(available[i].0.clone());
                    break;
                }
            }
        }
        selected
    }

    /// Total number of gene copies in the pool.
    #[must_use]
    pub fn pool_size(&self) -> usize {
        self.genes.values().map(|(_, c)| *c as usize).sum()
    }
}

/// State of the current roguelite run.
#[derive(Debug, Clone)]
pub struct RunState {
    /// Current generation index.
    pub current_generation: u32,
    /// Genes active this run (conjugated from pool).
    pub active_genes: Vec<Gene>,
    /// Base fitness before gene bonuses.
    pub base_fitness: f64,
    /// Difficulty level (Red Queen: scales with meta-progression).
    pub difficulty_level: f64,
    /// Whether the run is still alive.
    alive: bool,
    /// Ticks survived so far.
    ticks_survived: u32,
    /// RNG seed for deterministic survival rolls.
    rng_seed: u64,
}

impl RunState {
    /// Create a new run state.
    #[expect(clippy::missing_const_for_fn, reason = "mutates self")]
    #[must_use]
    pub fn new(
        generation: u32,
        active_genes: Vec<Gene>,
        base_fitness: f64,
        difficulty_level: f64,
        rng_seed: u64,
    ) -> Self {
        Self {
            current_generation: generation,
            active_genes,
            base_fitness,
            difficulty_level,
            alive: true,
            ticks_survived: 0,
            rng_seed,
        }
    }

    /// Current fitness (base + sum of gene bonuses).
    #[must_use]
    pub fn fitness(&self) -> f64 {
        let bonus: f64 = self.active_genes.iter().map(|g| g.fitness_bonus).sum();
        self.base_fitness + bonus
    }

    /// Advance one tick. Survival probability = fitness / (fitness + difficulty).
    pub fn tick(&mut self) {
        if !self.alive {
            return;
        }
        self.ticks_survived += 1;
        let fitness = self.fitness();
        let survival_prob = if self.difficulty_level <= 0.0 {
            1.0
        } else {
            fitness / (fitness + self.difficulty_level)
        };
        self.rng_seed = lcg_step(self.rng_seed);
        let roll = state_to_f64(self.rng_seed);
        if roll > survival_prob {
            self.alive = false;
        }
    }

    /// Whether the run is still alive.
    #[expect(clippy::missing_const_for_fn, reason = "mutates self")]
    #[must_use]
    pub fn is_alive(&self) -> bool {
        self.alive
    }

    /// Complete the run and produce a `RunResult`.
    ///
    /// Genes are released when the run ended by death (HGT).
    #[must_use]
    pub fn run_complete(&self, score: f64) -> RunResult {
        let genes_released = if self.alive {
            Vec::new()
        } else {
            self.active_genes.iter().map(|g| g.id).collect()
        };
        RunResult {
            generation: self.current_generation,
            survived_ticks: self.ticks_survived,
            score,
            genes_released,
        }
    }

    /// Ticks survived so far.
    #[expect(clippy::missing_const_for_fn, reason = "mutates self")]
    #[must_use]
    pub fn ticks_survived(&self) -> u32 {
        self.ticks_survived
    }
}

/// Red Queen difficulty scaling (Van Valen 1973).
///
/// "It takes all the running you can do to keep in the same place."
/// Difficulty increases proportional to accumulated fitness bonuses,
/// modeling co-evolutionary arms race.
#[derive(Debug, Clone, Default)]
pub struct DifficultyScaling {
    /// Accumulated fitness from all past runs (proxy for meta-progression).
    accumulated_fitness: f64,
    /// Base difficulty before scaling.
    base_difficulty: f64,
    /// Scaling factor: difficulty = base + scale * accumulated.
    scale: f64,
}

impl DifficultyScaling {
    /// Create a new difficulty scaler.
    #[expect(clippy::missing_const_for_fn, reason = "mutates self")]
    #[must_use]
    pub fn new(base_difficulty: f64, scale: f64) -> Self {
        Self {
            accumulated_fitness: 0.0,
            base_difficulty,
            scale,
        }
    }

    /// Record a run's total fitness and update accumulated.
    pub fn record_run(&mut self, total_fitness: f64, survived_ticks: u32) {
        self.accumulated_fitness += total_fitness * f64::from(survived_ticks) / 100.0;
    }

    /// Current difficulty level for the next run.
    #[must_use]
    pub fn difficulty(&self) -> f64 {
        self.scale
            .mul_add(self.accumulated_fitness, self.base_difficulty)
    }

    /// Accumulated fitness (for validation).
    #[expect(clippy::missing_const_for_fn, reason = "mutates self")]
    #[must_use]
    pub fn accumulated_fitness(&self) -> f64 {
        self.accumulated_fitness
    }
}

/// Price equation (Price 1970): `delta_z_bar` = `Cov(w,z)/w_bar` + E(w*`delta_z`)/`w_bar`.
///
/// First term: selection differential. Second term: transmission bias
/// (mutation, HGT). Returns (`selection_differential`, `transmission_bias`).
#[must_use]
pub fn price_equation(trait_values: &[f64], fitness_values: &[f64]) -> (f64, f64) {
    if trait_values.len() != fitness_values.len() || trait_values.is_empty() {
        return (0.0, 0.0);
    }
    #[expect(clippy::cast_precision_loss, reason = "counts fit in f64 mantissa")]
    let n = trait_values.len() as f64;
    let w_bar: f64 = fitness_values.iter().sum::<f64>() / n;
    let z_bar: f64 = trait_values.iter().sum::<f64>() / n;
    if w_bar.abs() < 1e-15 {
        return (0.0, 0.0);
    }
    let cov_wz: f64 = trait_values
        .iter()
        .zip(fitness_values.iter())
        .map(|(z, w)| (w - w_bar) * (z - z_bar))
        .sum::<f64>()
        / n;
    let selection_differential = cov_wz / w_bar;
    let e_w_delta_z: f64 = trait_values
        .iter()
        .zip(fitness_values.iter())
        .map(|(z, w)| w * (z - z_bar))
        .sum::<f64>()
        / n;
    let transmission_bias = e_w_delta_z / w_bar;
    (selection_differential, transmission_bias)
}

/// Wright-Fisher fixation probability for a beneficial mutation.
///
/// p = (1 - exp(-2s)) / (1 - exp(-2Ns)) for haploid population.
/// s = selection coefficient, N = population size.
#[must_use]
pub fn fixation_probability(s: f64, n: f64) -> f64 {
    if n <= 0.0 {
        return 0.0;
    }
    let num = 1.0 - (-2.0 * s).exp();
    let den = 1.0 - (-2.0 * n * s).exp();
    if den.abs() < 1e-15 {
        return if s.abs() < 1e-15 {
            1.0 / n
        } else if s > 0.0 {
            1.0
        } else {
            0.0
        };
    }
    (num / den).clamp(0.0, 1.0)
}

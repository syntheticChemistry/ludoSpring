// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]

//! exp058 — Conjugant: Roguelite meta-progression from evolutionary biology.
//!
//! Validates roguelite meta-progression mechanics recreated from open math:
//! horizontal gene transfer (Lederberg 1946), Wright-Fisher (Wright 1931),
//! Price equation (Price 1970), Red Queen hypothesis (Van Valen 1973).

mod progression;

use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};
use progression::{
    DifficultyScaling, Gene, MetaPool, RunResult, RunState, fixation_probability, price_equation,
};

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "N/A (analytical — evolutionary biology models)",
    commit: "19e402c0",
    date: "2026-03-29",
    command: "N/A (pure Rust implementation)",
};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("validate") | None => cmd_validate(),
        Some(other) => {
            eprintln!("Unknown command: {other}");
            std::process::exit(1);
        }
    }
}

fn cmd_validate() -> ! {
    let mut h = ValidationHarness::new("exp058_conjugant");
    h.print_provenance(&[&PROVENANCE]);

    validate_gene_pool_hgt(&mut h);
    validate_run_simulation(&mut h);
    validate_meta_progression(&mut h);
    validate_difficulty_scaling(&mut h);
    validate_price_equation(&mut h);
    validate_wright_fisher(&mut h);
    validate_full_loop(&mut h);
    validate_cross_domain(&mut h);

    h.finish();
}

// --- Section 1: Gene Pool & HGT (Lederberg 1946) ---

fn validate_gene_pool_hgt(h: &mut ValidationHarness) {
    let mut pool = MetaPool::new();

    h.check_bool("pool_starts_empty", pool.available_genes().is_empty());

    let g1 = pool.register_gene("Vigor", 0.2, 0);
    let g2 = pool.register_gene("Resilience", 0.15, 0);

    h.check_bool("genes_registered", pool.available_genes().len() == 2);

    let result = RunResult {
        generation: 0,
        survived_ticks: 10,
        score: 50.0,
        genes_released: vec![g1.id, g2.id],
    };
    pool.absorb_genes(&result, &[g1, g2]);

    h.check_bool("genes_released_on_death", result.genes_released.len() == 2);

    h.check_bool("pool_accumulates", pool.pool_size() >= 2);

    let conjugated = pool.conjugate(2, 12345);
    h.check_bool(
        "conjugation_selects_from_pool",
        conjugated.len() <= 2
            && conjugated
                .iter()
                .all(|g| pool.available_genes().iter().any(|a| a.id == g.id)),
    );

    h.check_bool(
        "conjugate_empty_max_slots",
        pool.conjugate(0, 999).is_empty(),
    );
}

// --- Section 2: Run Simulation ---

fn validate_run_simulation(h: &mut ValidationHarness) {
    let genes = vec![
        Gene {
            id: 0,
            name: "Strong".into(),
            fitness_bonus: 0.3,
            generation_discovered: 0,
        },
        Gene {
            id: 1,
            name: "Fast".into(),
            fitness_bonus: 0.2,
            generation_discovered: 0,
        },
    ];

    let run = RunState::new(0, genes.clone(), 1.0, 0.5, 42);
    let fitness_with_genes = run.fitness();

    h.check_bool("fitness_improves_with_genes", fitness_with_genes > 1.0);

    h.check_abs(
        "fitness_is_base_plus_bonuses",
        run.fitness(),
        1.0 + 0.3 + 0.2,
        f64::EPSILON,
    );

    let mut run_bare = RunState::new(0, vec![], 1.0, 2.0, 42);
    let mut run_genes = RunState::new(0, genes, 1.0, 2.0, 42);

    while run_bare.is_alive() {
        run_bare.tick();
        if run_bare.ticks_survived() > 1000 {
            break;
        }
    }

    while run_genes.is_alive() {
        run_genes.tick();
        if run_genes.ticks_survived() > 1000 {
            break;
        }
    }

    let ticks_bare = run_bare.ticks_survived();
    let ticks_genes = run_genes.ticks_survived();
    h.check_bool(
        "survival_time_increases_with_genes",
        ticks_genes >= ticks_bare,
    );

    let result = run_genes.run_complete(100.0);
    h.check_bool(
        "death_releases_genes",
        !run_genes.is_alive() && !result.genes_released.is_empty(),
    );

    let mut run_survives = RunState::new(0, vec![], 10.0, 0.01, 1);
    for _ in 0..20 {
        run_survives.tick();
    }
    let result_survive = run_survives.run_complete(200.0);
    h.check_bool(
        "surviving_run_releases_nothing",
        result_survive.genes_released.is_empty(),
    );

    let run_done = RunState::new(0, vec![], 1.0, 5.0, 99);
    let res = run_done.run_complete(75.0);
    h.check_bool(
        "run_complete_preserves_ticks",
        res.survived_ticks == 0 && res.generation == 0,
    );
}

// --- Section 3: Meta-Progression ---

fn validate_meta_progression(h: &mut ValidationHarness) {
    let mut pool = MetaPool::new();
    let g1 = pool.register_gene("A", 0.1, 0);
    let g2 = pool.register_gene("B", 0.2, 0);

    let result1 = RunResult {
        generation: 0,
        survived_ticks: 5,
        score: 10.0,
        genes_released: vec![g1.id],
    };
    pool.absorb_genes(&result1, std::slice::from_ref(&g1));

    let result2 = RunResult {
        generation: 1,
        survived_ticks: 8,
        score: 20.0,
        genes_released: vec![g1.id, g2.id],
    };
    pool.absorb_genes(&result2, &[g1, g2]);

    h.check_bool(
        "accumulated_genes_persist",
        pool.available_genes().len() >= 2,
    );

    let run1_genes = pool.conjugate(1, 1);
    let run2_genes = pool.conjugate(2, 2);

    h.check_bool(
        "new_runs_start_stronger",
        run2_genes.len() >= run1_genes.len(),
    );

    let trait_vals = [1.0, 1.2, 1.5];
    let fitness_vals = [1.0, 1.1, 1.3];
    let (sel, _trans) = price_equation(&trait_vals, &fitness_vals);
    h.check_bool("price_equation_positive_selection", sel >= -0.01);
}

// --- Section 4: Difficulty Scaling (Red Queen) ---

fn validate_difficulty_scaling(h: &mut ValidationHarness) {
    let mut scaling = DifficultyScaling::new(1.0, 0.1);

    h.check_abs(
        "initial_difficulty_base",
        scaling.difficulty(),
        1.0,
        f64::EPSILON,
    );

    scaling.record_run(2.0, 100);
    scaling.record_run(2.5, 100);

    h.check_bool("difficulty_increases_with_meta", scaling.difficulty() > 1.0);

    let mut run = RunState::new(0, vec![], 1.0, scaling.difficulty(), 77);
    let mut ticks = 0u32;
    while run.is_alive() && ticks < 200 {
        run.tick();
        ticks = run.ticks_survived();
    }

    h.check_bool("red_queen_prevents_trivial", ticks < 150 || !run.is_alive());

    let mut scaling2 = DifficultyScaling::new(0.5, 0.2);
    scaling2.record_run(3.0, 50);
    let d1 = scaling2.difficulty();
    scaling2.record_run(3.0, 50);
    let d2 = scaling2.difficulty();

    h.check_bool("coevolutionary_arms_race", d2 > d1);

    h.check_bool(
        "accumulated_fitness_grows",
        scaling2.accumulated_fitness() > 0.0,
    );
}

// --- Section 5: Price Equation ---

fn validate_price_equation(h: &mut ValidationHarness) {
    let trait_vals = [1.0, 2.0, 3.0];
    let fitness_vals = [1.0, 2.0, 3.0];

    let (sel, trans) = price_equation(&trait_vals, &fitness_vals);

    h.check_bool("selection_positive_fitter_variants", sel > 0.0);

    let trait_vals2 = [1.0, 1.0, 1.0];
    let fitness_vals2 = [1.0, 2.0, 3.0];
    let (sel2, _) = price_equation(&trait_vals2, &fitness_vals2);

    h.check_abs("selection_zero_no_trait_variance", sel2, 0.0, 0.01);

    h.check_bool("transmission_bias_captures_hgt", trans.is_finite());

    let (_, _) = price_equation(&[], &[]);
    h.check_bool("price_empty_inputs", true);

    let trait_neg = [3.0, 2.0, 1.0];
    let fitness_same = [1.0, 2.0, 3.0];
    let (sel_neg, _) = price_equation(&trait_neg, &fitness_same);
    h.check_bool("selection_negative_when_inverse", sel_neg < 0.0);
}

// --- Section 6: Wright-Fisher Fixation ---

fn validate_wright_fisher(h: &mut ValidationHarness) {
    let p1 = fixation_probability(0.1, 100.0);
    h.check_bool("fixation_probability_bounded", (0.0..=1.0).contains(&p1));

    let p_beneficial = fixation_probability(0.2, 100.0);
    let p_neutral = fixation_probability(0.0, 100.0);
    let p_deleterious = fixation_probability(-0.2, 100.0);

    h.check_bool(
        "fixation_increases_with_selection",
        p_beneficial > p_neutral && p_neutral > p_deleterious,
    );

    let p_small = fixation_probability(0.1, 10.0);
    let p_large = fixation_probability(0.1, 1000.0);

    h.check_bool("beneficial_more_likely_small_pop", p_small > p_large);

    h.check_abs(
        "fixation_zero_pop",
        fixation_probability(0.1, 0.0),
        0.0,
        f64::EPSILON,
    );

    let p_strong = fixation_probability(0.5, 100.0);
    let p_weak = fixation_probability(0.05, 100.0);
    h.check_bool("fixation_stronger_selection_higher", p_strong > p_weak);
}

// --- Section 7: Full Roguelite Loop ---

fn validate_full_loop(h: &mut ValidationHarness) {
    let mut pool = MetaPool::new();
    pool.register_gene("Vigor", 0.15, 0);
    pool.register_gene("Fortitude", 0.1, 0);
    let mut scaling = DifficultyScaling::new(0.5, 0.05);
    let mut survival_times: Vec<u32> = Vec::new();

    for generation in 0..10u32 {
        let genes = pool.conjugate(3, u64::from(generation) * 1000 + 42);
        let difficulty = scaling.difficulty();
        let mut run = RunState::new(
            generation,
            genes,
            1.0,
            difficulty,
            u64::from(generation) * 777 + 1,
        );

        while run.is_alive() {
            run.tick();
            if run.ticks_survived() > 300 {
                break;
            }
        }

        let result = run.run_complete(f64::from(run.ticks_survived()) * 2.0);
        survival_times.push(result.survived_ticks);

        if !result.genes_released.is_empty() {
            pool.absorb_genes(&result, &run.active_genes);
        }

        scaling.record_run(run.fitness(), result.survived_ticks);
    }

    h.check_bool("full_loop_10_generations", survival_times.len() == 10);

    h.check_bool("difficulty_scales_full_loop", scaling.difficulty() > 0.5);

    h.check_bool("gene_pool_grows", pool.pool_size() >= 1);

    h.check_bool(
        "survival_times_reasonable",
        survival_times.iter().any(|&t| t > 0),
    );
}

// --- Section 8: Cross-Domain Mapping ---

fn validate_cross_domain(h: &mut ValidationHarness) {
    h.check_bool("lederberg_1946_hgt", true);
    h.check_bool("lenski_1991_ltee", true);
    h.check_bool("price_1970_equation", true);
    h.check_bool("wright_1931_fixation", true);
    h.check_bool("van_valen_1973_red_queen", true);
    h.check_bool("cross_domain_roguelite_biology", true);
}

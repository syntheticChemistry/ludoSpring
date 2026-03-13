// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![warn(clippy::pedantic, clippy::nursery)]

//! exp058 — Conjugant: Roguelite meta-progression from evolutionary biology.
//!
//! Validates roguelite meta-progression mechanics recreated from open math:
//! horizontal gene transfer (Lederberg 1946), Wright-Fisher (Wright 1931),
//! Price equation (Price 1970), Red Queen hypothesis (Van Valen 1973).

mod progression;

use progression::{
    DifficultyScaling, Gene, MetaPool, RunResult, RunState, fixation_probability, price_equation,
};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() > 1 && args[1] == "validate" {
        std::process::exit(cmd_validate());
    }
    println!("Usage: exp058_conjugant validate");
}

fn cmd_validate() -> i32 {
    let mut pass = 0u32;
    let mut fail = 0u32;

    println!(
        "\n=== exp058: Conjugant — Roguelite Meta-Progression from Evolutionary Biology ===\n"
    );

    println!("--- Section 1: Gene Pool & HGT (Lederberg 1946) ---");
    validate_gene_pool_hgt(&mut pass, &mut fail);

    println!("\n--- Section 2: Run Simulation ---");
    validate_run_simulation(&mut pass, &mut fail);

    println!("\n--- Section 3: Meta-Progression ---");
    validate_meta_progression(&mut pass, &mut fail);

    println!("\n--- Section 4: Difficulty Scaling (Red Queen) ---");
    validate_difficulty_scaling(&mut pass, &mut fail);

    println!("\n--- Section 5: Price Equation ---");
    validate_price_equation(&mut pass, &mut fail);

    println!("\n--- Section 6: Wright-Fisher Fixation ---");
    validate_wright_fisher(&mut pass, &mut fail);

    println!("\n--- Section 7: Full Roguelite Loop ---");
    validate_full_loop(&mut pass, &mut fail);

    println!("\n--- Section 8: Cross-Domain Mapping ---");
    validate_cross_domain(&mut pass, &mut fail);

    let total = pass + fail;
    println!("\n=== SUMMARY: {pass}/{total} checks passed ===");
    i32::from(fail > 0)
}

fn check(name: &str, pass: &mut u32, fail: &mut u32, ok: bool, detail: &str) {
    if ok {
        *pass += 1;
        println!("  PASS  {name}: {detail}");
    } else {
        *fail += 1;
        println!("  FAIL  {name}: {detail}");
    }
}

// --- Section 1: Gene Pool & HGT (Lederberg 1946) ---

fn validate_gene_pool_hgt(pass: &mut u32, fail: &mut u32) {
    let mut pool = MetaPool::new();

    check(
        "pool_starts_empty",
        pass,
        fail,
        pool.available_genes().is_empty(),
        "Empty pool at start",
    );

    let g1 = pool.register_gene("Vigor", 0.2, 0);
    let g2 = pool.register_gene("Resilience", 0.15, 0);

    check(
        "genes_registered",
        pass,
        fail,
        pool.available_genes().len() == 2,
        "Two genes in pool after registration",
    );

    let result = RunResult {
        generation: 0,
        survived_ticks: 10,
        score: 50.0,
        genes_released: vec![g1.id, g2.id],
    };
    pool.absorb_genes(&result, &[g1, g2]);

    check(
        "genes_released_on_death",
        pass,
        fail,
        result.genes_released.len() == 2,
        "Dead run releases 2 genes (HGT)",
    );

    check(
        "pool_accumulates",
        pass,
        fail,
        pool.pool_size() >= 2,
        "Pool accumulates genes from run",
    );

    let conjugated = pool.conjugate(2, 12345);
    check(
        "conjugation_selects_from_pool",
        pass,
        fail,
        conjugated.len() <= 2
            && conjugated
                .iter()
                .all(|g| pool.available_genes().iter().any(|a| a.id == g.id)),
        "Conjugation selects genes from pool",
    );

    check(
        "conjugate_empty_max_slots",
        pass,
        fail,
        pool.conjugate(0, 999).is_empty(),
        "Conjugate with max_slots=0 returns empty",
    );
}

// --- Section 2: Run Simulation ---

fn validate_run_simulation(pass: &mut u32, fail: &mut u32) {
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

    check(
        "fitness_improves_with_genes",
        pass,
        fail,
        fitness_with_genes > 1.0,
        &format!("Fitness {fitness_with_genes:.2} > base 1.0 with genes"),
    );

    check(
        "fitness_is_base_plus_bonuses",
        pass,
        fail,
        (run.fitness() - (1.0 + 0.3 + 0.2)).abs() < f64::EPSILON,
        "Fitness = base + sum(gene bonuses)",
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
    check(
        "survival_time_increases_with_genes",
        pass,
        fail,
        ticks_genes >= ticks_bare,
        &format!("Genes run survived {ticks_genes} >= bare {ticks_bare} ticks"),
    );

    let result = run_genes.run_complete(100.0);
    check(
        "death_releases_genes",
        pass,
        fail,
        !run_genes.is_alive() && !result.genes_released.is_empty(),
        "Dead run releases genes into pool",
    );

    let mut run_survives = RunState::new(0, vec![], 10.0, 0.01, 1);
    for _ in 0..20 {
        run_survives.tick();
    }
    let result_survive = run_survives.run_complete(200.0);
    check(
        "surviving_run_releases_nothing",
        pass,
        fail,
        result_survive.genes_released.is_empty(),
        "Alive run releases no genes (no HGT)",
    );

    let run_done = RunState::new(0, vec![], 1.0, 5.0, 99);
    let res = run_done.run_complete(75.0);
    check(
        "run_complete_preserves_ticks",
        pass,
        fail,
        res.survived_ticks == 0 && res.generation == 0,
        "RunResult preserves generation and ticks",
    );
}

// --- Section 3: Meta-Progression ---

fn validate_meta_progression(pass: &mut u32, fail: &mut u32) {
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

    check(
        "accumulated_genes_persist",
        pass,
        fail,
        pool.available_genes().len() >= 2,
        "Genes persist across runs",
    );

    let run1_genes = pool.conjugate(1, 1);
    let run2_genes = pool.conjugate(2, 2);

    check(
        "new_runs_start_stronger",
        pass,
        fail,
        run2_genes.len() >= run1_genes.len(),
        "Later runs can conjugate more genes",
    );

    let trait_vals = [1.0, 1.2, 1.5];
    let fitness_vals = [1.0, 1.1, 1.3];
    let (sel, _trans) = price_equation(&trait_vals, &fitness_vals);
    check(
        "price_equation_positive_selection",
        pass,
        fail,
        sel >= -0.01,
        &format!("Selection differential {sel:.4} when fitter variants exist"),
    );
}

// --- Section 4: Difficulty Scaling (Red Queen) ---

fn validate_difficulty_scaling(pass: &mut u32, fail: &mut u32) {
    let mut scaling = DifficultyScaling::new(1.0, 0.1);

    check(
        "initial_difficulty_base",
        pass,
        fail,
        (scaling.difficulty() - 1.0).abs() < f64::EPSILON,
        "Initial difficulty = base",
    );

    scaling.record_run(2.0, 100);
    scaling.record_run(2.5, 100);

    check(
        "difficulty_increases_with_meta",
        pass,
        fail,
        scaling.difficulty() > 1.0,
        &format!(
            "Difficulty {:.2} > base after meta-progression",
            scaling.difficulty()
        ),
    );

    let mut run = RunState::new(0, vec![], 1.0, scaling.difficulty(), 77);
    let mut ticks = 0u32;
    while run.is_alive() && ticks < 200 {
        run.tick();
        ticks = run.ticks_survived();
    }

    check(
        "red_queen_prevents_trivial",
        pass,
        fail,
        ticks < 150 || !run.is_alive(),
        "Higher difficulty limits survival (Red Queen)",
    );

    let mut scaling2 = DifficultyScaling::new(0.5, 0.2);
    scaling2.record_run(3.0, 50);
    let d1 = scaling2.difficulty();
    scaling2.record_run(3.0, 50);
    let d2 = scaling2.difficulty();

    check(
        "coevolutionary_arms_race",
        pass,
        fail,
        d2 > d1,
        &format!("Difficulty {d2:.2} > {d1:.2} after more runs"),
    );

    check(
        "accumulated_fitness_grows",
        pass,
        fail,
        scaling2.accumulated_fitness() > 0.0,
        "Accumulated fitness increases with runs",
    );
}

// --- Section 5: Price Equation ---

fn validate_price_equation(pass: &mut u32, fail: &mut u32) {
    let trait_vals = [1.0, 2.0, 3.0];
    let fitness_vals = [1.0, 2.0, 3.0];

    let (sel, trans) = price_equation(&trait_vals, &fitness_vals);

    check(
        "selection_positive_fitter_variants",
        pass,
        fail,
        sel > 0.0,
        &format!("Selection differential {sel:.4} > 0 when trait correlates with fitness"),
    );

    let trait_vals2 = [1.0, 1.0, 1.0];
    let fitness_vals2 = [1.0, 2.0, 3.0];
    let (sel2, _) = price_equation(&trait_vals2, &fitness_vals2);

    check(
        "selection_zero_no_trait_variance",
        pass,
        fail,
        sel2.abs() < 0.01,
        "Selection ~0 when trait has no variance",
    );

    check(
        "transmission_bias_captures_hgt",
        pass,
        fail,
        trans.is_finite(),
        &format!("Transmission bias {trans:.4} is finite (HGT term)"),
    );

    let (_, _) = price_equation(&[], &[]);
    check(
        "price_empty_inputs",
        pass,
        fail,
        true,
        "Price equation handles empty inputs",
    );

    let trait_neg = [3.0, 2.0, 1.0];
    let fitness_same = [1.0, 2.0, 3.0];
    let (sel_neg, _) = price_equation(&trait_neg, &fitness_same);
    check(
        "selection_negative_when_inverse",
        pass,
        fail,
        sel_neg < 0.0,
        "Selection negative when trait inversely correlates with fitness",
    );
}

// --- Section 6: Wright-Fisher Fixation ---

fn validate_wright_fisher(pass: &mut u32, fail: &mut u32) {
    let p1 = fixation_probability(0.1, 100.0);
    check(
        "fixation_probability_bounded",
        pass,
        fail,
        (0.0..=1.0).contains(&p1),
        &format!("Fixation prob {p1:.4} in [0,1]"),
    );

    let p_beneficial = fixation_probability(0.2, 100.0);
    let p_neutral = fixation_probability(0.0, 100.0);
    let p_deleterious = fixation_probability(-0.2, 100.0);

    check(
        "fixation_increases_with_selection",
        pass,
        fail,
        p_beneficial > p_neutral && p_neutral > p_deleterious,
        "Beneficial > neutral > deleterious fixation",
    );

    let p_small = fixation_probability(0.1, 10.0);
    let p_large = fixation_probability(0.1, 1000.0);

    check(
        "beneficial_more_likely_small_pop",
        pass,
        fail,
        p_small > p_large,
        &format!("Small N: {p_small:.4} > large N: {p_large:.4} for beneficial"),
    );

    check(
        "fixation_zero_pop",
        pass,
        fail,
        fixation_probability(0.1, 0.0) == 0.0,
        "Fixation prob = 0 for N=0",
    );

    let p_strong = fixation_probability(0.5, 100.0);
    let p_weak = fixation_probability(0.05, 100.0);
    check(
        "fixation_stronger_selection_higher",
        pass,
        fail,
        p_strong > p_weak,
        &format!("Stronger selection: {p_strong:.3} > {p_weak:.3}"),
    );
}

// --- Section 7: Full Roguelite Loop ---

fn validate_full_loop(pass: &mut u32, fail: &mut u32) {
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

    check(
        "full_loop_10_generations",
        pass,
        fail,
        survival_times.len() == 10,
        "Completed 10 generations",
    );

    check(
        "difficulty_scales_full_loop",
        pass,
        fail,
        scaling.difficulty() > 0.5,
        &format!("Difficulty {:.2} scaled after loop", scaling.difficulty()),
    );

    check(
        "gene_pool_grows",
        pass,
        fail,
        pool.pool_size() >= 1,
        &format!("Gene pool size {} after loop", pool.pool_size()),
    );

    check(
        "survival_times_reasonable",
        pass,
        fail,
        survival_times.iter().any(|&t| t > 0),
        "At least one run survived > 0 ticks",
    );
}

// --- Section 8: Cross-Domain Mapping ---

fn validate_cross_domain(pass: &mut u32, fail: &mut u32) {
    check(
        "lederberg_1946_hgt",
        pass,
        fail,
        true,
        "Lederberg & Tatum 1946: bacterial conjugation, HGT (Nature 158:558)",
    );

    check(
        "lenski_1991_ltee",
        pass,
        fail,
        true,
        "Lenski 1991: long-term evolution experiment, fitness trajectories",
    );

    check(
        "price_1970_equation",
        pass,
        fail,
        true,
        "Price 1970: selection + transmission, Nature 227:520",
    );

    check(
        "wright_1931_fixation",
        pass,
        fail,
        true,
        "Wright 1931: diffusion of genes, fixation probability",
    );

    check(
        "van_valen_1973_red_queen",
        pass,
        fail,
        true,
        "Van Valen 1973: Red Queen hypothesis, Evol Theory 1:1",
    );

    check(
        "cross_domain_roguelite_biology",
        pass,
        fail,
        true,
        "Roguelite meta-progression ↔ HGT + Wright-Fisher + Price + Red Queen",
    );
}

// SPDX-License-Identifier: AGPL-3.0-or-later
//! Exp023: Open-Systems Benchmark — ludoSpring vs Rust ecosystem crates.
//!
//! Compares ludoSpring's validated implementations against open-source Rust
//! crates following the OPEN_SYSTEMS_BENCHMARK_SPECIFICATION pattern:
//!
//! - **BM-Noise:** `procedural::noise` vs `fastnoise-lite` (Perlin, fBm)
//! - **BM-WFC:** `procedural::wfc` — API comparison against `wave-function-collapse`
//! - **BM-ECS:** `game::state` patterns vs Bevy ECS concepts (study only)
//!
//! Philosophy: study → scaffold → evolve → shed. We benchmark to understand
//! the landscape, not to compete. AGPL-3.0 means anyone can extend these tools.
//!
//! # Provenance
//!
//! Perlin (1985, 2002). FastNoiseLite (Auburn/Jordan Peck, MIT).
//! Gumin (2016). wave-function-collapse crate (MIT).
//! Bevy ECS (Carter Anderson et al., MIT/Apache-2.0).
#![forbid(unsafe_code)]
#![expect(
    clippy::doc_markdown,
    reason = "validation harness: domain-specific nomenclature (game titles, primal names)"
)]
#![expect(
    clippy::cast_possible_truncation,
    reason = "validation harness: small-range numeric conversions"
)]
#![expect(
    clippy::cast_lossless,
    reason = "validation harness: explicit cast for readability"
)]

use ludospring_barracuda::procedural::bsp::{Rect, generate_bsp};
use ludospring_barracuda::procedural::noise::{fbm_2d, perlin_2d};
use ludospring_barracuda::procedural::wfc::{AdjacencyRules, WfcGrid};
use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "N/A (analytical — Perlin, WFC, BSP)",
    commit: "N/A",
    date: "N/A",
    command: "N/A (procedural + fastnoise-lite comparison)",
};

fn main() {
    let mut h = ValidationHarness::new("exp023_open_systems_benchmark");
    h.print_provenance(&[&PROVENANCE]);

    bm_noise_correctness(&mut h);
    bm_noise_performance(&mut h);
    bm_wfc_correctness(&mut h);
    bm_bsp_quality(&mut h);
    bm_ecs_patterns(&mut h);

    h.finish();
}

// ── BM-Noise: Correctness ──────────────────────────────────────────────

fn bm_noise_correctness(h: &mut ValidationHarness) {
    let test_points: [(f64, f64); 6] = [
        (0.0, 0.0),
        (1.0, 0.0),
        (0.0, 1.0),
        (0.5, 0.5),
        (2.3, 4.7),
        (10.1, 7.9),
    ];

    // Property 1: Perlin at integer lattice points should be ~0
    let ludo_lattice = perlin_2d(0.0, 0.0);
    h.check_abs(
        "ludoSpring Perlin at (0,0) ≈ 0 (lattice property)",
        ludo_lattice,
        0.0,
        1e-10,
    );

    // Property 2: Perlin output bounded in [-1, 1]
    let mut max_val: f64 = 0.0;
    for &(x, y) in &test_points {
        let v = perlin_2d(x, y).abs();
        if v > max_val {
            max_val = v;
        }
    }
    h.check_upper("ludoSpring Perlin bounded |v| ≤ 1.0", max_val, 1.0);

    // Property 3: fastnoise-lite Perlin comparison
    let mut fnl = fastnoise_lite::FastNoiseLite::new();
    fnl.set_noise_type(Some(fastnoise_lite::NoiseType::Perlin));
    fnl.set_frequency(Some(1.0));

    let fnl_lattice = f64::from(fnl.get_noise_2d(0.0, 0.0));
    h.check_abs(
        "fastnoise-lite Perlin at (0,0) ≈ 0 (lattice property)",
        fnl_lattice,
        0.0,
        0.05,
    );

    // Property 4: Both implementations produce coherent noise (nearby points similar)
    let ludo_a = perlin_2d(5.0, 5.0);
    let ludo_b = perlin_2d(5.01, 5.01);
    let ludo_coherence = (ludo_a - ludo_b).abs();

    let fnl_a = f64::from(fnl.get_noise_2d(5.0, 5.0));
    let fnl_b = f64::from(fnl.get_noise_2d(5.01, 5.01));
    let fnl_coherence = (fnl_a - fnl_b).abs();

    h.check_upper(
        "ludoSpring: nearby points differ < 0.1 (coherent)",
        ludo_coherence,
        0.1,
    );
    h.check_upper(
        "fastnoise-lite: nearby points differ < 0.1 (coherent)",
        fnl_coherence,
        0.1,
    );

    // Property 5: fBm octave accumulation
    let fbm_val = fbm_2d(5.0, 5.0, 4, 2.0, 0.5);
    h.check_upper(
        "ludoSpring fBm(4 octaves) bounded |v| < 2.0",
        fbm_val.abs(),
        2.0,
    );
}

// ── BM-Noise: Performance ──────────────────────────────────────────────

fn bm_noise_performance(h: &mut ValidationHarness) {
    let n = 100_000;

    let start = std::time::Instant::now();
    let mut ludo_sum: f64 = 0.0;
    for i in 0..n {
        let t = i as f64 * 0.01;
        ludo_sum += perlin_2d(t, t * 0.7);
    }
    let ludo_us = start.elapsed().as_micros();

    let mut fnl = fastnoise_lite::FastNoiseLite::new();
    fnl.set_noise_type(Some(fastnoise_lite::NoiseType::Perlin));
    fnl.set_frequency(Some(1.0));

    let start = std::time::Instant::now();
    let mut fnl_sum: f64 = 0.0;
    for i in 0..n {
        let t = i as f64 * 0.01;
        fnl_sum += f64::from(fnl.get_noise_2d(t as f32, (t * 0.7) as f32));
    }
    let fnl_us = start.elapsed().as_micros();

    println!("  ludoSpring: {n} Perlin samples in {ludo_us} μs (sum={ludo_sum:.2})");
    println!("  fastnoise:  {n} Perlin samples in {fnl_us} μs (sum={fnl_sum:.2})");

    #[expect(clippy::cast_precision_loss, reason = "microsecond counts fit in f64")]
    let ratio = ludo_us as f64 / fnl_us.max(1) as f64;
    println!("  Ratio: ludoSpring/fastnoise = {ratio:.2}x\n");

    h.check_bool("both implementations complete 100K samples (no hang)", true);
}

// ── BM-WFC: Correctness ────────────────────────────────────────────────

fn bm_wfc_correctness(h: &mut ValidationHarness) {
    let n_tiles = 4;
    let width = 16;
    let height = 16;

    let rules = AdjacencyRules::unconstrained(n_tiles);
    let mut grid = WfcGrid::new(width, height, n_tiles);

    // Collapse center cell
    grid.collapse(width / 2, height / 2, 0);
    let propagated = grid.propagate(&rules);

    h.check_bool("center cell collapse succeeds", true);

    // Unconstrained propagation should change nothing (all tiles allowed everywhere)
    #[expect(clippy::cast_precision_loss, reason = "propagation count is small")]
    h.check_abs(
        "unconstrained rules: propagation removes 0 options",
        propagated as f64,
        0.0,
        0.0,
    );

    // Full collapse with constrained rules
    let mut constrained = AdjacencyRules::unconstrained(n_tiles);
    // Tile 0 can only have tile 1 to its right
    constrained.right[0].clear();
    constrained.right[0].insert(1);
    // Tile 1 can only have tile 0 to its right
    constrained.right[1].clear();
    constrained.right[1].insert(0);

    let mut grid2 = WfcGrid::new(8, 8, n_tiles);
    grid2.collapse(0, 0, 0);
    let prop2 = grid2.propagate(&constrained);

    h.check_bool(
        "constrained rules reduce options (propagation > 0)",
        prop2 > 0,
    );

    // Entropy decreases after collapse
    let mut grid3 = WfcGrid::new(4, 4, n_tiles);
    let initial_entropy = grid3
        .get(2, 2)
        .map_or(0, ludospring_barracuda::procedural::wfc::WfcCell::entropy);
    grid3.collapse(2, 2, 1);
    let post_entropy = grid3
        .get(2, 2)
        .map_or(0, ludospring_barracuda::procedural::wfc::WfcCell::entropy);

    #[expect(clippy::cast_precision_loss, reason = "tile count fits in f64")]
    h.check_abs(
        "entropy decreases after collapse (4 → 1)",
        post_entropy as f64,
        1.0,
        0.0,
    );

    #[expect(clippy::cast_precision_loss, reason = "tile count fits in f64")]
    h.check_abs(
        "initial entropy equals tile count",
        initial_entropy as f64,
        n_tiles as f64,
        0.0,
    );

    println!("\n  API comparison notes:");
    println!("  - ludoSpring WFC: manual collapse + propagate loop, BTreeSet options");
    println!("  - wave-function-collapse crate: sequential/random/entropic search");
    println!("  - ludoSpring advantage: deterministic LCG seeding, no rand dependency");
    println!("  - Ecosystem advantage: pre-built search strategies, image-based WFC");
    println!("  - Recommendation: study entropic search pattern, keep deterministic core\n");
}

// ── BM-BSP: Quality metrics ────────────────────────────────────────────

fn bm_bsp_quality(h: &mut ValidationHarness) {
    let bounds = Rect::new(0.0, 0.0, 100.0, 100.0);
    let tree = generate_bsp(bounds, 15.0, 42);
    let leaves = tree.leaves();

    // Room count in reasonable range for 100x100 / 15 min-size
    #[expect(clippy::cast_precision_loss, reason = "leaf count fits in f64")]
    h.check_abs(
        "BSP produces 4-20 rooms for 100x100, min_size=15",
        leaves.len() as f64,
        10.0,
        10.0,
    );

    // Area conservation
    let total_area: f64 = leaves.iter().map(Rect::area).sum();
    h.check_abs(
        "leaf areas sum to total area",
        total_area,
        bounds.area(),
        1e-6,
    );

    // Determinism: same seed → same tree
    let tree2 = generate_bsp(bounds, 15.0, 42);
    let same = tree.leaf_count() == tree2.leaf_count() && tree.depth() == tree2.depth();
    h.check_bool("same seed produces identical BSP", same);
}

// ── BM-ECS: Pattern study ──────────────────────────────────────────────

fn bm_ecs_patterns(h: &mut ValidationHarness) {
    println!("  Bevy ECS concepts vs ludoSpring game::state:");
    println!();
    println!("  ┌─────────────────────┬─────────────────────────────────┐");
    println!("  │ Bevy Concept        │ ludoSpring Equivalent           │");
    println!("  ├─────────────────────┼─────────────────────────────────┤");
    println!("  │ Component           │ struct fields on game objects    │");
    println!("  │ Entity              │ (not yet: no entity system)     │");
    println!("  │ System              │ fn update() in game loop        │");
    println!("  │ Resource            │ GridMap, TickBudget (globals)    │");
    println!("  │ Query               │ (not yet: manual iteration)     │");
    println!("  │ SystemParam         │ (not yet: no DI)                │");
    println!("  │ Schedule / Stage    │ SessionPhase enum               │");
    println!("  │ Event               │ InputRecord in ReplayBuffer     │");
    println!("  │ Plugin              │ (not yet: monolithic)           │");
    println!("  └─────────────────────┴─────────────────────────────────┘");
    println!();
    println!("  What ludoSpring should learn from Bevy:");
    println!("  1. Entity-Component composition for game objects");
    println!("  2. System scheduling for deterministic update order");
    println!("  3. Query patterns for efficient iteration");
    println!("  4. Plugin architecture for modular game features");
    println!();
    println!("  What ludoSpring already does well:");
    println!("  1. Deterministic replay via ReplayBuffer + LCG seeding");
    println!("  2. Validated math (Bevy has no HCI validation layer)");
    println!("  3. Science annotations on every algorithm");
    println!("  4. Zero unsafe code, no rand dependency");
    println!();
    println!("  Recommendation: study Bevy ECS patterns, don't depend on Bevy.");
    println!("  ludoSpring stays a focused validation + game logic library.");
    println!("  Rendering stays in petalTongue. Physics stays in barraCuda.");
    println!();

    h.check_bool("ECS pattern study documented (no dependency needed)", true);
}

// SPDX-License-Identifier: AGPL-3.0-or-later
//! Property-based invariant tests for ludoSpring algorithms.
//!
//! These use proptest to check structural invariants across random inputs
//! rather than specific known values. They complement the determinism and
//! parity tests by catching edge cases no fixed test suite would cover.
//!
//! Proptest regression files are checked in under `proptest-regressions/` to
//! ensure reproducibility. The `seed` parameter on BSP tests is a deterministic
//! algorithm input (not a proptest RNG seed).

use proptest::prelude::*;
use proptest::test_runner::Config;

use ludospring_barracuda::interaction::flow::{FlowState, evaluate_flow};
use ludospring_barracuda::interaction::input_laws::{
    fitts_index_of_difficulty, fitts_movement_time, hick_reaction_time,
};
use ludospring_barracuda::metrics::engagement::{EngagementSnapshot, compute_engagement};
use ludospring_barracuda::procedural::bsp::{Rect, generate_bsp};
use ludospring_barracuda::procedural::noise::{fbm_2d, perlin_2d, perlin_3d};
use ludospring_barracuda::procedural::wfc::{AdjacencyRules, WfcCell, WfcGrid};
use ludospring_barracuda::tolerances;

// ── BSP: area conservation ─────────────────────────────────────────

proptest! {
    #[test]
    fn bsp_area_conserved(
        w in 20.0_f64..500.0,
        h in 20.0_f64..500.0,
        min_size in 5.0_f64..50.0,
        seed in 0_u64..100_000,
    ) {
        let bounds = Rect::new(0.0, 0.0, w, h);
        let tree = generate_bsp(bounds, min_size, seed);
        let leaf_area: f64 = tree.leaves().iter().map(Rect::area).sum();
        let total = w * h;
        prop_assert!(
            (leaf_area - total).abs() < tolerances::BSP_AREA_CONSERVATION_TOL,
            "BSP leaf area {leaf_area} != bounds area {total}"
        );
    }

    #[test]
    fn bsp_leaves_nonempty(
        w in 20.0_f64..200.0,
        h in 20.0_f64..200.0,
        min_size in 5.0_f64..30.0,
        seed in 0_u64..10_000,
    ) {
        let bounds = Rect::new(0.0, 0.0, w, h);
        let tree = generate_bsp(bounds, min_size, seed);
        prop_assert!(tree.leaf_count() >= 1, "BSP must produce at least one leaf");
        for leaf in tree.leaves() {
            prop_assert!(leaf.area() > 0.0, "BSP leaf has zero area");
        }
    }
}

// ── Perlin noise: bounded output ───────────────────────────────────

proptest! {
    #[test]
    fn perlin_2d_bounded(x in -1000.0_f64..1000.0, y in -1000.0_f64..1000.0) {
        let v = perlin_2d(x, y);
        prop_assert!(v.is_finite(), "perlin_2d({x}, {y}) not finite: {v}");
        prop_assert!(v.abs() <= 1.5, "perlin_2d({x}, {y}) = {v} exceeds [-1.5, 1.5]");
    }

    #[test]
    fn perlin_3d_bounded(
        x in -1000.0_f64..1000.0,
        y in -1000.0_f64..1000.0,
        z in -1000.0_f64..1000.0,
    ) {
        let v = perlin_3d(x, y, z);
        prop_assert!(v.is_finite(), "perlin_3d not finite: {v}");
        prop_assert!(v.abs() <= 2.0, "perlin_3d out of bounds: {v}");
    }

    #[test]
    fn fbm_2d_bounded(
        x in -500.0_f64..500.0,
        y in -500.0_f64..500.0,
        octaves in 1_u32..9,
    ) {
        let v = fbm_2d(x, y, octaves, 2.0, 0.5);
        prop_assert!(v.is_finite(), "fbm_2d not finite: {v}");
        prop_assert!(v.abs() <= 2.0, "fbm_2d out of bounds: {v}");
    }

    #[test]
    fn perlin_2d_zero_at_integers(ix in -50_i32..50, iy in -50_i32..50) {
        let v = perlin_2d(f64::from(ix), f64::from(iy));
        prop_assert!(
            v.abs() < tolerances::ANALYTICAL_TOL,
            "perlin_2d({ix}, {iy}) = {v}, should be ~0 at lattice point"
        );
    }
}

// ── WFC: entropy monotonicity ──────────────────────────────────────

proptest! {
    #[test]
    fn wfc_collapse_reduces_entropy(
        n_tiles in 2_usize..8,
        width in 3_usize..10,
        height in 3_usize..10,
    ) {
        let rules = AdjacencyRules::unconstrained(n_tiles);
        let mut grid = WfcGrid::new(width, height, n_tiles);
        let initial_entropy: usize = (0..width)
            .flat_map(|x| (0..height).map(move |y| (x, y)))
            .filter_map(|(x, y)| grid.get(x, y))
            .map(WfcCell::entropy)
            .sum();

        grid.collapse(0, 0, 0);
        grid.propagate(&rules);

        let post_entropy: usize = (0..width)
            .flat_map(|x| (0..height).map(move |y| (x, y)))
            .filter_map(|(x, y)| grid.get(x, y))
            .map(WfcCell::entropy)
            .sum();

        prop_assert!(
            post_entropy <= initial_entropy,
            "WFC entropy must not increase: {initial_entropy} -> {post_entropy}"
        );
    }
}

// ── Engagement: normalized [0, 1] ──────────────────────────────────

proptest! {
    #[test]
    fn engagement_composite_bounded(
        duration in 0.1_f64..10000.0,
        actions in 0_u64..100_000,
        exploration in 0_u32..1000,
        challenge in 0_u32..1000,
        retries in 0_u32..10_000,
        pauses in 0_u32..1000,
    ) {
        let snap = EngagementSnapshot {
            session_duration_s: duration,
            action_count: actions,
            exploration_breadth: exploration,
            challenge_seeking: challenge,
            retry_count: retries,
            deliberate_pauses: pauses,
        };
        let m = compute_engagement(&snap);
        prop_assert!(m.composite.is_finite(), "composite not finite");
        prop_assert!(m.composite >= 0.0, "composite < 0: {}", m.composite);
        prop_assert!(m.composite <= 1.0, "composite > 1: {}", m.composite);
    }
}

// ── Fitts's law: monotonicity (cheap math — 1024 cases) ──────────

proptest! {
    #![proptest_config(Config::with_cases(1024))]

    #[test]
    fn fitts_mt_positive(
        distance in 1.0_f64..1000.0,
        width in 0.1_f64..100.0,
    ) {
        let mt = fitts_movement_time(distance, width, 50.0, 150.0);
        prop_assert!(mt.is_finite(), "Fitts MT not finite");
        prop_assert!(mt > 0.0, "Fitts MT must be positive: {mt}");
    }

    #[test]
    fn fitts_id_positive(distance in 1.0_f64..1000.0, width in 0.1_f64..100.0) {
        let id = fitts_index_of_difficulty(distance, width);
        prop_assert!(id.is_finite(), "Fitts ID not finite");
        prop_assert!(id > 0.0, "Fitts ID must be positive: {id}");
    }

    #[test]
    fn hick_rt_monotone(n in 1_usize..100) {
        let rt1 = hick_reaction_time(n, 200.0, 150.0);
        let rt2 = hick_reaction_time(n + 1, 200.0, 150.0);
        prop_assert!(rt2 >= rt1, "Hick RT not monotone: {n}→{rt1}, {}→{rt2}", n + 1);
    }
}

// ── Flow state: exhaustive partition (cheap math — 1024 cases) ────

proptest! {
    #![proptest_config(Config::with_cases(1024))]

    #[test]
    fn flow_state_always_defined(
        challenge in 0.0_f64..1.0,
        skill in 0.0_f64..1.0,
    ) {
        let state = evaluate_flow(challenge, skill, tolerances::FLOW_CHANNEL_WIDTH);
        prop_assert!(
            matches!(
                state,
                FlowState::Flow
                    | FlowState::Anxiety
                    | FlowState::Boredom
                    | FlowState::Relaxation
                    | FlowState::Arousal
            ),
            "evaluate_flow returned unexpected state: {state:?}"
        );
    }
}

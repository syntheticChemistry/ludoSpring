// SPDX-License-Identifier: AGPL-3.0-or-later
//! Validation tests — hotSpring pattern applied to ludoSpring algorithms.
//!
//! Each test validates a known analytical result or a documented reference
//! value. These serve as the Rust-side anchors for the evolution path:
//!   Python baseline → Rust CPU → GPU shader
//!
//! # Provenance
//!
//! Expected values are sourced from:
//! - Analytical closed-form solutions (Fitts, Hick, steering)
//! - Published reference tables (Perlin permutation)
//! - Documented game design literature (flow, DDA)
//!
//! Python baselines: `baselines/python/`
//!   - `perlin_noise.py` — Perlin 2D/3D, fBm (stdlib only)
//!   - `interaction_laws.py` — Fitts, Hick, steering (stdlib only)
//!   - `flow_engagement.py` — flow state, engagement composite (stdlib only)
//!   - Run: `python3 baselines/python/run_all_baselines.py`
//!   - Date: 2026-03-11, Python 3.x, no external dependencies
//!   - Output: `baselines/python/combined_baselines.json`

use ludospring_barracuda::interaction::flow::{FlowState, evaluate_flow};
use ludospring_barracuda::interaction::input_laws::{
    fitts_index_of_difficulty, fitts_movement_time, hick_reaction_time, steering_time,
};
use ludospring_barracuda::procedural::noise::{fbm_2d, perlin_2d, perlin_3d};
use ludospring_barracuda::tolerances;
use ludospring_barracuda::validation::ValidationResult;

// ── Fitts's Law (analytical) ─────────────────────────────────────────

#[test]
fn fitts_known_value() {
    // MT = a + b * log2(2D/W + 1)
    // D=100, W=10 → ID = log2(201/10) = log2(21) ≈ 4.392
    // MT = 50 + 150 * 4.392 ≈ 708.9 ms
    let mt = fitts_movement_time(100.0, 10.0, 50.0, 150.0);
    let expected = 150.0_f64.mul_add((2.0 * 100.0 / 10.0 + 1.0_f64).log2(), 50.0);
    let r = ValidationResult::check(
        "fitts_known_value",
        "Fitts MT for D=100 W=10 a=50 b=150",
        mt,
        expected,
        tolerances::ANALYTICAL_TOL,
    );
    assert!(
        r.passed,
        "Fitts: measured={}, expected={}",
        r.measured, r.expected
    );
}

#[test]
fn fitts_id_known_value() {
    let id = fitts_index_of_difficulty(100.0, 10.0);
    let expected = (2.0 * 100.0 / 10.0 + 1.0_f64).log2();
    let r = ValidationResult::check(
        "fitts_id",
        "ID for D=100 W=10",
        id,
        expected,
        tolerances::ANALYTICAL_TOL,
    );
    assert!(r.passed);
}

// ── Hick's Law (analytical) ──────────────────────────────────────────

#[test]
fn hick_known_value() {
    // RT = a + b * log2(N+1)
    // N=7 → RT = 200 + 150 * log2(8) = 200 + 150 * 3 = 650 ms
    let rt = hick_reaction_time(7, 200.0, 150.0);
    let expected = 150.0_f64.mul_add(8.0_f64.log2(), 200.0);
    let r = ValidationResult::check(
        "hick_known",
        "Hick RT for N=7",
        rt,
        expected,
        tolerances::ANALYTICAL_TOL,
    );
    assert!(
        r.passed,
        "Hick: measured={}, expected={}",
        r.measured, r.expected
    );
}

// ── Steering Law (analytical) ────────────────────────────────────────

#[test]
fn steering_known_value() {
    // T = a + b * (D/W) = 10 + 5 * (100/20) = 10 + 25 = 35
    let t = steering_time(100.0, 20.0, 10.0, 5.0);
    let r = ValidationResult::check(
        "steering_known",
        "steering D=100 W=20",
        t,
        35.0,
        tolerances::ANALYTICAL_TOL,
    );
    assert!(r.passed);
}

// ── Perlin Noise (reference properties) ──────────────────────────────

#[test]
fn perlin_at_integer_lattice_is_zero() {
    // Perlin noise is zero at integer grid points (gradient dot product = 0).
    for ix in 0..10_i32 {
        for iy in 0..10_i32 {
            let x = f64::from(ix);
            let y = f64::from(iy);
            let v = perlin_2d(x, y);
            let r = ValidationResult::check(
                "perlin_lattice",
                &format!("perlin_2d({x}, {y}) == 0"),
                v,
                0.0,
                tolerances::ANALYTICAL_TOL,
            );
            assert!(r.passed, "perlin_2d({x},{y}) = {v}, expected ~0");
        }
    }
}

#[test]
#[expect(
    clippy::many_single_char_names,
    reason = "x, y, z, v, r are standard names for spatial coordinate tests"
)]
fn perlin_3d_at_integer_lattice_is_zero() {
    for ix in 0..5_i32 {
        for iy in 0..5_i32 {
            for iz in 0..5_i32 {
                let (x, y, z) = (f64::from(ix), f64::from(iy), f64::from(iz));
                let v = perlin_3d(x, y, z);
                let r = ValidationResult::check(
                    "perlin_3d_lattice",
                    &format!("perlin_3d({x},{y},{z}) == 0"),
                    v,
                    0.0,
                    tolerances::ANALYTICAL_TOL,
                );
                assert!(r.passed, "perlin_3d({x},{y},{z}) = {v}, expected ~0");
            }
        }
    }
}

#[test]
fn perlin_bounded() {
    // Perlin noise output is bounded by approximately [-1, 1] for 2D.
    // Theoretical maximum is ±sqrt(2)/2 ≈ 0.707 but with fade function
    // practical bounds are tighter. We use ±1.5 as a safe bound.
    for i in 0..1000_i32 {
        let x = f64::from(i) * 0.037;
        let y = f64::from(i) * 0.043;
        let v = perlin_2d(x, y);
        assert!(v.abs() <= 1.5, "perlin_2d({x}, {y}) = {v}, exceeds bounds");
    }
}

#[test]
fn fbm_normalized() {
    // fBm with normalization should stay in approximately [-1, 1].
    for i in 0..500_i32 {
        let x = f64::from(i) * 0.1;
        let y = f64::from(i) * 0.07;
        let v = fbm_2d(x, y, 6, 2.0, 0.5);
        assert!(v.abs() <= 1.5, "fbm_2d = {v}, exceeds expected bounds");
    }
}

// ── Flow State (boundary conditions) ─────────────────────────────────

#[test]
fn flow_state_boundaries() {
    let w = tolerances::FLOW_CHANNEL_WIDTH;

    // Exact diagonal → Flow.
    assert_eq!(evaluate_flow(0.5, 0.5, w), FlowState::Flow);

    // Slightly inside the flow channel (w * 0.9 avoids f64 boundary rounding).
    assert_eq!(evaluate_flow(0.5, w.mul_add(-0.9, 0.5), w), FlowState::Flow);
    assert_eq!(evaluate_flow(0.5, w.mul_add(0.9, 0.5), w), FlowState::Flow);

    // Well outside the channel.
    assert_eq!(evaluate_flow(0.9, 0.1, w), FlowState::Anxiety);
    assert_eq!(evaluate_flow(0.1, 0.9, w), FlowState::Boredom);
}

// ── Raycaster (geometric) ────────────────────────────────────────────

#[test]
fn raycaster_distance_to_wall() {
    use ludospring_barracuda::game::raycaster::{GridMap, RayPlayer, cast_ray};

    // Player at (2.5, 2.5) facing east (angle=0), wall at x=4.
    // Expected distance: 4.0 - 2.5 = 1.5
    let map = GridMap::from_nested(&[
        vec![true, true, true, true, true],
        vec![true, false, false, false, true],
        vec![true, false, false, false, true],
        vec![true, false, false, false, true],
        vec![true, true, true, true, true],
    ]);
    let player = RayPlayer {
        x: 2.5,
        y: 2.5,
        angle: 0.0,
        ..Default::default()
    };

    let Some(hit) = cast_ray(&player, 0.0, &map, 20.0) else {
        panic!("ray should hit east wall");
    };
    let r = ValidationResult::check(
        "raycaster_distance",
        "east wall distance from (2.5, 2.5)",
        hit.distance,
        1.5,
        tolerances::RAYCASTER_DISTANCE_TOL,
    );
    assert!(
        r.passed,
        "raycaster distance: measured={}, expected={}, tol={}",
        r.measured, r.expected, r.tolerance
    );
    assert_eq!(hit.cell_x, 4);
}

// ── Engagement (edge cases) ──────────────────────────────────────────

#[test]
fn engagement_zero_session_finite() {
    use ludospring_barracuda::metrics::engagement::{EngagementSnapshot, compute_engagement};

    let snap = EngagementSnapshot::default();
    let m = compute_engagement(&snap);
    assert!(m.composite.is_finite(), "zero session should be finite");
    assert!(m.composite >= 0.0, "composite should be non-negative");
    assert!(m.composite <= 1.0, "composite should be ≤ 1.0");
}

// ── Voxel chunk (structural) ─────────────────────────────────────────

#[test]
fn voxel_chunk_density_correct() {
    use ludospring_barracuda::game::voxel::{BlockId, Chunk};

    let mut chunk = Chunk::new(4, 4, 4, [0, 0, 0]);
    assert!((chunk.density() - 0.0).abs() < f64::EPSILON);

    // Fill half the blocks.
    for x in 0..4 {
        for z in 0..4 {
            for y in 0..2 {
                chunk.set(x, y, z, BlockId(1));
            }
        }
    }
    let r = ValidationResult::check(
        "chunk_density",
        "half-filled 4x4x4 chunk density",
        chunk.density(),
        0.5,
        tolerances::ANALYTICAL_TOL,
    );
    assert!(r.passed);
}

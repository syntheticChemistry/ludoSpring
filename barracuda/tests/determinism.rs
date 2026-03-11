// SPDX-License-Identifier: AGPL-3.0-or-later
//! Determinism tests — verify that repeated runs produce identical results.
//!
//! These tests ensure that all algorithms in ludoSpring are deterministic
//! when given the same inputs. Non-determinism would break:
//! - Validation against Python baselines (different results each run)
//! - Deterministic replay (game state diverges)
//! - GPU promotion correctness (can't compare CPU vs GPU if CPU varies)

use ludospring_barracuda::procedural::noise::{fbm_2d, fbm_3d, perlin_2d, perlin_3d};
use ludospring_barracuda::procedural::wfc::{AdjacencyRules, WfcCell, WfcGrid};

#[test]
fn perlin_2d_deterministic() {
    let coords = [(0.5, 0.7), (1.23, 4.56), (100.1, 200.2), (-3.17, 2.73)];
    for (x, y) in coords {
        let a = perlin_2d(x, y);
        let b = perlin_2d(x, y);
        assert_eq!(
            a.to_bits(),
            b.to_bits(),
            "perlin_2d({x}, {y}) not deterministic: {a} vs {b}"
        );
    }
}

#[test]
fn perlin_3d_deterministic() {
    let coords = [(0.5, 0.7, 0.3), (1.23, 4.56, 7.89), (100.1, 200.2, 300.3)];
    for (x, y, z) in coords {
        let a = perlin_3d(x, y, z);
        let b = perlin_3d(x, y, z);
        assert_eq!(
            a.to_bits(),
            b.to_bits(),
            "perlin_3d({x}, {y}, {z}) not deterministic"
        );
    }
}

#[test]
fn fbm_2d_deterministic() {
    for octaves in [1, 4, 8] {
        let a = fbm_2d(3.17, 2.73, octaves, 2.0, 0.5);
        let b = fbm_2d(3.17, 2.73, octaves, 2.0, 0.5);
        assert_eq!(
            a.to_bits(),
            b.to_bits(),
            "fbm_2d not deterministic with {octaves} octaves"
        );
    }
}

#[test]
fn fbm_3d_deterministic() {
    let a = fbm_3d(1.0, 2.0, 3.0, 4, 2.0, 0.5);
    let b = fbm_3d(1.0, 2.0, 3.0, 4, 2.0, 0.5);
    assert_eq!(a.to_bits(), b.to_bits(), "fbm_3d not deterministic");
}

#[test]
fn wfc_propagation_deterministic() {
    let mut rules = AdjacencyRules::unconstrained(4);
    rules.right[0] = [0, 1].into_iter().collect();
    rules.right[1] = [1, 2].into_iter().collect();
    rules.right[2] = [2, 3].into_iter().collect();
    rules.right[3] = [3, 0].into_iter().collect();

    for _ in 0..5 {
        let mut grid = WfcGrid::new(6, 6, 4);
        grid.collapse(0, 0, 0);
        let removed = grid.propagate(&rules);

        let mut grid2 = WfcGrid::new(6, 6, 4);
        grid2.collapse(0, 0, 0);
        let removed2 = grid2.propagate(&rules);

        assert_eq!(removed, removed2, "WFC propagation not deterministic");

        for y in 0..6 {
            for x in 0..6 {
                let a = grid.get(x, y).map(WfcCell::entropy);
                let b = grid2.get(x, y).map(WfcCell::entropy);
                assert_eq!(a, b, "WFC grid state differs at ({x}, {y})");
            }
        }
    }
}

#[test]
fn fitts_law_deterministic() {
    use ludospring_barracuda::interaction::input_laws::fitts_movement_time;

    let cases = [
        (100.0, 20.0, 50.0, 150.0),
        (200.0, 5.0, 50.0, 150.0),
        (50.0, 50.0, 0.0, 100.0),
    ];
    for (d, w, a, b) in cases {
        let r1 = fitts_movement_time(d, w, a, b);
        let r2 = fitts_movement_time(d, w, a, b);
        assert_eq!(
            r1.to_bits(),
            r2.to_bits(),
            "fitts not deterministic for d={d}, w={w}"
        );
    }
}

#[test]
fn engagement_deterministic() {
    use ludospring_barracuda::metrics::engagement::{EngagementSnapshot, compute_engagement};

    let snap = EngagementSnapshot {
        session_duration_s: 600.0,
        action_count: 200,
        exploration_breadth: 15,
        challenge_seeking: 8,
        retry_count: 20,
        deliberate_pauses: 12,
    };

    let m1 = compute_engagement(&snap);
    let m2 = compute_engagement(&snap);
    assert_eq!(
        m1.composite.to_bits(),
        m2.composite.to_bits(),
        "engagement not deterministic"
    );
}

#[test]
fn raycaster_deterministic() {
    use ludospring_barracuda::game::raycaster::{GridMap, RayPlayer, cast_screen};

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
        angle: 0.3,
        ..Default::default()
    };

    let hits1 = cast_screen(&player, 64, &map, 20.0);
    let hits2 = cast_screen(&player, 64, &map, 20.0);

    for (i, (h1, h2)) in hits1.iter().zip(hits2.iter()).enumerate() {
        match (h1, h2) {
            (Some(a), Some(b)) => {
                assert_eq!(
                    a.distance.to_bits(),
                    b.distance.to_bits(),
                    "raycaster not deterministic at ray {i}"
                );
            }
            (None, None) => {}
            _ => panic!("raycaster hit/miss mismatch at ray {i}"),
        }
    }
}

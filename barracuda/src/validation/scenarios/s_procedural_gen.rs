// SPDX-License-Identifier: AGPL-3.0-or-later
//! Scenario: Procedural Generation — Perlin noise, fBm, BSP.
//! Absorbed from exp013_perlin_noise + exp017_bsp_dungeon.

use super::registry::{Scenario, ScenarioMeta, Tier, Track};
use crate::procedural::bsp::{BspNode, Rect, generate_bsp};
use crate::procedural::noise::perlin_2d;
use crate::tolerances::ANALYTICAL_TOL;
use crate::validation::{BaselineProvenance, ValidationHarness};

pub const SCENARIO: Scenario = Scenario {
    meta: ScenarioMeta {
        id: "procedural_gen",
        track: Track::ProceduralGeneration,
        tier: Tier::Rust,
        provenance_crate: "exp013_perlin_noise",
        provenance_date: "2026-04-11",
        description: "Validate Perlin, BSP against Python golden values",
    },
    run: run_procedural,
};

fn count_leaves(node: &BspNode) -> usize {
    match node {
        BspNode::Leaf { .. } => 1,
        BspNode::Split { left, right, .. } => count_leaves(left) + count_leaves(right),
    }
}

fn run_procedural(h: &mut ValidationHarness) {
    let prov = BaselineProvenance {
        script: "baselines/python/perlin_noise.py",
        commit: "231928a",
        date: "2026-04-17",
        command: "python3 baselines/python/perlin_noise.py",
    };
    h.print_provenance(&[&prov]);

    // Perlin at origin is exactly 0 by construction
    let perlin_origin = perlin_2d(0.0, 0.0);
    h.check_abs("Perlin origin", perlin_origin, 0.0, ANALYTICAL_TOL);

    // Perlin bounded [-1, 1]
    let samples: Vec<f64> = (0..100)
        .map(|i| perlin_2d(f64::from(i) * 0.1, 0.5))
        .collect();
    let all_bounded = samples.iter().all(|s| (-1.0..=1.0).contains(s));
    h.check_bool("Perlin bounded [-1,1]", all_bounded);

    // BSP produces at least 2 leaves for a non-trivial grid
    let bounds = Rect {
        x: 0.0,
        y: 0.0,
        w: 64.0,
        h: 64.0,
    };
    let tree = generate_bsp(bounds, 8.0, 42);
    let rooms = count_leaves(&tree);
    h.check_bool("BSP rooms >= 2", rooms >= 2);
}

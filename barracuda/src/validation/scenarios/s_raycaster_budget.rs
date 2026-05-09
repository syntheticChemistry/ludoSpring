// SPDX-License-Identifier: AGPL-3.0-or-later
//! Scenario: Raycaster Budget — 60Hz headroom validation.
//! Absorbed from exp001_doom_raycaster_science + exp024_doom_terminal.

use super::registry::{Scenario, ScenarioMeta, Tier, Track};
use crate::game::raycaster::{GridMap, RayPlayer, cast_ray};
use crate::validation::{BaselineProvenance, ValidationHarness};
use std::f64::consts::PI;

pub const SCENARIO: Scenario = Scenario {
    meta: ScenarioMeta {
        id: "raycaster_budget",
        track: Track::PerformanceBudget,
        tier: Tier::Rust,
        provenance_crate: "exp001_doom_raycaster_science",
        provenance_date: "2026-04-11",
        description: "Validate DDA raycaster completes within 16.6ms budget",
    },
    run: run_raycaster_budget,
};

fn run_raycaster_budget(h: &mut ValidationHarness) {
    let prov = BaselineProvenance {
        script: "baselines/python/bench_cpu_parity.py",
        commit: "d0abf08",
        date: "2026-05-08",
        command: "python3 baselines/python/bench_cpu_parity.py",
    };
    h.print_provenance(&[&prov]);

    let size = 64;
    let mut map_data = vec![false; size * size];
    for i in 0..size {
        map_data[i] = true;
        map_data[(size - 1) * size + i] = true;
        map_data[i * size] = true;
        map_data[i * size + (size - 1)] = true;
    }
    let map = GridMap::new(size, size, map_data);
    let player = RayPlayer {
        x: 5.0,
        y: 5.0,
        angle: 0.0,
        fov: PI / 3.0,
        speed: 3.0,
        turn_speed: PI,
    };

    let columns = 320;
    let fov = 60.0_f64.to_radians();
    let half_fov = fov / 2.0;

    let start = std::time::Instant::now();
    let mut hit_count = 0_u32;
    for col in 0..columns {
        let angle_offset = (f64::from(col) / f64::from(columns)).mul_add(fov, -half_fov);
        let ray_angle = player.angle + angle_offset;
        if cast_ray(&player, ray_angle, &map, 64.0).is_some() {
            hit_count += 1;
        }
    }
    let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;

    h.check_bool("Raycaster within 60Hz budget (16.6ms)", elapsed_ms < 16.6);
    h.check_bool("Raycaster sub-millisecond", elapsed_ms < 1.0);
    h.check_bool("Raycaster produces hits", hit_count > 0);
}

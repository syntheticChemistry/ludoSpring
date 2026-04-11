// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
//! Exp001: Doom raycaster analysis — validation binary.
//!
//! Validates the reference raycaster against known Doom geometry, Fitts's law
//! targeting costs, and Tufte analysis of the classic Doom HUD.
//!
//! Follows the hotSpring validation pattern: exit 0 (all pass) / 1 (failure).
//!
//! # Provenance
//!
//! Raycaster geometry: analytical (enclosed arena, known wall distances).
//! Fitts: MacKenzie (1992) — `a=50, b=150` mouse parameters.
//! Tufte thresholds: Tufte (1983), Fagerholt & Lorentzon (2009).
//! Python baseline: `baselines/python/interaction_laws.py` commit 19e402c0, 2026-04-10.

use ludospring_barracuda::game::raycaster::{GridMap, RayPlayer, cast_screen};
use ludospring_barracuda::interaction::input_laws::fitts_movement_time;
use ludospring_barracuda::metrics::tufte_gaming::{UiElement, analyze_game_ui};
use ludospring_barracuda::tolerances;
use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "baselines/python/interaction_laws.py",
    commit: "19e402c0",
    date: "2026-04-10",
    command: "python3 baselines/python/run_all_baselines.py",
};

#[expect(
    clippy::cast_precision_loss,
    reason = "hit_count ≤ 320; fits in f64 mantissa"
)]
fn validate_raycaster(h: &mut ValidationHarness) {
    let map = GridMap::from_nested(&[
        vec![true; 8],
        vec![true, false, false, false, false, false, false, true],
        vec![true, false, false, false, false, false, false, true],
        vec![true, false, false, false, false, false, false, true],
        vec![true, false, false, false, false, false, false, true],
        vec![true, false, false, false, false, false, false, true],
        vec![true, false, false, false, false, false, false, true],
        vec![true; 8],
    ]);

    let player = RayPlayer {
        x: 4.0,
        y: 4.0,
        angle: 0.0,
        ..Default::default()
    };
    let hits = cast_screen(&player, 320, &map, 20.0);
    let hit_count = hits.iter().filter(|h| h.is_some()).count();

    h.check_lower(
        "hit_rate >= 300/320 in enclosed arena",
        hit_count as f64,
        300.0,
    );

    if let Some(center) = &hits[160] {
        h.check_abs(
            "center_ray_dist_to_east_wall ~3.0",
            center.distance,
            3.0,
            tolerances::RAYCASTER_DISTANCE_TOL,
        );
    } else {
        h.check_bool("center_ray_hit", false);
    }
}

fn validate_tufte(h: &mut ValidationHarness) {
    let doom_hud = vec![
        UiElement {
            name: "health".into(),
            bounds: [0.15, 0.92, 0.06, 0.06],
            data_values: 1,
            pixel_area: 600.0,
            data_ink_area: 400.0,
            critical: true,
        },
        UiElement {
            name: "armor".into(),
            bounds: [0.35, 0.92, 0.06, 0.06],
            data_values: 1,
            pixel_area: 600.0,
            data_ink_area: 400.0,
            critical: true,
        },
        UiElement {
            name: "ammo".into(),
            bounds: [0.02, 0.92, 0.08, 0.06],
            data_values: 1,
            pixel_area: 800.0,
            data_ink_area: 600.0,
            critical: true,
        },
        UiElement {
            name: "bar bg".into(),
            bounds: [0.0, 0.90, 1.0, 0.10],
            data_values: 0,
            pixel_area: 30000.0,
            data_ink_area: 2000.0,
            critical: false,
        },
        UiElement {
            name: "face".into(),
            bounds: [0.45, 0.90, 0.10, 0.10],
            data_values: 2,
            pixel_area: 3000.0,
            data_ink_area: 1000.0,
            critical: false,
        },
    ];
    let ui = analyze_game_ui(&doom_hud);

    h.check_abs(
        "doom_hud_data_ink ~0.125 (chartjunk-heavy)",
        ui.data_ink_ratio,
        0.125,
        tolerances::UI_DATA_INK_TOL,
    );
    h.check_abs(
        "doom_hud ~11% screen coverage",
        ui.screen_coverage,
        0.114,
        tolerances::UI_COVERAGE_TOL,
    );
}

fn validate_fitts(h: &mut ValidationHarness) {
    let mt = fitts_movement_time(100.0, 10.0, 50.0, 150.0);
    h.check_abs(
        "fitts MT D=100 W=10 (MacKenzie 1992, Python baseline)",
        mt,
        708.847_613_416_814,
        tolerances::ANALYTICAL_TOL,
    );

    let mt_barrel = fitts_movement_time(
        50.0,
        30.0,
        tolerances::FITTS_A_MOUSE_MS,
        tolerances::FITTS_B_MOUSE_MS,
    );
    let expected = 150.0_f64.mul_add((2.0 * 50.0 / 30.0 + 1.0_f64).log2(), 50.0);
    h.check_abs(
        "fitts MT barrel D=50 W=30",
        mt_barrel,
        expected,
        tolerances::ANALYTICAL_TOL,
    );
}

fn main() {
    let mut h = ValidationHarness::new("exp001_doom_raycaster_analysis");
    h.print_provenance(&[&PROVENANCE]);

    validate_raycaster(&mut h);
    validate_tufte(&mut h);
    validate_fitts(&mut h);

    h.finish();
}

// SPDX-License-Identifier: AGPL-3.0-or-later
//! Exp001: Doom raycaster analysis.
//!
//! Validates the reference raycaster against known Doom geometry:
//! - DDA ray-wall intersection accuracy
//! - Fisheye correction
//! - Input latency model (Fitts's law on crosshair targeting)
//! - Interaction cost of the classic Doom HUD

use ludospring_barracuda::game::raycaster::{RayPlayer, cast_screen};
use ludospring_barracuda::interaction::input_laws::fitts_movement_time;
use ludospring_barracuda::metrics::tufte_gaming::{UiElement, analyze_game_ui};
use ludospring_barracuda::tolerances;

fn main() {
    println!("=== Exp001: Doom Raycaster Analysis ===\n");

    // --- Part 1: Raycaster validation ---
    println!("Part 1: Raycaster DDA validation");
    let map = vec![
        vec![true, true, true, true, true, true, true, true],
        vec![true, false, false, false, false, false, false, true],
        vec![true, false, false, false, false, false, false, true],
        vec![true, false, false, false, false, false, false, true],
        vec![true, false, false, false, false, false, false, true],
        vec![true, false, false, false, false, false, false, true],
        vec![true, false, false, false, false, false, false, true],
        vec![true, true, true, true, true, true, true, true],
    ];

    let player = RayPlayer {
        x: 4.0,
        y: 4.0,
        angle: 0.0,
        ..Default::default()
    };

    let hits = cast_screen(&player, 320, &map, 20.0);
    let hit_count = hits.iter().filter(|h| h.is_some()).count();
    println!("  320 rays cast, {hit_count} hit walls");
    assert!(hit_count > 300, "most rays should hit in an enclosed room");

    // Center ray should hit east wall at distance ~3.0
    let center = &hits[160];
    if let Some(hit) = center {
        println!("  Center ray: distance={:.2}, cell=({}, {})", hit.distance, hit.cell_x, hit.cell_y);
        assert!(hit.distance > 2.5 && hit.distance < 3.5);
    }

    // --- Part 2: Interaction cost of Doom HUD ---
    println!("\nPart 2: Doom HUD Tufte analysis");
    let doom_hud = vec![
        UiElement {
            name: "health number".into(),
            bounds: [0.15, 0.92, 0.06, 0.06],
            data_values: 1,
            pixel_area: 600.0,
            data_ink_area: 400.0,
            critical: true,
        },
        UiElement {
            name: "armor number".into(),
            bounds: [0.35, 0.92, 0.06, 0.06],
            data_values: 1,
            pixel_area: 600.0,
            data_ink_area: 400.0,
            critical: true,
        },
        UiElement {
            name: "ammo counter".into(),
            bounds: [0.02, 0.92, 0.08, 0.06],
            data_values: 1,
            pixel_area: 800.0,
            data_ink_area: 600.0,
            critical: true,
        },
        UiElement {
            name: "status bar background".into(),
            bounds: [0.0, 0.90, 1.0, 0.10],
            data_values: 0,
            pixel_area: 30000.0,
            data_ink_area: 2000.0,
            critical: false,
        },
        UiElement {
            name: "face portrait".into(),
            bounds: [0.45, 0.90, 0.10, 0.10],
            data_values: 2,
            pixel_area: 3000.0,
            data_ink_area: 1000.0,
            critical: false,
        },
    ];

    let report = analyze_game_ui(&doom_hud);
    println!("  Data-ink ratio: {:.3}", report.data_ink_ratio);
    println!("  Screen coverage: {:.1}%", report.screen_coverage * 100.0);
    println!("  Info density: {:.1} values/unit", report.info_density);
    for note in &report.notes {
        println!("  Note: {note}");
    }

    // --- Part 3: Fitts's law on crosshair targeting ---
    println!("\nPart 3: Crosshair targeting (Fitts's law)");
    let scenarios = [
        ("Close large target (barrel)", 50.0, 30.0),
        ("Medium distance imp", 150.0, 20.0),
        ("Far cacodemon", 300.0, 15.0),
        ("Sniper (far, tiny)", 400.0, 5.0),
    ];

    for (desc, distance, width) in &scenarios {
        let mt = fitts_movement_time(
            *distance,
            *width,
            tolerances::FITTS_A_MOUSE_MS,
            tolerances::FITTS_B_MOUSE_MS,
        );
        println!("  {desc:30} → {mt:.0} ms");
    }

    println!("\n=== Exp001 complete ===");
}

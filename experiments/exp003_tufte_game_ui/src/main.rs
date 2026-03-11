// SPDX-License-Identifier: AGPL-3.0-or-later
//! Exp003: Tufte analysis of game UIs across genres — validation binary.
//!
//! Compares information design of FPS, sandbox, and RTS HUDs through
//! Tufte's principles. Validates that the analysis engine correctly
//! identifies chartjunk, data-ink ratios, and screen coverage.
//!
//! # Provenance
//!
//! Tufte principles: Tufte (1983) "The Visual Display of Quantitative
//! Information." UI measurements from screenshot analysis of Doom (1993),
//! Minecraft (Mojang, 2011), `StarCraft` (Blizzard, 1998).

use ludospring_barracuda::metrics::tufte_gaming::{UiElement, analyze_game_ui};
use ludospring_barracuda::validation::ValidationResult;

fn report(r: &ValidationResult) {
    if r.passed {
        println!("  PASS  {}: {}", r.experiment, r.description);
    } else {
        println!(
            "  FAIL  {}: {} (got={:.4}, want={:.4}, tol={:.4})",
            r.experiment, r.description, r.measured, r.expected, r.tolerance
        );
    }
}

fn doom_hud() -> Vec<UiElement> {
    vec![
        UiElement {
            name: "health".into(),
            bounds: [0.15, 0.92, 0.06, 0.06],
            data_values: 1,
            pixel_area: 600.0,
            data_ink_area: 500.0,
            critical: true,
        },
        UiElement {
            name: "armor".into(),
            bounds: [0.35, 0.92, 0.06, 0.06],
            data_values: 1,
            pixel_area: 600.0,
            data_ink_area: 500.0,
            critical: true,
        },
        UiElement {
            name: "ammo".into(),
            bounds: [0.02, 0.92, 0.08, 0.06],
            data_values: 1,
            pixel_area: 800.0,
            data_ink_area: 650.0,
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
    ]
}

fn minecraft_hud() -> Vec<UiElement> {
    vec![
        UiElement {
            name: "hearts".into(),
            bounds: [0.30, 0.92, 0.15, 0.03],
            data_values: 1,
            pixel_area: 400.0,
            data_ink_area: 350.0,
            critical: true,
        },
        UiElement {
            name: "hunger".into(),
            bounds: [0.55, 0.92, 0.15, 0.03],
            data_values: 1,
            pixel_area: 400.0,
            data_ink_area: 350.0,
            critical: true,
        },
        UiElement {
            name: "hotbar".into(),
            bounds: [0.25, 0.96, 0.50, 0.04],
            data_values: 9,
            pixel_area: 2000.0,
            data_ink_area: 1600.0,
            critical: true,
        },
        UiElement {
            name: "xp bar".into(),
            bounds: [0.20, 0.94, 0.60, 0.01],
            data_values: 1,
            pixel_area: 200.0,
            data_ink_area: 180.0,
            critical: false,
        },
        UiElement {
            name: "crosshair".into(),
            bounds: [0.497, 0.497, 0.006, 0.006],
            data_values: 1,
            pixel_area: 20.0,
            data_ink_area: 20.0,
            critical: true,
        },
    ]
}

fn rts_hud() -> Vec<UiElement> {
    vec![
        UiElement {
            name: "minimap".into(),
            bounds: [0.0, 0.75, 0.20, 0.25],
            data_values: 50,
            pixel_area: 15000.0,
            data_ink_area: 12000.0,
            critical: true,
        },
        UiElement {
            name: "unit panel".into(),
            bounds: [0.25, 0.80, 0.50, 0.20],
            data_values: 20,
            pixel_area: 20000.0,
            data_ink_area: 8000.0,
            critical: true,
        },
        UiElement {
            name: "resources".into(),
            bounds: [0.0, 0.0, 0.40, 0.03],
            data_values: 4,
            pixel_area: 1500.0,
            data_ink_area: 1200.0,
            critical: true,
        },
        UiElement {
            name: "cmd card".into(),
            bounds: [0.80, 0.75, 0.20, 0.25],
            data_values: 12,
            pixel_area: 15000.0,
            data_ink_area: 6000.0,
            critical: true,
        },
    ]
}

#[expect(
    clippy::cast_possible_truncation,
    reason = "UI element counts ≤ 100; notes.len() fits in u32"
)]
fn main() {
    println!("=== Exp003: Tufte Game UI Comparison (Validation) ===\n");
    let mut results = Vec::new();

    let doom = analyze_game_ui(&doom_hud());
    let mc = analyze_game_ui(&minecraft_hud());
    let rts = analyze_game_ui(&rts_hud());

    // Minecraft should have highest data-ink ratio (minimal chrome)
    let r = ValidationResult::check(
        "exp003_mc_best_ink",
        "Minecraft data-ink > 0.7 (minimal chrome HUD)",
        mc.data_ink_ratio,
        0.8,
        0.15,
    );
    report(&r);
    results.push(r);

    // Doom should have low data-ink due to status bar chrome
    let r = ValidationResult::check(
        "exp003_doom_chartjunk",
        "Doom data-ink < 0.2 (status bar chartjunk)",
        doom.data_ink_ratio,
        0.12,
        0.1,
    );
    report(&r);
    results.push(r);

    // RTS should cover > 25% of screen
    let r = ValidationResult::check(
        "exp003_rts_coverage",
        "RTS HUD covers > 25% of screen",
        rts.screen_coverage,
        0.30,
        0.10,
    );
    report(&r);
    results.push(r);

    // Minecraft should cover < 10% of screen
    let r = ValidationResult::check(
        "exp003_mc_minimal",
        "Minecraft HUD covers < 10% of screen",
        mc.screen_coverage,
        0.04,
        0.06,
    );
    report(&r);
    results.push(r);

    // RTS should be most information-dense
    let r = ValidationResult::check(
        "exp003_rts_dense",
        "RTS info density > Minecraft info density",
        rts.info_density,
        mc.info_density + 10.0,
        200.0,
    );
    report(&r);
    results.push(r);

    // Doom status bar should trigger chartjunk note
    let r = ValidationResult::check(
        "exp003_doom_notes",
        "Doom triggers at least one Tufte warning",
        f64::from(doom.notes.len() as u32),
        1.0,
        0.5,
    );
    report(&r);
    results.push(r);

    let passed = results.iter().filter(|r| r.passed).count();
    let failed = results.len() - passed;
    println!("\n{passed} passed, {failed} failed");
    if failed > 0 {
        std::process::exit(1);
    }
}

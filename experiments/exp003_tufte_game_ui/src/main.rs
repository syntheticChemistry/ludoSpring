// SPDX-License-Identifier: AGPL-3.0-or-later
//! Exp003: Tufte analysis of game UIs across genres.
//!
//! Compares the information design of different game genres' HUDs through
//! Tufte's principles. This reveals which genre UIs are most suitable as
//! templates for scientific visualization interaction.

use ludospring_barracuda::metrics::tufte_gaming::{UiElement, analyze_game_ui};

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
            name: "status bar bg".into(),
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
            name: "command card".into(),
            bounds: [0.80, 0.75, 0.20, 0.25],
            data_values: 12,
            pixel_area: 15000.0,
            data_ink_area: 6000.0,
            critical: true,
        },
    ]
}

fn print_report(genre: &str, elements: &[UiElement]) {
    let report = analyze_game_ui(elements);
    println!(
        "  {genre:15} │ data-ink: {:.3} │ coverage: {:5.1}% │ density: {:6.1} │ elements: {}",
        report.data_ink_ratio,
        report.screen_coverage * 100.0,
        report.info_density,
        report.elements.len(),
    );
    for note in &report.notes {
        println!("  {genre:15} │ ⚠ {note}");
    }
    for elem in &report.elements {
        for rec in &elem.recommendations {
            println!("  {genre:15} │   → {rec}");
        }
    }
}

fn main() {
    println!("=== Exp003: Tufte Game UI Comparison ===\n");
    println!(
        "  {:15} │ {:14} │ {:12} │ {:12} │ elements",
        "Genre", "data-ink", "coverage", "density"
    );
    println!(
        "  {:─>15} │ {:─>14} │ {:─>12} │ {:─>12} │ ────────",
        "", "", "", ""
    );

    print_report("Doom (FPS)", &doom_hud());
    print_report("Minecraft", &minecraft_hud());
    print_report("RTS (SC2-like)", &rts_hud());

    println!("\n  Key insights:");
    println!("  • Minecraft's HUD has highest data-ink ratio — minimal chrome, maximal info");
    println!("  • RTS HUDs are information-dense but cover 30%+ of screen");
    println!("  • Doom's status bar background is pure chartjunk (30k px, 0 data values)");
    println!("  • For chemistry UI: Minecraft pattern (minimal, iconic) suits exploration");
    println!("  • For systems biology: RTS pattern (dense, panel-based) suits monitoring");

    println!("\n=== Exp003 complete ===");
}

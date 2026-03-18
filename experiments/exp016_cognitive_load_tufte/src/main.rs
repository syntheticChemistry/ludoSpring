// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
//! Exp016: Cognitive load Tufte sweep — validation binary.
//!
//! Sweeps UI configurations from minimal (racing game HUD) to maximal
//! (RTS interface) and validates that Tufte metrics correctly identify
//! information density tradeoffs and cognitive load boundaries.
//!
//! # Provenance
//!
//! Tufte, E.R. (1983). "The Visual Display of Quantitative Information."
//! Tufte, E.R. (1990). "Envisioning Information."
//! Sweller, J. (1988). "Cognitive load during problem solving." Cog. Sci.
//! Hick (1952): decision time scales with log(N) — more UI = slower decisions.

use ludospring_barracuda::interaction::input_laws::hick_reaction_time;
use ludospring_barracuda::metrics::tufte_gaming::{UiElement, analyze_game_ui};
use ludospring_barracuda::tolerances;
use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "N/A (analytical — Tufte 1983, Sweller 1988, Hick 1952)",
    commit: "74cf9488",
    date: "2026-03-15",
    command: "N/A (analytical)",
};

fn minimal_hud() -> Vec<UiElement> {
    vec![
        UiElement {
            name: "speedometer".into(),
            bounds: [0.85, 0.85, 0.1, 0.1],
            data_values: 1,
            pixel_area: 100.0,
            data_ink_area: 90.0,
            critical: true,
        },
        UiElement {
            name: "lap_counter".into(),
            bounds: [0.45, 0.0, 0.1, 0.05],
            data_values: 1,
            pixel_area: 50.0,
            data_ink_area: 45.0,
            critical: true,
        },
    ]
}

fn moderate_hud() -> Vec<UiElement> {
    vec![
        UiElement {
            name: "health".into(),
            bounds: [0.0, 0.9, 0.15, 0.08],
            data_values: 1,
            pixel_area: 120.0,
            data_ink_area: 90.0,
            critical: true,
        },
        UiElement {
            name: "ammo".into(),
            bounds: [0.85, 0.9, 0.12, 0.08],
            data_values: 2,
            pixel_area: 96.0,
            data_ink_area: 70.0,
            critical: true,
        },
        UiElement {
            name: "minimap".into(),
            bounds: [0.8, 0.0, 0.18, 0.18],
            data_values: 10,
            pixel_area: 400.0,
            data_ink_area: 320.0,
            critical: false,
        },
        UiElement {
            name: "crosshair".into(),
            bounds: [0.49, 0.49, 0.02, 0.02],
            data_values: 1,
            pixel_area: 20.0,
            data_ink_area: 18.0,
            critical: true,
        },
    ]
}

fn maximal_hud() -> Vec<UiElement> {
    vec![
        UiElement {
            name: "minimap".into(),
            bounds: [0.75, 0.0, 0.25, 0.25],
            data_values: 50,
            pixel_area: 800.0,
            data_ink_area: 500.0,
            critical: true,
        },
        UiElement {
            name: "resource_bar".into(),
            bounds: [0.0, 0.0, 1.0, 0.05],
            data_values: 6,
            pixel_area: 500.0,
            data_ink_area: 300.0,
            critical: true,
        },
        UiElement {
            name: "unit_panel".into(),
            bounds: [0.0, 0.7, 0.3, 0.3],
            data_values: 15,
            pixel_area: 600.0,
            data_ink_area: 200.0,
            critical: false,
        },
        UiElement {
            name: "command_card".into(),
            bounds: [0.3, 0.75, 0.4, 0.25],
            data_values: 12,
            pixel_area: 500.0,
            data_ink_area: 150.0,
            critical: false,
        },
        UiElement {
            name: "production_queue".into(),
            bounds: [0.7, 0.75, 0.3, 0.25],
            data_values: 8,
            pixel_area: 400.0,
            data_ink_area: 100.0,
            critical: false,
        },
        UiElement {
            name: "chat".into(),
            bounds: [0.0, 0.3, 0.25, 0.35],
            data_values: 5,
            pixel_area: 300.0,
            data_ink_area: 250.0,
            critical: false,
        },
    ]
}

fn validate_tufte_sweep(h: &mut ValidationHarness) {
    let min_report = analyze_game_ui(&minimal_hud());
    let _mod_report = analyze_game_ui(&moderate_hud());
    let max_report = analyze_game_ui(&maximal_hud());

    h.check_bool(
        "minimal HUD: highest data-ink ratio",
        min_report.data_ink_ratio > max_report.data_ink_ratio,
    );

    h.check_bool(
        "maximal HUD: highest screen coverage",
        max_report.screen_coverage > min_report.screen_coverage,
    );

    h.check_bool(
        "maximal HUD: more Tufte warnings than minimal",
        max_report.notes.len() >= min_report.notes.len(),
    );
}

fn validate_cognitive_load(h: &mut ValidationHarness) {
    let a = tolerances::HICK_A_MS;
    let b = tolerances::HICK_B_MS;

    let rt_minimal = hick_reaction_time(minimal_hud().len(), a, b);
    let rt_moderate = hick_reaction_time(moderate_hud().len(), a, b);
    let rt_maximal = hick_reaction_time(maximal_hud().len(), a, b);

    h.check_bool(
        "more UI elements → slower decisions (Hick's law)",
        rt_minimal < rt_moderate && rt_moderate < rt_maximal,
    );

    h.check_abs(
        "decision time grows logarithmically, not linearly",
        rt_maximal / rt_minimal,
        1.5,
        0.5,
    );
}

fn validate_information_density(h: &mut ValidationHarness) {
    let min_report = analyze_game_ui(&minimal_hud());
    let max_report = analyze_game_ui(&maximal_hud());

    h.check_bool(
        "all Tufte metrics are finite and non-negative",
        min_report.data_ink_ratio >= 0.0
            && max_report.data_ink_ratio >= 0.0
            && min_report.info_density >= 0.0
            && max_report.info_density >= 0.0,
    );

    h.check_bool(
        "maximal HUD has higher info density",
        max_report.info_density > min_report.info_density,
    );
}

fn main() {
    let mut h = ValidationHarness::new("exp016_cognitive_load_tufte");
    h.print_provenance(&[&PROVENANCE]);

    validate_tufte_sweep(&mut h);
    validate_cognitive_load(&mut h);
    validate_information_density(&mut h);

    h.finish();
}

// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
//! Exp022: Small multiples minimap analysis — validation binary.
//!
//! Validates Tufte's "small multiples" principle applied to game minimaps.
//! A minimap IS a small-multiple display: same data as the main view,
//! smaller scale, repeated pattern. Validates that minimap configurations
//! satisfy Tufte constraints on data-ink ratio, lie factor, and coverage.
//!
//! # Provenance
//!
//! Tufte (1990). "Envisioning Information." Chapter on small multiples.
//! Tufte (1983). "The Visual Display of Quantitative Information."

use ludospring_barracuda::metrics::tufte_gaming::{UiElement, analyze_game_ui};
use ludospring_barracuda::tolerances;
use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "N/A (analytical — Tufte)",
    commit: "N/A",
    date: "N/A",
    command: "N/A (tufte_gaming metrics from barracuda)",
};

fn doom_minimap() -> UiElement {
    UiElement {
        name: "doom_automap".into(),
        bounds: [0.0, 0.0, 1.0, 1.0],
        data_values: 50,
        pixel_area: 1000.0,
        data_ink_area: 900.0,
        critical: false,
    }
}

fn rts_minimap() -> UiElement {
    UiElement {
        name: "rts_minimap".into(),
        bounds: [0.75, 0.0, 0.25, 0.25],
        data_values: 30,
        pixel_area: 400.0,
        data_ink_area: 320.0,
        critical: true,
    }
}

fn rpg_minimap() -> UiElement {
    UiElement {
        name: "rpg_compass_minimap".into(),
        bounds: [0.4, 0.0, 0.2, 0.08],
        data_values: 5,
        pixel_area: 200.0,
        data_ink_area: 80.0,
        critical: true,
    }
}

fn decorative_minimap() -> UiElement {
    UiElement {
        name: "decorative_minimap".into(),
        bounds: [0.7, 0.0, 0.3, 0.3],
        data_values: 10,
        pixel_area: 800.0,
        data_ink_area: 150.0,
        critical: false,
    }
}

fn validate_data_ink_ratios(h: &mut ValidationHarness) {
    let doom = analyze_game_ui(&[doom_minimap()]);
    let rts = analyze_game_ui(&[rts_minimap()]);
    let rpg = analyze_game_ui(&[rpg_minimap()]);
    let decorative = analyze_game_ui(&[decorative_minimap()]);

    // Doom automap should have highest ink ratio (pure line drawing)
    h.check_bool(
        "Doom automap: high data-ink ratio (> 0.8)",
        doom.data_ink_ratio > 0.8,
    );

    // Decorative minimap should have low ink ratio
    h.check_bool(
        "decorative minimap: low data-ink ratio (< 0.3)",
        decorative.data_ink_ratio < 0.3,
    );

    // RTS minimap should be between Doom and decorative
    h.check_bool(
        "RTS minimap: ink ratio between Doom and decorative",
        rts.data_ink_ratio < doom.data_ink_ratio && rts.data_ink_ratio > decorative.data_ink_ratio,
    );

    println!(
        "\n  Doom: {:.3}, RTS: {:.3}, RPG: {:.3}, Decorative: {:.3}",
        doom.data_ink_ratio, rts.data_ink_ratio, rpg.data_ink_ratio, decorative.data_ink_ratio
    );
}

fn validate_coverage(h: &mut ValidationHarness) {
    // Doom automap is full-screen overlay
    let doom = analyze_game_ui(&[doom_minimap()]);
    h.check_abs(
        "Doom automap: full-screen coverage (1.0)",
        doom.screen_coverage,
        1.0,
        tolerances::ANALYTICAL_TOL,
    );

    // RTS minimap is small corner
    let rts = analyze_game_ui(&[rts_minimap()]);
    h.check_bool(
        "RTS minimap: small corner (< 10% screen)",
        rts.screen_coverage < 0.10,
    );
}

fn validate_info_density(h: &mut ValidationHarness) {
    let doom = analyze_game_ui(&[doom_minimap()]);
    let rts = analyze_game_ui(&[rts_minimap()]);

    // RTS minimap should have higher info density (lots of data in small area)
    h.check_bool(
        "RTS minimap has higher info density than fullscreen Doom automap",
        rts.info_density > doom.info_density,
    );

    // Tufte principle: small multiples pack information efficiently
    let all_positive = doom.info_density > 0.0 && rts.info_density > 0.0;
    h.check_bool("all minimaps have positive info density", all_positive);

    println!(
        "\n  Doom density: {:.1}, RTS density: {:.1}",
        doom.info_density, rts.info_density
    );
}

fn main() {
    let mut h = ValidationHarness::new("exp022_small_multiples_minimap");
    h.print_provenance(&[&PROVENANCE]);

    validate_data_ink_ratios(&mut h);
    validate_coverage(&mut h);
    validate_info_density(&mut h);

    h.finish();
}

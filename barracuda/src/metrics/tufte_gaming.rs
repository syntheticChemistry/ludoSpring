// SPDX-License-Identifier: AGPL-3.0-or-later
//! Tufte constraints applied to game user interfaces.
//!
//! Edward Tufte's principles of information design apply equally to scientific
//! visualizations and game HUDs. A minimap is a small-multiple display. A
//! health bar is a bar chart. A damage number is data-ink. By scoring game
//! UIs through Tufte constraints, we can quantify visual clarity across genres.
//!
//! # Genre-specific UI patterns
//!
//! | Genre | Primary displays | Tufte concern |
//! |-------|-----------------|---------------|
//! | FPS | Health, ammo, crosshair, minimap | Data-ink ratio (minimize HUD chrome) |
//! | RTS | Minimap, unit list, resource bars | Information density, small multiples |
//! | RPG | Stats, inventory, dialog | Chartjunk in decorative frames |
//! | Puzzle | Board state, score, timer | Lie factor (visual ≠ logical size) |
//! | Sandbox | Inventory, world state, toolbars | Information density per pixel |

/// A game UI element for Tufte analysis.
#[derive(Debug, Clone)]
pub struct UiElement {
    /// Human-readable name (e.g., "health bar", "minimap", "ammo counter").
    pub name: String,
    /// Bounding box: [x, y, width, height] in normalized screen coords (0–1).
    pub bounds: [f64; 4],
    /// How many distinct data values this element conveys.
    pub data_values: usize,
    /// Total pixel area (estimated).
    pub pixel_area: f64,
    /// Pixel area that directly encodes data (non-decorative).
    pub data_ink_area: f64,
    /// Whether this element is critical (must always be visible).
    pub critical: bool,
}

/// Tufte analysis result for a game UI.
#[derive(Debug, Clone)]
pub struct GameUiTufteReport {
    /// Overall data-ink ratio (0.0–1.0, higher = more Tufte-compliant).
    pub data_ink_ratio: f64,
    /// Information density: data values per unit screen area.
    pub info_density: f64,
    /// Fraction of screen occupied by HUD elements.
    pub screen_coverage: f64,
    /// Per-element analysis.
    pub elements: Vec<ElementAnalysis>,
    /// Genre-specific notes.
    pub notes: Vec<String>,
}

/// Analysis of a single UI element.
#[derive(Debug, Clone)]
pub struct ElementAnalysis {
    /// Element name.
    pub name: String,
    /// Element's data-ink ratio.
    pub data_ink_ratio: f64,
    /// Whether the element could be simplified.
    pub simplifiable: bool,
    /// Specific recommendations.
    pub recommendations: Vec<String>,
}

/// Analyze a set of game UI elements through Tufte constraints.
#[must_use]
pub fn analyze_game_ui(elements: &[UiElement]) -> GameUiTufteReport {
    let total_pixel_area: f64 = elements.iter().map(|e| e.pixel_area).sum();
    let total_data_ink: f64 = elements.iter().map(|e| e.data_ink_area).sum();
    let total_data_values: usize = elements.iter().map(|e| e.data_values).sum();
    let total_screen_coverage: f64 = elements.iter().map(|e| e.bounds[2] * e.bounds[3]).sum();

    let data_ink_ratio = if total_pixel_area > 0.0 {
        total_data_ink / total_pixel_area
    } else {
        0.0
    };

    let info_density = if total_screen_coverage > 0.0 {
        total_data_values as f64 / total_screen_coverage
    } else {
        0.0
    };

    let element_analyses: Vec<ElementAnalysis> = elements
        .iter()
        .map(|e| {
            let ratio = if e.pixel_area > 0.0 {
                e.data_ink_area / e.pixel_area
            } else {
                0.0
            };
            let simplifiable = ratio < 0.5;
            let mut recs = Vec::new();

            if ratio < 0.3 {
                recs.push(format!(
                    "'{}': data-ink ratio {ratio:.2} — over 70% decoration. Consider Tufte sparkline style.",
                    e.name
                ));
            }
            if e.bounds[2] * e.bounds[3] > 0.05 && e.data_values < 3 {
                recs.push(format!(
                    "'{}': large area ({:.0}% of screen) for {} data values. Consider shrinking.",
                    e.name,
                    e.bounds[2] * e.bounds[3] * 100.0,
                    e.data_values
                ));
            }

            ElementAnalysis {
                name: e.name.clone(),
                data_ink_ratio: ratio,
                simplifiable,
                recommendations: recs,
            }
        })
        .collect();

    let mut notes = Vec::new();
    if total_screen_coverage > 0.25 {
        notes.push(format!(
            "HUD covers {:.0}% of screen — consider progressive disclosure.",
            total_screen_coverage * 100.0
        ));
    }
    if data_ink_ratio < 0.4 {
        notes.push("Overall data-ink ratio below 0.4 — significant chartjunk.".into());
    }

    GameUiTufteReport {
        data_ink_ratio,
        info_density,
        screen_coverage: total_screen_coverage,
        elements: element_analyses,
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minimal_hud_scores_high() {
        let elements = vec![
            UiElement {
                name: "health number".into(),
                bounds: [0.02, 0.95, 0.05, 0.03],
                data_values: 1,
                pixel_area: 100.0,
                data_ink_area: 90.0,
                critical: true,
            },
            UiElement {
                name: "ammo counter".into(),
                bounds: [0.93, 0.95, 0.05, 0.03],
                data_values: 1,
                pixel_area: 80.0,
                data_ink_area: 70.0,
                critical: true,
            },
        ];
        let report = analyze_game_ui(&elements);
        assert!(report.data_ink_ratio > 0.8);
        assert!(report.screen_coverage < 0.01);
    }

    #[test]
    fn cluttered_hud_scores_low() {
        let elements = vec![UiElement {
            name: "decorative frame".into(),
            bounds: [0.0, 0.8, 1.0, 0.2],
            data_values: 2,
            pixel_area: 10000.0,
            data_ink_area: 500.0,
            critical: false,
        }];
        let report = analyze_game_ui(&elements);
        assert!(report.data_ink_ratio < 0.1);
        assert!(!report.notes.is_empty());
    }
}

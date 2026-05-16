// SPDX-License-Identifier: AGPL-3.0-or-later
//! Meta-tier validation for petalTongue rendering pipeline.
//!
//! Uses the `meta.observe` and `meta.intent` signal patterns to create
//! structured validation cases that exercise petalTongue's scene graph
//! compilation without requiring a live renderer. Each validation case
//! declares its intent (what the scene *should* achieve visually) and
//! observes the structural properties of the composed payload.
//!
//! # Architecture
//!
//! ```text
//! GameDataChannel ──► compose_scene() ──► ScenePayload
//!                                              │
//!                                    MetaValidationCase
//!                                              │
//!                                   ┌──────────┼──────────┐
//!                                   ▼          ▼          ▼
//!                              TufteCheck  StructCheck  IntentCheck
//! ```
//!
//! This module operates at the "meta tier" — it doesn't render pixels but
//! validates the structural and semantic properties that ensure correct
//! rendering when petalTongue processes the payload.

use super::scene::{SceneData, ScenePayload};

/// A meta-tier validation case.
#[derive(Debug, Clone)]
pub struct MetaValidationCase {
    /// Case name for reporting.
    pub name: String,
    /// The intent declaration (what this visualization should achieve).
    pub intent: RenderIntent,
    /// The payload to validate.
    pub payload: ScenePayload,
}

/// Declared rendering intent — what the visualization *should* convey.
///
/// Maps to petalTongue's Tufte-aware constraints:
/// - Data-ink ratio (how much of the rendered area carries information)
/// - Information density (data points per visual unit)
/// - Temporal coherence (for animated/streaming data)
#[derive(Debug, Clone)]
pub enum RenderIntent {
    /// Dense analytical display — high data-ink, minimal chrome.
    AnalyticalDense {
        /// Minimum data-ink ratio expected (0.0–1.0).
        min_data_ink: f64,
    },
    /// Interactive exploration — moderate density, clickable regions.
    Interactive {
        /// Minimum number of interactive targets.
        min_targets: usize,
    },
    /// Status indicator — single value with context.
    StatusGauge {
        /// Whether thresholds should be visible.
        show_thresholds: bool,
    },
    /// Spatial layout — 2D map with positioned entities.
    SpatialMap {
        /// Minimum entity count for meaningful display.
        min_entities: usize,
    },
    /// Narrative flow — sequential progression.
    NarrativeSequence {
        /// Minimum node count for a meaningful graph.
        min_nodes: usize,
    },
}

/// Result of a meta-tier validation check.
#[derive(Debug, Clone)]
pub struct MetaValidationResult {
    /// Case name.
    pub name: String,
    /// Overall pass/fail.
    pub passed: bool,
    /// Individual check results.
    pub checks: Vec<CheckResult>,
}

/// A single check within a validation case.
#[derive(Debug, Clone)]
pub struct CheckResult {
    /// Check name.
    pub check: &'static str,
    /// Whether this check passed.
    pub passed: bool,
    /// Human-readable detail.
    pub detail: String,
}

/// Run a meta-tier validation case, returning structured results.
#[must_use]
pub fn validate_meta(case: &MetaValidationCase) -> MetaValidationResult {
    let mut checks = Vec::new();

    checks.push(check_structural_integrity(&case.payload));
    checks.push(check_data_density(&case.payload));
    checks.extend(check_intent_alignment(&case.intent, &case.payload));

    let passed = checks.iter().all(|c| c.passed);

    MetaValidationResult {
        name: case.name.clone(),
        passed,
        checks,
    }
}

fn check_structural_integrity(payload: &ScenePayload) -> CheckResult {
    let issues = super::scene::validate_payload(payload);
    CheckResult {
        check: "structural_integrity",
        passed: issues.is_empty(),
        detail: if issues.is_empty() {
            "payload structure valid".into()
        } else {
            format!("issues: {}", issues.join(", "))
        },
    }
}

fn check_data_density(payload: &ScenePayload) -> CheckResult {
    let point_count = match &payload.data {
        SceneData::TimeSeries { x_values, .. } => x_values.len(),
        SceneData::Bar { values, .. } | SceneData::Heatmap { values, .. } => values.len(),
        SceneData::FieldMap { entities, .. } => entities.len(),
        SceneData::GameScene { nodes, .. } => nodes.len(),
        SceneData::Gauge { .. } => 1,
    };

    CheckResult {
        check: "data_density",
        passed: point_count > 0,
        detail: format!("{point_count} data points"),
    }
}

fn check_intent_alignment(intent: &RenderIntent, payload: &ScenePayload) -> Vec<CheckResult> {
    match intent {
        RenderIntent::AnalyticalDense { min_data_ink } => {
            let estimated_ink = estimate_data_ink(payload);
            vec![CheckResult {
                check: "intent_data_ink",
                passed: estimated_ink >= *min_data_ink,
                detail: format!(
                    "estimated data-ink {estimated_ink:.3} >= required {min_data_ink:.3}"
                ),
            }]
        }
        RenderIntent::Interactive { min_targets } => {
            let targets = count_interactive_targets(payload);
            vec![CheckResult {
                check: "intent_interactive_targets",
                passed: targets >= *min_targets,
                detail: format!("{targets} targets >= required {min_targets}"),
            }]
        }
        RenderIntent::StatusGauge { show_thresholds } => {
            let is_gauge = matches!(payload.data, SceneData::Gauge { .. });
            let mut results = vec![CheckResult {
                check: "intent_gauge_type",
                passed: is_gauge,
                detail: format!("binding_type={}, is_gauge={is_gauge}", payload.binding_type),
            }];
            if *show_thresholds {
                results.push(CheckResult {
                    check: "intent_gauge_thresholds",
                    passed: is_gauge,
                    detail: "threshold display requires gauge binding".into(),
                });
            }
            results
        }
        RenderIntent::SpatialMap { min_entities } => {
            let entity_count = match &payload.data {
                SceneData::FieldMap { entities, .. } => entities.len(),
                _ => 0,
            };
            vec![CheckResult {
                check: "intent_spatial_entities",
                passed: entity_count >= *min_entities,
                detail: format!("{entity_count} entities >= required {min_entities}"),
            }]
        }
        RenderIntent::NarrativeSequence { min_nodes } => {
            let node_count = match &payload.data {
                SceneData::GameScene { nodes, .. } => nodes.len(),
                _ => 0,
            };
            vec![CheckResult {
                check: "intent_narrative_nodes",
                passed: node_count >= *min_nodes,
                detail: format!("{node_count} nodes >= required {min_nodes}"),
            }]
        }
    }
}

/// Estimate data-ink ratio based on payload structure.
///
/// Heuristic: ratio of meaningful data elements to total visual budget.
/// TimeSeries with many points → high ink. Gauge with 1 value → lower ink
/// (more chrome for context). This mirrors petalTongue's Tufte calculations.
const fn estimate_data_ink(payload: &ScenePayload) -> f64 {
    match &payload.data {
        SceneData::TimeSeries { x_values, .. } => {
            let n = x_values.len();
            if n > 50 {
                0.85
            } else if n > 10 {
                0.7
            } else {
                0.5
            }
        }
        SceneData::Heatmap { values, .. } => {
            if values.len() > 100 {
                0.9
            } else {
                0.75
            }
        }
        SceneData::Bar { values, .. } => {
            if values.len() > 10 {
                0.7
            } else {
                0.6
            }
        }
        SceneData::Gauge { .. } => 0.4,
        SceneData::FieldMap { entities, .. } => {
            if entities.len() > 20 {
                0.8
            } else {
                0.6
            }
        }
        SceneData::GameScene { nodes, .. } => {
            if nodes.len() > 10 {
                0.75
            } else {
                0.6
            }
        }
    }
}

/// Count interactive targets in a payload.
const fn count_interactive_targets(payload: &ScenePayload) -> usize {
    match &payload.data {
        SceneData::GameScene { nodes, .. } => nodes.len(),
        SceneData::FieldMap { entities, .. } => entities.len(),
        SceneData::Bar { values, .. } | SceneData::Heatmap { values, .. } => values.len(),
        SceneData::TimeSeries { x_values, .. } => x_values.len(),
        SceneData::Gauge { .. } => 1,
    }
}

/// Generate a full meta-tier validation suite for all ludoSpring game channels.
///
/// Creates validation cases that exercise the entire GameChannelType → petalTongue
/// rendering intent mapping, providing test coverage for the scene composition layer.
#[must_use]
#[allow(
    clippy::too_many_lines,
    reason = "suite builder constructing diverse validation scenarios"
)]
pub fn game_science_validation_suite() -> Vec<MetaValidationCase> {
    use super::{GameChannelType, GameDataChannel};
    use crate::visualization::GameDataPoint;

    let mut cases = Vec::new();

    let engagement_channel = GameDataChannel {
        name: "player_engagement_60s".into(),
        channel_type: GameChannelType::EngagementCurve,
        data: (0..60)
            .map(|i| GameDataPoint {
                x: f64::from(i),
                y: f64::from(i).mul_add(0.01, 0.4),
                label: None,
                category: None,
            })
            .collect(),
        x_unit: "seconds".into(),
        y_unit: "engagement".into(),
    };
    cases.push(MetaValidationCase {
        name: "engagement_analytical_density".into(),
        intent: RenderIntent::AnalyticalDense { min_data_ink: 0.7 },
        payload: super::scene::compose_scene(&engagement_channel),
    });

    let combat_channel = GameDataChannel {
        name: "tactical_grid".into(),
        channel_type: GameChannelType::CombatGrid,
        data: vec![
            GameDataPoint {
                x: 2.0,
                y: 3.0,
                label: Some("warrior".into()),
                category: None,
            },
            GameDataPoint {
                x: 8.0,
                y: 5.0,
                label: Some("mage".into()),
                category: None,
            },
            GameDataPoint {
                x: 14.0,
                y: 2.0,
                label: Some("goblin_1".into()),
                category: None,
            },
            GameDataPoint {
                x: 15.0,
                y: 6.0,
                label: Some("goblin_2".into()),
                category: None,
            },
            GameDataPoint {
                x: 10.0,
                y: 10.0,
                label: Some("chest".into()),
                category: None,
            },
        ],
        x_unit: "tile_x".into(),
        y_unit: "tile_y".into(),
    };
    cases.push(MetaValidationCase {
        name: "combat_spatial_layout".into(),
        intent: RenderIntent::SpatialMap { min_entities: 3 },
        payload: super::scene::compose_scene(&combat_channel),
    });

    let dialogue_channel = GameDataChannel {
        name: "quest_dialogue".into(),
        channel_type: GameChannelType::DialogueTree,
        data: vec![
            GameDataPoint {
                x: 0.0,
                y: 0.0,
                label: Some("npc_greeting".into()),
                category: Some("npc".into()),
            },
            GameDataPoint {
                x: 1.0,
                y: 0.0,
                label: Some("ask_about_dragon".into()),
                category: Some("choice".into()),
            },
            GameDataPoint {
                x: 2.0,
                y: 0.0,
                label: Some("dragon_lore".into()),
                category: Some("npc".into()),
            },
            GameDataPoint {
                x: 3.0,
                y: 0.0,
                label: Some("accept_quest".into()),
                category: Some("choice".into()),
            },
            GameDataPoint {
                x: 4.0,
                y: 0.0,
                label: Some("quest_details".into()),
                category: Some("npc".into()),
            },
        ],
        x_unit: String::new(),
        y_unit: String::new(),
    };
    cases.push(MetaValidationCase {
        name: "dialogue_narrative_flow".into(),
        intent: RenderIntent::NarrativeSequence { min_nodes: 4 },
        payload: super::scene::compose_scene(&dialogue_channel),
    });

    let dice_channel = GameDataChannel {
        name: "skill_check_result".into(),
        channel_type: GameChannelType::DiceResult,
        data: vec![GameDataPoint {
            x: 0.0,
            y: 72.0,
            label: Some("athletics".into()),
            category: None,
        }],
        x_unit: String::new(),
        y_unit: "percent".into(),
    };
    cases.push(MetaValidationCase {
        name: "dice_status_gauge".into(),
        intent: RenderIntent::StatusGauge {
            show_thresholds: true,
        },
        payload: super::scene::compose_scene(&dice_channel),
    });

    let exploration_channel = GameDataChannel {
        name: "dungeon_map".into(),
        channel_type: GameChannelType::ExplorationMap,
        data: (0..12)
            .map(|i| GameDataPoint {
                x: f64::from(i % 4) * 8.0,
                y: f64::from(i / 4) * 8.0,
                label: Some(format!("poi_{i}")),
                category: None,
            })
            .collect(),
        x_unit: "tile_x".into(),
        y_unit: "tile_y".into(),
    };
    cases.push(MetaValidationCase {
        name: "exploration_spatial_density".into(),
        intent: RenderIntent::SpatialMap { min_entities: 8 },
        payload: super::scene::compose_scene(&exploration_channel),
    });

    let npc_channel = GameDataChannel {
        name: "merchant_disposition".into(),
        channel_type: GameChannelType::NpcStatus,
        data: vec![GameDataPoint {
            x: 0.0,
            y: 65.0,
            label: Some("neutral_friendly".into()),
            category: None,
        }],
        x_unit: String::new(),
        y_unit: "disposition".into(),
    };
    cases.push(MetaValidationCase {
        name: "npc_status_gauge".into(),
        intent: RenderIntent::StatusGauge {
            show_thresholds: false,
        },
        payload: super::scene::compose_scene(&npc_channel),
    });

    let ui_channel = GameDataChannel {
        name: "main_menu_analysis".into(),
        channel_type: GameChannelType::UiAnalysis,
        data: (0..64)
            .map(|i| GameDataPoint {
                x: f64::from(i % 8),
                y: f64::from(i) * 0.1,
                label: None,
                category: None,
            })
            .collect(),
        x_unit: "grid_x".into(),
        y_unit: "data_ink".into(),
    };
    cases.push(MetaValidationCase {
        name: "ui_tufte_analysis".into(),
        intent: RenderIntent::AnalyticalDense { min_data_ink: 0.6 },
        payload: super::scene::compose_scene(&ui_channel),
    });

    cases
}

#[cfg(test)]
#[allow(clippy::manual_string_new, clippy::suboptimal_flops)]
mod tests {
    use super::*;

    #[test]
    fn validation_suite_all_pass() {
        let suite = game_science_validation_suite();
        assert!(!suite.is_empty());
        for case in &suite {
            let result = validate_meta(case);
            assert!(
                result.passed,
                "case '{}' failed: {:?}",
                result.name,
                result
                    .checks
                    .iter()
                    .filter(|c| !c.passed)
                    .collect::<Vec<_>>()
            );
        }
    }

    #[test]
    fn suite_covers_all_intents() {
        let suite = game_science_validation_suite();
        let has_analytical = suite
            .iter()
            .any(|c| matches!(c.intent, RenderIntent::AnalyticalDense { .. }));
        let has_interactive = suite
            .iter()
            .any(|c| matches!(c.intent, RenderIntent::Interactive { .. }));
        let has_gauge = suite
            .iter()
            .any(|c| matches!(c.intent, RenderIntent::StatusGauge { .. }));
        let has_spatial = suite
            .iter()
            .any(|c| matches!(c.intent, RenderIntent::SpatialMap { .. }));
        let has_narrative = suite
            .iter()
            .any(|c| matches!(c.intent, RenderIntent::NarrativeSequence { .. }));

        assert!(has_analytical, "missing AnalyticalDense case");
        assert!(has_gauge, "missing StatusGauge case");
        assert!(has_spatial, "missing SpatialMap case");
        assert!(has_narrative, "missing NarrativeSequence case");
        // Interactive not yet in suite — will be added when we wire petalTongue IPC
        let _ = has_interactive;
    }

    #[test]
    fn failing_intent_detected() {
        use crate::visualization::GameDataPoint;

        let sparse_channel = super::super::GameDataChannel {
            name: "sparse".into(),
            channel_type: super::super::GameChannelType::EngagementCurve,
            data: vec![
                GameDataPoint {
                    x: 0.0,
                    y: 1.0,
                    label: None,
                    category: None,
                },
                GameDataPoint {
                    x: 1.0,
                    y: 2.0,
                    label: None,
                    category: None,
                },
            ],
            x_unit: "x".into(),
            y_unit: "y".into(),
        };

        let case = MetaValidationCase {
            name: "impossible_density".into(),
            intent: RenderIntent::AnalyticalDense { min_data_ink: 0.95 },
            payload: super::super::scene::compose_scene(&sparse_channel),
        };

        let result = validate_meta(&case);
        assert!(!result.passed, "sparse data should not meet 0.95 data-ink");
    }

    #[test]
    fn spatial_intent_fails_for_gauge() {
        use crate::visualization::GameDataPoint;

        let gauge_channel = super::super::GameDataChannel {
            name: "score".into(),
            channel_type: super::super::GameChannelType::AccessibilityReport,
            data: vec![GameDataPoint {
                x: 0.0,
                y: 50.0,
                label: None,
                category: None,
            }],
            x_unit: "".into(),
            y_unit: "score".into(),
        };

        let case = MetaValidationCase {
            name: "wrong_intent".into(),
            intent: RenderIntent::SpatialMap { min_entities: 5 },
            payload: super::super::scene::compose_scene(&gauge_channel),
        };

        let result = validate_meta(&case);
        assert!(!result.passed, "gauge should not satisfy spatial intent");
    }
}

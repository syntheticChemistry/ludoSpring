// SPDX-License-Identifier: AGPL-3.0-or-later
//! Scene composition — mapping game channels to petalTongue DataBinding.
//!
//! This module bridges ludoSpring's `GameChannelType` analytics with
//! petalTongue's scene graph pipeline. Each game channel maps to a
//! specific `DataBinding` variant that petalTongue's Grammar of Graphics
//! compiler can transform into rendered output.
//!
//! The mapping is:
//! - `EngagementCurve` → `TimeSeries` (line plot over session time)
//! - `DifficultyProfile` → `TimeSeries` (difficulty vs progress)
//! - `FlowTimeline` → `Bar` (flow state duration per phase)
//! - `UiAnalysis` → `Heatmap` (Tufte data-ink analysis grid)
//! - `InteractionCostMap` → `Heatmap` (Fitts/Hick cost surface)
//! - `GenerationPreview` → `FieldMap` (2D tile/spatial preview)
//! - `AccessibilityReport` → `Gauge` (0–100 accessibility score)
//! - `DialogueTree` → `GameScene` (branching node graph)
//! - `CharacterSheet` → `Gauge` (multi-stat gauges)
//! - `CombatGrid` → `FieldMap` (tactical spatial layout)
//! - `VoiceDisplay` → `Bar` (voice priority stack)
//! - `NpcStatus` → `Gauge` (disposition/trust meters)
//! - `DiceResult` → `Gauge` (roll result vs target)
//! - `ExplorationMap` → `FieldMap` (fog-of-war spatial)
//! - `NarrationStream` → `TimeSeries` (text event timeline)
//!
//! # petalTongue Pipeline
//!
//! ```text
//! GameDataChannel → ScenePayload → JSON-RPC → petalTongue
//!                                              │
//!                                      DataBindingCompiler
//!                                              │
//!                                       GrammarExpr → SceneGraph → Render
//! ```

use super::{GameChannelType, GameDataChannel};

/// A scene payload ready for petalTongue consumption via `visualization.render.scene`.
///
/// This is the wire format: a JSON object matching petalTongue's `DataBinding`
/// enum variants. The receiver's `DataBindingCompiler` transforms it into a
/// renderable scene graph.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "ipc", derive(serde::Serialize, serde::Deserialize))]
pub struct ScenePayload {
    /// Binding type (petalTongue DataBinding variant name).
    pub binding_type: &'static str,
    /// Unique channel identifier.
    pub id: String,
    /// Display label.
    pub label: String,
    /// Structured data payload (variant-specific fields).
    pub data: SceneData,
}

/// Variant-specific data for scene composition.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "ipc", derive(serde::Serialize, serde::Deserialize))]
pub enum SceneData {
    /// Time-series: x/y arrays with axis labels.
    TimeSeries {
        /// X-axis label.
        x_label: String,
        /// Y-axis label.
        y_label: String,
        /// Unit of measurement.
        unit: String,
        /// X values (time/progress).
        x_values: Vec<f64>,
        /// Y values (metric).
        y_values: Vec<f64>,
    },
    /// Bar chart: named categories with values.
    Bar {
        /// Category labels.
        categories: Vec<String>,
        /// Values per category.
        values: Vec<f64>,
        /// Unit of measurement.
        unit: String,
    },
    /// Heatmap: 2D grid of values.
    Heatmap {
        /// Row count.
        rows: usize,
        /// Column count.
        cols: usize,
        /// Flattened row-major values.
        values: Vec<f64>,
        /// Value interpretation label.
        metric: String,
    },
    /// Gauge: single value with range.
    Gauge {
        /// Current value.
        value: f64,
        /// Minimum of range.
        min: f64,
        /// Maximum of range.
        max: f64,
        /// Unit label.
        unit: String,
    },
    /// FieldMap: spatial 2D layout with entities.
    FieldMap {
        /// Map width in tiles.
        width: u32,
        /// Map height in tiles.
        height: u32,
        /// Entity positions as (x, y, label) tuples.
        entities: Vec<(f64, f64, String)>,
    },
    /// GameScene: structured node graph (dialogue trees, etc.).
    GameScene {
        /// Nodes with (id, label, type).
        nodes: Vec<(String, String, String)>,
        /// Edges with (from_id, to_id).
        edges: Vec<(String, String)>,
    },
}

/// Integer square root ceiling (avoids f64 cast precision issues).
const fn isqrt_ceil(n: usize) -> usize {
    if n == 0 {
        return 0;
    }
    let mut x = 1;
    while x * x < n {
        x += 1;
    }
    x
}

/// Compose a `ScenePayload` from a `GameDataChannel`.
///
/// Maps the channel type to the appropriate petalTongue DataBinding variant
/// and transforms the data points into the expected structure.
#[must_use]
#[allow(
    clippy::too_many_lines,
    reason = "single cohesive match over all 15 GameChannelType variants"
)]
pub fn compose_scene(channel: &GameDataChannel) -> ScenePayload {
    match channel.channel_type {
        GameChannelType::EngagementCurve
        | GameChannelType::DifficultyProfile
        | GameChannelType::NarrationStream => ScenePayload {
            binding_type: "timeseries",
            id: channel.name.clone(),
            label: channel.name.clone(),
            data: SceneData::TimeSeries {
                x_label: channel.x_unit.clone(),
                y_label: channel.y_unit.clone(),
                unit: channel.y_unit.clone(),
                x_values: channel.data.iter().map(|p| p.x).collect(),
                y_values: channel.data.iter().map(|p| p.y).collect(),
            },
        },

        GameChannelType::FlowTimeline | GameChannelType::VoiceDisplay => ScenePayload {
            binding_type: "bar",
            id: channel.name.clone(),
            label: channel.name.clone(),
            data: SceneData::Bar {
                categories: channel
                    .data
                    .iter()
                    .map(|p| p.label.clone().unwrap_or_default())
                    .collect(),
                values: channel.data.iter().map(|p| p.y).collect(),
                unit: channel.y_unit.clone(),
            },
        },

        GameChannelType::UiAnalysis | GameChannelType::InteractionCostMap => {
            let n = isqrt_ceil(channel.data.len());
            ScenePayload {
                binding_type: "heatmap",
                id: channel.name.clone(),
                label: channel.name.clone(),
                data: SceneData::Heatmap {
                    rows: n,
                    cols: n,
                    values: channel.data.iter().map(|p| p.y).collect(),
                    metric: channel.y_unit.clone(),
                },
            }
        }

        GameChannelType::AccessibilityReport
        | GameChannelType::CharacterSheet
        | GameChannelType::NpcStatus
        | GameChannelType::DiceResult => {
            let value = channel.data.first().map_or(0.0, |p| p.y);
            ScenePayload {
                binding_type: "gauge",
                id: channel.name.clone(),
                label: channel.name.clone(),
                data: SceneData::Gauge {
                    value,
                    min: 0.0,
                    max: 100.0,
                    unit: channel.y_unit.clone(),
                },
            }
        }

        GameChannelType::GenerationPreview
        | GameChannelType::CombatGrid
        | GameChannelType::ExplorationMap => ScenePayload {
            binding_type: "fieldmap",
            id: channel.name.clone(),
            label: channel.name.clone(),
            data: SceneData::FieldMap {
                width: 32,
                height: 32,
                entities: channel
                    .data
                    .iter()
                    .map(|p| (p.x, p.y, p.label.clone().unwrap_or_default()))
                    .collect(),
            },
        },

        GameChannelType::DialogueTree => ScenePayload {
            binding_type: "gamescene",
            id: channel.name.clone(),
            label: channel.name.clone(),
            data: SceneData::GameScene {
                nodes: channel
                    .data
                    .iter()
                    .enumerate()
                    .map(|(i, p)| {
                        (
                            format!("node_{i}"),
                            p.label.clone().unwrap_or_default(),
                            p.category.clone().unwrap_or_else(|| "choice".into()),
                        )
                    })
                    .collect(),
                edges: channel
                    .data
                    .windows(2)
                    .enumerate()
                    .map(|(i, _)| (format!("node_{i}"), format!("node_{}", i + 1)))
                    .collect(),
            },
        },
    }
}

/// Validate that a `ScenePayload` meets petalTongue's structural requirements.
///
/// Returns a list of validation issues (empty = valid).
#[must_use]
pub fn validate_payload(payload: &ScenePayload) -> Vec<String> {
    let mut issues = Vec::new();

    if payload.id.is_empty() {
        issues.push("id must not be empty".into());
    }

    match &payload.data {
        SceneData::TimeSeries {
            x_values, y_values, ..
        } => {
            if x_values.len() != y_values.len() {
                issues.push(format!(
                    "x_values ({}) and y_values ({}) must have same length",
                    x_values.len(),
                    y_values.len()
                ));
            }
        }
        SceneData::Heatmap {
            rows, cols, values, ..
        } => {
            let expected = rows * cols;
            if values.len() != expected {
                issues.push(format!(
                    "heatmap has {} values but rows*cols = {expected}",
                    values.len()
                ));
            }
        }
        SceneData::Gauge {
            value, min, max, ..
        } => {
            if value < min || value > max {
                issues.push(format!("gauge value {value} outside [{min}, {max}]"));
            }
        }
        SceneData::FieldMap { entities, .. } => {
            if entities.is_empty() {
                issues.push("fieldmap has no entities".into());
            }
        }
        SceneData::GameScene { nodes, .. } => {
            if nodes.is_empty() {
                issues.push("gamescene has no nodes".into());
            }
        }
        SceneData::Bar {
            categories, values, ..
        } => {
            if categories.len() != values.len() {
                issues.push(format!(
                    "bar categories ({}) and values ({}) must have same length",
                    categories.len(),
                    values.len()
                ));
            }
        }
    }

    issues
}

#[cfg(test)]
#[allow(clippy::manual_string_new, clippy::suboptimal_flops)]
mod tests {
    use super::*;
    use crate::visualization::GameDataPoint;

    fn sample_engagement_channel() -> GameDataChannel {
        GameDataChannel {
            name: "session_engagement".into(),
            channel_type: GameChannelType::EngagementCurve,
            data: (0..10)
                .map(|i| GameDataPoint {
                    x: f64::from(i),
                    y: 0.5 + f64::from(i) * 0.05,
                    label: None,
                    category: None,
                })
                .collect(),
            x_unit: "seconds".into(),
            y_unit: "engagement".into(),
        }
    }

    fn sample_flow_channel() -> GameDataChannel {
        GameDataChannel {
            name: "flow_phases".into(),
            channel_type: GameChannelType::FlowTimeline,
            data: vec![
                GameDataPoint {
                    x: 0.0,
                    y: 12.0,
                    label: Some("anxiety".into()),
                    category: None,
                },
                GameDataPoint {
                    x: 1.0,
                    y: 45.0,
                    label: Some("flow".into()),
                    category: None,
                },
                GameDataPoint {
                    x: 2.0,
                    y: 8.0,
                    label: Some("boredom".into()),
                    category: None,
                },
            ],
            x_unit: "phase".into(),
            y_unit: "duration_s".into(),
        }
    }

    fn sample_accessibility_channel() -> GameDataChannel {
        GameDataChannel {
            name: "accessibility_score".into(),
            channel_type: GameChannelType::AccessibilityReport,
            data: vec![GameDataPoint {
                x: 0.0,
                y: 78.0,
                label: None,
                category: None,
            }],
            x_unit: "".into(),
            y_unit: "score".into(),
        }
    }

    fn sample_dialogue_channel() -> GameDataChannel {
        GameDataChannel {
            name: "dialogue_branch".into(),
            channel_type: GameChannelType::DialogueTree,
            data: vec![
                GameDataPoint {
                    x: 0.0,
                    y: 0.0,
                    label: Some("greeting".into()),
                    category: Some("npc".into()),
                },
                GameDataPoint {
                    x: 1.0,
                    y: 0.0,
                    label: Some("ask_quest".into()),
                    category: Some("choice".into()),
                },
                GameDataPoint {
                    x: 2.0,
                    y: 0.0,
                    label: Some("accept".into()),
                    category: Some("choice".into()),
                },
            ],
            x_unit: "".into(),
            y_unit: "".into(),
        }
    }

    fn sample_combat_channel() -> GameDataChannel {
        GameDataChannel {
            name: "combat_grid".into(),
            channel_type: GameChannelType::CombatGrid,
            data: vec![
                GameDataPoint {
                    x: 5.0,
                    y: 3.0,
                    label: Some("player".into()),
                    category: None,
                },
                GameDataPoint {
                    x: 12.0,
                    y: 8.0,
                    label: Some("enemy".into()),
                    category: None,
                },
                GameDataPoint {
                    x: 7.0,
                    y: 7.0,
                    label: Some("ally".into()),
                    category: None,
                },
            ],
            x_unit: "tile_x".into(),
            y_unit: "tile_y".into(),
        }
    }

    #[test]
    fn engagement_maps_to_timeseries() {
        let payload = compose_scene(&sample_engagement_channel());
        assert_eq!(payload.binding_type, "timeseries");
        if let SceneData::TimeSeries {
            x_values, y_values, ..
        } = &payload.data
        {
            assert_eq!(x_values.len(), 10);
            assert_eq!(y_values.len(), 10);
        } else {
            panic!("expected TimeSeries");
        }
    }

    #[test]
    fn flow_maps_to_bar() {
        let payload = compose_scene(&sample_flow_channel());
        assert_eq!(payload.binding_type, "bar");
        if let SceneData::Bar {
            categories, values, ..
        } = &payload.data
        {
            assert_eq!(categories.len(), 3);
            assert_eq!(values.len(), 3);
            assert_eq!(categories[1], "flow");
        } else {
            panic!("expected Bar");
        }
    }

    #[test]
    fn accessibility_maps_to_gauge() {
        let payload = compose_scene(&sample_accessibility_channel());
        assert_eq!(payload.binding_type, "gauge");
        if let SceneData::Gauge {
            value, min, max, ..
        } = &payload.data
        {
            assert!((value - 78.0).abs() < f64::EPSILON);
            assert!((min - 0.0).abs() < f64::EPSILON);
            assert!((max - 100.0).abs() < f64::EPSILON);
        } else {
            panic!("expected Gauge");
        }
    }

    #[test]
    fn dialogue_maps_to_gamescene() {
        let payload = compose_scene(&sample_dialogue_channel());
        assert_eq!(payload.binding_type, "gamescene");
        if let SceneData::GameScene { nodes, edges } = &payload.data {
            assert_eq!(nodes.len(), 3);
            assert_eq!(edges.len(), 2);
            assert_eq!(nodes[0].1, "greeting");
            assert_eq!(nodes[0].2, "npc");
        } else {
            panic!("expected GameScene");
        }
    }

    #[test]
    fn combat_maps_to_fieldmap() {
        let payload = compose_scene(&sample_combat_channel());
        assert_eq!(payload.binding_type, "fieldmap");
        if let SceneData::FieldMap {
            entities,
            width,
            height,
        } = &payload.data
        {
            assert_eq!(entities.len(), 3);
            assert_eq!(*width, 32);
            assert_eq!(*height, 32);
        } else {
            panic!("expected FieldMap");
        }
    }

    #[test]
    fn valid_payload_passes_validation() {
        let payload = compose_scene(&sample_engagement_channel());
        let issues = validate_payload(&payload);
        assert!(issues.is_empty(), "unexpected issues: {issues:?}");
    }

    #[test]
    fn mismatched_timeseries_fails_validation() {
        let mut payload = compose_scene(&sample_engagement_channel());
        if let SceneData::TimeSeries {
            ref mut y_values, ..
        } = payload.data
        {
            y_values.push(999.0);
        }
        let issues = validate_payload(&payload);
        assert!(!issues.is_empty());
    }

    #[test]
    fn gauge_out_of_range_fails_validation() {
        let payload = ScenePayload {
            binding_type: "gauge",
            id: "test".into(),
            label: "test".into(),
            data: SceneData::Gauge {
                value: 150.0,
                min: 0.0,
                max: 100.0,
                unit: "x".into(),
            },
        };
        let issues = validate_payload(&payload);
        assert!(!issues.is_empty());
    }

    #[test]
    fn empty_id_fails_validation() {
        let payload = ScenePayload {
            binding_type: "gauge",
            id: String::new(),
            label: "test".into(),
            data: SceneData::Gauge {
                value: 50.0,
                min: 0.0,
                max: 100.0,
                unit: "x".into(),
            },
        };
        let issues = validate_payload(&payload);
        assert!(issues.iter().any(|i| i.contains("id must not be empty")));
    }

    #[test]
    fn all_channel_types_compose_without_panic() {
        let channel_types = [
            GameChannelType::EngagementCurve,
            GameChannelType::DifficultyProfile,
            GameChannelType::FlowTimeline,
            GameChannelType::UiAnalysis,
            GameChannelType::InteractionCostMap,
            GameChannelType::GenerationPreview,
            GameChannelType::AccessibilityReport,
            GameChannelType::DialogueTree,
            GameChannelType::CharacterSheet,
            GameChannelType::CombatGrid,
            GameChannelType::VoiceDisplay,
            GameChannelType::NpcStatus,
            GameChannelType::DiceResult,
            GameChannelType::ExplorationMap,
            GameChannelType::NarrationStream,
        ];

        for ct in channel_types {
            let channel = GameDataChannel {
                name: format!("test_{ct:?}"),
                channel_type: ct,
                data: vec![
                    GameDataPoint {
                        x: 1.0,
                        y: 2.0,
                        label: Some("a".into()),
                        category: Some("c".into()),
                    },
                    GameDataPoint {
                        x: 3.0,
                        y: 4.0,
                        label: Some("b".into()),
                        category: Some("d".into()),
                    },
                ],
                x_unit: "x".into(),
                y_unit: "y".into(),
            };
            let payload = compose_scene(&channel);
            assert!(!payload.id.is_empty(), "empty id for {ct:?}");
        }
    }
}

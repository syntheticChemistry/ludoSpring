// SPDX-License-Identifier: AGPL-3.0-or-later
//! JSON-RPC method result types.
//!
//! Each struct maps to the `result` field of a successful JSON-RPC response.
//! All use f64 for numeric values (JSON number type) and String for text.

use serde::Serialize;

/// Result for `game.evaluate_flow`.
#[derive(Debug, Serialize)]
pub struct FlowResult {
    /// Flow state name.
    pub state: String,
}

/// Result for `game.fitts_cost`.
#[derive(Debug, Serialize)]
pub struct FittsCostResult {
    /// Movement time in milliseconds.
    pub movement_time_ms: f64,
    /// Index of difficulty in bits.
    pub index_of_difficulty: f64,
}

/// Result for `game.engagement`.
#[derive(Debug, Serialize)]
pub struct EngagementResult {
    /// Actions per minute.
    pub actions_per_minute: f64,
    /// Exploration rate.
    pub exploration_rate: f64,
    /// Challenge appetite.
    pub challenge_appetite: f64,
    /// Persistence score.
    pub persistence: f64,
    /// Deliberation score.
    pub deliberation: f64,
    /// Composite engagement (0.0–1.0).
    pub composite: f64,
}

/// Result for `game.generate_noise`.
#[derive(Debug, Serialize)]
pub struct NoiseResult {
    /// Noise value at the requested coordinates.
    pub value: f64,
}

/// Result for `game.analyze_ui`.
#[derive(Debug, Serialize)]
pub struct UiAnalysisResult {
    /// Overall data-ink ratio.
    pub data_ink_ratio: f64,
    /// Information density.
    pub info_density: f64,
    /// Screen coverage fraction.
    pub screen_coverage: f64,
    /// Advisory notes.
    pub notes: Vec<String>,
}

/// Result for `game.accessibility`.
#[derive(Debug, Serialize)]
pub struct AccessibilityResult {
    /// Visual accessibility score (0.0–1.0).
    pub score: f64,
    /// Issues found.
    pub issues: Vec<String>,
    /// Strengths found.
    pub strengths: Vec<String>,
}

/// Result for `game.difficulty_adjustment`.
#[derive(Debug, Serialize)]
pub struct DifficultyAdjustmentResult {
    /// Recommended adjustment in \[-1.0, 1.0\] (negative = easier, positive = harder).
    pub adjustment: f64,
    /// Estimated current player skill (0.0–1.0).
    pub estimated_skill: f64,
    /// Trend: positive = improving, negative = declining.
    pub trend: f64,
}

/// Result for `game.wfc_step`.
#[derive(Debug, Serialize)]
pub struct WfcStepResult {
    /// Whether the grid is fully collapsed.
    pub fully_collapsed: bool,
    /// Whether any contradiction exists.
    pub has_contradiction: bool,
    /// Number of options removed by propagation.
    pub options_removed: usize,
}

// SPDX-License-Identifier: AGPL-3.0-or-later
//! JSON-RPC method parameter types.
//!
//! Each struct maps to the `params` field of a specific JSON-RPC method.
//! All fields use the caller's native types; type coercion happens in handlers.

use serde::Deserialize;

/// Parameters for `game.evaluate_flow`.
#[derive(Debug, Deserialize)]
pub struct EvaluateFlowParams {
    /// Normalized challenge level (0.0–1.0).
    pub challenge: f64,
    /// Normalized player skill (0.0–1.0).
    pub skill: f64,
    /// Optional custom channel width (defaults to tolerance constant).
    pub channel_width: Option<f64>,
}

/// Parameters for `game.fitts_cost`.
#[derive(Debug, Deserialize)]
pub struct FittsCostParams {
    /// Distance to target center (pixels or arbitrary units).
    pub distance: f64,
    /// Target width along movement axis.
    pub target_width: f64,
    /// Optional Fitts `a` parameter (defaults to mouse constant).
    pub a: Option<f64>,
    /// Optional Fitts `b` parameter (defaults to mouse constant).
    pub b: Option<f64>,
}

/// Parameters for `game.engagement`.
#[derive(Debug, Deserialize)]
pub struct EngagementParams {
    /// Session duration in seconds.
    pub session_duration_s: f64,
    /// Number of meaningful actions.
    pub action_count: u64,
    /// Distinct areas explored.
    pub exploration_breadth: u32,
    /// Voluntary difficulty increases.
    pub challenge_seeking: u32,
    /// Retry count after failure.
    pub retry_count: u32,
    /// Deliberate pauses (thinking, not frustrated).
    pub deliberate_pauses: u32,
}

/// Parameters for `game.generate_noise`.
#[derive(Debug, Deserialize)]
pub struct GenerateNoiseParams {
    /// X coordinate.
    pub x: f64,
    /// Y coordinate.
    pub y: f64,
    /// Z coordinate (if absent, uses 2D noise).
    pub z: Option<f64>,
    /// Number of fBm octaves (default 4).
    pub octaves: Option<u32>,
    /// Lacunarity (default 2.0).
    pub lacunarity: Option<f64>,
    /// Persistence (default 0.5).
    pub persistence: Option<f64>,
}

/// Parameters for `game.analyze_ui`.
#[derive(Debug, Deserialize)]
pub struct AnalyzeUiParams {
    /// UI elements to analyze.
    pub elements: Vec<UiElementParam>,
}

/// A UI element for IPC transport.
#[derive(Debug, Deserialize)]
pub struct UiElementParam {
    /// Element name.
    pub name: String,
    /// Bounding box \[x, y, width, height\] in normalized screen coords.
    pub bounds: [f64; 4],
    /// Number of distinct data values conveyed.
    pub data_values: usize,
    /// Total pixel area.
    pub pixel_area: f64,
    /// Pixel area that directly encodes data.
    pub data_ink_area: f64,
    /// Whether the element must always be visible.
    pub critical: bool,
}

/// Parameters for `game.accessibility`.
#[derive(Debug, Deserialize)]
#[expect(
    clippy::struct_excessive_bools,
    reason = "mirrors VisualAccessibilityFeatures; each bool is a distinct IGDA/XAG feature"
)]
pub struct AccessibilityParams {
    /// Audio cues for visual events.
    pub audio_cues: bool,
    /// Natural language descriptions.
    pub descriptions: bool,
    /// Braille output support.
    pub braille: bool,
    /// Haptic feedback.
    pub haptic: bool,
    /// Color-independent information encoding.
    pub color_independent: bool,
    /// User-scalable text.
    pub scalable_text: bool,
}

/// Parameters for `game.wfc_step`.
#[derive(Debug, Deserialize)]
pub struct WfcStepParams {
    /// Grid width.
    pub width: usize,
    /// Grid height.
    pub height: usize,
    /// Number of tile types.
    pub n_tiles: usize,
    /// Cell to collapse: `(x, y, tile_id)`.
    pub collapse: Option<(usize, usize, u16)>,
}

/// Parameters for `game.difficulty_adjustment`.
#[derive(Debug, Deserialize)]
pub struct DifficultyAdjustmentParams {
    /// Recent outcomes (0.0 = fail, 1.0 = success).
    pub outcomes: Vec<f64>,
    /// Target success rate (default: `DDA_TARGET_SUCCESS_RATE` from tolerances).
    pub target_success_rate: Option<f64>,
}

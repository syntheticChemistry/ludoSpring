// SPDX-License-Identifier: AGPL-3.0-or-later
//! Visualization data channels for game analytics.
//!
//! Defines `DataChannel` types that any visualization-capable primal can
//! consume: engagement curves, difficulty profiles, UI analysis,
//! procedural generation previews, and interaction cost maps.
//! Discovered at runtime via the `visualization` capability.

#[cfg(feature = "ipc")]
pub mod push_client;

#[cfg(feature = "ipc")]
pub use push_client::PetalTonguePushClient;

/// A data channel for visualization consumers.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "ipc", derive(serde::Serialize, serde::Deserialize))]
pub struct GameDataChannel {
    /// Channel name (e.g., "`engagement_curve`", "`difficulty_profile`").
    pub name: String,
    /// Channel type for visualization routing.
    pub channel_type: GameChannelType,
    /// Data points.
    pub data: Vec<GameDataPoint>,
    /// Units for X axis.
    pub x_unit: String,
    /// Units for Y axis.
    pub y_unit: String,
}

/// Types of game data channels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "ipc", derive(serde::Serialize, serde::Deserialize))]
pub enum GameChannelType {
    /// Time series of engagement metrics.
    EngagementCurve,
    /// Difficulty profile over game progress.
    DifficultyProfile,
    /// Flow state transitions.
    FlowTimeline,
    /// UI element Tufte analysis.
    UiAnalysis,
    /// Interaction cost map (Fitts + Hick).
    InteractionCostMap,
    /// Procedural generation preview.
    GenerationPreview,
    /// Accessibility score breakdown.
    AccessibilityReport,
}

/// A data point in a game data channel.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "ipc", derive(serde::Serialize, serde::Deserialize))]
pub struct GameDataPoint {
    /// X value (typically time or progress).
    pub x: f64,
    /// Y value (metric value).
    pub y: f64,
    /// Optional label.
    pub label: Option<String>,
    /// Optional category for coloring.
    pub category: Option<String>,
}

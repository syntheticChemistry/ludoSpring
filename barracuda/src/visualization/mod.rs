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
pub use push_client::{PetalTonguePushClient, VisualizationPushClient};

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
///
/// Analytics channels route to petalTongue DataBinding types.
/// RPGPT channels route to petalTongue scene graph or grammar expressions
/// for game-specific UI composition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "ipc", derive(serde::Serialize, serde::Deserialize))]
pub enum GameChannelType {
    // ── Analytics channels (existing) ────────────────────────────────────
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

    // ── RPGPT game UI channels ───────────────────────────────────────────
    /// Dialogue tree with branching choices and skill check indicators.
    /// Renders as scene graph with selectable nodes.
    DialogueTree,
    /// Character sheet — stats, conditions, inventory, equipped items.
    /// Renders as multi-panel scene graph with gauge sub-bindings.
    CharacterSheet,
    /// Tactical combat grid — zones, positions, action indicators.
    /// Renders as FieldMap with interactive entity markers.
    CombatGrid,
    /// Internal voice display — passive check outputs, priority-ordered.
    /// Renders as stacked cards in scene graph, max 3 visible.
    VoiceDisplay,
    /// NPC interaction panel — name, disposition gauge, trust, portrait.
    /// Renders as scene graph with gauge + text sub-bindings.
    NpcStatus,
    /// Dice roll result with signed provenance and degree of success.
    /// Renders as animated bar/arc with BearDog signature badge.
    DiceResult,
    /// Exploration map with fog of war, points of interest, party marker.
    /// Renders as FieldMap with overlay geometry.
    ExplorationMap,
    /// Session narration stream — AI-generated text with voice attribution.
    /// Renders as streaming text panel with speaker indicators.
    NarrationStream,
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

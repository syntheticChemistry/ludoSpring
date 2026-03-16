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

/// Parameters for `game.begin_session` (provenance trio).
#[derive(Debug, Deserialize)]
pub struct BeginSessionParams {
    /// Human-readable session name.
    pub session_name: String,
}

/// Parameters for `game.record_action` (provenance trio).
#[derive(Debug, Deserialize)]
pub struct RecordActionParams {
    /// Session ID from `game.begin_session`.
    pub session_id: String,
    /// Game action as JSON (event type, metadata, etc.).
    pub action: serde_json::Value,
}

/// Parameters for `game.complete_session` (provenance trio).
#[derive(Debug, Deserialize)]
pub struct CompleteSessionParams {
    /// Session ID to dehydrate, commit, and attribute.
    pub session_id: String,
}

/// Parameters for `game.npc_dialogue` (Squirrel AI).
#[derive(Debug, Deserialize)]
pub struct NpcDialogueParams {
    /// NPC name for logging and context.
    pub npc_name: String,
    /// System prompt encoding NPC personality, knowledge bounds, trust level.
    pub personality_prompt: String,
    /// Player's dialogue input.
    pub player_input: String,
    /// Conversation history (role/content pairs).
    #[serde(default)]
    pub history: Vec<serde_json::Value>,
}

/// Parameters for `game.narrate_action` (Squirrel AI).
#[derive(Debug, Deserialize)]
pub struct NarrateActionParams {
    /// Description of the action to narrate.
    pub action: String,
    /// Surrounding context for the narration.
    pub context: String,
}

/// Parameters for `game.voice_check` (Squirrel AI).
#[derive(Debug, Deserialize)]
pub struct VoiceCheckParams {
    /// Internal voice name (e.g. "Logic", "Empathy").
    pub voice_name: String,
    /// Voice personality constraint.
    pub voice_personality: String,
    /// Summary of current game state for the voice to react to.
    pub game_state: String,
}

/// Parameters for `game.push_scene` (petalTongue).
#[derive(Debug, Deserialize)]
pub struct PushSceneParams {
    /// Session ID for petalTongue routing.
    pub session_id: String,
    /// Channel name (e.g. "DialogueTree", "CombatGrid").
    pub channel: String,
    /// Scene payload as JSON.
    pub scene: serde_json::Value,
}

/// Parameters for `game.query_vertices` (rhizoCrypt DAG).
#[derive(Debug, Deserialize)]
pub struct QueryVerticesParams {
    /// Session ID to query.
    pub session_id: String,
    /// Optional event type filter.
    pub event_type: Option<String>,
    /// Optional agent filter.
    pub agent: Option<String>,
    /// Max results (default 50).
    pub limit: Option<u32>,
}

/// Parameters for `game.mint_certificate` (loamSpine).
#[derive(Debug, Deserialize)]
pub struct MintCertificateParams {
    /// Certificate type (e.g. "NpcPersonality", "Ruleset", "CharacterSheet").
    pub cert_type: String,
    /// Certificate owner.
    pub owner: String,
    /// Certificate payload.
    pub payload: serde_json::Value,
}

/// Parameters for `game.storage_put` (NestGate).
#[derive(Debug, Deserialize)]
pub struct StoragePutParams {
    /// Storage key.
    pub key: String,
    /// Data to store.
    pub data: serde_json::Value,
    /// Optional metadata.
    #[serde(default = "default_json_object")]
    pub metadata: serde_json::Value,
}

fn default_json_object() -> serde_json::Value {
    serde_json::json!({})
}

/// Parameters for `game.storage_get` (NestGate).
#[derive(Debug, Deserialize)]
pub struct StorageGetParams {
    /// Storage key to retrieve.
    pub key: String,
}

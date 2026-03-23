// SPDX-License-Identifier: AGPL-3.0-or-later
//! JSON-RPC method parameter types.
//!
//! Each struct maps to the `params` field of a specific JSON-RPC method.
//! All fields use the caller's native types; type coercion happens in handlers.

use serde::{Deserialize, Serialize};

/// Parameters for `game.evaluate_flow`.
#[derive(Debug, Serialize, Deserialize)]
pub struct EvaluateFlowParams {
    /// Normalized challenge level (0.0–1.0).
    pub challenge: f64,
    /// Normalized player skill (0.0–1.0).
    pub skill: f64,
    /// Optional custom channel width (defaults to tolerance constant).
    pub channel_width: Option<f64>,
}

/// Parameters for `game.fitts_cost`.
#[derive(Debug, Serialize, Deserialize)]
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
#[derive(Debug, Serialize, Deserialize)]
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
#[derive(Debug, Serialize, Deserialize)]
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
#[derive(Debug, Serialize, Deserialize)]
pub struct AnalyzeUiParams {
    /// UI elements to analyze.
    pub elements: Vec<UiElementParam>,
}

/// A UI element for IPC transport.
#[derive(Debug, Serialize, Deserialize)]
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
#[derive(Debug, Serialize, Deserialize)]
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
#[derive(Debug, Serialize, Deserialize)]
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
#[derive(Debug, Serialize, Deserialize)]
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

/// Parameters for `tools.call` (MCP tool invocation).
#[derive(Debug, Deserialize)]
pub struct ToolsCallParams {
    /// Tool name (must match a tools.list entry).
    pub name: String,
    /// Tool arguments.
    pub arguments: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evaluate_flow_deserializes_full_and_minimal() {
        let full: EvaluateFlowParams = serde_json::from_value(serde_json::json!({
            "challenge": 0.3,
            "skill": 0.7,
            "channel_width": 0.12
        }))
        .expect("valid");
        assert!((full.challenge - 0.3).abs() < f64::EPSILON);
        assert!((full.skill - 0.7).abs() < f64::EPSILON);
        assert!((full.channel_width.expect("opt") - 0.12).abs() < f64::EPSILON);

        let minimal: EvaluateFlowParams = serde_json::from_value(serde_json::json!({
            "challenge": 0.0,
            "skill": 1.0
        }))
        .expect("valid");
        assert!(minimal.channel_width.is_none());
    }

    #[test]
    fn evaluate_flow_rejects_missing_required() {
        let err = serde_json::from_value::<EvaluateFlowParams>(serde_json::json!({
            "challenge": 0.5
        }));
        assert!(err.is_err());
    }

    #[test]
    fn fitts_cost_deserializes() {
        let p: FittsCostParams = serde_json::from_value(serde_json::json!({
            "distance": 10.0,
            "target_width": 2.0,
            "a": 1.0,
            "b": 2.0
        }))
        .expect("valid");
        assert!((p.distance - 10.0).abs() < f64::EPSILON);
        assert!((p.a.expect("a") - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn engagement_deserializes() {
        let p: EngagementParams = serde_json::from_value(serde_json::json!({
            "session_duration_s": 60.0,
            "action_count": 10,
            "exploration_breadth": 2,
            "challenge_seeking": 1,
            "retry_count": 0,
            "deliberate_pauses": 3
        }))
        .expect("valid");
        assert_eq!(p.action_count, 10);
    }

    #[test]
    fn generate_noise_deserializes_2d_and_3d() {
        let two: GenerateNoiseParams = serde_json::from_value(serde_json::json!({
            "x": 1.0,
            "y": 2.0
        }))
        .expect("valid");
        assert!(two.z.is_none());

        let three: GenerateNoiseParams = serde_json::from_value(serde_json::json!({
            "x": 0.0,
            "y": 0.0,
            "z": 3.0,
            "octaves": 2,
            "lacunarity": 2.5,
            "persistence": 0.4
        }))
        .expect("valid");
        assert!((three.z.expect("z") - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn analyze_ui_deserializes() {
        let p: AnalyzeUiParams = serde_json::from_value(serde_json::json!({
            "elements": [{
                "name": "hud",
                "bounds": [0.0, 0.0, 1.0, 0.1],
                "data_values": 2,
                "pixel_area": 50.0,
                "data_ink_area": 40.0,
                "critical": false
            }]
        }))
        .expect("valid");
        assert_eq!(p.elements.len(), 1);
        assert_eq!(p.elements[0].name, "hud");
    }

    #[test]
    fn accessibility_deserializes_and_rejects_incomplete() {
        let p: AccessibilityParams = serde_json::from_value(serde_json::json!({
            "audio_cues": true,
            "descriptions": false,
            "braille": false,
            "haptic": true,
            "color_independent": true,
            "scalable_text": true
        }))
        .expect("valid");
        assert!(p.audio_cues);

        let err = serde_json::from_value::<AccessibilityParams>(serde_json::json!({
            "audio_cues": true
        }));
        assert!(err.is_err());
    }

    #[test]
    fn wfc_step_deserializes() {
        let p: WfcStepParams = serde_json::from_value(serde_json::json!({
            "width": 8,
            "height": 8,
            "n_tiles": 4,
            "collapse": [0, 0, 1]
        }))
        .expect("valid");
        assert_eq!(p.collapse, Some((0, 0, 1)));
    }

    #[test]
    fn difficulty_adjustment_deserializes() {
        let p: DifficultyAdjustmentParams = serde_json::from_value(serde_json::json!({
            "outcomes": [0.5, 1.0],
            "target_success_rate": 0.75
        }))
        .expect("valid");
        assert_eq!(p.outcomes.len(), 2);
        assert!((p.target_success_rate.expect("t") - 0.75).abs() < f64::EPSILON);
    }

    #[test]
    fn provenance_and_squirrel_params_deserialize() {
        let _: BeginSessionParams = serde_json::from_value(serde_json::json!({
            "session_name": "run-1"
        }))
        .expect("valid");

        let _: RecordActionParams = serde_json::from_value(serde_json::json!({
            "session_id": "sid",
            "action": {"type": "jump"}
        }))
        .expect("valid");

        let _: CompleteSessionParams = serde_json::from_value(serde_json::json!({
            "session_id": "sid"
        }))
        .expect("valid");

        let with_history: NpcDialogueParams = serde_json::from_value(serde_json::json!({
            "npc_name": "Bob",
            "personality_prompt": "friendly",
            "player_input": "hey",
            "history": [{"role": "user", "content": "hi"}]
        }))
        .expect("valid");
        assert_eq!(with_history.history.len(), 1);

        let no_history: NpcDialogueParams = serde_json::from_value(serde_json::json!({
            "npc_name": "Bob",
            "personality_prompt": "friendly",
            "player_input": "hey"
        }))
        .expect("valid");
        assert!(no_history.history.is_empty());

        let _: NarrateActionParams = serde_json::from_value(serde_json::json!({
            "action": "open door",
            "context": "dungeon"
        }))
        .expect("valid");

        let _: VoiceCheckParams = serde_json::from_value(serde_json::json!({
            "voice_name": "Logic",
            "voice_personality": "cold",
            "game_state": "in combat"
        }))
        .expect("valid");

        let _: PushSceneParams = serde_json::from_value(serde_json::json!({
            "session_id": "s",
            "channel": "ui",
            "scene": {"nodes": []}
        }))
        .expect("valid");

        let _: QueryVerticesParams = serde_json::from_value(serde_json::json!({
            "session_id": "s",
            "event_type": "move",
            "agent": "p1",
            "limit": 10
        }))
        .expect("valid");

        let _: MintCertificateParams = serde_json::from_value(serde_json::json!({
            "cert_type": "Ruleset",
            "owner": "alice",
            "payload": {"v": 1}
        }))
        .expect("valid");
    }

    #[test]
    fn storage_put_defaults_metadata_object() {
        let p: StoragePutParams = serde_json::from_value(serde_json::json!({
            "key": "k",
            "data": {"x": 1}
        }))
        .expect("valid");
        assert_eq!(p.metadata, serde_json::json!({}));

        let p2: StoragePutParams = serde_json::from_value(serde_json::json!({
            "key": "k",
            "data": {"x": 1},
            "metadata": {"version": 2}
        }))
        .expect("valid");
        assert_eq!(p2.metadata["version"], 2);
    }

    #[test]
    fn storage_get_deserializes() {
        let p: StorageGetParams = serde_json::from_value(serde_json::json!({
            "key": "save-1"
        }))
        .expect("valid");
        assert_eq!(p.key, "save-1");
    }

    #[test]
    fn storage_get_rejects_missing_key() {
        let err = serde_json::from_value::<StorageGetParams>(serde_json::json!({}));
        assert!(err.is_err());
    }
}

// SPDX-License-Identifier: AGPL-3.0-or-later
//! Game telemetry event types — the portable protocol.
//!
//! Any game (Rust, Unity, Godot, web) can emit these events as NDJSON.
//! ludoSpring ingests them and maps to its analysis engines.
//!
//! # Wire format
//!
//! One JSON object per line (NDJSON). The `event_type` field uses
//! `snake_case` strings so non-Rust producers don't need enum awareness.
//!
//! ```json
//! {"timestamp_ms":1234567,"session_id":"abc","event_type":"player_move","payload":{"x":1.0,"y":2.0}}
//! ```

use serde::{Deserialize, Serialize};

/// Core event envelope — the unit of telemetry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryEvent {
    /// Milliseconds since session start (or Unix epoch — producer's choice).
    pub timestamp_ms: u64,
    /// Opaque session identifier.
    pub session_id: String,
    /// Event type tag (`snake_case` string for cross-language portability).
    pub event_type: EventType,
    /// Type-specific payload. Rust producers use typed structs; others write
    /// raw JSON matching the same schema.
    #[serde(default)]
    pub payload: serde_json::Value,
}

/// Telemetry event types.
///
/// `snake_case` serde representation for JSON portability.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    /// Game session started.
    SessionStart,
    /// Game session ended.
    SessionEnd,
    /// Player moved (position update).
    PlayerMove,
    /// Player performed a discrete action (attack, use item, interact, etc.).
    PlayerAction,
    /// Player took damage.
    PlayerDamage,
    /// Player died.
    PlayerDeath,
    /// Player encountered a challenge (enemy, puzzle, obstacle).
    ChallengeEncounter,
    /// Player completed a challenge.
    ChallengeComplete,
    /// Player failed a challenge.
    ChallengeFail,
    /// Player discovered a new area, item, or point of interest.
    ExplorationDiscover,
    /// Player interacted with a UI element.
    UiInteract,
    /// UI layout snapshot (element positions and sizes).
    UiLayout,
    /// Raw input event (keypress, button, mouse move).
    InputRaw,
    /// Producer-defined event type (extensibility).
    #[serde(untagged)]
    Custom(String),
}

// ── Typed payloads ─────────────────────────────────────────────────

/// Payload for `session_start`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionStartPayload {
    /// Game name / title.
    #[serde(default)]
    pub game_name: String,
    /// Game version string.
    #[serde(default)]
    pub game_version: String,
    /// Genre hint for analysis calibration.
    #[serde(default)]
    pub genre: String,
}

/// Payload for `session_end`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionEndPayload {
    /// Total session duration in seconds.
    #[serde(default)]
    pub duration_s: f64,
    /// Reason for ending (quit, death, disconnect, etc.).
    #[serde(default)]
    pub reason: String,
}

/// Payload for `player_move`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlayerMovePayload {
    /// X position.
    pub x: f64,
    /// Y position.
    pub y: f64,
    /// Z position (0.0 for 2D games).
    #[serde(default)]
    pub z: f64,
    /// Facing angle in radians (optional).
    #[serde(default)]
    pub angle: f64,
    /// Speed / velocity magnitude.
    #[serde(default)]
    pub speed: f64,
}

/// Payload for `player_action`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlayerActionPayload {
    /// Action name (attack, defend, `use_item`, jump, etc.).
    pub action: String,
    /// Target of the action (if any).
    #[serde(default)]
    pub target: String,
    /// Action succeeded.
    #[serde(default)]
    pub success: bool,
}

/// Payload for `player_damage`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlayerDamagePayload {
    /// Damage amount.
    pub amount: f64,
    /// Damage source.
    #[serde(default)]
    pub source: String,
    /// Remaining health after damage.
    #[serde(default)]
    pub health_remaining: f64,
}

/// Payload for `player_death`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlayerDeathPayload {
    /// Cause of death.
    #[serde(default)]
    pub cause: String,
    /// Respawn available.
    #[serde(default)]
    pub respawn: bool,
}

/// Payload for challenge events (encounter, complete, fail).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChallengePayload {
    /// Challenge identifier or name.
    #[serde(default)]
    pub challenge_id: String,
    /// Difficulty level (0.0–1.0 normalized).
    #[serde(default)]
    pub difficulty: f64,
    /// Challenge type (combat, puzzle, platforming, etc.).
    #[serde(default)]
    pub challenge_type: String,
}

/// Payload for `exploration_discover`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExplorationPayload {
    /// Name or ID of discovered area/item.
    #[serde(default)]
    pub discovery_id: String,
    /// Category (area, item, npc, secret, etc.).
    #[serde(default)]
    pub category: String,
    /// Position of discovery.
    #[serde(default)]
    pub x: f64,
    /// Position of discovery.
    #[serde(default)]
    pub y: f64,
}

/// Payload for `ui_interact`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UiInteractPayload {
    /// UI element name.
    #[serde(default)]
    pub element: String,
    /// Cursor/pointer distance traveled to reach element (pixels).
    #[serde(default)]
    pub distance_px: f64,
    /// Element width (pixels).
    #[serde(default)]
    pub target_width_px: f64,
    /// Number of options visible when interacting (for Hick's law).
    #[serde(default)]
    pub n_options: usize,
    /// Time spent on the interaction (ms).
    #[serde(default)]
    pub duration_ms: f64,
}

/// A single UI element descriptor for `ui_layout` events.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UiElementDescriptor {
    /// Element name.
    pub name: String,
    /// Bounding box: [x, y, width, height] in normalized screen coords (0–1).
    #[serde(default)]
    pub bounds: [f64; 4],
    /// Number of data values the element conveys.
    #[serde(default)]
    pub data_values: usize,
    /// Total pixel area.
    #[serde(default)]
    pub pixel_area: f64,
    /// Pixel area encoding data (non-decorative).
    #[serde(default)]
    pub data_ink_area: f64,
    /// Whether the element is always-visible critical info.
    #[serde(default)]
    pub critical: bool,
}

/// Payload for `ui_layout`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UiLayoutPayload {
    /// All visible UI elements.
    pub elements: Vec<UiElementDescriptor>,
}

/// Payload for `input_raw`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct InputRawPayload {
    /// Input type (key, `mouse_button`, `mouse_move`, gamepad, etc.).
    #[serde(default)]
    pub input_type: String,
    /// Key or button name.
    #[serde(default)]
    pub key: String,
    /// Whether this is press (true) or release (false).
    #[serde(default)]
    pub pressed: bool,
}

impl TelemetryEvent {
    /// Parse a typed payload from this event's JSON value.
    ///
    /// Returns `None` if the payload doesn't match the expected type.
    #[must_use]
    pub fn parse_payload<T: for<'de> Deserialize<'de>>(&self) -> Option<T> {
        serde_json::from_value(self.payload.clone()).ok()
    }
}

#[cfg(test)]
#[expect(clippy::expect_used, reason = "test module: fail-fast on setup errors")]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_event() {
        let evt = TelemetryEvent {
            timestamp_ms: 1000,
            session_id: "test-session".into(),
            event_type: EventType::PlayerMove,
            payload: serde_json::to_value(PlayerMovePayload {
                x: 1.5,
                y: 2.5,
                ..Default::default()
            })
            .ok()
            .unwrap_or_default(),
        };
        let json = serde_json::to_string(&evt).expect("serialize");
        let back: TelemetryEvent = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.event_type, EventType::PlayerMove);
        assert_eq!(back.session_id, "test-session");
    }

    #[test]
    fn snake_case_event_type() {
        let json =
            r#"{"timestamp_ms":0,"session_id":"s","event_type":"session_start","payload":{}}"#;
        let evt: TelemetryEvent = serde_json::from_str(json).expect("parse");
        assert_eq!(evt.event_type, EventType::SessionStart);
    }

    #[test]
    fn parse_typed_payload() {
        let evt = TelemetryEvent {
            timestamp_ms: 0,
            session_id: String::new(),
            event_type: EventType::PlayerDamage,
            payload: serde_json::json!({"amount": 25.0, "source": "goblin", "health_remaining": 75.0}),
        };
        let dmg: PlayerDamagePayload = evt.parse_payload().expect("typed parse");
        assert!((dmg.amount - 25.0).abs() < f64::EPSILON);
        assert_eq!(dmg.source, "goblin");
    }
}

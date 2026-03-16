// SPDX-License-Identifier: AGPL-3.0-or-later
//! RPGPT scene binding types for petalTongue visualization.
//!
//! These types compose ludoSpring game state into structured payloads that
//! petalTongue's Grammar of Graphics engine can render. Each type maps to
//! a specific `GameChannelType` and is serialized as JSON for the
//! `visualization.render` / `visualization.render.scene` JSON-RPC methods.
//!
//! # Design
//!
//! petalTongue is discovered at runtime via capability (`visualization.render`).
//! No compile-time dependency on petalTongue crates — these types define the
//! wire format that any visualization-capable primal can consume.
//!
//! Scene types follow the Grammar of Graphics model:
//! - **Data** — game state values
//! - **Variables** — mapped to aesthetic channels (x, y, color, size, label)
//! - **Geometry** — rendering shape (gauge, bar, tile, text, point)
//! - **Coordinates** — layout system (cartesian, polar, tree)

use crate::game::rpgpt::voice::VoiceId;

/// A dialogue choice presented to the player.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "ipc", derive(serde::Serialize, serde::Deserialize))]
pub struct DialogueChoice {
    /// Unique identifier for this choice.
    pub id: String,
    /// Display text shown to the player.
    pub text: String,
    /// Skill check required (if any).
    pub skill_check: Option<DialogueSkillCheck>,
    /// Whether this choice is currently available.
    pub available: bool,
    /// Trust level required to see this option.
    pub trust_gate: Option<u8>,
}

/// Skill check indicator attached to a dialogue choice.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "ipc", derive(serde::Serialize, serde::Deserialize))]
pub struct DialogueSkillCheck {
    /// Voice/skill used for the check.
    pub voice: VoiceId,
    /// Difficulty class.
    pub dc: u8,
    /// Player's effective pool size.
    pub pool_size: u8,
    /// Estimated probability of success (0.0..=1.0).
    pub success_probability: f64,
}

/// Scene payload for the `DialogueTree` channel.
///
/// Renders as a branching tree with the current narration at the top
/// and selectable choices below. Internal voices appear as margin notes.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "ipc", derive(serde::Serialize, serde::Deserialize))]
pub struct DialogueTreeScene {
    /// Current narration text (AI-generated).
    pub narration: String,
    /// Speaker name (NPC or narrator).
    pub speaker: String,
    /// Available choices for the player.
    pub choices: Vec<DialogueChoice>,
    /// Active internal voice outputs (max 3).
    pub voices: Vec<VoiceNote>,
    /// Session DAG vertex ID for provenance.
    pub vertex_id: Option<String>,
}

/// An internal voice note displayed alongside dialogue.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "ipc", derive(serde::Serialize, serde::Deserialize))]
pub struct VoiceNote {
    /// Which internal voice is speaking.
    pub voice: VoiceId,
    /// The voice's observation or interjection.
    pub text: String,
    /// Priority (higher = more prominent display).
    pub priority: u8,
    /// Whether this was triggered by a passive check (vs. active).
    pub passive: bool,
}

/// A single stat for character sheet display.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "ipc", derive(serde::Serialize, serde::Deserialize))]
pub struct StatEntry {
    /// Stat name (e.g., "Logic", "Endurance").
    pub name: String,
    /// Current value.
    pub value: f64,
    /// Maximum value (for gauge rendering).
    pub max: f64,
    /// Modifier from conditions/equipment.
    pub modifier: f64,
}

/// A condition affecting the character.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "ipc", derive(serde::Serialize, serde::Deserialize))]
pub struct ConditionDisplay {
    /// Condition name.
    pub name: String,
    /// Severity or stack count.
    pub severity: u8,
    /// Duration remaining (turns or "permanent").
    pub duration: String,
    /// Source of the condition.
    pub source: String,
}

/// Scene payload for the `CharacterSheet` channel.
///
/// Renders as a multi-panel composition: stat gauges on the left,
/// conditions in the center, inventory on the right.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "ipc", derive(serde::Serialize, serde::Deserialize))]
pub struct CharacterSheetScene {
    /// Character name.
    pub name: String,
    /// Current HP.
    pub hp: f64,
    /// Maximum HP.
    pub hp_max: f64,
    /// Character stats (voice skills in Dialogue plane).
    pub stats: Vec<StatEntry>,
    /// Active conditions.
    pub conditions: Vec<ConditionDisplay>,
    /// Inventory item names (detail via inspect interaction).
    pub inventory: Vec<String>,
    /// Currently active plane.
    pub active_plane: String,
    /// Active ruleset name.
    pub ruleset: String,
}

/// An entity on the combat grid.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "ipc", derive(serde::Serialize, serde::Deserialize))]
pub struct GridEntity {
    /// Entity identifier.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Grid X position.
    pub x: u32,
    /// Grid Y position.
    pub y: u32,
    /// Team affiliation for coloring.
    pub team: String,
    /// HP fraction (0.0..=1.0) for health indicator.
    pub hp_fraction: f64,
    /// Whether this entity has acted this round.
    pub acted: bool,
}

/// Scene payload for the `CombatGrid` channel.
///
/// Renders as a FieldMap with entity markers, zone overlays,
/// and action-range indicators.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "ipc", derive(serde::Serialize, serde::Deserialize))]
pub struct CombatGridScene {
    /// Grid width in cells.
    pub width: u32,
    /// Grid height in cells.
    pub height: u32,
    /// Terrain values per cell (row-major, for FieldMap rendering).
    pub terrain: Vec<f64>,
    /// Entities on the grid.
    pub entities: Vec<GridEntity>,
    /// Current round number.
    pub round: u32,
    /// Whose turn it is.
    pub active_entity: Option<String>,
    /// Remaining actions for the active entity.
    pub actions_remaining: u8,
}

/// NPC status for the interaction panel.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "ipc", derive(serde::Serialize, serde::Deserialize))]
pub struct NpcStatusScene {
    /// NPC name.
    pub name: String,
    /// NPC role/title.
    pub role: String,
    /// Current trust level (0-255).
    pub trust: u8,
    /// Trust thresholds for display (friendly, allied, bonded).
    pub trust_thresholds: [u8; 3],
    /// Current disposition description.
    pub disposition: String,
    /// Visible conditions on the NPC.
    pub conditions: Vec<String>,
    /// Whether the NPC is currently in conversation.
    pub in_conversation: bool,
}

/// Dice result display with provenance.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "ipc", derive(serde::Serialize, serde::Deserialize))]
pub struct DiceResultScene {
    /// Individual die values.
    pub dice: Vec<u8>,
    /// Number of successes (for pool systems).
    pub successes: u8,
    /// Total (for sum systems).
    pub total: u16,
    /// Degree of success label.
    pub degree: String,
    /// BearDog Ed25519 signature (hex) for provenance.
    pub signature: Option<String>,
    /// Brief narrative interpretation.
    pub interpretation: String,
}

/// Map point of interest.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "ipc", derive(serde::Serialize, serde::Deserialize))]
pub struct MapMarker {
    /// Marker identifier.
    pub id: String,
    /// Display label.
    pub label: String,
    /// Grid X.
    pub x: u32,
    /// Grid Y.
    pub y: u32,
    /// Marker type for icon selection.
    pub marker_type: String,
    /// Whether this has been visited.
    pub visited: bool,
}

/// Scene payload for the `ExplorationMap` channel.
///
/// Renders as a FieldMap with fog of war, markers, and party position.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "ipc", derive(serde::Serialize, serde::Deserialize))]
pub struct ExplorationMapScene {
    /// Map width in cells.
    pub width: u32,
    /// Map height in cells.
    pub height: u32,
    /// Terrain elevation/type values (row-major FieldMap).
    pub terrain: Vec<f64>,
    /// Fog of war mask (true = revealed).
    pub revealed: Vec<bool>,
    /// Points of interest.
    pub markers: Vec<MapMarker>,
    /// Party X position.
    pub party_x: u32,
    /// Party Y position.
    pub party_y: u32,
}

/// Scene payload for the `NarrationStream` channel.
///
/// Renders as a scrolling text panel with speaker attribution.
/// Pushed incrementally via `visualization.render.stream` (append).
#[derive(Debug, Clone)]
#[cfg_attr(feature = "ipc", derive(serde::Serialize, serde::Deserialize))]
pub struct NarrationEntry {
    /// Speaker name (NPC, narrator, or voice name).
    pub speaker: String,
    /// Narration text.
    pub text: String,
    /// Whether this is an internal voice (rendered differently).
    pub is_voice: bool,
    /// Timestamp or turn number.
    pub sequence: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dialogue_tree_composes() {
        let scene = DialogueTreeScene {
            narration: "The innkeeper eyes you warily.".into(),
            speaker: "Maren".into(),
            choices: vec![
                DialogueChoice {
                    id: "ask_rooms".into(),
                    text: "I need a room for the night.".into(),
                    skill_check: None,
                    available: true,
                    trust_gate: None,
                },
                DialogueChoice {
                    id: "ask_disappearances".into(),
                    text: "Tell me about the disappearances.".into(),
                    skill_check: Some(DialogueSkillCheck {
                        voice: VoiceId::Rhetoric,
                        dc: 12,
                        pool_size: 4,
                        success_probability: 0.65,
                    }),
                    available: true,
                    trust_gate: Some(30),
                },
            ],
            voices: vec![VoiceNote {
                voice: VoiceId::Empathy,
                text: "She's hiding something. Her hands are shaking.".into(),
                priority: 8,
                passive: true,
            }],
            vertex_id: Some("v-00a1".into()),
        };
        assert_eq!(scene.choices.len(), 2);
        assert_eq!(scene.voices.len(), 1);
        assert!(scene.choices[1].skill_check.is_some());
    }

    #[test]
    fn character_sheet_composes() {
        let sheet = CharacterSheetScene {
            name: "Harlan Cole".into(),
            hp: 35.0,
            hp_max: 42.0,
            stats: vec![
                StatEntry {
                    name: "Logic".into(),
                    value: 6.0,
                    max: 10.0,
                    modifier: 0.0,
                },
                StatEntry {
                    name: "Empathy".into(),
                    value: 8.0,
                    max: 10.0,
                    modifier: 1.0,
                },
            ],
            conditions: vec![ConditionDisplay {
                name: "Shaken".into(),
                severity: 1,
                duration: "3 turns".into(),
                source: "Failed composure check".into(),
            }],
            inventory: vec!["Lantern".into(), "Old Key".into(), "Journal".into()],
            active_plane: "Dialogue".into(),
            ruleset: "Custom (Disco Elysium model)".into(),
        };
        assert_eq!(sheet.stats.len(), 2);
        assert!(sheet.hp < sheet.hp_max);
    }

    #[test]
    fn combat_grid_composes() {
        let grid = CombatGridScene {
            width: 10,
            height: 10,
            terrain: vec![0.0; 100],
            entities: vec![
                GridEntity {
                    id: "player".into(),
                    name: "Harlan".into(),
                    x: 3,
                    y: 5,
                    team: "party".into(),
                    hp_fraction: 0.83,
                    acted: false,
                },
                GridEntity {
                    id: "guard_1".into(),
                    name: "Town Guard".into(),
                    x: 7,
                    y: 4,
                    team: "hostile".into(),
                    hp_fraction: 1.0,
                    acted: false,
                },
            ],
            round: 1,
            active_entity: Some("player".into()),
            actions_remaining: 3,
        };
        assert_eq!(grid.entities.len(), 2);
        assert_eq!(grid.width * grid.height, grid.terrain.len() as u32);
    }

    #[test]
    fn npc_status_composes() {
        let npc = NpcStatusScene {
            name: "Maren".into(),
            role: "Innkeeper".into(),
            trust: 45,
            trust_thresholds: [30, 60, 90],
            disposition: "Cautious but warming".into(),
            conditions: vec!["Nervous".into()],
            in_conversation: true,
        };
        assert!(npc.trust > npc.trust_thresholds[0]);
        assert!(npc.trust < npc.trust_thresholds[1]);
    }

    #[test]
    fn dice_result_composes() {
        let result = DiceResultScene {
            dice: vec![4, 5, 2, 6, 3],
            successes: 3,
            total: 20,
            degree: "Success".into(),
            signature: Some("abcdef0123456789".into()),
            interpretation: "The guard believes your story.".into(),
        };
        assert_eq!(result.dice.iter().filter(|&&d| d >= 4).count(), 3);
        assert_eq!(result.successes, 3);
    }

    #[test]
    fn exploration_map_composes() {
        let map = ExplorationMapScene {
            width: 20,
            height: 15,
            terrain: vec![0.5; 300],
            revealed: vec![false; 300],
            markers: vec![MapMarker {
                id: "tavern".into(),
                label: "The Drowned Rat".into(),
                x: 10,
                y: 7,
                marker_type: "building".into(),
                visited: true,
            }],
            party_x: 10,
            party_y: 8,
        };
        assert_eq!(map.terrain.len(), (map.width * map.height) as usize);
        assert_eq!(map.revealed.len(), map.terrain.len());
    }
}

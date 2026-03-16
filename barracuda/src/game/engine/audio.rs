// SPDX-License-Identifier: AGPL-3.0-or-later
//! Audio narration layer — first-class game experience for ears.
//!
//! An audio game isn't an accessibility afterthought — it's a mode of play.
//! You could play this while driving cross-country. It happens to be
//! blind-user friendly, but the design intent is broader: petalTongue's
//! philosophy is that all humans of all ability are first class.
//!
//! This module produces structured narration cues from game state. These
//! cues are sent to petalTongue's audio sonification engine, which
//! renders them as synthesized speech, spatial audio, and sound effects.
//!
//! # Design
//!
//! Every game state change produces a `NarrationCue`. Cues carry semantic
//! meaning (what happened, who's involved, where) rather than specific
//! audio instructions. petalTongue's modality compilers choose the output:
//! - Audio: speech synthesis + spatial effects
//! - Visual: text in narration panel + animations
//! - Haptic: vibration patterns for significant events
//! - Braille: refreshable display output
//!
//! All modalities receive the same cues. The engine never assumes which
//! modality the human is using.

use super::action::{ActionOutcome, Effect, TriggerEvent};
use super::entity::{EntityId, EntityKind};
use super::world::Direction;

/// Priority of a narration cue (affects queuing and interruption).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CuePriority {
    /// Background ambient — can be dropped if queue is full.
    Ambient,
    /// Normal game narration.
    Normal,
    /// Important event (NPC speaks, clue found).
    Important,
    /// Critical alert (combat start, danger, plane transition).
    Critical,
}

/// Spatial position hint for directional audio.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpatialHint {
    /// Relative direction from the player.
    pub direction: Direction,
    /// Distance in tiles (closer = louder).
    pub distance: u32,
}

/// A structured narration cue produced by the engine.
///
/// These are modality-agnostic: petalTongue decides how to render them
/// based on the active modalities (audio, visual, haptic, braille).
#[derive(Debug, Clone)]
pub struct NarrationCue {
    /// The narration text (for speech synthesis and text display).
    pub text: String,
    /// Priority for queuing.
    pub priority: CuePriority,
    /// Speaker (NPC name, voice name, or "narrator").
    pub speaker: String,
    /// Optional spatial position for directional audio.
    pub spatial: Option<SpatialHint>,
    /// Sound effect tag (petalTongue maps to audio assets).
    pub sound_effect: Option<SoundEffect>,
    /// Category for filtering and presentation.
    pub category: CueCategory,
}

/// Categories of narration for filtering and presentation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CueCategory {
    /// Player action feedback ("You move north", "You pick up the lantern").
    ActionFeedback,
    /// NPC speech or reaction.
    NpcDialogue,
    /// Internal voice interjection.
    InternalVoice,
    /// Environment description.
    Environment,
    /// Combat event (hit, miss, damage).
    Combat,
    /// Discovery (clue, secret, item).
    Discovery,
    /// System event (plane transition, turn change, session).
    System,
    /// Ambient atmosphere (weather, crowd noise, creaking).
    Atmosphere,
}

/// Sound effect tags — petalTongue maps these to actual audio.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum SoundEffect {
    /// Footstep on the given surface type.
    Footstep(String),
    /// Door opening or closing.
    Door {
        /// True if opening, false if closing.
        opening: bool,
    },
    /// Dice rolling (number of dice).
    DiceRoll(u8),
    /// Combat hit.
    Hit,
    /// Combat miss.
    Miss,
    /// Item pickup.
    Pickup,
    /// Danger alert (combat start, trap trigger).
    Alert,
    /// Discovery chime (clue found, secret revealed).
    Discovery,
    /// Plane transition whoosh/shift.
    PlaneShift,
    /// Ambient loop (wind, rain, crowd, fire).
    Ambient(String),
    /// Custom sound effect tag.
    Custom(String),
}

/// Shared context for narration compilation — reduces argument sprawl.
///
/// Follows the healthSpring `TissueContext` pattern: groups related
/// parameters that flow through the entire narration pipeline.
pub struct NarrationContext<'a> {
    /// Entity registry for name resolution and spatial queries.
    pub entities: &'a super::entity::EntityRegistry,
    /// Accumulated narration cues (mutated during compilation).
    pub cues: &'a mut Vec<NarrationCue>,
}

/// Compiles game outcomes into narration cues.
///
/// This is the bridge between the engine's action resolution and
/// petalTongue's multi-modal rendering. Every outcome becomes one
/// or more cues that any modality can present.
#[must_use]
pub fn compile_outcome(
    outcome: &ActionOutcome,
    entities: &super::entity::EntityRegistry,
) -> Vec<NarrationCue> {
    let mut cues = Vec::new();
    let mut ctx = NarrationContext {
        entities,
        cues: &mut cues,
    };
    compile_effect_cues(&outcome.effect, outcome.narration.as_deref(), &mut ctx);
    compile_trigger_cues(&outcome.triggers, ctx.cues);
    cues
}

/// Translate an [`Effect`] into narration cues with entity context.
fn compile_effect_cues(effect: &Effect, narration: Option<&str>, ctx: &mut NarrationContext<'_>) {
    match effect {
        Effect::Moved { entity, to_x, to_y } => {
            compile_movement_cues(*entity, *to_x, *to_y, narration, ctx);
        }
        Effect::DialogueAdvanced { speaker, .. } => {
            let speaker_name = ctx
                .entities
                .get(*speaker)
                .map_or_else(|| "Someone".into(), |e| e.name.clone());
            if let Some(text) = narration {
                ctx.cues.push(NarrationCue {
                    text: text.to_owned(),
                    priority: CuePriority::Important,
                    speaker: speaker_name,
                    spatial: None,
                    sound_effect: None,
                    category: CueCategory::NpcDialogue,
                });
            }
        }
        Effect::ItemAcquired { item_name, .. } => {
            ctx.cues.push(NarrationCue {
                text: narration.map_or_else(|| format!("You acquired {item_name}."), str::to_owned),
                priority: CuePriority::Normal,
                speaker: "narrator".into(),
                spatial: None,
                sound_effect: Some(SoundEffect::Pickup),
                category: CueCategory::Discovery,
            });
        }
        Effect::Damaged { target, amount, .. } => {
            compile_damage_cues(*target, *amount, narration, ctx);
        }
        Effect::Revealed { info, .. } => {
            ctx.cues.push(NarrationCue {
                text: narration.map_or_else(|| info.clone(), str::to_owned),
                priority: CuePriority::Important,
                speaker: "narrator".into(),
                spatial: None,
                sound_effect: Some(SoundEffect::Discovery),
                category: CueCategory::Discovery,
            });
        }
        Effect::NoEffect { reason } => {
            push_narration_cue(narration, CuePriority::Ambient, ctx.cues);
            let _ = reason;
        }
        Effect::TurnEnded { .. } | Effect::Interacted { .. } | Effect::ItemUsed { .. } => {
            push_narration_cue(narration, CuePriority::Normal, ctx.cues);
        }
    }
}

/// Generate movement narration plus NPC proximity awareness cues.
fn compile_movement_cues(
    entity: EntityId,
    to_x: u32,
    to_y: u32,
    narration: Option<&str>,
    ctx: &mut NarrationContext<'_>,
) {
    if let Some(text) = narration {
        ctx.cues.push(NarrationCue {
            text: text.to_owned(),
            priority: CuePriority::Normal,
            speaker: "narrator".into(),
            spatial: None,
            sound_effect: Some(SoundEffect::Footstep("stone".into())),
            category: CueCategory::ActionFeedback,
        });
    }

    if let Some(e) = ctx.entities.get(entity) {
        if e.kind == EntityKind::Player {
            let nearby_npcs: Vec<_> = ctx
                .entities
                .within_range(to_x, to_y, crate::tolerances::NPC_PROXIMITY_TILES)
                .filter(|n| n.kind == EntityKind::Npc && n.visible)
                .collect();
            for npc in nearby_npcs {
                let dir = direction_from_to(to_x, to_y, npc.x, npc.y);
                let dir_text = dir.map_or_else(|| "nearby".into(), |d| format!("to the {d:?}"));
                ctx.cues.push(NarrationCue {
                    text: format!("{} is {dir_text}.", npc.name),
                    priority: CuePriority::Ambient,
                    speaker: "narrator".into(),
                    spatial: dir.map(|d| SpatialHint {
                        direction: d,
                        distance: npc.distance_to(to_x, to_y),
                    }),
                    sound_effect: None,
                    category: CueCategory::Environment,
                });
            }
        }
    }
}

/// Generate damage narration with hit/miss sound effects.
fn compile_damage_cues(
    target: EntityId,
    amount: i32,
    narration: Option<&str>,
    ctx: &mut NarrationContext<'_>,
) {
    let target_name = ctx
        .entities
        .get(target)
        .map_or_else(|| "Something".into(), |e| e.name.clone());
    ctx.cues.push(NarrationCue {
        text: narration.map_or_else(
            || format!("{target_name} takes {amount} damage!"),
            str::to_owned,
        ),
        priority: CuePriority::Important,
        speaker: "narrator".into(),
        spatial: None,
        sound_effect: Some(if amount > 0 {
            SoundEffect::Hit
        } else {
            SoundEffect::Miss
        }),
        category: CueCategory::Combat,
    });
}

/// Push a simple narrator cue if narration text is present.
fn push_narration_cue(
    narration: Option<&str>,
    priority: CuePriority,
    cues: &mut Vec<NarrationCue>,
) {
    if let Some(text) = narration {
        cues.push(NarrationCue {
            text: text.to_owned(),
            priority,
            speaker: "narrator".into(),
            spatial: None,
            sound_effect: None,
            category: CueCategory::ActionFeedback,
        });
    }
}

/// Translate [`TriggerEvent`]s into system and discovery narration cues.
fn compile_trigger_cues(triggers: &[TriggerEvent], cues: &mut Vec<NarrationCue>) {
    for trigger in triggers {
        match trigger {
            TriggerEvent::ConversationStart { .. } => {}
            TriggerEvent::EncounterStart { .. } => {
                cues.push(NarrationCue {
                    text: "Combat begins!".into(),
                    priority: CuePriority::Critical,
                    speaker: "narrator".into(),
                    spatial: None,
                    sound_effect: Some(SoundEffect::Alert),
                    category: CueCategory::System,
                });
            }
            TriggerEvent::PlaneTransition { to_plane, .. } => {
                cues.push(NarrationCue {
                    text: format!("The world shifts... entering {to_plane} mode."),
                    priority: CuePriority::Critical,
                    speaker: "narrator".into(),
                    spatial: None,
                    sound_effect: Some(SoundEffect::PlaneShift),
                    category: CueCategory::System,
                });
            }
            TriggerEvent::ClueFound { clue_id } => {
                cues.push(NarrationCue {
                    text: format!("New evidence: {clue_id}"),
                    priority: CuePriority::Important,
                    speaker: "narrator".into(),
                    spatial: None,
                    sound_effect: Some(SoundEffect::Discovery),
                    category: CueCategory::Discovery,
                });
            }
            TriggerEvent::ZoneTransition { zone_name } => {
                cues.push(NarrationCue {
                    text: format!("Entering {zone_name}."),
                    priority: CuePriority::Normal,
                    speaker: "narrator".into(),
                    spatial: None,
                    sound_effect: Some(SoundEffect::Footstep("transition".into())),
                    category: CueCategory::System,
                });
            }
            TriggerEvent::Custom { name, .. } => {
                cues.push(NarrationCue {
                    text: name.clone(),
                    priority: CuePriority::Normal,
                    speaker: "narrator".into(),
                    spatial: None,
                    sound_effect: None,
                    category: CueCategory::System,
                });
            }
        }
    }
}

/// Derive a direction from one position to another.
#[expect(
    clippy::cast_possible_wrap,
    reason = "grid coords are small positive numbers"
)]
const fn direction_from_to(from_x: u32, from_y: u32, to_x: u32, to_y: u32) -> Option<Direction> {
    let dx = to_x as i32 - from_x as i32;
    let dy = to_y as i32 - from_y as i32;
    match (dx.signum(), dy.signum()) {
        (0, -1) => Some(Direction::North),
        (0, 1) => Some(Direction::South),
        (1, 0) => Some(Direction::East),
        (-1, 0) => Some(Direction::West),
        (1, -1) => Some(Direction::NorthEast),
        (-1, -1) => Some(Direction::NorthWest),
        (1, 1) => Some(Direction::SouthEast),
        (-1, 1) => Some(Direction::SouthWest),
        _ => None,
    }
}

/// Compile a scene description for a newly entered area (used on zone
/// transitions and session start). Describes what the player sees/hears.
#[must_use]
pub fn compile_area_description(
    world: &super::world::TileWorld,
    entities: &super::entity::EntityRegistry,
    player_x: u32,
    player_y: u32,
) -> NarrationCue {
    let mut parts = vec![format!("You are in {}.", world.name)];

    let nearby_npcs: Vec<_> = entities
        .within_range(
            player_x,
            player_y,
            crate::tolerances::AREA_DESCRIPTION_RANGE_TILES,
        )
        .filter(|e| e.kind == EntityKind::Npc && e.visible)
        .collect();

    if !nearby_npcs.is_empty() {
        let names: Vec<_> = nearby_npcs.iter().map(|n| n.name.as_str()).collect();
        parts.push(format!("You can see: {}.", names.join(", ")));
    }

    let nearby_items: Vec<_> = entities
        .within_range(player_x, player_y, crate::tolerances::ITEM_PROXIMITY_TILES)
        .filter(|e| e.kind == EntityKind::Item && e.visible)
        .collect();

    if !nearby_items.is_empty() {
        let names: Vec<_> = nearby_items.iter().map(|n| n.name.as_str()).collect();
        parts.push(format!("Items nearby: {}.", names.join(", ")));
    }

    let exits: Vec<_> = Direction::CARDINAL
        .iter()
        .filter(|d| world.can_move(player_x, player_y, **d))
        .map(|d| format!("{d:?}"))
        .collect();

    if !exits.is_empty() {
        parts.push(format!("Exits: {}.", exits.join(", ")));
    }

    NarrationCue {
        text: parts.join(" "),
        priority: CuePriority::Important,
        speaker: "narrator".into(),
        spatial: None,
        sound_effect: Some(SoundEffect::Ambient("interior".into())),
        category: CueCategory::Environment,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::engine::action::{ActionCost, ActionOutcome, Effect, TriggerEvent};
    use crate::game::engine::entity::{Entity, EntityId, EntityRegistry, Faction};
    use crate::game::engine::world::{Terrain, TileWorld};

    fn test_entities() -> EntityRegistry {
        let mut reg = EntityRegistry::new();
        reg.spawn(Entity::player(0, "Harlan", 5, 5));
        reg.spawn(Entity::npc(0, "Maren", 6, 5, Faction::Neutral));
        reg
    }

    #[test]
    fn movement_produces_cues() {
        let reg = test_entities();
        let outcome = ActionOutcome {
            effect: Effect::Moved {
                entity: EntityId(0),
                to_x: 5,
                to_y: 4,
            },
            cost: ActionCost::One,
            narration: Some("You step forward carefully.".into()),
            triggers: Vec::new(),
        };

        let cues = compile_outcome(&outcome, &reg);
        assert!(!cues.is_empty());
        assert_eq!(cues[0].category, CueCategory::ActionFeedback);
        assert!(cues[0].sound_effect.is_some());
    }

    #[test]
    fn npc_proximity_produces_spatial_cue() {
        let reg = test_entities();
        let outcome = ActionOutcome {
            effect: Effect::Moved {
                entity: EntityId(0),
                to_x: 5,
                to_y: 5,
            },
            cost: ActionCost::One,
            narration: Some("You shift your weight.".into()),
            triggers: Vec::new(),
        };

        let cues = compile_outcome(&outcome, &reg);
        let env_cues: Vec<_> = cues
            .iter()
            .filter(|c| c.category == CueCategory::Environment)
            .collect();
        assert!(!env_cues.is_empty());
        assert!(env_cues[0].text.contains("Maren"));
    }

    #[test]
    fn encounter_start_is_critical() {
        let reg = test_entities();
        let outcome = ActionOutcome {
            effect: Effect::NoEffect {
                reason: "trigger".into(),
            },
            cost: ActionCost::Free,
            narration: None,
            triggers: vec![TriggerEvent::EncounterStart {
                hostiles: vec![EntityId(1)],
            }],
        };

        let cues = compile_outcome(&outcome, &reg);
        let critical: Vec<_> = cues
            .iter()
            .filter(|c| c.priority == CuePriority::Critical)
            .collect();
        assert!(!critical.is_empty());
    }

    #[test]
    fn item_acquisition_has_pickup_sound() {
        let reg = test_entities();
        let outcome = ActionOutcome {
            effect: Effect::ItemAcquired {
                actor: EntityId(0),
                item_name: "Lantern".into(),
            },
            cost: ActionCost::One,
            narration: None,
            triggers: Vec::new(),
        };

        let cues = compile_outcome(&outcome, &reg);
        assert_eq!(cues.len(), 1);
        assert_eq!(cues[0].sound_effect, Some(SoundEffect::Pickup));
    }

    #[test]
    fn area_description_includes_exits() {
        let world = TileWorld::new(5, 5, "Test Room", Terrain::Open);
        let mut reg = EntityRegistry::new();
        reg.spawn(Entity::player(0, "P", 2, 2));
        reg.spawn(Entity::npc(0, "Guard", 3, 2, Faction::Neutral));

        let cue = compile_area_description(&world, &reg, 2, 2);
        assert!(cue.text.contains("Test Room"));
        assert!(cue.text.contains("Guard"));
        assert!(cue.text.contains("Exits"));
        assert_eq!(cue.priority, CuePriority::Important);
    }

    #[test]
    fn direction_inference() {
        assert_eq!(direction_from_to(5, 5, 5, 3), Some(Direction::North));
        assert_eq!(direction_from_to(5, 5, 7, 7), Some(Direction::SouthEast));
        assert_eq!(direction_from_to(5, 5, 5, 5), None); // same position
    }

    #[test]
    fn cue_priority_ordering() {
        assert!(CuePriority::Ambient < CuePriority::Normal);
        assert!(CuePriority::Normal < CuePriority::Important);
        assert!(CuePriority::Important < CuePriority::Critical);
    }

    #[test]
    fn plane_transition_trigger() {
        let reg = test_entities();
        let outcome = ActionOutcome {
            effect: Effect::NoEffect {
                reason: "trigger".into(),
            },
            cost: ActionCost::Free,
            narration: None,
            triggers: vec![TriggerEvent::PlaneTransition {
                to_plane: "Tactical".into(),
                trigger: "ambush".into(),
            }],
        };

        let cues = compile_outcome(&outcome, &reg);
        let plane_cue = cues
            .iter()
            .find(|c| c.category == CueCategory::System)
            .unwrap();
        assert!(plane_cue.text.contains("Tactical"));
        assert_eq!(plane_cue.sound_effect, Some(SoundEffect::PlaneShift));
    }
}

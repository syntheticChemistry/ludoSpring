// SPDX-License-Identifier: AGPL-3.0-or-later
//! Game engine tolerances — NPC proximity, frame rate, entity limits.

/// NPC spatial proximity threshold for narration cues (tiles).
///
/// Justification: NPCs within 3 tiles are "nearby" and generate
/// awareness cues in the audio narration system. Matches typical
/// 2D RPG "adjacent room" distance.
pub const NPC_PROXIMITY_TILES: u32 = 3;

/// Extended spatial range for area descriptions (tiles).
///
/// Justification: Area description cues include NPCs within 5 tiles,
/// giving the player awareness of nearby characters outside immediate
/// proximity.
pub const AREA_DESCRIPTION_RANGE_TILES: u32 = 5;

/// Item proximity threshold for area descriptions (tiles).
///
/// Justification: Items within 3 tiles are mentioned in area narration
/// to support blind-accessible gameplay without requiring examine actions.
pub const ITEM_PROXIMITY_TILES: u32 = 3;

/// Default vertex query limit for NPC memory retrieval.
///
/// Justification: 50 vertices balances memory context richness against
/// Squirrel context window limits (~4096 tokens).
pub const DEFAULT_VERTEX_QUERY_LIMIT: u32 = 50;

/// Target frame rate for game loop budget calculations (Hz).
///
/// Justification: 60 Hz is the standard interactive rate for smooth
/// gameplay. petalTongue's interaction stream targets this rate.
pub const TARGET_FRAME_RATE_HZ: f64 = 60.0;

/// Default player sight radius for fog of war (tiles).
///
/// Justification: 5 tiles gives good visibility in a 10x10 room
/// while maintaining meaningful fog for larger spaces. Adjustable
/// per-session via `GameSession::with_sight_radius()`.
pub const DEFAULT_SIGHT_RADIUS: u32 = 5;

/// Trigger detection range for movement events (tiles).
///
/// Justification: Range 1 detects triggers on the destination tile
/// and adjacent tiles, catching doorway triggers the player walks
/// past. Range 0 (previous value) missed adjacent triggers.
pub const TRIGGER_DETECTION_RANGE: u32 = 1;

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

/// Game-state comparison tolerance for trust, flow, and engagement values.
///
/// Justification: game state values (NPC trust, dialogue option counts,
/// engagement scores) are computed from small integer operations where
/// f64 representation error is negligible. 0.01 catches real logic bugs
/// while absorbing floating-point arithmetic noise.
pub const GAME_STATE_TOL: f64 = 0.01;

/// Tolerance for NPC trust value equality across plane transitions.
///
/// Justification: trust values are computed from discrete integer events
/// (faction + personal + relationship + debt). `f64::EPSILON` (~2.2e-16)
/// is correct for exact bit-equality, but intermediate rounding during
/// serialization/deserialization can introduce error up to ~1e-14.
/// Using `f64::EPSILON` is overly strict; 1e-12 absorbs serialization
/// noise while remaining tight enough to catch real trust drift.
pub const TRUST_EQUALITY_TOL: f64 = 1e-12;

/// Milliseconds per second — unit conversion constant.
///
/// Used in telemetry timestamp-to-duration calculations.
pub const MS_PER_SECOND: f64 = 1000.0;

/// Seconds per minute — unit conversion constant.
///
/// Used in engagement metrics to convert session duration from seconds
/// to minutes for APM (actions per minute) calculations.
pub const SECONDS_PER_MINUTE: f64 = 60.0;

/// Default simulation timestep (seconds) — 1/60 s for 60 Hz.
///
/// Derived from [`TARGET_FRAME_RATE_HZ`]. Used in `TickBudget::default()`.
pub const DEFAULT_DT_S: f64 = 1.0 / TARGET_FRAME_RATE_HZ;

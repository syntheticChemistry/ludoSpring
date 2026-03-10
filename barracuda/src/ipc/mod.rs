// SPDX-License-Identifier: AGPL-3.0-or-later
//! JSON-RPC 2.0 IPC server for ludoSpring.
//!
//! Exposes game science capabilities to biomeOS and other primals:
//! - `game.analyze_ui` — run Tufte constraints on a game UI description
//! - `game.evaluate_flow` — compute flow state for challenge/skill pair
//! - `game.fitts_cost` — compute Fitts's law movement time
//! - `game.engagement` — compute engagement metrics from behavior snapshot
//! - `game.generate_noise` — generate a noise field sample
//! - `game.wfc_step` — perform one WFC collapse + propagation step
//! - `game.accessibility_score` — score accessibility dimensions

/// JSON-RPC method names.
pub const METHOD_ANALYZE_UI: &str = "game.analyze_ui";
/// Evaluate flow state.
pub const METHOD_EVALUATE_FLOW: &str = "game.evaluate_flow";
/// Compute Fitts's law cost.
pub const METHOD_FITTS_COST: &str = "game.fitts_cost";
/// Compute engagement metrics.
pub const METHOD_ENGAGEMENT: &str = "game.engagement";
/// Generate noise field.
pub const METHOD_GENERATE_NOISE: &str = "game.generate_noise";
/// Wave function collapse step.
pub const METHOD_WFC_STEP: &str = "game.wfc_step";
/// Accessibility scoring.
pub const METHOD_ACCESSIBILITY: &str = "game.accessibility";

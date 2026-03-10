// SPDX-License-Identifier: AGPL-3.0-or-later
//! Domain-specific tolerances — no magic numbers.
//!
//! Every numerical threshold in ludoSpring lives here with a citation.

/// Fitts's law default parameters (mouse, desktop).
/// Source: MacKenzie, I.S. (1992). "Fitts' law as a research and design tool
/// in human-computer interaction." Human-Computer Interaction, 7(1).
pub const FITTS_A_MOUSE_MS: f64 = 50.0;
/// Fitts's law slope for mouse input.
pub const FITTS_B_MOUSE_MS: f64 = 150.0;

/// Hick's law default parameters.
/// Source: Hick (1952), Hyman (1953).
pub const HICK_A_MS: f64 = 200.0;
/// Hick's law processing time per bit.
pub const HICK_B_MS: f64 = 150.0;

/// Flow channel width (normalized challenge-skill space).
/// Source: Chen, J. (2007). "Flow in Games." M.S. Thesis, USC.
pub const FLOW_CHANNEL_WIDTH: f64 = 0.15;

/// Target success rate for dynamic difficulty adjustment.
/// Source: Hunicke, R. (2005). SIGCHI '05.
pub const DDA_TARGET_SUCCESS_RATE: f64 = 0.7;

/// Minimum data-ink ratio for a "clean" game UI.
/// Source: Tufte, E.R. (1983). "The Visual Display of Quantitative Information."
pub const TUFTE_MIN_DATA_INK_RATIO: f64 = 0.4;

/// Maximum HUD screen coverage before recommending progressive disclosure.
pub const MAX_HUD_COVERAGE: f64 = 0.25;

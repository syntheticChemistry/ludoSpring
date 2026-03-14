// SPDX-License-Identifier: AGPL-3.0-or-later
//! Domain-specific tolerances — no magic numbers.
//!
//! Every numerical threshold in ludoSpring lives here with a citation.
//! Constants are organized by domain and sorted alphabetically within.

// ── Interaction: Fitts's Law ─────────────────────────────────────────

/// Fitts's law intercept (mouse, desktop) in milliseconds.
///
/// Source: `MacKenzie`, I.S. (1992). "Fitts' law as a research and design tool
/// in human-computer interaction." Human-Computer Interaction, 7(1), pp. 91–139.
pub const FITTS_A_MOUSE_MS: f64 = 50.0;

/// Fitts's law slope (mouse, desktop) in milliseconds per bit.
///
/// Source: `MacKenzie` (1992), Table 2, average across experiments.
pub const FITTS_B_MOUSE_MS: f64 = 150.0;

// ── Interaction: Hick's Law ──────────────────────────────────────────

/// Hick's law base reaction time in milliseconds.
///
/// Source: Hick, W.E. (1952). "On the rate of gain of information."
/// Quarterly Journal of Experimental Psychology, 4(1), pp. 11–26.
pub const HICK_A_MS: f64 = 200.0;

/// Hick's law processing time per bit in milliseconds.
///
/// Source: Hyman, R. (1953). "Stimulus information as a determinant of
/// reaction time." Journal of Experimental Psychology, 45(3), pp. 188–196.
pub const HICK_B_MS: f64 = 150.0;

// ── Interaction: Steering Law ───────────────────────────────────────

/// Steering law intercept in milliseconds.
///
/// Source: Accot, J. & Zhai, S. (1997). "Beyond Fitts' law: models for
/// trajectory-based HCI tasks." CHI '97, pp. 295–302. Empirical fit
/// for mouse-based tunnel steering; 10ms accounts for initial latency.
pub const STEERING_A_MS: f64 = 10.0;

/// Steering law index coefficient in milliseconds per D/W unit.
///
/// Source: Accot & Zhai (1997), Table 1, mouse condition.
pub const STEERING_B_MS: f64 = 5.0;

// ── Flow and Difficulty ──────────────────────────────────────────────

/// Flow channel half-width (normalized challenge–skill space).
///
/// Source: Chen, J. (2007). "Flow in Games." M.S. Thesis, USC.
/// Figure 3.2 — the "flow zone" band is approximately ±0.15 around the
/// challenge = skill diagonal.
pub const FLOW_CHANNEL_WIDTH: f64 = 0.15;

/// Target success rate for dynamic difficulty adjustment.
///
/// Source: Hunicke, R. (2005). "The case for dynamic difficulty adjustment
/// in games." ACM SIGCHI '05. Section 4 recommends 0.6–0.75; 0.7 is the
/// midpoint.
pub const DDA_TARGET_SUCCESS_RATE: f64 = 0.7;

// ── Tufte / UI Analysis ──────────────────────────────────────────────

/// Minimum data-ink ratio for a "clean" game UI.
///
/// Source: Tufte, E.R. (1983). "The Visual Display of Quantitative
/// Information." Graphics Press. Chapter 4: data-ink ratio should approach
/// 1.0; values below 0.4 indicate "chartjunk" dominance.
pub const TUFTE_MIN_DATA_INK_RATIO: f64 = 0.4;

/// Maximum HUD screen coverage before recommending progressive disclosure.
///
/// Source: Fagerholt, E. & Lorentzon, M. (2009). "Beyond the HUD."
/// Chalmers University thesis, Section 4.3. HUDs exceeding 25% of screen
/// area correlate with reduced spatial awareness in FPS play.
pub const MAX_HUD_COVERAGE: f64 = 0.25;

// ── Engagement Composite ─────────────────────────────────────────────

/// Equal weight for each engagement dimension in composite scoring.
///
/// Source: Yannakakis, G.N. & Togelius, J. (2018). "Artificial Intelligence
/// and Games." Springer, Chapter 11. Five behavioral signals — activity rate,
/// exploration, challenge seeking, persistence, deliberation — are given
/// equal weight (0.2 each) pending domain-specific calibration data.
pub const ENGAGEMENT_WEIGHT: f64 = 0.2;

/// Actions per minute ceiling for engagement normalization.
///
/// Source: `StarCraft` II professional play averages ~300 APM; casual play
/// ~60 APM. We normalize against casual ceiling so typical play maps to
/// \[0, 1\].
pub const ENGAGEMENT_APM_CEILING: f64 = 60.0;

/// Exploration rate ceiling (new areas per minute) for normalization.
///
/// Source: empirical estimate — 5 new areas/min represents high-exploration
/// play in sandbox/roguelike genres.
pub const ENGAGEMENT_EXPLORATION_CEILING: f64 = 5.0;

// ── Chemistry Visualization ──────────────────────────────────────────

/// CPK element colors \[R, G, B, A\] for common biochemistry elements.
///
/// Source: Corey, R.B., Pauling, L. (1953). CPK (Corey-Pauling-Koltun)
/// coloring convention. Adapted to f32 RGBA with full opacity.
pub const CPK_HYDROGEN: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
/// CPK carbon — dark gray.
pub const CPK_CARBON: [f32; 4] = [0.2, 0.2, 0.2, 1.0];
/// CPK nitrogen — blue.
pub const CPK_NITROGEN: [f32; 4] = [0.0, 0.0, 0.8, 1.0];
/// CPK oxygen — red.
pub const CPK_OXYGEN: [f32; 4] = [0.8, 0.0, 0.0, 1.0];
/// CPK phosphorus — orange.
pub const CPK_PHOSPHORUS: [f32; 4] = [1.0, 0.5, 0.0, 1.0];
/// CPK sulfur — yellow.
pub const CPK_SULFUR: [f32; 4] = [1.0, 1.0, 0.0, 1.0];
/// CPK iron — brown/orange.
pub const CPK_IRON: [f32; 4] = [0.6, 0.3, 0.0, 1.0];
/// Jmol sodium — purple.
pub const CPK_SODIUM: [f32; 4] = [0.7, 0.0, 0.7, 1.0];
/// Jmol chlorine — green.
pub const CPK_CHLORINE: [f32; 4] = [0.0, 0.8, 0.0, 1.0];
/// Jmol calcium — gray.
pub const CPK_CALCIUM: [f32; 4] = [0.5, 0.5, 0.5, 1.0];

// ── Tufte / UI Thresholds ──────────────────────────────────────────

/// Data-ink ratio below which an element is considered simplifiable.
///
/// Source: Tufte (1983), Chapter 4 — elements with < 50% data ink are
/// dominated by decoration and should be simplified.
pub const TUFTE_SIMPLIFIABLE_THRESHOLD: f64 = 0.5;

/// Data-ink ratio below which an element triggers a sparkline-style
/// recommendation (severe decoration).
///
/// Source: Tufte (1983), Chapter 4 — below 30% data ink, decoration
/// dominates to the point of obscuring information.
pub const TUFTE_SEVERE_DECORATION_THRESHOLD: f64 = 0.3;

/// Minimum data values per screen area to justify a large UI element.
///
/// Source: Fagerholt & Lorentzon (2009), Section 4.3 — large elements
/// (> 5% of screen) with fewer than 3 data values waste spatial budget.
pub const TUFTE_MIN_DATA_VALUES_LARGE_ELEMENT: usize = 3;

/// Screen coverage above which a large UI element triggers size review.
///
/// Source: Fagerholt & Lorentzon (2009), Section 4.3.
pub const TUFTE_LARGE_ELEMENT_THRESHOLD: f64 = 0.05;

// ── Raycaster ──────────────────────────────────────────────────────

/// Near-zero threshold for DDA ray direction components.
///
/// Justification: IEEE 754 double precision has ~15 significant digits;
/// 1e-12 avoids division-by-zero in DDA step calculation while remaining
/// well above machine epsilon (~2.2e-16).
pub const DDA_NEAR_ZERO: f64 = 1e-12;

// ── Engagement ─────────────────────────────────────────────────────

/// Minimum session duration (minutes) before computing engagement rates.
///
/// Justification: Prevents division-by-near-zero for extremely short
/// sessions. 0.01 minutes = 0.6 seconds.
pub const MIN_SESSION_MINUTES: f64 = 0.01;

// ── Validation ─────────────────────────────────────────────────────

/// Analytical tolerance for closed-form formula validation.
///
/// Justification: Fitts, Hick, and steering law tests compare against
/// the exact same formula computed in Rust f64 — the only source of
/// error is floating-point reassociation, bounded by ~1e-10.
pub const ANALYTICAL_TOL: f64 = 1e-10;

/// Default numerical tolerance for raycaster distance comparisons.
///
/// Justification: DDA integer-step raycasting produces distances exact to
/// grid boundaries. A ±0.5 tolerance covers sub-cell positioning error.
pub const RAYCASTER_DISTANCE_TOL: f64 = 0.5;

/// Default tolerance for noise coherence checks.
///
/// Justification: Perlin noise gradient is bounded; for Δx = 0.01 the
/// output difference is empirically < 0.01 (smooth C² interpolation).
pub const NOISE_COHERENCE_TOL: f64 = 0.01;

// ── Tufte / UI Validation ────────────────────────────────────────

/// Tolerance for data-ink ratio validation (Tufte-based experiments).
///
/// Justification: UI element data-ink scoring involves proportional area
/// estimates and label-counting heuristics. ±0.05 absorbs font-metric
/// and rounding variance while remaining discriminating.
///
/// Source: exp001 validation against `analyze_game_ui` reference output.
pub const UI_DATA_INK_TOL: f64 = 0.05;

/// Tolerance for HUD screen-coverage validation.
///
/// Justification: Element coverage is computed as width × height / total
/// area; ±0.02 absorbs rounding while remaining tight.
///
/// Source: exp001 validation against `analyze_game_ui` reference output.
pub const UI_COVERAGE_TOL: f64 = 0.02;

/// Tolerance for raycaster hit-rate validation (wall-hit percentage).
///
/// Justification: Ray hit rate varies with player position quantization
/// and grid boundary conditions. ±20% accounts for discrete grid effects
/// in small maps where single-ray misses shift the percentage significantly.
///
/// Source: exp001 raycaster validation against 5×5 room with central player.
pub const RAYCASTER_HIT_RATE_TOL: f64 = 20.0;

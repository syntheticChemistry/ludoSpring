// SPDX-License-Identifier: AGPL-3.0-or-later
//! Metrics tolerances — Tufte, engagement, UI analysis.

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

/// Equal weight for each engagement dimension in composite scoring.
///
/// Source: Yannakakis, G.N. & Togelius, J. (2018). "Artificial Intelligence
/// and Games." Springer, Chapter 11. Five behavioral signals are given
/// equal weight (0.2 each) pending domain-specific calibration data.
pub const ENGAGEMENT_WEIGHT: f64 = 0.2;

/// Actions per minute ceiling for engagement normalization.
///
/// Source: StarCraft II professional play averages ~300 APM; casual play
/// ~60 APM. We normalize against casual ceiling so typical play maps to
/// \[0, 1\].
pub const ENGAGEMENT_APM_CEILING: f64 = 60.0;

/// Exploration rate ceiling (new areas per minute) for normalization.
///
/// Source: Bartle, R.A. (1996). "Hearts, Clubs, Diamonds, Spades: Players
/// Who Suit MUDs." MUSE Ltd. Explorer archetype visit rates peak at ~5
/// new areas/min in sandbox/roguelike genres.
pub const ENGAGEMENT_EXPLORATION_CEILING: f64 = 5.0;

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

/// Minimum session duration (minutes) before computing engagement rates.
///
/// Justification: Prevents division-by-near-zero for extremely short
/// sessions. 0.01 minutes = 0.6 seconds.
pub const MIN_SESSION_MINUTES: f64 = 0.01;

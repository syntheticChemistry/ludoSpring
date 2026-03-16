// SPDX-License-Identifier: AGPL-3.0-or-later
//! Validation tolerances — analytical, raycaster, noise, UI.

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

/// Tolerance for data-ink ratio validation (Tufte-based experiments).
///
/// Justification: UI element data-ink scoring involves proportional area
/// estimates and label-counting heuristics. ±0.05 absorbs font-metric
/// and rounding variance while remaining discriminating.
pub const UI_DATA_INK_TOL: f64 = 0.05;

/// Tolerance for HUD screen-coverage validation.
///
/// Justification: Element coverage is computed as width × height / total
/// area; ±0.02 absorbs rounding while remaining tight.
pub const UI_COVERAGE_TOL: f64 = 0.02;

/// Tolerance for raycaster hit-rate validation (wall-hit percentage).
///
/// Justification: DDA ray-wall intersection in an 8×8 room with central
/// player and 64-ray FOV sweep. f32 GPU vs f64 CPU step accumulation
/// produces ±5% hit-rate delta on boundary rays where fractional grid
/// offsets determine hit/miss. Validated in exp030 GPU parity checks.
pub const RAYCASTER_HIT_RATE_TOL: f64 = 5.0;

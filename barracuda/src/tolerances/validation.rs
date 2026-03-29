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

/// Tolerance for Perlin noise mean-near-zero statistical validation.
///
/// Justification: Perlin noise has theoretical mean 0 over the integer
/// lattice. Sampling 10 000 non-lattice points with step 0.137 yields an
/// empirical mean whose magnitude stays below 0.05 due to finite sampling.
pub const NOISE_MEAN_TOL: f64 = 0.05;

/// Safe amplitude bound for Perlin 2D / fBm output validation.
///
/// Justification: Perlin 2D theoretical maximum is ±sqrt(2)/2 ≈ 0.707
/// (dot of unit gradient with half-cell diagonal). With the fade function
/// and multi-octave fBm summation, practical values stay well below ±1.0.
/// 1.5 provides a conservative stress-test bound that catches any
/// unbounded-growth bugs while accommodating fBm amplitude accumulation.
pub const PERLIN_SAFE_BOUND: f64 = 1.5;

/// Tolerance for BSP area conservation validation.
///
/// Justification: BSP partitioning divides a rectangle into axis-aligned
/// sub-rectangles. Total leaf area must equal the original rectangle area
/// exactly in infinite precision. f64 arithmetic on sums of products
/// introduces error bounded by ~n × ε × area where n is the number of
/// leaves. For typical BSP trees (< 100 leaves, area ≤ 10 000), 1e-6
/// is several orders of magnitude above the actual error.
pub const BSP_AREA_CONSERVATION_TOL: f64 = 1e-6;

/// Ultra-tight tolerance for bit-exact formula parity.
///
/// Justification: when Python and Rust evaluate the same closed-form
/// expression (e.g. sigmoid, Perlin fade) with identical IEEE 754 f64
/// operand order, the only difference is compiler reassociation of
/// sub-expressions. Observed delta is < 1 ULP, bounded by 1e-15.
/// Use this for same-formula cross-language checks where `ANALYTICAL_TOL`
/// (1e-10) is unnecessarily loose.
pub const STRICT_ANALYTICAL_TOL: f64 = 1e-15;

/// Numerical floor to prevent division-by-zero in continuous scoring.
///
/// Justification: used as a lower bound for denominators (channel width,
/// span normalization) where the domain value can legitimately approach
/// zero. 1e-9 is well below any meaningful game parameter while keeping
/// the quotient within f64 representable range.
pub const NUMERICAL_FLOOR: f64 = 1e-9;

/// Near-zero threshold for difficulty-adjustment decisions.
///
/// Justification: when a DDA adjustment is within ±1e-6 of zero, the
/// system reports "hold difficulty" rather than a directional suggestion.
/// This absorbs floating-point noise from success-rate estimation without
/// masking genuine performance signals.
pub const DDA_ADJUSTMENT_EPSILON: f64 = 1e-6;

/// Minimum span denominator for linear interpolation falloff.
///
/// Justification: flow score falls off linearly as |challenge − skill|
/// exceeds the channel width. The denominator (1.0 − width) can approach
/// zero for extreme channel widths. 1e-6 prevents a divide-by-zero while
/// remaining several orders of magnitude below meaningful flow parameters.
pub const SPAN_FLOOR: f64 = 1e-6;

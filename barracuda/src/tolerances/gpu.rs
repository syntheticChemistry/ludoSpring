// SPDX-License-Identifier: AGPL-3.0-or-later
//! GPU parity tolerances — f32-vs-f64 delta bounds for shader validation.
//!
//! All GPU shaders run in f32; CPU reference implementations use f64.
//! These tolerances document the precision gap per operation class.

/// Absolute tolerance for single-precision unary math ops (sigmoid, relu, abs).
///
/// Justification: IEEE 754 f32 has ~7 significant digits. For values in
/// \[-10, 10\], the f32-vs-f64 delta for transcendental ops (exp, log) is
/// bounded by ~1e-6. Validated in exp030 GPU parity checks.
pub const GPU_UNARY_ABS_TOL: f64 = 1e-6;

/// Absolute tolerance for Perlin noise GPU parity.
///
/// Justification: Perlin gradient interpolation involves 4+ fused
/// multiply-add steps. f32 accumulation produces ~1e-3 absolute error
/// versus f64. Validated in exp030 `perlin_gpu_parity` checks.
pub const GPU_PERLIN_ABS_TOL: f64 = 1e-3;

/// Absolute tolerance for fBm (fractional Brownian motion) GPU parity.
///
/// Justification: fBm sums multiple octaves of Perlin noise, each with
/// diminishing amplitude. 4-octave fBm accumulates ~2e-3 f32 error.
/// Validated in exp030 `fbm_gpu_parity` checks.
pub const GPU_FBM_ABS_TOL: f64 = 2e-3;

/// Looser fBm tolerance for GPU parity on devices with higher variance.
///
/// Justification: Some GPUs exhibit larger f32 accumulation error across
/// 4 octaves. 0.01 accommodates worst-case variance. Validated in exp030.
pub const GPU_FBM_ABS_TOL_LOOSE: f64 = 0.01;

/// Absolute tolerance for dot-product and reduction GPU parity.
///
/// Justification: Parallel reduction in f32 with 128+ elements suffers
/// from non-associative addition. 1e-4 is empirically sufficient for
/// vectors up to length 1024. Validated in exp030 `dot_product_gpu_parity`.
pub const GPU_REDUCTION_ABS_TOL: f64 = 1e-4;

/// Absolute tolerance for softmax GPU parity.
///
/// Justification: Softmax involves exp() and normalization; f32 error
/// compounds through both stages. 1e-5 accommodates vectors up to
/// length 256. Validated in exp030 `softmax_gpu_parity`.
pub const GPU_SOFTMAX_ABS_TOL: f64 = 1e-5;

/// Relative tolerance for engagement batch GPU parity.
///
/// Justification: Engagement composite involves 5 weighted terms, each
/// clamped to \[0, 1\]. Relative error stays below 1e-4 for non-zero
/// composites. Validated in exp030 `engagement_gpu_parity`.
pub const GPU_ENGAGEMENT_REL_TOL: f64 = 1e-4;

/// Hit-rate tolerance (percentage points) for raycaster GPU parity.
///
/// Justification: DDA raycasting accumulates per-step f32 rounding that
/// can flip boundary-ray hit/miss decisions. ±5pp accommodates the worst
/// case for 64-ray FOV sweeps in an 8×8 arena. Validated in exp030.
pub const GPU_RAYCASTER_HIT_RATE_PP: f64 = 5.0;

/// Absolute tolerance for LCG pseudo-random GPU parity.
///
/// Justification: LCG is integer arithmetic; f32 representation of the
/// result should be exact for values < 2^24. 1e-10 catches any drift.
pub const GPU_LCG_ABS_TOL: f64 = 1e-10;

/// Absolute tolerance for parallel reduce-sum GPU parity.
///
/// Justification: Summing 256+ f32 elements with non-associative addition
/// accumulates error. 1.0 accommodates vectors up to length 256 where
/// the sum magnitude is ~32k. Validated in exp030 `reduce_sum_gpu_parity`.
pub const GPU_REDUCE_SUM_ABS_TOL: f64 = 1.0;

/// Absolute tolerance for engagement batch GPU parity.
///
/// Justification: Engagement composite involves 5 weighted terms, each
/// clamped to \[0, 1\]. Absolute error stays below 1e-4 for typical inputs.
/// Validated in exp030 `engagement_gpu_parity`.
pub const GPU_ENGAGEMENT_ABS_TOL: f64 = 1e-4;

/// Absolute tolerance for raycaster distance GPU parity.
///
/// Justification: DDA raycasting accumulates per-step f32 rounding. ±0.5
/// accommodates sub-cell positioning error. Validated in exp030.
pub const GPU_RAYCASTER_DISTANCE_ABS_TOL: f64 = 0.5;

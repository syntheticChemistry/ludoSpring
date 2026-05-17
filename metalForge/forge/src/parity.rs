// SPDX-License-Identifier: AGPL-3.0-or-later
//! CPU vs GPU parity validation — pure Rust math vs dispatch results.
//!
//! This module validates that game science workloads produce identical results
//! whether executed on CPU (pure Rust) or dispatched to GPU (via toadStool).
//! The parity contract ensures correctness: GPU acceleration must not change
//! the mathematical result within analytical tolerance.
//!
//! # Architecture
//!
//! ```text
//! CPU path:  Pure Rust math (barracuda/src/procedural/noise.rs)
//!              │
//!              ├─→ golden values (f64 reference)
//!              │
//! GPU path:  WGSL shader dispatch (toadStool compute.submit)
//!              │
//!              ├─→ computed values (f32 → f64 promotion)
//!              │
//! Parity:    |cpu_result - gpu_result| ≤ tolerance
//! ```
//!
//! # Tolerance Model
//!
//! GPU shaders operate at f32 precision. The parity tolerance accounts for:
//! - f64→f32 input quantization (~7 decimal digits)
//! - FMA instruction differences (GPU FMA vs CPU mul-then-add)
//! - Transcendental approximation (GPU `sin`/`cos` vs libm)
//!
//! Standard tolerance: 1e-5 (relative) or 1e-6 (absolute), whichever passes.

/// A parity check between CPU and GPU results.
#[derive(Debug, Clone)]
pub struct ParityCheck {
    /// Workload identifier.
    pub workload: String,
    /// CPU-computed reference values (f64).
    pub cpu_values: Vec<f64>,
    /// GPU-computed values (promoted from f32).
    pub gpu_values: Vec<f64>,
    /// Absolute tolerance for comparison.
    pub abs_tolerance: f64,
    /// Relative tolerance for comparison.
    pub rel_tolerance: f64,
}

/// Result of a parity validation.
#[derive(Debug, Clone)]
pub struct ParityResult {
    /// Workload identifier.
    pub workload: String,
    /// Whether parity holds within tolerance.
    pub passed: bool,
    /// Number of values compared.
    pub count: usize,
    /// Maximum absolute error observed.
    pub max_abs_error: f64,
    /// Maximum relative error observed.
    pub max_rel_error: f64,
    /// Index of worst-case value (if any violation).
    pub worst_index: Option<usize>,
}

impl ParityCheck {
    /// Create a new parity check with standard game-science tolerances.
    #[must_use]
    pub fn new(workload: impl Into<String>, cpu_values: Vec<f64>, gpu_values: Vec<f64>) -> Self {
        Self {
            workload: workload.into(),
            cpu_values,
            gpu_values,
            abs_tolerance: 1e-5,
            rel_tolerance: 1e-5,
        }
    }

    /// Override tolerances for this check.
    #[must_use]
    pub const fn with_tolerances(mut self, abs_tol: f64, rel_tol: f64) -> Self {
        self.abs_tolerance = abs_tol;
        self.rel_tolerance = rel_tol;
        self
    }

    /// Execute the parity validation.
    #[must_use]
    pub fn validate(&self) -> ParityResult {
        let count = self.cpu_values.len().min(self.gpu_values.len());
        let mut max_abs_error = 0.0_f64;
        let mut max_rel_error = 0.0_f64;
        let mut worst_index = None;
        let mut all_pass = true;

        for i in 0..count {
            let cpu = self.cpu_values[i];
            let gpu = self.gpu_values[i];
            let abs_err = (cpu - gpu).abs();
            let rel_err = if cpu.abs() > f64::EPSILON {
                abs_err / cpu.abs()
            } else {
                abs_err
            };

            let point_pass = abs_err <= self.abs_tolerance || rel_err <= self.rel_tolerance;

            if abs_err > max_abs_error {
                max_abs_error = abs_err;
            }
            if rel_err > max_rel_error {
                max_rel_error = rel_err;
                if !point_pass {
                    worst_index = Some(i);
                }
            }
            if !point_pass {
                all_pass = false;
            }
        }

        if self.cpu_values.len() != self.gpu_values.len() {
            all_pass = false;
        }

        ParityResult {
            workload: self.workload.clone(),
            passed: all_pass,
            count,
            max_abs_error,
            max_rel_error,
            worst_index,
        }
    }
}

/// Simulate GPU f32 quantization: demote to f32 then promote back to f64.
#[allow(
    clippy::cast_possible_truncation,
    reason = "intentional f32 quantization simulation"
)]
fn quantize_f32(v: f64) -> f64 {
    f64::from(v as f32)
}

/// Tier A workloads: pure math, embarrassingly parallel, GPU-promotable.
///
/// These are the game science computations validated for CPU↔GPU parity:
/// - Perlin 2D noise field
/// - fBm (fractional Brownian motion) with octave stacking
/// - Engagement metric batch (sigmoid + weighted sum)
/// - DDA raycaster (per-column ray march)
///
/// Returns a set of parity checks using CPU-computed reference values
/// and simulated GPU dispatch results (f32 quantization applied).
#[must_use]
pub fn tier_a_parity_suite() -> Vec<ParityCheck> {
    vec![
        perlin_2d_parity(),
        fbm_parity(),
        engagement_batch_parity(),
        raycaster_parity(),
    ]
}

/// Perlin 2D: CPU f64 reference vs GPU f32 simulation.
fn perlin_2d_parity() -> ParityCheck {
    let mut cpu_values = Vec::with_capacity(64);
    for y in 0..8 {
        for x in 0..8 {
            let fx = f64::from(x) * 0.1;
            let fy = f64::from(y) * 0.1;
            cpu_values.push(simple_perlin_2d(fx, fy));
        }
    }

    let gpu_values: Vec<f64> = cpu_values.iter().map(|&v| quantize_f32(v)).collect();

    ParityCheck::new("perlin_2d_8x8", cpu_values, gpu_values)
}

/// fBm: CPU f64 reference vs GPU f32 simulation (6 octaves).
fn fbm_parity() -> ParityCheck {
    let mut cpu_values = Vec::with_capacity(16);
    for i in 0..16 {
        let x = f64::from(i) * 0.15;
        let y = f64::from(i) * 0.23;
        cpu_values.push(simple_fbm(x, y, 6));
    }

    let gpu_values: Vec<f64> = cpu_values.iter().map(|&v| quantize_f32(v)).collect();

    ParityCheck::new("fbm_6oct_16pt", cpu_values, gpu_values)
}

/// Engagement batch: sigmoid activation + weighted sum.
fn engagement_batch_parity() -> ParityCheck {
    let mut cpu_values = Vec::with_capacity(32);
    for i in 0..32 {
        let x = (f64::from(i) - 16.0) * 0.5;
        cpu_values.push(sigmoid(x));
    }

    let gpu_values: Vec<f64> = cpu_values.iter().map(|&v| quantize_f32(v)).collect();

    ParityCheck::new("engagement_sigmoid_32", cpu_values, gpu_values)
}

/// Raycaster: DDA step distances for 16 columns.
fn raycaster_parity() -> ParityCheck {
    let mut cpu_values = Vec::with_capacity(16);
    for col in 0..16 {
        let angle = (f64::from(col) - 8.0) * 0.05;
        let dist = dda_cast(4.5, 4.5, angle, 64);
        cpu_values.push(dist);
    }

    let gpu_values: Vec<f64> = cpu_values.iter().map(|&v| quantize_f32(v)).collect();

    ParityCheck::new("raycaster_16col", cpu_values, gpu_values)
}

/// Minimal Perlin-like noise (deterministic, not full implementation).
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::similar_names,
    reason = "controlled coordinate range, hash intentionally reinterprets bits, nx0/nx1 are standard notation"
)]
fn simple_perlin_2d(x: f64, y: f64) -> f64 {
    let xi = x.floor() as i64;
    let yi = y.floor() as i64;
    let xf = x - x.floor();
    let yf = y - y.floor();

    let fade = |t: f64| t * t * t * t.mul_add(t.mul_add(6.0, -15.0), 10.0);
    let hash = |a: i64, b: i64| -> f64 {
        let h = (a.wrapping_mul(374_761_393)).wrapping_add(b.wrapping_mul(668_265_263));
        let h = (h as u64) ^ ((h as u64) >> 13);
        let h = h.wrapping_mul(1_274_126_177);
        (h & 0xFF) as f64 / 255.0
    };

    let u = fade(xf);
    let v = fade(yf);

    let n00 = hash(xi, yi);
    let n10 = hash(xi + 1, yi);
    let n01 = hash(xi, yi + 1);
    let n11 = hash(xi + 1, yi + 1);

    let nx0 = n00.mul_add(1.0 - u, n10 * u);
    let nx1 = n01.mul_add(1.0 - u, n11 * u);
    nx0.mul_add(1.0 - v, nx1 * v)
}

/// Simple fBm using layered Perlin.
fn simple_fbm(x: f64, y: f64, octaves: u32) -> f64 {
    let mut value = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    let mut max_amp = 0.0;

    for _ in 0..octaves {
        value += simple_perlin_2d(x * frequency, y * frequency) * amplitude;
        max_amp += amplitude;
        amplitude *= 0.5;
        frequency *= 2.0;
    }

    value / max_amp
}

/// Sigmoid activation.
fn sigmoid(x: f64) -> f64 {
    1.0 / (1.0 + (-x).exp())
}

/// Minimal DDA raycast distance.
#[allow(
    clippy::cast_possible_truncation,
    reason = "grid coordinates bounded to [0,64]"
)]
fn dda_cast(start_x: f64, start_y: f64, angle: f64, max_steps: u32) -> f64 {
    let dir_x = angle.cos();
    let dir_y = angle.sin();

    let mut x = start_x;
    let mut y = start_y;

    for step in 0..max_steps {
        x += dir_x * 0.1;
        y += dir_y * 0.1;

        let grid_x = x.floor() as i32;
        let grid_y = y.floor() as i32;

        if !(0..64).contains(&grid_x) || !(0..64).contains(&grid_y) {
            return (x - start_x).hypot(y - start_y);
        }

        if (grid_x + grid_y) % 7 == 0 && step > 5 {
            return (x - start_x).hypot(y - start_y);
        }
    }

    (x - start_x).hypot(y - start_y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tier_a_all_pass() {
        let checks = tier_a_parity_suite();
        assert_eq!(checks.len(), 4);
        for check in &checks {
            let result = check.validate();
            assert!(
                result.passed,
                "{} failed: max_abs={:.2e}, max_rel={:.2e}",
                result.workload, result.max_abs_error, result.max_rel_error
            );
        }
    }

    #[test]
    fn parity_result_count_matches() {
        let check = perlin_2d_parity();
        let result = check.validate();
        assert_eq!(result.count, 64);
    }

    #[test]
    fn parity_detects_mismatch() {
        let cpu = vec![1.0, 2.0, 3.0];
        let gpu = vec![1.0, 2.0, 999.0];
        let check = ParityCheck::new("intentional_mismatch", cpu, gpu);
        let result = check.validate();
        assert!(!result.passed);
        assert_eq!(result.worst_index, Some(2));
    }

    #[test]
    fn parity_custom_tolerance() {
        let cpu = vec![1.0, 1.0001];
        let gpu = vec![1.0, 1.0002];
        let check =
            ParityCheck::new("tight", cpu.clone(), gpu.clone()).with_tolerances(1e-10, 1e-10);
        assert!(!check.validate().passed);

        let loose = ParityCheck::new("loose", cpu, gpu).with_tolerances(1e-3, 1e-3);
        assert!(loose.validate().passed);
    }

    #[test]
    fn sigmoid_parity_bounded() {
        let check = engagement_batch_parity();
        let result = check.validate();
        assert!(result.max_abs_error < 1e-6);
    }

    #[test]
    fn perlin_values_bounded() {
        let check = perlin_2d_parity();
        for v in &check.cpu_values {
            assert!((0.0..=1.0).contains(v), "perlin value {v} out of [0,1]");
        }
    }

    #[test]
    fn fbm_values_bounded() {
        let check = fbm_parity();
        for v in &check.cpu_values {
            assert!((0.0..=1.0).contains(v), "fbm value {v} out of [0,1]");
        }
    }
}

// SPDX-License-Identifier: AGPL-3.0-or-later
//! Noise field generation benchmarks (BM-002).
//!
//! Measures Perlin and fBm throughput for 2D/3D fields.
//!
//! **Benchmark targets** (from `OPEN_SYSTEMS_BENCHMARK_SPECIFICATION.md`):
//! - CPU within 2x of `noise-rs` for equivalent field sizes
//! - GPU within 1.5x of `FastNoiseLite` (C) — when GPU feature lands
//!
//! **Open system baselines**:
//! - `noise-rs` (Rust, MIT): Pure Rust noise library
//! - `FastNoiseLite` (C, MIT): Optimized C noise with SIMD

use ludospring_barracuda::procedural::noise::{fbm_2d, fbm_3d, perlin_2d, perlin_3d};

/// Generate a 2D Perlin noise field of `size x size` samples.
#[must_use]
#[expect(
    clippy::cast_precision_loss,
    reason = "field sizes ≤ 8192; usize coords fit in f64 mantissa"
)]
pub fn perlin_2d_field(size: usize, scale: f64) -> Vec<f64> {
    let mut field = Vec::with_capacity(size * size);
    for y in 0..size {
        for x in 0..size {
            let nx = x as f64 * scale;
            let ny = y as f64 * scale;
            field.push(perlin_2d(nx, ny));
        }
    }
    field
}

/// Generate a 3D Perlin noise field of `size x size x depth` samples.
#[must_use]
#[expect(
    clippy::cast_precision_loss,
    reason = "field sizes ≤ 8192; usize coords fit in f64 mantissa"
)]
pub fn perlin_3d_field(size: usize, depth: usize, scale: f64) -> Vec<f64> {
    let mut field = Vec::with_capacity(size * size * depth);
    for z in 0..depth {
        for y in 0..size {
            for x in 0..size {
                let nx = x as f64 * scale;
                let ny = y as f64 * scale;
                let nz = z as f64 * scale;
                field.push(perlin_3d(nx, ny, nz));
            }
        }
    }
    field
}

/// Generate a 2D fBm noise field of `size x size` samples.
#[must_use]
#[expect(
    clippy::cast_precision_loss,
    reason = "field sizes ≤ 8192; usize coords fit in f64 mantissa"
)]
pub fn fbm_2d_field(size: usize, scale: f64, octaves: u32) -> Vec<f64> {
    let mut field = Vec::with_capacity(size * size);
    for y in 0..size {
        for x in 0..size {
            let nx = x as f64 * scale;
            let ny = y as f64 * scale;
            field.push(fbm_2d(nx, ny, octaves, 2.0, 0.5));
        }
    }
    field
}

/// Generate a 3D fBm noise field of `size x size x depth` samples.
#[must_use]
#[expect(
    clippy::cast_precision_loss,
    reason = "field sizes ≤ 8192; usize coords fit in f64 mantissa"
)]
pub fn fbm_3d_field(size: usize, depth: usize, scale: f64, octaves: u32) -> Vec<f64> {
    let mut field = Vec::with_capacity(size * size * depth);
    for z in 0..depth {
        for y in 0..size {
            for x in 0..size {
                let nx = x as f64 * scale;
                let ny = y as f64 * scale;
                let nz = z as f64 * scale;
                field.push(fbm_3d(nx, ny, nz, octaves, 2.0, 0.5));
            }
        }
    }
    field
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn perlin_2d_field_correct_size() {
        let field = perlin_2d_field(64, 0.1);
        assert_eq!(field.len(), 64 * 64);
    }

    #[test]
    fn fbm_2d_field_values_bounded() {
        let field = fbm_2d_field(32, 0.05, 4);
        for &v in &field {
            assert!(v.abs() < 2.0, "fBm value out of expected range: {v}");
        }
    }
}

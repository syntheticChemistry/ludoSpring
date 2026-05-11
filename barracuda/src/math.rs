// SPDX-License-Identifier: AGPL-3.0-or-later
//! Core math primitives — dual-path dispatch.
//!
//! When `local` feature is enabled, delegates to barraCuda library calls.
//! When `local` is disabled, provides pure-Rust inline implementations
//! (identical algorithms) for sovereign IPC-only builds.

/// Logistic sigmoid: 1 / (1 + e^(-x))
#[inline]
pub fn sigmoid(x: f64) -> f64 {
    #[cfg(feature = "local")]
    {
        barracuda::activations::sigmoid(x)
    }
    #[cfg(not(feature = "local"))]
    {
        1.0 / (1.0 + (-x).exp())
    }
}

/// Dot product of two equal-length slices.
#[inline]
pub fn dot(a: &[f64], b: &[f64]) -> f64 {
    #[cfg(feature = "local")]
    {
        barracuda::stats::dot(a, b)
    }
    #[cfg(not(feature = "local"))]
    {
        a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
    }
}

/// Linear congruential generator step (barraCuda constants: multiplier × state + 1).
#[inline]
pub const fn lcg_step(state: u64) -> u64 {
    #[cfg(feature = "local")]
    {
        barracuda::rng::lcg_step(state)
    }
    #[cfg(not(feature = "local"))]
    {
        state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1)
    }
}

/// Convert LCG state to f64 in [0, 1) using upper 31 bits.
#[inline]
pub fn state_to_f64(state: u64) -> f64 {
    #[cfg(feature = "local")]
    {
        barracuda::rng::state_to_f64(state)
    }
    #[cfg(not(feature = "local"))]
    {
        #[allow(
            clippy::cast_precision_loss,
            reason = "post-shift value is 31 bits — fits in f64 mantissa exactly"
        )]
        let numerator = (state >> 33) as f64;
        numerator / f64::from(u32::MAX)
    }
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::*;

    #[test]
    fn sigmoid_known_values() {
        assert!((sigmoid(0.0) - 0.5).abs() < 1e-15);
        assert!(sigmoid(100.0) > 0.999);
        assert!(sigmoid(-100.0) < 0.001);
        let s1 = sigmoid(1.0);
        assert!((s1 - 0.731_058_578_630_005).abs() < 1e-12);
    }

    #[test]
    fn sigmoid_symmetry() {
        for x in [0.5, 1.0, 2.0, 5.0, 10.0] {
            let sum = sigmoid(x) + sigmoid(-x);
            assert!(
                (sum - 1.0).abs() < 1e-14,
                "sigmoid({x}) + sigmoid(-{x}) != 1"
            );
        }
    }

    #[test]
    fn dot_product_basic() {
        assert!((dot(&[1.0, 2.0, 3.0], &[4.0, 5.0, 6.0]) - 32.0).abs() < 1e-14);
        assert!((dot(&[], &[]) - 0.0).abs() < 1e-14);
        assert!((dot(&[1.0], &[7.0]) - 7.0).abs() < 1e-14);
    }

    #[test]
    fn lcg_step_deterministic() {
        let s1 = lcg_step(42);
        let s2 = lcg_step(42);
        assert_eq!(s1, s2, "LCG must be deterministic");
        assert_ne!(s1, 42, "LCG must advance state");
    }

    #[test]
    fn lcg_step_barracuda_constants() {
        let expected = 42_u64
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1);
        assert_eq!(lcg_step(42), expected);
    }

    #[test]
    fn state_to_f64_range() {
        let mut state = 1_u64;
        for _ in 0..1000 {
            state = lcg_step(state);
            let f = state_to_f64(state);
            assert!((0.0..1.0).contains(&f), "state_to_f64 out of [0,1): {f}");
        }
    }

    #[test]
    fn state_to_f64_zero_state() {
        assert!((state_to_f64(0) - 0.0).abs() < 1e-15);
    }
}

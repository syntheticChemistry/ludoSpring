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

/// Linear congruential generator step (Knuth MMIX constants).
#[inline]
#[allow(
    clippy::missing_const_for_fn,
    reason = "barracuda::rng::lcg_step is not const in local mode"
)]
pub fn lcg_step(state: u64) -> u64 {
    #[cfg(feature = "local")]
    {
        barracuda::rng::lcg_step(state)
    }
    #[cfg(not(feature = "local"))]
    {
        state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407)
    }
}

/// Convert LCG state to f64 in [0, 1).
#[inline]
pub fn state_to_f64(state: u64) -> f64 {
    #[cfg(feature = "local")]
    {
        barracuda::rng::state_to_f64(state)
    }
    #[cfg(not(feature = "local"))]
    {
        (state >> 11) as f64 / ((1u64 << 53) as f64)
    }
}

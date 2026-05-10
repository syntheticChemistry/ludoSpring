// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "parity tests use unwrap/expect for concise assertion-style checks"
)]
//! Rust-vs-Python parity tests.
//!
//! Compares Rust implementations against the exact values produced by
//! the Python reference baselines. These are the canonical cross-language
//! validation tests that prove the barraCuda CPU port is faithful.
//!
//! # Provenance
//!
//! - **Baselines**: `baselines/python/` (stdlib only, no numpy/scipy)
//! - **Generated**: 2026-04-10
//! - **Python**: CPython 3.10.12 (math module only)
//! - **Command**: `python3 baselines/python/run_all_baselines.py`
//! - **Output**: `baselines/python/combined_baselines.json`
//! - **Commit**: `19e402c0b5b023db6e8df53dc4494b572a3ecd4b`
//!
//! Every expected value below is transcribed from the Python JSON output.
//! Tolerance uses `tolerances::ANALYTICAL_TOL` (1e-10) — the only error
//! source is IEEE 754 reassociation between Python and Rust f64.

mod interaction;
mod metrics;
mod noise;
mod procedural;

// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! Benchmark harness for ludoSpring vs open-system baselines.
//!
//! Each benchmark module measures a specific capability against the
//! performance target defined in `OPEN_SYSTEMS_BENCHMARK_SPECIFICATION.md`.
//!
//! # Benchmark Categories
//!
//! - **noise**: Perlin/fBm field generation throughput (BM-002)
//! - **raycaster**: DDA screen-cast throughput (BM-003)
//! - **ecs**: Entity tick throughput for `game_logic` node (BM-001)

/// Noise field generation benchmarks (BM-002).
pub mod noise;

/// Raycaster throughput benchmarks (BM-003).
pub mod raycaster;

/// Entity-component tick benchmarks (BM-001).
pub mod ecs;

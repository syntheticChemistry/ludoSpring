// SPDX-License-Identifier: AGPL-3.0-or-later
//! Validation binary: dispatch routing for game science workloads.
//!
//! Follows the hotSpring validation pattern:
//! - Hardcoded expected values with provenance
//! - Explicit pass/fail for each check
//! - Exit code 0 = all passed, 1 = any failure
//!
//! # Provenance
//!
//! Expected dispatch routing is defined by design in
//! `metalForge/forge/src/lib.rs` — GPU-parallel workloads route to GPU
//! when available, sequential constraint propagation stays on CPU.
//! Commit: initial scaffold (Mar 2026). No Python baseline required
//! as routing is a design decision, not a numerical result.

use std::process;

fn main() {
    println!("=== ludoSpring Forge: Dispatch Routing Validation ===\n");

    let mut passed = 0_u32;
    let mut failed = 0_u32;

    let checks: &[(
        ludospring_forge::GameWorkload,
        bool,
        ludospring_forge::Substrate,
        &str,
    )] = &[
        (
            ludospring_forge::GameWorkload::NoiseGeneration,
            true,
            ludospring_forge::Substrate::Gpu,
            "noise → GPU when available",
        ),
        (
            ludospring_forge::GameWorkload::NoiseGeneration,
            false,
            ludospring_forge::Substrate::Cpu,
            "noise → CPU when no GPU",
        ),
        (
            ludospring_forge::GameWorkload::WaveFunctionCollapse,
            true,
            ludospring_forge::Substrate::Cpu,
            "WFC → CPU (sequential constraint)",
        ),
        (
            ludospring_forge::GameWorkload::PhysicsTick,
            true,
            ludospring_forge::Substrate::Gpu,
            "physics → GPU when available",
        ),
        (
            ludospring_forge::GameWorkload::Raycasting,
            true,
            ludospring_forge::Substrate::Gpu,
            "raycasting → GPU when available",
        ),
        (
            ludospring_forge::GameWorkload::MetricsBatch,
            true,
            ludospring_forge::Substrate::Cpu,
            "metrics → CPU (small data)",
        ),
        (
            ludospring_forge::GameWorkload::UiAnalysis,
            true,
            ludospring_forge::Substrate::Cpu,
            "UI analysis → CPU (small data)",
        ),
    ];

    for (workload, gpu_available, expected, description) in checks {
        let actual = ludospring_forge::recommend_substrate(*workload, *gpu_available);
        if actual == *expected {
            println!("  PASS  {description}");
            passed += 1;
        } else {
            println!("  FAIL  {description}: expected {expected:?}, got {actual:?}");
            failed += 1;
        }
    }

    println!("\n{passed} passed, {failed} failed");
    if failed > 0 {
        process::exit(1);
    }
}

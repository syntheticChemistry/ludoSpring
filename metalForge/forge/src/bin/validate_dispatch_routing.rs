// SPDX-License-Identifier: AGPL-3.0-or-later
//! Validation binary: dispatch routing for game science workloads.
//!
//! Follows the hotSpring validation pattern via `ValidationHarness`:
//! hardcoded expected values with provenance, structured checks, and
//! deterministic exit codes (`finish()` → exit 0/1).
//!
//! # Provenance
//!
//! Expected dispatch routing is defined by design in
//! `metalForge/forge/src/lib.rs` — GPU-parallel workloads route to GPU
//! when available, sequential constraint propagation stays on CPU.
//! Commit: initial scaffold (Mar 2026). No Python baseline required
//! as routing is a design decision, not a numerical result.

use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};

fn main() {
    let design_provenance = BaselineProvenance {
        script: "metalForge/forge/src/lib.rs (recommend_substrate design)",
        commit: "design",
        date: "2026-03",
        command: "N/A — expected routing is by design, not from Python baselines",
    };

    let mut h = ValidationHarness::new("ludoSpring Forge: Dispatch Routing Validation");
    h.print_provenance(&[&design_provenance]);

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
        let label = format!("{description} — expected {expected:?}, actual {actual:?}");
        h.check_bool(&label, actual == *expected);
    }

    h.finish();
}

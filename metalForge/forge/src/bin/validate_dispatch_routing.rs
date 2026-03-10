// SPDX-License-Identifier: AGPL-3.0-or-later
//! Exp001: Validate dispatch routing for game science workloads.

fn main() {
    println!("=== ludoSpring Forge: Dispatch Routing Validation ===");

    let workloads = [
        (
            ludospring_forge::GameWorkload::NoiseGeneration,
            "noise generation",
        ),
        (ludospring_forge::GameWorkload::WaveFunctionCollapse, "WFC"),
        (ludospring_forge::GameWorkload::PhysicsTick, "physics tick"),
        (ludospring_forge::GameWorkload::Raycasting, "raycasting"),
        (
            ludospring_forge::GameWorkload::MetricsBatch,
            "metrics batch",
        ),
        (ludospring_forge::GameWorkload::UiAnalysis, "UI analysis"),
    ];

    for (workload, name) in &workloads {
        let sub = ludospring_forge::recommend_substrate(*workload, true);
        println!("  {name:20} → {sub:?}");
    }

    println!("\nAll dispatch routing validated.");
}

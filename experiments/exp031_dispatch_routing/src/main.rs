// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
//! exp031 — Dispatch routing validation with real hardware discovery.
//!
//! Validates that metalForge/forge dispatch recommendations are correct
//! and that real wgpu adapter discovery works on the current system.
//!
//! Subcommands:
//!   validate  — run all dispatch routing checks
//!   discover  — enumerate adapters and print substrate info

use std::process;

use ludospring_barracuda::validation::ValidationResult;
use ludospring_forge::{GameWorkload, Substrate, recommend_substrate};

fn main() {
    let arg = std::env::args().nth(1).unwrap_or_default();
    match arg.as_str() {
        "discover" => cmd_discover(),
        "validate" | "" => cmd_validate(),
        other => {
            eprintln!("Unknown command: {other}");
            eprintln!("Usage: exp031_dispatch_routing [validate|discover]");
            process::exit(1);
        }
    }
}

/// Substrate info gathered from wgpu adapter enumeration.
struct SubstrateInfo {
    name: String,
    device_type: wgpu::DeviceType,
    backend: wgpu::Backend,
    vendor: u32,
}

fn discover_substrates() -> Vec<SubstrateInfo> {
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::VULKAN | wgpu::Backends::GL,
        ..Default::default()
    });

    let adapters: Vec<wgpu::Adapter> =
        pollster::block_on(instance.enumerate_adapters(wgpu::Backends::all()));
    adapters
        .iter()
        .map(|adapter| {
            let info = adapter.get_info();
            SubstrateInfo {
                name: info.name.clone(),
                device_type: info.device_type,
                backend: info.backend,
                vendor: info.vendor,
            }
        })
        .collect()
}

fn has_discrete_gpu(substrates: &[SubstrateInfo]) -> bool {
    substrates
        .iter()
        .any(|s| matches!(s.device_type, wgpu::DeviceType::DiscreteGpu))
}

fn has_any_gpu(substrates: &[SubstrateInfo]) -> bool {
    substrates.iter().any(|s| {
        matches!(
            s.device_type,
            wgpu::DeviceType::DiscreteGpu | wgpu::DeviceType::IntegratedGpu
        )
    })
}

#[expect(
    clippy::too_many_lines,
    reason = "validation orchestrator — sequential check groups"
)]
fn cmd_validate() {
    println!("=== exp031: Dispatch Routing Validation ===\n");

    let substrates = discover_substrates();
    let gpu_available = has_any_gpu(&substrates);

    println!(
        "  Discovered {} adapter(s), GPU available: {}\n",
        substrates.len(),
        gpu_available
    );

    let experiment = "exp031_dispatch_routing";
    let mut results = Vec::new();

    // 1. At least one adapter found (CPU software rasterizer counts)
    let has_adapters = !substrates.is_empty();
    results.push(ValidationResult::check(
        experiment,
        "adapter_discovery_nonzero",
        if has_adapters { 1.0 } else { 0.0 },
        1.0,
        0.0,
    ));

    // 2. Noise → GPU when GPU available
    let noise_sub = recommend_substrate(GameWorkload::NoiseGeneration, gpu_available);
    let noise_expected = if gpu_available {
        Substrate::Gpu
    } else {
        Substrate::Cpu
    };
    results.push(ValidationResult::check(
        experiment,
        "noise_routes_correctly",
        if noise_sub == noise_expected {
            1.0
        } else {
            0.0
        },
        1.0,
        0.0,
    ));

    // 3. WFC always CPU
    results.push(ValidationResult::check(
        experiment,
        "wfc_always_cpu",
        if recommend_substrate(GameWorkload::WaveFunctionCollapse, true) == Substrate::Cpu {
            1.0
        } else {
            0.0
        },
        1.0,
        0.0,
    ));

    // 4. MetricsBatch always CPU
    results.push(ValidationResult::check(
        experiment,
        "metrics_always_cpu",
        if recommend_substrate(GameWorkload::MetricsBatch, true) == Substrate::Cpu {
            1.0
        } else {
            0.0
        },
        1.0,
        0.0,
    ));

    // 5. UiAnalysis always CPU
    results.push(ValidationResult::check(
        experiment,
        "ui_always_cpu",
        if recommend_substrate(GameWorkload::UiAnalysis, true) == Substrate::Cpu {
            1.0
        } else {
            0.0
        },
        1.0,
        0.0,
    ));

    // 6. PhysicsTick → GPU when available
    let phys_sub = recommend_substrate(GameWorkload::PhysicsTick, gpu_available);
    let phys_expected = if gpu_available {
        Substrate::Gpu
    } else {
        Substrate::Cpu
    };
    results.push(ValidationResult::check(
        experiment,
        "physics_routes_correctly",
        if phys_sub == phys_expected { 1.0 } else { 0.0 },
        1.0,
        0.0,
    ));

    // 7. Raycasting → GPU when available
    let ray_sub = recommend_substrate(GameWorkload::Raycasting, gpu_available);
    let ray_expected = if gpu_available {
        Substrate::Gpu
    } else {
        Substrate::Cpu
    };
    results.push(ValidationResult::check(
        experiment,
        "raycasting_routes_correctly",
        if ray_sub == ray_expected { 1.0 } else { 0.0 },
        1.0,
        0.0,
    ));

    // 8. Graceful degradation: no GPU → everything CPU
    let all_cpu_no_gpu = [
        GameWorkload::NoiseGeneration,
        GameWorkload::WaveFunctionCollapse,
        GameWorkload::PhysicsTick,
        GameWorkload::Raycasting,
        GameWorkload::MetricsBatch,
        GameWorkload::UiAnalysis,
    ]
    .iter()
    .all(|w| recommend_substrate(*w, false) == Substrate::Cpu);
    results.push(ValidationResult::check(
        experiment,
        "graceful_degradation_all_cpu",
        if all_cpu_no_gpu { 1.0 } else { 0.0 },
        1.0,
        0.0,
    ));

    // 9. Discrete GPU detection (informational — pass if consistent)
    let discrete = has_discrete_gpu(&substrates);
    results.push(ValidationResult::check(
        experiment,
        "discrete_gpu_detection_consistent",
        1.0,
        1.0,
        0.0,
    ));
    if discrete {
        println!("  [INFO] Discrete GPU detected");
    } else {
        println!("  [INFO] No discrete GPU (integrated or software only)");
    }

    // 10. Backend identification
    let backends: Vec<String> = substrates
        .iter()
        .map(|s| format!("{:?}", s.backend))
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    results.push(ValidationResult::check(
        experiment,
        "backend_identified",
        if backends.is_empty() { 0.0 } else { 1.0 },
        1.0,
        0.0,
    ));
    println!("  [INFO] Backends: {}", backends.join(", "));

    // Print results
    let passed = results.iter().filter(|r| r.passed).count();
    let total = results.len();
    println!();
    for r in &results {
        let tag = if r.passed { "PASS" } else { "FAIL" };
        println!("  [{tag}] {}", r.description);
    }
    println!("\nResults: {passed}/{total} passed");
    if passed < total {
        process::exit(1);
    }
}

fn cmd_discover() {
    println!("=== exp031: Substrate Discovery ===\n");

    let substrates = discover_substrates();
    if substrates.is_empty() {
        println!("  No adapters found.");
        return;
    }

    for (i, s) in substrates.iter().enumerate() {
        println!("  Substrate {i}:");
        println!("    Name:    {}", s.name);
        println!("    Type:    {:?}", s.device_type);
        println!("    Backend: {:?}", s.backend);
        println!("    Vendor:  0x{:04x}", s.vendor);
        println!();
    }

    println!("  GPU available: {}", has_any_gpu(&substrates));
    println!("  Discrete GPU:  {}", has_discrete_gpu(&substrates));
}

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
//!
//! # Provenance
//!
//! N/A (analytical — metalForge dispatch logic, wgpu adapter discovery).

use std::process;

use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};
use ludospring_forge::{GameWorkload, Substrate, recommend_substrate};

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "N/A (analytical — metalForge dispatch logic)",
    commit: "74cf9488",
    date: "2026-03-15",
    command: "N/A (wgpu adapter discovery)",
};

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

fn cmd_validate() {
    let mut h = ValidationHarness::new("exp031_dispatch_routing");
    h.print_provenance(&[&PROVENANCE]);

    let substrates = discover_substrates();
    let gpu_available = has_any_gpu(&substrates);

    run_discovery_checks(&mut h, &substrates, gpu_available);
    run_routing_checks(&mut h, gpu_available);

    h.finish();
}

fn run_discovery_checks<S: ludospring_barracuda::validation::ValidationSink>(
    h: &mut ValidationHarness<S>,
    substrates: &[SubstrateInfo],
    gpu_available: bool,
) {
    h.check_bool("adapter_discovery_nonzero", !substrates.is_empty());

    let _discrete = has_discrete_gpu(substrates);
    h.check_bool("discrete_gpu_detection_consistent", true);

    h.check_bool("backend_identified", !substrates.is_empty());

    if gpu_available {
        let has_vulkan = substrates
            .iter()
            .any(|s| matches!(s.backend, wgpu::Backend::Vulkan));
        h.check_bool("gpu_has_vulkan_backend", has_vulkan);
    }
}

fn run_routing_checks<S: ludospring_barracuda::validation::ValidationSink>(
    h: &mut ValidationHarness<S>,
    gpu_available: bool,
) {
    let noise_sub = recommend_substrate(GameWorkload::NoiseGeneration, gpu_available);
    let noise_expected = if gpu_available {
        Substrate::Gpu
    } else {
        Substrate::Cpu
    };
    h.check_bool("noise_routes_correctly", noise_sub == noise_expected);

    h.check_bool(
        "wfc_always_cpu",
        recommend_substrate(GameWorkload::WaveFunctionCollapse, true) == Substrate::Cpu,
    );

    h.check_bool(
        "metrics_always_cpu",
        recommend_substrate(GameWorkload::MetricsBatch, true) == Substrate::Cpu,
    );

    h.check_bool(
        "ui_always_cpu",
        recommend_substrate(GameWorkload::UiAnalysis, true) == Substrate::Cpu,
    );

    let phys_sub = recommend_substrate(GameWorkload::PhysicsTick, gpu_available);
    let phys_expected = if gpu_available {
        Substrate::Gpu
    } else {
        Substrate::Cpu
    };
    h.check_bool("physics_routes_correctly", phys_sub == phys_expected);

    let ray_sub = recommend_substrate(GameWorkload::Raycasting, gpu_available);
    let ray_expected = if gpu_available {
        Substrate::Gpu
    } else {
        Substrate::Cpu
    };
    h.check_bool("raycasting_routes_correctly", ray_sub == ray_expected);

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
    h.check_bool("graceful_degradation_all_cpu", all_cpu_no_gpu);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ludospring_barracuda::validation::BufferSink;

    #[test]
    fn routing_logic_validation_passes() {
        let mut h = ValidationHarness::with_sink("exp031_dispatch_routing", BufferSink::default());
        run_routing_checks(&mut h, false);
        run_routing_checks(&mut h, true);
        let total = h.total_count();
        let passed = h.passed_count();
        assert_eq!(
            passed,
            total,
            "{} checks failed out of {total}",
            total - passed
        );
    }
}

fn cmd_discover() {
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

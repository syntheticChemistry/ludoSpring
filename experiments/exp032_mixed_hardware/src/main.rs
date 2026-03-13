// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
//! exp032 — Mixed hardware substrate validation.
//!
//! Validates `PCIe` transfer cost modeling, mixed pipelines (CPU → GPU → CPU),
//! NPU mock substrates, and substrate scoring for cross-system dispatch.
//! Evolving locally for absorption into barraCuda/toadStool metalForge.
//!
//! Subcommands:
//!   validate  — run all mixed hardware checks
//!   pcie      — probe `PCIe` topology from sysfs
//!   demo      — run a mixed pipeline demonstration

use std::process;

use ludospring_barracuda::validation::ValidationResult;

fn main() {
    let arg = std::env::args().nth(1).unwrap_or_default();
    match arg.as_str() {
        "pcie" => cmd_pcie(),
        "demo" => cmd_demo(),
        "validate" | "" => cmd_validate(),
        other => {
            eprintln!("Unknown command: {other}");
            eprintln!("Usage: exp032_mixed_hardware [validate|pcie|demo]");
            process::exit(1);
        }
    }
}

// ---------------------------------------------------------------------------
// Transfer cost model (evolving locally for barraCuda absorption)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq)]
enum BandwidthTier {
    PciE3x16,
    PciE4x16,
    PciE5x16,
    NvLink,
    SharedMemory,
    Unknown,
}

impl BandwidthTier {
    const fn bandwidth_gbps(self) -> f64 {
        match self {
            Self::PciE3x16 => 15.75,
            Self::PciE4x16 => 31.5,
            Self::PciE5x16 => 63.0,
            Self::NvLink => 300.0,
            Self::SharedMemory => 1000.0,
            Self::Unknown => 8.0,
        }
    }

    /// Estimated one-way transfer time in microseconds for `bytes` of data.
    #[expect(
        clippy::cast_precision_loss,
        reason = "validation counts fit in f64 mantissa"
    )]
    fn transfer_time_us(self, bytes: usize) -> f64 {
        let gb = bytes as f64 / 1e9;
        let seconds = gb / self.bandwidth_gbps();
        seconds * 1e6
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SubstrateType {
    Cpu,
    NvidiaGpu,
    AmdGpu,
    IntelGpu,
    Npu,
}

#[allow(dead_code)]
struct SubstrateProfile {
    substrate_type: SubstrateType,
    name: String,
    bandwidth: BandwidthTier,
    supports_f64: bool,
    flops_gflops: f64,
}

struct MixedPipelineStage {
    name: String,
    target: SubstrateType,
    compute_us: f64,
    data_bytes: usize,
}

struct MixedPipelineResult {
    total_us: f64,
    transfer_us: f64,
    compute_us: f64,
    stages: Vec<(String, f64)>,
}

fn run_mixed_pipeline(
    stages: &[MixedPipelineStage],
    _substrates: &[SubstrateProfile],
    bandwidth: BandwidthTier,
) -> MixedPipelineResult {
    let mut total_us = 0.0;
    let mut total_transfer = 0.0;
    let mut total_compute = 0.0;
    let mut stage_timings = Vec::new();
    let mut prev_substrate = SubstrateType::Cpu;

    for stage in stages {
        let mut stage_time = stage.compute_us;

        if stage.target != prev_substrate {
            let transfer = bandwidth.transfer_time_us(stage.data_bytes);
            stage_time += transfer;
            total_transfer += transfer;
        }

        total_compute += stage.compute_us;
        total_us += stage_time;
        stage_timings.push((stage.name.clone(), stage_time));
        prev_substrate = stage.target;
    }

    MixedPipelineResult {
        total_us,
        transfer_us: total_transfer,
        compute_us: total_compute,
        stages: stage_timings,
    }
}

/// Score a substrate for a workload. Higher is better.
fn score_substrate(
    substrate: &SubstrateProfile,
    data_bytes: usize,
    parallel_factor: f64,
    bandwidth: BandwidthTier,
) -> f64 {
    let compute_benefit = substrate.flops_gflops * parallel_factor;
    let transfer_penalty = bandwidth.transfer_time_us(data_bytes) * 0.001;
    compute_benefit - transfer_penalty
}

/// Detect `PCIe` link info from sysfs (best-effort).
struct PcieLinkInfo {
    device: String,
    vendor: String,
    pcie_gen: u8,
    width: u8,
}

fn detect_pcie_links() -> Vec<PcieLinkInfo> {
    let mut links = Vec::new();
    let pci_path = std::path::Path::new("/sys/bus/pci/devices");
    if !pci_path.exists() {
        return links;
    }

    let Ok(entries) = std::fs::read_dir(pci_path) else {
        return links;
    };

    for entry in entries.flatten() {
        let dev_path = entry.path();
        let class_path = dev_path.join("class");
        let Ok(class) = std::fs::read_to_string(&class_path) else {
            continue;
        };
        let class = class.trim();
        // 0x03 = display controller (GPU), 0x12 = processing accelerator (NPU)
        if !class.starts_with("0x03") && !class.starts_with("0x12") {
            continue;
        }

        let vendor = std::fs::read_to_string(dev_path.join("vendor"))
            .unwrap_or_default()
            .trim()
            .to_string();

        let link_speed = std::fs::read_to_string(dev_path.join("current_link_speed"))
            .unwrap_or_default()
            .trim()
            .to_string();
        let pcie_gen = if link_speed.contains("16") {
            4
        } else if link_speed.contains("32") {
            5
        } else if link_speed.contains('8') {
            3
        } else {
            0
        };

        let width = std::fs::read_to_string(dev_path.join("current_link_width"))
            .unwrap_or_default()
            .trim()
            .parse::<u8>()
            .unwrap_or(0);

        links.push(PcieLinkInfo {
            device: entry.file_name().to_string_lossy().to_string(),
            vendor,
            pcie_gen,
            width,
        });
    }

    links
}

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

#[expect(
    clippy::too_many_lines,
    clippy::similar_names,
    reason = "validation orchestrator — sequential check groups"
)]
fn cmd_validate() {
    println!("=== exp032: Mixed Hardware Validation ===\n");

    let experiment = "exp032_mixed_hardware";
    let mut results = Vec::new();

    // 1. Transfer cost model: PCIe 4x16 sanity
    let pcie4_1mb = BandwidthTier::PciE4x16.transfer_time_us(1_000_000);
    results.push(ValidationResult::check(
        experiment,
        "pcie4_1mb_transfer_bounded",
        pcie4_1mb,
        31.75,
        50.0,
    ));

    // 2. NvLink faster than PCIe
    let nvlink_1mb = BandwidthTier::NvLink.transfer_time_us(1_000_000);
    results.push(ValidationResult::check(
        experiment,
        "nvlink_faster_than_pcie",
        if nvlink_1mb < pcie4_1mb { 1.0 } else { 0.0 },
        1.0,
        0.0,
    ));

    // 3. Shared memory fastest
    let shared_1mb = BandwidthTier::SharedMemory.transfer_time_us(1_000_000);
    results.push(ValidationResult::check(
        experiment,
        "shared_mem_fastest",
        if shared_1mb < nvlink_1mb { 1.0 } else { 0.0 },
        1.0,
        0.0,
    ));

    // 4. Mixed pipeline: CPU → GPU → CPU completes
    let stages = vec![
        MixedPipelineStage {
            name: "preprocess".to_string(),
            target: SubstrateType::Cpu,
            compute_us: 100.0,
            data_bytes: 1_000_000,
        },
        MixedPipelineStage {
            name: "gpu_compute".to_string(),
            target: SubstrateType::NvidiaGpu,
            compute_us: 50.0,
            data_bytes: 1_000_000,
        },
        MixedPipelineStage {
            name: "postprocess".to_string(),
            target: SubstrateType::Cpu,
            compute_us: 80.0,
            data_bytes: 1_000_000,
        },
    ];
    let result = run_mixed_pipeline(&stages, &[], BandwidthTier::PciE4x16);
    results.push(ValidationResult::check(
        experiment,
        "mixed_pipeline_completes",
        if result.total_us > 0.0 { 1.0 } else { 0.0 },
        1.0,
        0.0,
    ));

    // 5. Transfer cost included in pipeline
    results.push(ValidationResult::check(
        experiment,
        "transfer_cost_nonzero",
        if result.transfer_us > 0.0 { 1.0 } else { 0.0 },
        1.0,
        0.0,
    ));

    // 6. Pipeline total > compute alone
    results.push(ValidationResult::check(
        experiment,
        "total_exceeds_compute",
        if result.total_us > result.compute_us {
            1.0
        } else {
            0.0
        },
        1.0,
        0.0,
    ));

    // 7. GPU substrate scores higher for parallel work
    let gpu_profile = SubstrateProfile {
        substrate_type: SubstrateType::NvidiaGpu,
        name: "RTX 4070".to_string(),
        bandwidth: BandwidthTier::PciE4x16,
        supports_f64: true,
        flops_gflops: 29_000.0,
    };
    let cpu_profile = SubstrateProfile {
        substrate_type: SubstrateType::Cpu,
        name: "Ryzen 9".to_string(),
        bandwidth: BandwidthTier::SharedMemory,
        supports_f64: true,
        flops_gflops: 200.0,
    };
    let large_parallel_score_gpu =
        score_substrate(&gpu_profile, 10_000_000, 1000.0, BandwidthTier::PciE4x16);
    let large_parallel_score_cpu = score_substrate(
        &cpu_profile,
        10_000_000,
        1000.0,
        BandwidthTier::SharedMemory,
    );
    results.push(ValidationResult::check(
        experiment,
        "gpu_preferred_large_parallel",
        if large_parallel_score_gpu > large_parallel_score_cpu {
            1.0
        } else {
            0.0
        },
        1.0,
        0.0,
    ));

    // 8. Transfer cost dominance: massive data + zero parallelism → CPU wins
    let massive_transfer_gpu =
        score_substrate(&gpu_profile, 10_000_000_000, 0.0, BandwidthTier::PciE3x16);
    let zero_transfer_cpu = score_substrate(&cpu_profile, 0, 0.0, BandwidthTier::SharedMemory);
    let transfer_dominates = zero_transfer_cpu > massive_transfer_gpu;
    results.push(ValidationResult::check(
        experiment,
        "transfer_cost_dominance",
        if transfer_dominates { 1.0 } else { 0.0 },
        1.0,
        0.0,
    ));

    // 9. NPU mock substrate scoring for inference
    let npu_profile = SubstrateProfile {
        substrate_type: SubstrateType::Npu,
        name: "BrainChip AKD1000".to_string(),
        bandwidth: BandwidthTier::PciE3x16,
        supports_f64: false,
        flops_gflops: 50.0,
    };
    let npu_score = score_substrate(&npu_profile, 1_000, 100.0, BandwidthTier::PciE3x16);
    results.push(ValidationResult::check(
        experiment,
        "npu_mock_scores_positive",
        if npu_score > 0.0 { 1.0 } else { 0.0 },
        1.0,
        0.0,
    ));

    // 10. PCIe link detection (best-effort, pass if no crash)
    let links = detect_pcie_links();
    results.push(ValidationResult::check(
        experiment,
        "pcie_detection_no_crash",
        1.0,
        1.0,
        0.0,
    ));
    println!("  [INFO] Detected {} PCIe GPU/NPU link(s)", links.len());

    // 11. CPU-only pipeline works (no transfers)
    let cpu_only_stages = vec![
        MixedPipelineStage {
            name: "step1".to_string(),
            target: SubstrateType::Cpu,
            compute_us: 50.0,
            data_bytes: 1000,
        },
        MixedPipelineStage {
            name: "step2".to_string(),
            target: SubstrateType::Cpu,
            compute_us: 50.0,
            data_bytes: 1000,
        },
    ];
    let cpu_result = run_mixed_pipeline(&cpu_only_stages, &[], BandwidthTier::SharedMemory);
    results.push(ValidationResult::check(
        experiment,
        "cpu_only_no_transfer",
        cpu_result.transfer_us,
        0.0,
        0.0,
    ));

    // 12. Bandwidth tier ordering is consistent
    let tiers = [
        BandwidthTier::PciE3x16,
        BandwidthTier::PciE4x16,
        BandwidthTier::PciE5x16,
        BandwidthTier::NvLink,
        BandwidthTier::SharedMemory,
    ];
    let monotonic = tiers
        .windows(2)
        .all(|w| w[0].bandwidth_gbps() < w[1].bandwidth_gbps());
    results.push(ValidationResult::check(
        experiment,
        "bandwidth_tier_monotonic",
        if monotonic { 1.0 } else { 0.0 },
        1.0,
        0.0,
    ));

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

fn cmd_pcie() {
    println!("=== exp032: PCIe Topology Probe ===\n");

    let links = detect_pcie_links();
    if links.is_empty() {
        println!("  No GPU/NPU PCIe devices found (or /sys/bus/pci not available).");
        return;
    }

    for link in &links {
        println!("  Device: {}", link.device);
        println!("    Vendor: {}", link.vendor);
        println!("    PCIe Gen: {}", link.pcie_gen);
        println!("    Width: x{}", link.width);
        let bw = match (link.pcie_gen, link.width) {
            (4, 16) => BandwidthTier::PciE4x16,
            (5, 16) => BandwidthTier::PciE5x16,
            (3, 16) => BandwidthTier::PciE3x16,
            _ => BandwidthTier::Unknown,
        };
        println!(
            "    Estimated BW: {:.1} GB/s ({:?})",
            bw.bandwidth_gbps(),
            bw
        );
        println!();
    }
}

fn cmd_demo() {
    println!("=== exp032: Mixed Pipeline Demo ===\n");

    let stages = vec![
        MixedPipelineStage {
            name: "CPU preprocess (noise gen)".to_string(),
            target: SubstrateType::Cpu,
            compute_us: 200.0,
            data_bytes: 4_000_000,
        },
        MixedPipelineStage {
            name: "GPU compute (engagement batch)".to_string(),
            target: SubstrateType::NvidiaGpu,
            compute_us: 30.0,
            data_bytes: 4_000_000,
        },
        MixedPipelineStage {
            name: "NPU inference (player intent)".to_string(),
            target: SubstrateType::Npu,
            compute_us: 10.0,
            data_bytes: 100_000,
        },
        MixedPipelineStage {
            name: "CPU postprocess (report)".to_string(),
            target: SubstrateType::Cpu,
            compute_us: 50.0,
            data_bytes: 100_000,
        },
    ];

    let result = run_mixed_pipeline(&stages, &[], BandwidthTier::PciE4x16);

    println!("  Pipeline stages:");
    for (name, time) in &result.stages {
        println!("    {name}: {time:.1} us");
    }
    println!();
    println!("  Total:     {:.1} us", result.total_us);
    println!("  Compute:   {:.1} us", result.compute_us);
    println!("  Transfer:  {:.1} us", result.transfer_us);
    println!(
        "  Overhead:  {:.1}%",
        result.transfer_us / result.total_us * 100.0
    );
}

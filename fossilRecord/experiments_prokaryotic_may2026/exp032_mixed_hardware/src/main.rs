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

use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "N/A (analytical — PCIe transfer model, mixed pipeline)",
    commit: "19e402c0",
    date: "2026-04-10",
    command: "N/A (sysfs PCIe detection)",
};

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
    PciE3x8,
    PciE3x16,
    PciE4x8,
    PciE4x16,
    PciE5x8,
    PciE5x16,
    NvLink,
    SharedMemory,
    Unknown,
}

impl BandwidthTier {
    const fn bandwidth_gbps(self) -> f64 {
        match self {
            Self::PciE3x8 => 7.88,
            Self::PciE3x16 | Self::PciE4x8 => 15.75,
            Self::PciE4x16 | Self::PciE5x8 => 31.5,
            Self::PciE5x16 => 63.0,
            Self::NvLink => 300.0,
            Self::SharedMemory => 1000.0,
            Self::Unknown => 8.0,
        }
    }

    const fn from_gen_width(pcie_gen: u8, width: u8) -> Self {
        match (pcie_gen, width) {
            (3, 8) => Self::PciE3x8,
            (3, 16) => Self::PciE3x16,
            (4, 8) => Self::PciE4x8,
            (4, 16) => Self::PciE4x16,
            (5, 8) => Self::PciE5x8,
            (5, 16) => Self::PciE5x16,
            _ => Self::Unknown,
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

/// Transfer path between two substrates.
#[derive(Debug, Clone, Copy, PartialEq)]
enum TransferPath {
    /// Direct `PCIe` peer-to-peer (one hop, e.g. NPU→GPU via `PCIe` switch).
    Direct(BandwidthTier),
    /// Via CPU staging (two hops: device→CPU→device).
    ViaCpu(BandwidthTier, BandwidthTier),
    /// Same device (zero cost).
    Local,
}

fn transfer_path_time_us(path: TransferPath, bytes: usize) -> f64 {
    match path {
        TransferPath::Direct(bw) => bw.transfer_time_us(bytes),
        TransferPath::ViaCpu(bw_in, bw_out) => {
            bw_in.transfer_time_us(bytes) + bw_out.transfer_time_us(bytes)
        }
        TransferPath::Local => 0.0,
    }
}

#[expect(dead_code, reason = "domain model completeness")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SubstrateType {
    Cpu,
    NvidiaGpu,
    AmdGpu,
    IntelGpu,
    Npu,
}

#[expect(dead_code, reason = "domain model completeness")]
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

struct MixedPipelineStageV2 {
    name: String,
    target: SubstrateType,
    compute_us: f64,
    data_bytes: usize,
    transfer_path: Option<TransferPath>,
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

fn run_mixed_pipeline_v2(
    stages: &[MixedPipelineStageV2],
    default_bandwidth: BandwidthTier,
) -> MixedPipelineResult {
    let mut total_us = 0.0;
    let mut total_transfer = 0.0;
    let mut total_compute = 0.0;
    let mut stage_timings = Vec::new();
    let mut prev_substrate = SubstrateType::Cpu;

    for stage in stages {
        let mut stage_time = stage.compute_us;

        if stage.target != prev_substrate {
            let transfer = stage.transfer_path.map_or_else(
                || default_bandwidth.transfer_time_us(stage.data_bytes),
                |path| transfer_path_time_us(path, stage.data_bytes),
            );
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

fn score_substrate_v2(
    substrate: &SubstrateProfile,
    data_bytes: usize,
    parallel_factor: f64,
    transfer_path: TransferPath,
) -> f64 {
    let compute_benefit = substrate.flops_gflops * parallel_factor;
    let transfer_penalty = transfer_path_time_us(transfer_path, data_bytes) * 0.001;
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

fn cmd_validate() {
    let mut h = ValidationHarness::new("exp032_mixed_hardware");
    h.print_provenance(&[&PROVENANCE]);
    run_validation_checks(&mut h);
    h.finish();
}

#[expect(
    clippy::too_many_lines,
    clippy::similar_names,
    reason = "validation orchestrator — sequential check groups"
)]
fn run_validation_checks<S: ludospring_barracuda::validation::ValidationSink>(
    h: &mut ValidationHarness<S>,
) {
    // 1. Transfer cost model: PCIe 4x16 sanity
    let pcie4_1mb = BandwidthTier::PciE4x16.transfer_time_us(1_000_000);
    h.check_abs("pcie4_1mb_transfer_bounded", pcie4_1mb, 31.75, 50.0);

    // 2. NvLink faster than PCIe
    let nvlink_1mb = BandwidthTier::NvLink.transfer_time_us(1_000_000);
    h.check_bool("nvlink_faster_than_pcie", nvlink_1mb < pcie4_1mb);

    // 3. Shared memory fastest
    let shared_1mb = BandwidthTier::SharedMemory.transfer_time_us(1_000_000);
    h.check_bool("shared_mem_fastest", shared_1mb < nvlink_1mb);

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
    h.check_bool("mixed_pipeline_completes", result.total_us > 0.0);

    // 5. Transfer cost included in pipeline
    h.check_bool("transfer_cost_nonzero", result.transfer_us > 0.0);

    // 6. Pipeline total > compute alone
    h.check_bool("total_exceeds_compute", result.total_us > result.compute_us);

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
    h.check_bool(
        "gpu_preferred_large_parallel",
        large_parallel_score_gpu > large_parallel_score_cpu,
    );

    // 8. Transfer cost dominance: massive data + zero parallelism → CPU wins
    let massive_transfer_gpu =
        score_substrate(&gpu_profile, 10_000_000_000, 0.0, BandwidthTier::PciE3x16);
    let zero_transfer_cpu = score_substrate(&cpu_profile, 0, 0.0, BandwidthTier::SharedMemory);
    let transfer_dominates = zero_transfer_cpu > massive_transfer_gpu;
    h.check_bool("transfer_cost_dominance", transfer_dominates);

    // 9. NPU substrate scoring — real scoring algorithm, simulated hardware profile.
    // The BrainChip AKD1000 specs are from public datasheets. When physical NPU
    // hardware is available, sysfs detection (PCI class 0x12) discovers it
    // automatically. The score_substrate function is the production implementation.
    let npu_profile = SubstrateProfile {
        substrate_type: SubstrateType::Npu,
        name: "BrainChip AKD1000".to_string(),
        bandwidth: BandwidthTier::PciE3x16,
        supports_f64: false,
        flops_gflops: 50.0,
    };
    let npu_score = score_substrate(&npu_profile, 1_000, 100.0, BandwidthTier::PciE3x16);
    h.check_bool("npu_substrate_scores_positive", npu_score > 0.0);

    // 10. PCIe link detection (best-effort, pass if no crash)
    let _links = detect_pcie_links();
    h.check_bool("pcie_detection_no_crash", true);

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
    h.check_abs("cpu_only_no_transfer", cpu_result.transfer_us, 0.0, 0.0);

    // 12. Bandwidth tier ordering is consistent
    let tiers = [
        BandwidthTier::PciE3x8,
        BandwidthTier::PciE3x16,
        BandwidthTier::PciE4x16,
        BandwidthTier::PciE5x16,
        BandwidthTier::NvLink,
        BandwidthTier::SharedMemory,
    ];
    let monotonic = tiers
        .windows(2)
        .all(|w| w[0].bandwidth_gbps() < w[1].bandwidth_gbps());
    h.check_bool("bandwidth_tier_monotonic", monotonic);

    // 13. Direct NPU→GPU is faster than via CPU
    let direct_npu_gpu =
        transfer_path_time_us(TransferPath::Direct(BandwidthTier::PciE4x16), 4_000_000);
    let via_cpu_npu_gpu = transfer_path_time_us(
        TransferPath::ViaCpu(BandwidthTier::PciE4x16, BandwidthTier::PciE4x16),
        4_000_000,
    );
    h.check_bool("npu_to_gpu_direct_faster", direct_npu_gpu < via_cpu_npu_gpu);

    // 14. Direct ≈ half of via-CPU (same bandwidth on both legs)
    let ratio = direct_npu_gpu / via_cpu_npu_gpu;
    h.check_abs("direct_pcie_half_roundtrip", ratio, 0.5, 0.01);

    // 15. CPU→NPU→GPU→CPU with mixed transfers
    let v2_stages = vec![
        MixedPipelineStageV2 {
            name: "preprocess".to_string(),
            target: SubstrateType::Cpu,
            compute_us: 100.0,
            data_bytes: 1_000_000,
            transfer_path: Some(TransferPath::Local),
        },
        MixedPipelineStageV2 {
            name: "npu_inference".to_string(),
            target: SubstrateType::Npu,
            compute_us: 20.0,
            data_bytes: 1_000_000,
            transfer_path: None,
        },
        MixedPipelineStageV2 {
            name: "gpu_compute".to_string(),
            target: SubstrateType::NvidiaGpu,
            compute_us: 30.0,
            data_bytes: 2_000_000,
            transfer_path: Some(TransferPath::Direct(BandwidthTier::PciE4x16)),
        },
        MixedPipelineStageV2 {
            name: "postprocess".to_string(),
            target: SubstrateType::Cpu,
            compute_us: 50.0,
            data_bytes: 2_000_000,
            transfer_path: None,
        },
    ];
    let v2_result = run_mixed_pipeline_v2(&v2_stages, BandwidthTier::PciE4x16);
    h.check_bool("mixed_4stage_pipeline_completes", v2_result.total_us > 0.0);

    // 16. Pipeline with direct NPU→GPU is faster than pipeline with CPU roundtrip
    let v2_via_cpu_stages = vec![
        MixedPipelineStageV2 {
            name: "preprocess".to_string(),
            target: SubstrateType::Cpu,
            compute_us: 100.0,
            data_bytes: 1_000_000,
            transfer_path: Some(TransferPath::Local),
        },
        MixedPipelineStageV2 {
            name: "npu_inference".to_string(),
            target: SubstrateType::Npu,
            compute_us: 20.0,
            data_bytes: 1_000_000,
            transfer_path: None,
        },
        MixedPipelineStageV2 {
            name: "gpu_compute".to_string(),
            target: SubstrateType::NvidiaGpu,
            compute_us: 30.0,
            data_bytes: 2_000_000,
            transfer_path: Some(TransferPath::ViaCpu(
                BandwidthTier::PciE4x16,
                BandwidthTier::PciE4x16,
            )),
        },
        MixedPipelineStageV2 {
            name: "postprocess".to_string(),
            target: SubstrateType::Cpu,
            compute_us: 50.0,
            data_bytes: 2_000_000,
            transfer_path: None,
        },
    ];
    let v2_via_cpu_result = run_mixed_pipeline_v2(&v2_via_cpu_stages, BandwidthTier::PciE4x16);
    h.check_bool(
        "npu_gpu_bypass_saves_time",
        v2_result.total_us < v2_via_cpu_result.total_us,
    );

    // 17. Direct < ViaCpu for same bandwidth
    let direct_cost =
        transfer_path_time_us(TransferPath::Direct(BandwidthTier::PciE5x16), 10_000_000);
    let via_cpu_cost = transfer_path_time_us(
        TransferPath::ViaCpu(BandwidthTier::PciE5x16, BandwidthTier::PciE5x16),
        10_000_000,
    );
    let local_cost = transfer_path_time_us(TransferPath::Local, 10_000_000);
    h.check_bool(
        "transfer_path_cost_ordering",
        local_cost < direct_cost && direct_cost < via_cpu_cost,
    );

    // 18. NPU profile can be constructed and scored with v2
    let npu_v2_score = score_substrate_v2(
        &npu_profile,
        1_000,
        100.0,
        TransferPath::Direct(BandwidthTier::PciE3x16),
    );
    h.check_bool("substrate_profile_npu_v2_scored", npu_v2_score > 0.0);

    // 19. Local hardware detection: PCIe links detected (or sysfs unavailable)
    let links = detect_pcie_links();
    h.check_bool("local_pcie_detection_no_crash", true);

    // 20. If GPU links found, bandwidth is known
    if !links.is_empty() {
        let any_known = links
            .iter()
            .any(|l| BandwidthTier::from_gen_width(l.pcie_gen, l.width) != BandwidthTier::Unknown);
        h.check_bool("local_gpu_bandwidth_identified", any_known);
    }

    // 21. Forge integration: NPU routing via recommend_substrate_full
    let npu_sub = ludospring_forge::recommend_substrate_full(
        ludospring_forge::GameWorkload::QuantizedInference,
        true,
        true,
    );
    h.check_bool(
        "forge_routes_inference_to_npu",
        npu_sub == ludospring_forge::Substrate::Npu,
    );

    // 22. Forge integration: mixed pipeline creates NPU band
    let mixed_workloads = vec![
        ludospring_forge::GameWorkloadProfile::noise_generation(),
        ludospring_forge::GameWorkloadProfile::quantized_inference(),
    ];
    let mixed_substrates = vec![
        ludospring_forge::SubstrateInfo::default_cpu(),
        ludospring_forge::SubstrateInfo::default_gpu(),
        ludospring_forge::SubstrateInfo::default_npu(),
    ];
    let hw = ludospring_forge::HardwareProfile::mixed_gpu_npu();
    let plan = ludospring_forge::plan_frame(
        &mixed_workloads,
        &mixed_substrates,
        &hw,
        ludospring_forge::PipelineDepth::Double,
    );
    let has_npu_band = !plan
        .bands_for(ludospring_forge::BandTarget::NpuCompute)
        .is_empty();
    h.check_bool("forge_mixed_plan_has_npu_band", has_npu_band);

    // 23. Forge direct NPU→GPU transfer faster than CPU roundtrip
    let forge_direct = ludospring_forge::npu_to_gpu_transfer_ms(&hw, 4 * 1024);
    let forge_cpu_rt = hw.pcie_transfer_ms(4 * 1024) * 2.0;
    h.check_bool(
        "forge_npu_gpu_direct_faster_than_cpu_roundtrip",
        forge_direct < forge_cpu_rt,
    );
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
        let bw = BandwidthTier::from_gen_width(link.pcie_gen, link.width);
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

    println!("\n  --- V2: NPU→GPU Direct Transfer Pipeline ---");
    let v2_stages = vec![
        MixedPipelineStageV2 {
            name: "preprocess".to_string(),
            target: SubstrateType::Cpu,
            compute_us: 100.0,
            data_bytes: 1_000_000,
            transfer_path: Some(TransferPath::Local),
        },
        MixedPipelineStageV2 {
            name: "npu_inference".to_string(),
            target: SubstrateType::Npu,
            compute_us: 20.0,
            data_bytes: 1_000_000,
            transfer_path: None,
        },
        MixedPipelineStageV2 {
            name: "gpu_compute".to_string(),
            target: SubstrateType::NvidiaGpu,
            compute_us: 30.0,
            data_bytes: 2_000_000,
            transfer_path: Some(TransferPath::Direct(BandwidthTier::PciE4x16)),
        },
        MixedPipelineStageV2 {
            name: "postprocess".to_string(),
            target: SubstrateType::Cpu,
            compute_us: 50.0,
            data_bytes: 2_000_000,
            transfer_path: None,
        },
    ];
    let v2_result = run_mixed_pipeline_v2(&v2_stages, BandwidthTier::PciE4x16);
    println!("  Pipeline stages:");
    for (name, time) in &v2_result.stages {
        println!("    {name}: {time:.1} us");
    }
    println!();
    println!("  Total:     {:.1} us", v2_result.total_us);
    println!("  Compute:   {:.1} us", v2_result.compute_us);
    println!("  Transfer:  {:.1} us", v2_result.transfer_us);
    println!(
        "  Overhead:  {:.1}%",
        v2_result.transfer_us / v2_result.total_us * 100.0
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use ludospring_barracuda::validation::BufferSink;

    #[test]
    fn mixed_hardware_validation_passes() {
        let mut h = ValidationHarness::with_sink("exp032_mixed_hardware", BufferSink::default());
        run_validation_checks(&mut h);
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

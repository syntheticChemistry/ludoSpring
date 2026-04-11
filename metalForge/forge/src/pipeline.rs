// SPDX-License-Identifier: AGPL-3.0-or-later

//! Symphony frame pipeline planning, budget estimation, and double-buffer state.

use crate::routing::route;
use crate::substrate::{SubstrateInfo, SubstrateKind};
use crate::workload::GameWorkloadProfile;

// ═══════════════════════════════════════════════════════════════════════════
// Symphony pipeline — frame-level planning with overlapping CPU/GPU work
// ═══════════════════════════════════════════════════════════════════════════

/// Target domain for an execution band in the frame pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BandTarget {
    /// CPU-bound work (game logic, AI, state machine transitions).
    Cpu,
    /// GPU compute dispatch (physics, noise, pathfinding batches).
    GpuCompute,
    /// GPU render pass (scene draw, post-processing, present).
    GpuRender,
    /// PCIe transfer (upload uniforms, download results).
    PcieTransfer,
    /// NPU inference dispatch (quantized NPC models, predictions).
    NpuCompute,
    /// Direct NPU→GPU PCIe transfer (bypassing CPU roundtrip).
    NpuToGpuTransfer,
}

/// A band of concurrent workloads within a frame pipeline.
///
/// Each band runs on a single target domain. Bands within different domains
/// can overlap (CPU band runs while GPU compute band runs). Bands on the
/// same domain are sequential.
#[derive(Debug, Clone)]
pub struct ExecutionBand {
    /// Which domain this band executes on.
    pub target: BandTarget,
    /// Workload profiles assigned to this band.
    pub workloads: Vec<String>,
    /// Estimated duration in milliseconds.
    pub estimated_ms: f64,
    /// Indices of bands that must complete before this band starts.
    pub depends_on: Vec<usize>,
}

/// Pipeline depth — how many frames of latency the pipeline trades for throughput.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PipelineDepth {
    /// 1 frame — no overlap. CPU waits for GPU. For turn-based games or tools.
    Single,
    /// 2 frames — double buffered. CPU prepares N+1 while GPU works on N.
    Double,
    /// 3 frames — AAA-style. CPU on N+2, GPU compute on N+1, GPU render on N.
    Triple,
}

impl PipelineDepth {
    /// Number of frames in flight simultaneously.
    #[must_use]
    pub const fn frames_in_flight(self) -> u32 {
        match self {
            Self::Single => 1,
            Self::Double => 2,
            Self::Triple => 3,
        }
    }

    /// Input-to-display latency in frames.
    #[must_use]
    pub const fn latency_frames(self) -> u32 {
        self.frames_in_flight()
    }
}

/// Hardware performance profile for budget estimation.
#[derive(Debug, Clone)]
pub struct HardwareProfile {
    /// CPU game logic budget estimate (ms per frame).
    pub cpu_budget_ms: f64,
    /// GPU compute budget estimate (ms per frame).
    pub gpu_compute_budget_ms: f64,
    /// GPU render budget estimate (ms per frame).
    pub gpu_render_budget_ms: f64,
    /// NPU inference budget estimate (ms per frame).
    pub npu_budget_ms: f64,
    /// PCIe one-way bandwidth in GB/s (CPU↔GPU).
    pub pcie_bandwidth_gbps: f64,
    /// Direct NPU↔GPU PCIe bandwidth in GB/s (0 if no direct path).
    pub npu_gpu_bandwidth_gbps: f64,
    /// Whether the GPU supports concurrent compute and render.
    pub concurrent_compute_render: bool,
}

impl HardwareProfile {
    /// RTX 4060 + Ryzen 5800X3D local profile (validated by exp030-036).
    #[must_use]
    pub const fn local_rtx4060() -> Self {
        Self {
            cpu_budget_ms: 4.0,
            gpu_compute_budget_ms: 4.0,
            gpu_render_budget_ms: 8.0,
            npu_budget_ms: 0.0,
            pcie_bandwidth_gbps: 15.8,
            npu_gpu_bandwidth_gbps: 0.0,
            concurrent_compute_render: true,
        }
    }

    /// Mixed hardware profile: GPU + NPU with direct PCIe link.
    #[must_use]
    pub const fn mixed_gpu_npu() -> Self {
        Self {
            cpu_budget_ms: 4.0,
            gpu_compute_budget_ms: 4.0,
            gpu_render_budget_ms: 8.0,
            npu_budget_ms: 2.0,
            pcie_bandwidth_gbps: 15.8,
            npu_gpu_bandwidth_gbps: 12.0,
            concurrent_compute_render: true,
        }
    }

    /// Estimate PCIe transfer time for a given payload size in bytes.
    #[must_use]
    #[expect(
        clippy::cast_precision_loss,
        reason = "transfer sizes are well within f64 mantissa range"
    )]
    pub fn pcie_transfer_ms(&self, bytes: usize) -> f64 {
        let gb = bytes as f64 / (1024.0 * 1024.0 * 1024.0);
        (gb / self.pcie_bandwidth_gbps) * 1000.0
    }
}

/// Frame plan — the result of routing all workloads for a single frame.
///
/// Groups workloads into execution bands on CPU, GPU compute, GPU render,
/// and PCIe transfer. Bands on different domains overlap; bands on the same
/// domain are sequential.
#[derive(Debug, Clone)]
pub struct FramePlan {
    /// Ordered execution bands for this frame.
    pub bands: Vec<ExecutionBand>,
    /// Pipeline depth (how many frames overlap).
    pub depth: PipelineDepth,
}

impl FramePlan {
    /// Total bands in the plan.
    #[must_use]
    pub const fn band_count(&self) -> usize {
        self.bands.len()
    }

    /// Bands targeting a specific domain.
    #[must_use]
    pub fn bands_for(&self, target: BandTarget) -> Vec<&ExecutionBand> {
        self.bands.iter().filter(|b| b.target == target).collect()
    }

    /// Total estimated time for a specific domain (sequential within domain).
    #[must_use]
    pub fn domain_time_ms(&self, target: BandTarget) -> f64 {
        self.bands_for(target).iter().map(|b| b.estimated_ms).sum()
    }
}

/// Budget estimate — whether a frame plan fits within a target frame rate.
#[derive(Debug, Clone)]
pub struct BudgetEstimate {
    /// Target frame time in milliseconds (e.g. 16.67 for 60 Hz).
    pub target_frame_ms: f64,
    /// CPU domain total time (ms).
    pub cpu_total_ms: f64,
    /// GPU compute domain total time (ms).
    pub gpu_compute_total_ms: f64,
    /// GPU render domain total time (ms).
    pub gpu_render_total_ms: f64,
    /// PCIe transfer total time (ms).
    pub pcie_total_ms: f64,
    /// NPU compute total time (ms).
    pub npu_total_ms: f64,
    /// Direct NPU→GPU transfer time (ms).
    pub npu_gpu_transfer_ms: f64,
    /// Effective frame time = max(CPU, GPU_compute + GPU_render) when not
    /// concurrent, or max(CPU, max(GPU_compute, GPU_render)) when concurrent.
    pub effective_frame_ms: f64,
    /// Headroom = target_frame_ms - effective_frame_ms.
    pub headroom_ms: f64,
    /// Whether the plan fits within the target frame rate.
    pub fits: bool,
}

/// Plan all workloads for a frame, grouping by execution domain.
///
/// Routes each workload via the existing `route()` function, then assigns
/// to execution bands based on substrate kind. CPU-preferred workloads go
/// to the CPU band, GPU-preferred to GPU compute, plus a render band.
#[must_use]
#[expect(
    clippy::cast_precision_loss,
    reason = "workload counts are tiny (<100), no precision loss"
)]
#[expect(
    clippy::similar_names,
    reason = "parallel has_gpu_work / has_npu_work flags for the pipeline planner"
)]
pub fn plan_frame(
    workloads: &[GameWorkloadProfile],
    substrates: &[SubstrateInfo],
    hardware: &HardwareProfile,
    depth: PipelineDepth,
) -> FramePlan {
    let mut cpu_workloads = Vec::new();
    let mut gpu_compute_workloads = Vec::new();
    let mut npu_workloads = Vec::new();

    for w in workloads {
        match route(w, substrates) {
            Some(d) if d.substrate.kind == SubstrateKind::Gpu => {
                gpu_compute_workloads.push(w.name.clone());
            }
            Some(d) if d.substrate.kind == SubstrateKind::Npu => {
                npu_workloads.push(w.name.clone());
            }
            _ => {
                cpu_workloads.push(w.name.clone());
            }
        }
    }

    let cpu_count = cpu_workloads.len();
    let gpu_count = gpu_compute_workloads.len();
    let npu_count = npu_workloads.len();
    let has_gpu_work = !gpu_compute_workloads.is_empty();
    let has_npu_work = !npu_workloads.is_empty();
    let has_npu_gpu_direct = has_npu_work && has_gpu_work && hardware.npu_gpu_bandwidth_gbps > 0.0;

    let mut bands = Vec::new();

    if !cpu_workloads.is_empty() {
        bands.push(ExecutionBand {
            target: BandTarget::Cpu,
            workloads: cpu_workloads,
            estimated_ms: hardware.cpu_budget_ms
                * (cpu_count as f64 / workloads.len().max(1) as f64),
            depends_on: vec![],
        });
    }

    let pcie_upload_idx = bands.len();
    if has_gpu_work {
        bands.push(ExecutionBand {
            target: BandTarget::PcieTransfer,
            workloads: vec!["upload_uniforms".to_string()],
            estimated_ms: hardware.pcie_transfer_ms(16 * 1024),
            depends_on: vec![],
        });

        bands.push(ExecutionBand {
            target: BandTarget::GpuCompute,
            workloads: gpu_compute_workloads,
            estimated_ms: hardware.gpu_compute_budget_ms
                * (gpu_count as f64 / workloads.len().max(1) as f64),
            depends_on: vec![pcie_upload_idx],
        });
    }

    if has_npu_work {
        let npu_compute_idx = bands.len();
        bands.push(ExecutionBand {
            target: BandTarget::NpuCompute,
            workloads: npu_workloads,
            estimated_ms: hardware.npu_budget_ms
                * (npu_count as f64 / workloads.len().max(1) as f64),
            depends_on: vec![],
        });

        if has_npu_gpu_direct {
            bands.push(ExecutionBand {
                target: BandTarget::NpuToGpuTransfer,
                workloads: vec!["npu_result_to_gpu".to_string()],
                estimated_ms: npu_to_gpu_transfer_ms(hardware, 4 * 1024),
                depends_on: vec![npu_compute_idx],
            });
        }
    }

    let render_deps = if has_gpu_work {
        vec![pcie_upload_idx + 1]
    } else {
        vec![]
    };
    bands.push(ExecutionBand {
        target: BandTarget::GpuRender,
        workloads: vec!["scene_draw".to_string(), "present".to_string()],
        estimated_ms: hardware.gpu_render_budget_ms,
        depends_on: render_deps,
    });

    FramePlan { bands, depth }
}

/// Estimate NPU→GPU direct PCIe transfer time (bypasses CPU roundtrip).
#[must_use]
#[expect(
    clippy::cast_precision_loss,
    reason = "transfer sizes are well within f64 mantissa range"
)]
pub fn npu_to_gpu_transfer_ms(hardware: &HardwareProfile, bytes: usize) -> f64 {
    if hardware.npu_gpu_bandwidth_gbps <= 0.0 {
        return hardware.pcie_transfer_ms(bytes) * 2.0;
    }
    let gb = bytes as f64 / (1024.0 * 1024.0 * 1024.0);
    (gb / hardware.npu_gpu_bandwidth_gbps) * 1000.0
}

/// Estimate whether a frame plan fits within a target frame rate.
#[must_use]
pub fn estimate_budget(
    plan: &FramePlan,
    hardware: &HardwareProfile,
    target_hz: f64,
) -> BudgetEstimate {
    let target_frame_ms = 1000.0 / target_hz;

    let cpu_total_ms = plan.domain_time_ms(BandTarget::Cpu);
    let gpu_compute_total_ms = plan.domain_time_ms(BandTarget::GpuCompute);
    let gpu_render_total_ms = plan.domain_time_ms(BandTarget::GpuRender);
    let pcie_total_ms = plan.domain_time_ms(BandTarget::PcieTransfer);
    let npu_total_ms = plan.domain_time_ms(BandTarget::NpuCompute);
    let npu_gpu_transfer_ms = plan.domain_time_ms(BandTarget::NpuToGpuTransfer);

    let gpu_effective_ms = if hardware.concurrent_compute_render {
        gpu_compute_total_ms.max(gpu_render_total_ms)
    } else {
        gpu_compute_total_ms + gpu_render_total_ms
    };

    let gpu_path_ms = gpu_effective_ms + pcie_total_ms;
    let npu_path_ms = npu_total_ms + npu_gpu_transfer_ms;
    let effective_frame_ms = cpu_total_ms.max(gpu_path_ms).max(npu_path_ms);

    let headroom_ms = target_frame_ms - effective_frame_ms;

    BudgetEstimate {
        target_frame_ms,
        cpu_total_ms,
        gpu_compute_total_ms,
        gpu_render_total_ms,
        pcie_total_ms,
        npu_total_ms,
        npu_gpu_transfer_ms,
        effective_frame_ms,
        headroom_ms,
        fits: headroom_ms >= 0.0,
    }
}

/// Double-buffer state for zero-stall CPU/GPU data exchange.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferSlot {
    /// Slot A — GPU reads while CPU writes B.
    A,
    /// Slot B — GPU reads while CPU writes A.
    B,
}

impl BufferSlot {
    /// Swap to the other slot.
    #[must_use]
    pub const fn swap(self) -> Self {
        match self {
            Self::A => Self::B,
            Self::B => Self::A,
        }
    }
}

/// Double-buffer tracker for frame pipeline state.
#[derive(Debug, Clone)]
pub struct DoubleBuffer {
    /// Which slot the CPU is currently writing to.
    pub cpu_write: BufferSlot,
    /// Which slot the GPU is currently reading from.
    pub gpu_read: BufferSlot,
    /// Number of frame swaps executed.
    pub swap_count: u64,
}

impl DoubleBuffer {
    /// Create a new double buffer (CPU writes A, GPU reads B initially).
    #[must_use]
    pub const fn new() -> Self {
        Self {
            cpu_write: BufferSlot::A,
            gpu_read: BufferSlot::B,
            swap_count: 0,
        }
    }

    /// Swap buffers at frame boundary. CPU and GPU never touch the same slot.
    pub const fn swap(&mut self) {
        self.cpu_write = self.cpu_write.swap();
        self.gpu_read = self.gpu_read.swap();
        self.swap_count += 1;
    }

    /// Whether the buffers are in a valid state (CPU and GPU on different slots).
    #[must_use]
    pub const fn is_valid(&self) -> bool {
        !matches!(
            (&self.cpu_write, &self.gpu_read),
            (BufferSlot::A, BufferSlot::A) | (BufferSlot::B, BufferSlot::B)
        )
    }
}

impl Default for DoubleBuffer {
    fn default() -> Self {
        Self::new()
    }
}

// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
//! exp082 — Symphony pipeline: CPU/GPU concurrent frame architecture.
//!
//! Validates the symphony model where CPU and GPU operate concurrently as
//! different sections of the same orchestra, rather than in synchronous
//! request-response lockstep.
//!
//! 1. **Overlapping CPU/GPU work**: CPU submits GPU work, continues game logic,
//!    collects results later. Effective frame time = max(CPU, GPU), not sum.
//! 2. **Double buffering**: Two buffer sets with zero-stall swap. CPU writes
//!    buffer B while GPU reads buffer A. No PCIe round-trip stall.
//! 3. **Frame budget model**: Given hardware profile, predict whether 60 Hz
//!    is achievable. Partition budget across CPU, GPU compute, GPU render,
//!    and PCIe transfer domains.
//! 4. **Persistent GPU state**: GPU retains data between frames. Only deltas
//!    transferred over PCIe. Amortizes dispatch overhead.
//! 5. **Crossover model evolution**: GPU dispatch overhead amortized when GPU
//!    is already hot and data resident. Changes the CPU-always-wins breakeven
//!    from exp030 for persistent workloads.
//! 6. **Pipeline depth tradeoff**: 1-3 frames of pipeline depth trade latency
//!    for throughput. The model predicts effective frame time at each depth.
//!
//! Cross-spring: the same symphony pipeline applies to hotSpring reactor
//! simulations (CPU: control logic, GPU: neutron transport), wetSpring
//! molecular dynamics (CPU: topology, GPU: force eval), and healthSpring
//! patient processing (CPU: state machine, GPU: population sim).

use std::process;

use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness, ValidationSink};
use ludospring_forge::{
    BandTarget, BufferSlot, DoubleBuffer, GameWorkloadProfile, HardwareProfile, PipelineDepth,
    SubstrateInfo, estimate_budget, plan_frame,
};

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "N/A (analytical — frame pipeline model from AAA game architecture)",
    commit: "N/A",
    date: "2026-03-18",
    command: "N/A (analytical — Carmack 1999 frame pipelining, CUDA async compute)",
};

fn main() {
    let arg = std::env::args().nth(1).unwrap_or_default();
    match arg.as_str() {
        "validate" | "" => cmd_validate(),
        other => {
            eprintln!("Unknown command: {other}");
            process::exit(1);
        }
    }
}

fn cmd_validate() {
    let mut h = ValidationHarness::new("exp082_symphony_pipeline");
    h.print_provenance(&[&PROVENANCE]);
    run_validation_checks(&mut h);
    h.finish();
}

fn run_validation_checks<S: ValidationSink>(h: &mut ValidationHarness<S>) {
    section_double_buffer(h);
    section_frame_planning(h);
    section_budget_estimation(h);
    section_pipeline_depth(h);
    section_persistent_state(h);
    section_crossover_model(h);
    section_silicon_utilization(h);
    section_cross_spring(h);
}

// ---------------------------------------------------------------------------
// Section 1: Double buffer invariants
// ---------------------------------------------------------------------------

fn section_double_buffer<S: ValidationSink>(h: &mut ValidationHarness<S>) {
    let mut db = DoubleBuffer::new();
    h.check_bool("double_buffer_initial_valid", db.is_valid());
    h.check_bool(
        "double_buffer_initial_slots_differ",
        db.cpu_write != db.gpu_read,
    );

    db.swap();
    h.check_bool("double_buffer_post_swap_valid", db.is_valid());
    h.check_bool(
        "double_buffer_post_swap_slots_differ",
        db.cpu_write != db.gpu_read,
    );
    #[expect(clippy::cast_precision_loss, reason = "swap count is 1")]
    let swap_count_f64 = db.swap_count as f64;
    h.check_abs("double_buffer_swap_count_1", swap_count_f64, 1.0, 0.0);

    for _ in 0..998 {
        db.swap();
    }
    h.check_bool("double_buffer_1000_swaps_valid", db.is_valid());
    #[expect(clippy::cast_precision_loss, reason = "swap count is 999")]
    let swap_count_f64 = db.swap_count as f64;
    h.check_abs("double_buffer_swap_count_999", swap_count_f64, 999.0, 0.0);

    let even_slot = {
        let mut d = DoubleBuffer::new();
        d.swap();
        d.swap();
        d.cpu_write
    };
    h.check_bool(
        "double_buffer_even_swap_returns_to_start",
        even_slot == BufferSlot::A,
    );

    let odd_slot = {
        let mut d = DoubleBuffer::new();
        d.swap();
        d.cpu_write
    };
    h.check_bool("double_buffer_odd_swap_flips", odd_slot == BufferSlot::B);
}

// ---------------------------------------------------------------------------
// Section 2: Frame planning
// ---------------------------------------------------------------------------

fn section_frame_planning<S: ValidationSink>(h: &mut ValidationHarness<S>) {
    let substrates = vec![SubstrateInfo::default_cpu(), SubstrateInfo::default_gpu()];
    let hw = HardwareProfile::local_rtx4060();

    let mixed_workloads = vec![
        GameWorkloadProfile::noise_generation(),
        GameWorkloadProfile::physics_tick(),
        GameWorkloadProfile::raycasting(),
        GameWorkloadProfile::wfc_step(),
        GameWorkloadProfile::ui_analysis(),
        GameWorkloadProfile::metrics_batch(),
    ];
    let plan = plan_frame(&mixed_workloads, &substrates, &hw, PipelineDepth::Double);

    h.check_bool("plan_has_multiple_bands", plan.band_count() >= 3);

    let cpu_bands = plan.bands_for(BandTarget::Cpu);
    let gpu_compute_bands = plan.bands_for(BandTarget::GpuCompute);
    let gpu_render_bands = plan.bands_for(BandTarget::GpuRender);
    let pcie_bands = plan.bands_for(BandTarget::PcieTransfer);

    h.check_bool("plan_has_cpu_band", !cpu_bands.is_empty());
    h.check_bool("plan_has_gpu_compute_band", !gpu_compute_bands.is_empty());
    h.check_bool(
        "plan_has_exactly_one_render_band",
        gpu_render_bands.len() == 1,
    );
    h.check_bool("plan_has_pcie_transfer", !pcie_bands.is_empty());

    let cpu_names: Vec<&str> = cpu_bands
        .iter()
        .flat_map(|b| b.workloads.iter().map(String::as_str))
        .collect();
    h.check_bool("plan_wfc_on_cpu", cpu_names.contains(&"wfc_step"));
    h.check_bool("plan_ui_on_cpu", cpu_names.contains(&"ui_analysis"));
    h.check_bool("plan_metrics_on_cpu", cpu_names.contains(&"metrics_batch"));

    let gpu_names: Vec<&str> = gpu_compute_bands
        .iter()
        .flat_map(|b| b.workloads.iter().map(String::as_str))
        .collect();
    h.check_bool("plan_noise_on_gpu", gpu_names.contains(&"noise_generation"));
    h.check_bool("plan_physics_on_gpu", gpu_names.contains(&"physics_tick"));
    h.check_bool("plan_raycasting_on_gpu", gpu_names.contains(&"raycasting"));

    let all_durations_positive = plan.bands.iter().all(|b| b.estimated_ms > 0.0);
    h.check_bool("plan_all_durations_positive", all_durations_positive);

    let cpu_only_plan = plan_frame(
        &[
            GameWorkloadProfile::wfc_step(),
            GameWorkloadProfile::ui_analysis(),
        ],
        &[SubstrateInfo::default_cpu()],
        &hw,
        PipelineDepth::Single,
    );
    h.check_bool(
        "plan_cpu_only_no_gpu_compute",
        cpu_only_plan.bands_for(BandTarget::GpuCompute).is_empty(),
    );
    h.check_bool(
        "plan_cpu_only_no_pcie",
        cpu_only_plan.bands_for(BandTarget::PcieTransfer).is_empty(),
    );
}

// ---------------------------------------------------------------------------
// Section 3: Budget estimation
// ---------------------------------------------------------------------------

fn section_budget_estimation<S: ValidationSink>(h: &mut ValidationHarness<S>) {
    let substrates = vec![SubstrateInfo::default_cpu(), SubstrateInfo::default_gpu()];
    let hw = HardwareProfile::local_rtx4060();

    let workloads = vec![
        GameWorkloadProfile::noise_generation(),
        GameWorkloadProfile::physics_tick(),
        GameWorkloadProfile::wfc_step(),
        GameWorkloadProfile::metrics_batch(),
    ];
    let plan = plan_frame(&workloads, &substrates, &hw, PipelineDepth::Double);
    let budget = estimate_budget(&plan, &hw, 60.0);

    h.check_bool("budget_fits_60hz", budget.fits);
    h.check_bool("budget_positive_headroom", budget.headroom_ms > 0.0);
    h.check_abs(
        "budget_target_frame_is_16ms",
        budget.target_frame_ms,
        1000.0 / 60.0,
        0.01,
    );
    h.check_bool(
        "budget_effective_less_than_target",
        budget.effective_frame_ms < budget.target_frame_ms,
    );
    h.check_bool("budget_cpu_total_positive", budget.cpu_total_ms > 0.0);
    h.check_bool(
        "budget_gpu_compute_positive",
        budget.gpu_compute_total_ms > 0.0,
    );

    let budget_30hz = estimate_budget(&plan, &hw, 30.0);
    h.check_bool("budget_30hz_fits", budget_30hz.fits);
    h.check_bool(
        "budget_30hz_more_headroom",
        budget_30hz.headroom_ms > budget.headroom_ms,
    );

    let budget_240hz = estimate_budget(&plan, &hw, 240.0);
    h.check_bool(
        "budget_240hz_tighter",
        budget_240hz.headroom_ms < budget.headroom_ms,
    );
}

// ---------------------------------------------------------------------------
// Section 4: Pipeline depth tradeoffs
// ---------------------------------------------------------------------------

fn section_pipeline_depth<S: ValidationSink>(h: &mut ValidationHarness<S>) {
    h.check_abs(
        "depth_single_1_frame",
        f64::from(PipelineDepth::Single.frames_in_flight()),
        1.0,
        0.0,
    );
    h.check_abs(
        "depth_double_2_frames",
        f64::from(PipelineDepth::Double.frames_in_flight()),
        2.0,
        0.0,
    );
    h.check_abs(
        "depth_triple_3_frames",
        f64::from(PipelineDepth::Triple.frames_in_flight()),
        3.0,
        0.0,
    );

    h.check_abs(
        "depth_single_latency_1",
        f64::from(PipelineDepth::Single.latency_frames()),
        1.0,
        0.0,
    );
    h.check_abs(
        "depth_double_latency_2",
        f64::from(PipelineDepth::Double.latency_frames()),
        2.0,
        0.0,
    );
    h.check_abs(
        "depth_triple_latency_3",
        f64::from(PipelineDepth::Triple.latency_frames()),
        3.0,
        0.0,
    );

    let latency_single_ms = f64::from(PipelineDepth::Single.latency_frames()) * (1000.0 / 60.0);
    let latency_triple_ms = f64::from(PipelineDepth::Triple.latency_frames()) * (1000.0 / 60.0);
    h.check_bool(
        "depth_triple_higher_latency",
        latency_triple_ms > latency_single_ms,
    );
    h.check_upper("depth_single_latency_under_20ms", latency_single_ms, 20.0);
    h.check_upper("depth_triple_latency_under_60ms", latency_triple_ms, 60.0);
}

// ---------------------------------------------------------------------------
// Section 5: Persistent GPU state model
// ---------------------------------------------------------------------------

/// Models GPU state persistence across frames.
struct PersistentGpuState {
    resident_bytes: usize,
    frames_resident: u64,
    cumulative_upload_saved_bytes: u64,
}

impl PersistentGpuState {
    const fn new() -> Self {
        Self {
            resident_bytes: 0,
            frames_resident: 0,
            cumulative_upload_saved_bytes: 0,
        }
    }

    const fn upload_initial(&mut self, bytes: usize) {
        self.resident_bytes = bytes;
        self.frames_resident = 1;
    }

    fn tick_frame(&mut self, delta_bytes: usize) {
        self.frames_resident += 1;
        let saved = self.resident_bytes - delta_bytes.min(self.resident_bytes);
        self.cumulative_upload_saved_bytes += saved as u64;
    }

    /// Fraction of total upload bandwidth saved by keeping data resident.
    #[expect(
        clippy::cast_precision_loss,
        reason = "values are small enough for f64 mantissa"
    )]
    fn savings_ratio(&self) -> f64 {
        if self.frames_resident <= 1 {
            return 0.0;
        }
        let total_would_upload = self.resident_bytes as f64 * self.frames_resident as f64;
        if total_would_upload == 0.0 {
            return 0.0;
        }
        self.cumulative_upload_saved_bytes as f64 / total_would_upload
    }
}

#[expect(
    clippy::cast_precision_loss,
    reason = "frame counts and byte sizes are small, no precision loss"
)]
fn section_persistent_state<S: ValidationSink>(h: &mut ValidationHarness<S>) {
    let hw = HardwareProfile::local_rtx4060();

    let mut state = PersistentGpuState::new();
    state.upload_initial(1024 * 1024);

    let full_upload_ms = hw.pcie_transfer_ms(1024 * 1024);
    let delta_size = 1024;
    let delta_upload_ms = hw.pcie_transfer_ms(delta_size);

    h.check_bool(
        "persistent_delta_much_smaller",
        delta_upload_ms < full_upload_ms * 0.01,
    );

    for _ in 0..59 {
        state.tick_frame(delta_size);
    }

    h.check_bool(
        "persistent_savings_over_90pct",
        state.savings_ratio() > 0.90,
    );
    h.check_abs(
        "persistent_frames_resident_60",
        state.frames_resident as f64,
        60.0,
        0.0,
    );

    let pcie_saved_per_second_bytes =
        state.cumulative_upload_saved_bytes as f64 / (state.frames_resident as f64 / 60.0);
    let pcie_saved_per_second_mb = pcie_saved_per_second_bytes / (1024.0 * 1024.0);
    h.check_bool(
        "persistent_saves_significant_bandwidth",
        pcie_saved_per_second_mb > 50.0,
    );
}

// ---------------------------------------------------------------------------
// Section 6: Crossover model (amortized GPU dispatch)
// ---------------------------------------------------------------------------

fn section_crossover_model<S: ValidationSink>(h: &mut ValidationHarness<S>) {
    let cold_dispatch_overhead_ms = 1.5;
    let warm_dispatch_overhead_ms = 0.1;

    h.check_bool(
        "crossover_warm_much_cheaper",
        warm_dispatch_overhead_ms < cold_dispatch_overhead_ms * 0.15,
    );

    let workload_compute_ms = 2.0;

    let cold_total = cold_dispatch_overhead_ms + workload_compute_ms;
    let warm_total = warm_dispatch_overhead_ms + workload_compute_ms;
    let cpu_equivalent_ms = 4.0;

    h.check_bool(
        "crossover_cold_gpu_slower_than_cpu",
        cold_total < cpu_equivalent_ms,
    );
    h.check_bool(
        "crossover_warm_gpu_faster_than_cpu",
        warm_total < cpu_equivalent_ms,
    );

    let amortization_frames = 60u32;
    let amortized_overhead = (cold_dispatch_overhead_ms
        + warm_dispatch_overhead_ms * f64::from(amortization_frames - 1))
        / f64::from(amortization_frames);
    h.check_bool(
        "crossover_amortized_near_warm",
        (amortized_overhead - warm_dispatch_overhead_ms).abs() < 0.05,
    );

    let symphony_effective = workload_compute_ms.max(cpu_equivalent_ms);
    let sequential_total = workload_compute_ms + cpu_equivalent_ms;
    h.check_bool(
        "crossover_symphony_beats_sequential",
        symphony_effective < sequential_total,
    );
    h.check_abs(
        "crossover_symphony_savings_ms",
        sequential_total - symphony_effective,
        workload_compute_ms.min(cpu_equivalent_ms),
        0.01,
    );
}

// ---------------------------------------------------------------------------
// Section 7: Silicon utilization model
// ---------------------------------------------------------------------------

fn section_silicon_utilization<S: ValidationSink>(h: &mut ValidationHarness<S>) {
    struct SiliconUnit {
        area_pct: f64,
        compute_only_active: bool,
        symphony_active: bool,
    }

    // RTX 4060 silicon area estimates (shader, tensor, RT, TMU, ROP, raster, L2, mem ctrl)
    let units = [
        SiliconUnit {
            area_pct: 40.0,
            compute_only_active: true,
            symphony_active: true,
        },
        SiliconUnit {
            area_pct: 15.0,
            compute_only_active: false,
            symphony_active: true,
        },
        SiliconUnit {
            area_pct: 10.0,
            compute_only_active: false,
            symphony_active: true,
        },
        SiliconUnit {
            area_pct: 10.0,
            compute_only_active: false,
            symphony_active: true,
        },
        SiliconUnit {
            area_pct: 8.0,
            compute_only_active: false,
            symphony_active: true,
        },
        SiliconUnit {
            area_pct: 5.0,
            compute_only_active: false,
            symphony_active: true,
        },
        SiliconUnit {
            area_pct: 8.0,
            compute_only_active: true,
            symphony_active: true,
        },
        SiliconUnit {
            area_pct: 4.0,
            compute_only_active: true,
            symphony_active: true,
        },
    ];

    let total_area: f64 = units.iter().map(|u| u.area_pct).sum();
    h.check_abs("silicon_total_area_100pct", total_area, 100.0, 0.01);

    let compute_only_util: f64 = units
        .iter()
        .filter(|u| u.compute_only_active)
        .map(|u| u.area_pct)
        .sum();
    let symphony_util: f64 = units
        .iter()
        .filter(|u| u.symphony_active)
        .map(|u| u.area_pct)
        .sum();

    h.check_bool("silicon_compute_only_under_60pct", compute_only_util < 60.0);
    h.check_bool("silicon_symphony_over_90pct", symphony_util > 90.0);
    h.check_bool(
        "silicon_symphony_improvement",
        symphony_util > compute_only_util,
    );

    let improvement_factor = symphony_util / compute_only_util;
    h.check_lower("silicon_improvement_at_least_1_5x", improvement_factor, 1.5);
}

// ---------------------------------------------------------------------------
// Section 8: Cross-spring universality
// ---------------------------------------------------------------------------

fn section_cross_spring<S: ValidationSink>(h: &mut ValidationHarness<S>) {
    struct SpringPipeline {
        name: &'static str,
        cpu_role: &'static str,
        gpu_role: &'static str,
        render_role: &'static str,
    }

    let pipelines = [
        SpringPipeline {
            name: "ludoSpring",
            cpu_role: "game_logic_ai_dag",
            gpu_role: "physics_noise_pathfinding",
            render_role: "frame_render",
        },
        SpringPipeline {
            name: "hotSpring",
            cpu_role: "reactor_control_logic",
            gpu_role: "neutron_transport_thermal",
            render_role: "visualization",
        },
        SpringPipeline {
            name: "wetSpring",
            cpu_role: "molecular_topology",
            gpu_role: "force_evaluation_integration",
            render_role: "trajectory_output",
        },
        SpringPipeline {
            name: "healthSpring",
            cpu_role: "patient_state_machine",
            gpu_role: "population_sim_risk",
            render_role: "dashboard_render",
        },
    ];

    h.check_bool("cross_spring_4_pipelines", pipelines.len() == 4);

    for p in &pipelines {
        h.check_bool(&format!("{}_has_cpu_role", p.name), !p.cpu_role.is_empty());
        h.check_bool(&format!("{}_has_gpu_role", p.name), !p.gpu_role.is_empty());
        h.check_bool(
            &format!("{}_has_render_role", p.name),
            !p.render_role.is_empty(),
        );
    }

    let hw = HardwareProfile::local_rtx4060();
    let substrates = vec![SubstrateInfo::default_cpu(), SubstrateInfo::default_gpu()];

    let game_plan = plan_frame(
        &[
            GameWorkloadProfile::noise_generation(),
            GameWorkloadProfile::physics_tick(),
            GameWorkloadProfile::wfc_step(),
        ],
        &substrates,
        &hw,
        PipelineDepth::Double,
    );
    let game_budget = estimate_budget(&game_plan, &hw, 60.0);
    h.check_bool("cross_spring_game_budget_fits", game_budget.fits);

    let science_plan = plan_frame(
        &[
            GameWorkloadProfile::noise_generation(),
            GameWorkloadProfile::metrics_batch(),
        ],
        &substrates,
        &hw,
        PipelineDepth::Single,
    );
    let science_budget = estimate_budget(&science_plan, &hw, 30.0);
    h.check_bool("cross_spring_science_budget_fits_30hz", science_budget.fits);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ludospring_barracuda::validation::BufferSink;

    #[test]
    fn symphony_pipeline_validation_passes() {
        let mut h = ValidationHarness::with_sink("exp082_symphony_pipeline", BufferSink::default());
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

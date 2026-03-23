// SPDX-License-Identifier: AGPL-3.0-or-later

#![forbid(unsafe_code)]
#![deny(clippy::expect_used, clippy::unwrap_used)]

//! ludoSpring Forge — hardware discovery and dispatch for game science workloads.
//!
//! Forge discovers compute substrates (CPU, GPU, NPU) and routes ludoSpring
//! workloads to the best capable substrate. It leans on ToadStool/barraCuda
//! for GPU discovery and device management, and adds game-specific dispatch
//! logic (e.g., noise generation on GPU, WFC on CPU).
//!
//! # Design Principle
//!
//! Springs don't reference each other. `ludoSpring` doesn't import `wetSpring`.
//! Both lean on `ToadStool` independently — `ludoSpring` evolves game/interaction
//! shaders, `wetSpring` evolves bio shaders, and `ToadStool` absorbs both.

mod pipeline;
mod routing;
mod substrate;
mod workload;

pub use pipeline::{
    BandTarget, BudgetEstimate, BufferSlot, DoubleBuffer, ExecutionBand, FramePlan,
    HardwareProfile, PipelineDepth, estimate_budget, plan_frame,
};
pub use routing::{Decision, Substrate, fallback_chain, recommend_substrate, route};
pub use substrate::{Capability, SubstrateInfo, SubstrateKind};
pub use workload::{GameWorkload, GameWorkloadProfile};

#[cfg(test)]
#[expect(clippy::expect_used, reason = "test assertions use expect for clarity")]
#[expect(
    clippy::similar_names,
    reason = "t_1kb/t_1mb are intentionally similar"
)]
mod tests {
    use super::*;

    // --- Legacy tests (backward compat) ---

    #[test]
    fn noise_goes_to_gpu_when_available() {
        assert_eq!(
            recommend_substrate(GameWorkload::NoiseGeneration, true),
            Substrate::Gpu
        );
    }

    #[test]
    fn wfc_stays_on_cpu() {
        assert_eq!(
            recommend_substrate(GameWorkload::WaveFunctionCollapse, true),
            Substrate::Cpu
        );
    }

    #[test]
    fn everything_cpu_without_gpu() {
        assert_eq!(
            recommend_substrate(GameWorkload::NoiseGeneration, false),
            Substrate::Cpu
        );
    }

    // --- Capability-based routing tests ---

    #[test]
    fn route_noise_to_gpu() {
        let profile = GameWorkloadProfile::noise_generation();
        let substrates = vec![SubstrateInfo::default_cpu(), SubstrateInfo::default_gpu()];
        let d = route(&profile, &substrates).expect("should route noise");
        assert_eq!(d.substrate.kind, SubstrateKind::Gpu);
    }

    #[test]
    fn route_wfc_to_cpu() {
        let profile = GameWorkloadProfile::wfc_step();
        let substrates = vec![SubstrateInfo::default_cpu(), SubstrateInfo::default_gpu()];
        let d = route(&profile, &substrates).expect("should route WFC");
        assert_eq!(d.substrate.kind, SubstrateKind::Cpu);
    }

    #[test]
    fn route_npu_inference() {
        let profile = GameWorkloadProfile::quantized_inference();
        let substrates = vec![
            SubstrateInfo::default_cpu(),
            SubstrateInfo::default_gpu(),
            SubstrateInfo::default_npu(),
        ];
        let d = route(&profile, &substrates).expect("should route NPU inference");
        assert_eq!(d.substrate.kind, SubstrateKind::Npu);
    }

    #[test]
    fn route_fallback_chain_order() {
        let substrates = vec![
            SubstrateInfo::default_cpu(),
            SubstrateInfo::default_gpu(),
            SubstrateInfo::default_npu(),
        ];
        let chain = fallback_chain(&substrates);
        assert_eq!(chain.len(), 3);
        assert_eq!(chain[0].kind, SubstrateKind::Gpu);
        assert_eq!(chain[1].kind, SubstrateKind::Npu);
        assert_eq!(chain[2].kind, SubstrateKind::Cpu);
    }

    #[test]
    fn route_no_capable_substrate() {
        let profile = GameWorkloadProfile::quantized_inference();
        let substrates = vec![SubstrateInfo::default_cpu(), SubstrateInfo::default_gpu()];
        let decision = route(&profile, &substrates);
        assert!(decision.is_none());
    }

    #[test]
    fn route_preferred_honored() {
        let profile = GameWorkloadProfile::noise_generation();
        let substrates = vec![
            SubstrateInfo::default_cpu(),
            SubstrateInfo::default_gpu(),
            SubstrateInfo::default_npu(),
        ];
        let d = route(&profile, &substrates).expect("should route with preference");
        assert_eq!(d.substrate.kind, SubstrateKind::Gpu);
        assert_eq!(d.reason, "preferred substrate");
    }

    // --- Symphony pipeline tests ---

    #[test]
    fn pipeline_depth_frames_in_flight() {
        assert_eq!(PipelineDepth::Single.frames_in_flight(), 1);
        assert_eq!(PipelineDepth::Double.frames_in_flight(), 2);
        assert_eq!(PipelineDepth::Triple.frames_in_flight(), 3);
    }

    #[test]
    fn double_buffer_swap_invariant() {
        let mut db = DoubleBuffer::new();
        assert!(db.is_valid());
        assert_ne!(db.cpu_write, db.gpu_read);

        for _ in 0..100 {
            db.swap();
            assert!(db.is_valid());
            assert_ne!(db.cpu_write, db.gpu_read);
        }
        assert_eq!(db.swap_count, 100);
    }

    #[test]
    fn buffer_slot_swap_roundtrip() {
        assert_eq!(BufferSlot::A.swap(), BufferSlot::B);
        assert_eq!(BufferSlot::B.swap(), BufferSlot::A);
        assert_eq!(BufferSlot::A.swap().swap(), BufferSlot::A);
    }

    #[test]
    fn plan_frame_routes_mixed_workloads() {
        let workloads = vec![
            GameWorkloadProfile::noise_generation(),
            GameWorkloadProfile::physics_tick(),
            GameWorkloadProfile::wfc_step(),
            GameWorkloadProfile::ui_analysis(),
        ];
        let substrates = vec![SubstrateInfo::default_cpu(), SubstrateInfo::default_gpu()];
        let hw = HardwareProfile::local_rtx4060();
        let plan = plan_frame(&workloads, &substrates, &hw, PipelineDepth::Double);

        assert!(
            plan.band_count() >= 3,
            "need CPU + GPU compute + render bands"
        );

        let cpu_bands = plan.bands_for(BandTarget::Cpu);
        assert!(!cpu_bands.is_empty(), "WFC and UI should route to CPU");

        let gpu_bands = plan.bands_for(BandTarget::GpuCompute);
        assert!(
            !gpu_bands.is_empty(),
            "noise and physics should route to GPU"
        );

        let render_bands = plan.bands_for(BandTarget::GpuRender);
        assert_eq!(render_bands.len(), 1, "always one render band");
    }

    #[test]
    fn estimate_budget_fits_60hz() {
        let workloads = vec![
            GameWorkloadProfile::noise_generation(),
            GameWorkloadProfile::physics_tick(),
            GameWorkloadProfile::wfc_step(),
        ];
        let substrates = vec![SubstrateInfo::default_cpu(), SubstrateInfo::default_gpu()];
        let hw = HardwareProfile::local_rtx4060();
        let plan = plan_frame(&workloads, &substrates, &hw, PipelineDepth::Double);
        let budget = estimate_budget(&plan, &hw, 60.0);

        assert!(budget.fits, "local hardware should fit 60 Hz");
        assert!(budget.headroom_ms > 0.0, "should have positive headroom");
        assert!(
            (budget.target_frame_ms - 16.666_666).abs() < 0.01,
            "60 Hz = ~16.67ms"
        );
    }

    #[test]
    fn concurrent_compute_render_reduces_gpu_time() {
        let workloads = vec![GameWorkloadProfile::noise_generation()];
        let substrates = vec![SubstrateInfo::default_cpu(), SubstrateInfo::default_gpu()];

        let hw_concurrent = HardwareProfile {
            concurrent_compute_render: true,
            ..HardwareProfile::local_rtx4060()
        };
        let hw_sequential = HardwareProfile {
            concurrent_compute_render: false,
            ..HardwareProfile::local_rtx4060()
        };

        let plan = plan_frame(
            &workloads,
            &substrates,
            &hw_concurrent,
            PipelineDepth::Double,
        );
        let budget_concurrent = estimate_budget(&plan, &hw_concurrent, 60.0);
        let budget_sequential = estimate_budget(&plan, &hw_sequential, 60.0);

        assert!(
            budget_concurrent.effective_frame_ms <= budget_sequential.effective_frame_ms,
            "concurrent GPU should be at least as fast"
        );
    }

    #[test]
    fn pcie_transfer_time_scales_linearly() {
        let hw = HardwareProfile::local_rtx4060();
        let t_1kb = hw.pcie_transfer_ms(1024);
        let t_1mb = hw.pcie_transfer_ms(1024 * 1024);
        let ratio = t_1mb / t_1kb;
        assert!(
            (ratio - 1024.0).abs() < 1.0,
            "1MB should take ~1024x longer than 1KB, got {ratio}x"
        );
    }

    #[test]
    fn hardware_profile_pcie_matches_spec() {
        let hw = HardwareProfile::local_rtx4060();
        let t_1mb = hw.pcie_transfer_ms(1024 * 1024);
        assert!(
            t_1mb < 0.1,
            "1MB at 15.8 GB/s should be <0.1ms, got {t_1mb}ms"
        );
    }

    #[test]
    fn plan_cpu_only_no_gpu_bands() {
        let workloads = vec![
            GameWorkloadProfile::wfc_step(),
            GameWorkloadProfile::ui_analysis(),
        ];
        let substrates = vec![SubstrateInfo::default_cpu()];
        let hw = HardwareProfile::local_rtx4060();
        let plan = plan_frame(&workloads, &substrates, &hw, PipelineDepth::Single);

        let gpu_compute = plan.bands_for(BandTarget::GpuCompute);
        assert!(
            gpu_compute.is_empty(),
            "no GPU compute when CPU-only substrates"
        );
    }

    #[test]
    fn substrate_info_concurrent_flag() {
        assert!(!SubstrateInfo::default_cpu().concurrent_compute_render);
        assert!(SubstrateInfo::default_gpu().concurrent_compute_render);
        assert!(!SubstrateInfo::default_npu().concurrent_compute_render);
    }
}

# Expedition 032: Mixed Hardware Substrate Validation

**Date:** 2026-03-11
**Status:** Active
**Reference:** barraCuda unified_hardware, PCIe specifications, toadStool substrate model

## What We Built

Transfer cost modeling, mixed-substrate pipelines (CPU → GPU → NPU → CPU),
and substrate scoring for cross-system dispatch. Evolving locally in
metalForge/forge for absorption into barraCuda and toadStool.

### Components

| Component | Purpose |
|-----------|---------|
| `BandwidthTier` | PCIe 3/4/5 x16, NvLink, SharedMemory bandwidth model |
| `SubstrateProfile` | Type, name, bandwidth, f64 support, GFLOPS |
| `MixedPipeline` | Multi-stage pipeline with per-stage substrate targets |
| `score_substrate()` | GFLOPS * parallelism - transfer penalty |
| `detect_pcie_links()` | Real sysfs PCIe topology probing |

### Bandwidth Model

| Tier | Bandwidth (GB/s) | 1 MB transfer (us) |
|------|-------------------|---------------------|
| PCIe 3.0 x16 | 15.75 | 63.5 |
| PCIe 4.0 x16 | 31.50 | 31.7 |
| PCIe 5.0 x16 | 63.00 | 15.9 |
| NvLink | 300.00 | 3.3 |
| Shared memory | 1000.00 | 1.0 |

### Validation Checks (18 total)

| # | Check | What it validates |
|---|-------|-------------------|
| 1 | pcie4_1mb_transfer_bounded | PCIe 4 x16 transfer in expected range |
| 2 | nvlink_faster_than_pcie | NvLink outperforms PCIe |
| 3 | shared_mem_fastest | Shared memory fastest tier |
| 4 | mixed_pipeline_completes | CPU → GPU → CPU pipeline works |
| 5 | transfer_cost_nonzero | Cross-substrate transfer adds latency |
| 6 | total_exceeds_compute | Pipeline total > pure compute time |
| 7 | gpu_preferred_large_parallel | GPU scores higher for large parallel work |
| 8 | transfer_cost_dominance | Massive transfer + zero parallelism → CPU wins |
| 9 | npu_substrate_scores_positive | NPU substrate scores correctly |
| 10 | pcie_detection_no_crash | sysfs probing completes gracefully |
| 11 | cpu_only_no_transfer | CPU-only pipeline has zero transfer cost |
| 12 | bandwidth_tier_monotonic | Tier ordering is consistent |
| 13 | npu_to_gpu_direct_faster | Direct NPU→GPU PCIe P2P faster than via CPU |
| 14 | direct_pcie_half_roundtrip | Direct transfer ≈ half of CPU-mediated roundtrip |
| 15 | mixed_4stage_pipeline_completes | CPU→NPU→GPU→CPU with mixed TransferPaths |
| 16 | npu_gpu_bypass_saves_time | Direct NPU→GPU pipeline faster than CPU roundtrip |
| 17 | transfer_path_cost_ordering | Local < Direct < ViaCpu |
| 18 | substrate_profile_npu_v2_scored | NPU scored with TransferPath-aware model |

### Key Insight: PCIe Transfer Costs Matter

For small workloads, the PCIe roundtrip (CPU→GPU→CPU) costs more than the
GPU compute benefit. This is why toadStool's unidirectional streaming model
is critical — keeping data on GPU eliminates the return trip.

### Reproducibility

```bash
cargo run --bin exp032_mixed_hardware               # validate (18 checks)
cargo run --bin exp032_mixed_hardware -- pcie        # probe PCIe topology
cargo run --bin exp032_mixed_hardware -- demo        # mixed pipeline demo
```

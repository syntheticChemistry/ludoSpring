# Expedition 031: Dispatch Routing with Hardware Discovery

**Date:** 2026-03-11
**Status:** Active
**Reference:** barraCuda dispatch, toadStool substrate model, wgpu adapter API

## What We Built

Real-time hardware discovery via wgpu adapter enumeration, validating that
metalForge/forge dispatch recommendations match actual hardware capabilities.

### Components

| Component | Purpose |
|-----------|---------|
| `discover_substrates()` | wgpu adapter enumeration → SubstrateInfo vec |
| Routing validation | GameWorkload → Substrate recommendation checks |
| Graceful degradation | All workloads route to CPU when no GPU detected |
| Backend identification | Vulkan, GL, Metal backend detection |

### Validation Checks (10 total)

| # | Check | What it validates |
|---|-------|-------------------|
| 1 | adapter_discovery_nonzero | At least one adapter found |
| 2 | noise_routes_correctly | Noise → GPU (if available), CPU (if not) |
| 3 | wfc_always_cpu | WFC constraint propagation always stays on CPU |
| 4 | metrics_always_cpu | Engagement metrics batch stays on CPU |
| 5 | ui_always_cpu | UI analysis stays on CPU |
| 6 | physics_routes_correctly | Physics → GPU (if available), CPU (if not) |
| 7 | raycasting_routes_correctly | Raycasting → GPU (if available), CPU (if not) |
| 8 | graceful_degradation_all_cpu | No GPU → everything routes to CPU |
| 9 | discrete_gpu_detection_consistent | Discrete GPU detection is coherent |
| 10 | backend_identified | At least one backend identified |

### Connection to Upstream

Forge dispatch evolves locally, then absorbs into:
- **barraCuda**: `dispatch::DispatchConfig`, `dispatch::dispatch_for`
- **toadStool**: Substrate discovery and workload routing
- **metalForge** (root): Cross-system hardware characterization

### Reproducibility

```bash
cargo run --bin exp031_dispatch_routing              # validate (10 checks)
cargo run --bin exp031_dispatch_routing -- discover   # enumerate substrates
```

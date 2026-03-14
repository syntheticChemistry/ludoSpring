# Expedition 033: NUCLEUS Atomic Pipeline Simulation

**Date:** 2026-03-11
**Status:** Active
**Reference:** biomeOS nucleus_complete.toml, gaming_niche_deploy.toml

## What We Built

A local simulation of the Tower → Node → Nest NUCLEUS atomic deployment
pattern, proving that ludoSpring game workloads compose correctly through
biomeOS-style coordination.

### NUCLEUS Architecture (from biomeOS graphs)

```
Tower = BearDog + Songbird             (crypto + network)
Node  = Tower + ToadStool              (crypto + network + compute)
Nest  = Node + NestGate                (+ storage/provenance)
NUCLEUS = BearDog + Songbird + ToadStool + NestGate + Squirrel
```

### Simulated Components

| Atomic | Simulation | Capabilities |
|--------|-----------|--------------|
| TowerAtomic | Capability resolution (6 capabilities) | crypto.hash, crypto.sign, discovery.query, game.* |
| NodeAtomic | Tower + compute dispatch via forge | GameWorkload → Substrate routing |
| NestAtomic | Node + provenance recording | ProvenanceRecord per pipeline stage |

### Pipeline Flow

1. **Noise generation** (Node dispatches to GPU)
2. **Engagement analysis** (Node dispatches to CPU)
3. **Physics tick** (Node dispatches to GPU)
4. **Provenance seal** (Nest records all stages)

### Validation Checks (19 total)

| # | Check | What it validates |
|---|-------|-------------------|
| 1 | tower_resolves_game_noise | Tower resolves game.generate_noise capability |
| 2 | tower_resolves_game_engagement | Tower resolves game.engagement capability |
| 3 | node_noise_to_gpu | Node routes noise to GPU substrate |
| 4 | node_metrics_to_cpu | Node routes metrics to CPU substrate |
| 5 | nest_provenance_recorded | Nest records 4 provenance entries |
| 6 | pipeline_produces_result | Pipeline completes with nonzero timing |
| 7 | tower_before_node_enforced | Tower initializes before Node can dispatch |
| 8 | pipeline_stage_timings | Every stage has timing metadata |
| 9 | topology_matches_deploy_graph | Phase order matches gaming_niche_deploy.toml |
| 10 | cpu_only_pipeline_works | Full pipeline works without GPU |
| 11 | dispatch_log_complete | Dispatch log captures all operations |
| 12 | node_routes_via_capability | NodeV2 routes noise to GPU via capability routing |
| 13 | node_npu_routes_quantized | Quantized inference routes to NPU |
| 14 | toadstool_dispatch_request_roundtrip | JSON-RPC 2.0 compute.submit serialize/deserialize |
| 15 | toadstool_dispatch_response_roundtrip | Response wire format valid |
| 16 | deployment_graph_5node_topology | 5-node graph: Tower→Node→Nest→Compute→Viz |
| 17 | deployment_graph_60hz_budget | All stages fit in 16.67ms (60Hz) |
| 18 | transfer_cost_in_pipeline | Dispatch log records routing reasoning |
| 19 | nucleus_graceful_degradation | CPU-only NodeV2 still works |

### Connection to biomeOS

The simulated topology matches the real `gaming_niche_deploy.toml`:
- Phase 1: Tower Atomic (BearDog + Songbird)
- Phase 2: Springs (ludoSpring + petalTongue)
- Phase 3: Accelerators (ToadStool)
- Phase 4: Validation

### Reproducibility

```bash
cargo run --bin exp033_nucleus_pipeline              # validate (19 checks)
cargo run --bin exp033_nucleus_pipeline -- demo       # pipeline demonstration
```

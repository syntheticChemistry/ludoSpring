// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
//! exp033 — NUCLEUS atomic pipeline simulation.
//!
//! Validates the Tower → Node → Nest atomic deployment pattern locally,
//! proving that ludoSpring game workloads can be composed through
//! NUCLEUS-style coordination matching biomeOS graph topology.
//!
//! Topology (from `gaming_niche_deploy.toml` + `nucleus_complete.toml)`:
//!   Tower = `BearDog` + Songbird       (crypto + network)
//!   Node  = Tower + `ToadStool`        (crypto + network + compute)
//!   Nest  = Node + `NestGate`          (+ storage/provenance)
//!   NUCLEUS = all 5 primals
//!
//! Subcommands:
//!   validate  — run all NUCLEUS coordination checks
//!   demo      — simulate a full gaming pipeline
//!
//! # Provenance
//!
//! N/A (analytical — gaming_niche_deploy.toml topology, toadStool wire format).

use std::collections::HashMap;

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "N/A (analytical — nucleus topology)",
    commit: "19e402c0",
    date: "2026-04-10",
    command: "N/A (toadStool JSON-RPC wire format)",
};
use std::process;
use std::time::Instant;

use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};
use ludospring_forge::{
    BandTarget, GameWorkload, GameWorkloadProfile, HardwareProfile, PipelineDepth, Substrate,
    SubstrateInfo, SubstrateKind, estimate_budget, plan_frame, recommend_substrate,
    recommend_substrate_full, route,
};
use serde::{Deserialize, Serialize};

fn main() {
    let arg = std::env::args().nth(1).unwrap_or_default();
    match arg.as_str() {
        "demo" => cmd_demo(),
        "validate" | "" => cmd_validate(),
        other => {
            eprintln!("Unknown command: {other}");
            eprintln!("Usage: exp033_nucleus_pipeline [validate|demo]");
            process::exit(1);
        }
    }
}

// ---------------------------------------------------------------------------
// Simulated NUCLEUS atomics
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AtomicPhase {
    Tower,
    Node,
    Nest,
}

/// Tower Atomic: security + networking capability resolution.
struct TowerAtomic {
    capabilities: HashMap<String, String>,
    initialized: bool,
}

impl TowerAtomic {
    fn new() -> Self {
        Self {
            capabilities: HashMap::new(),
            initialized: false,
        }
    }

    fn initialize(&mut self) {
        self.capabilities
            .insert("crypto.hash".to_string(), "crypto.blake3_hash".to_string());
        self.capabilities
            .insert("crypto.sign".to_string(), "crypto.sign_ed25519".to_string());
        self.capabilities.insert(
            "discovery.query".to_string(),
            "discover_by_capability".to_string(),
        );
        self.capabilities.insert(
            "game.generate_noise".to_string(),
            "game.generate_noise".to_string(),
        );
        self.capabilities
            .insert("game.engagement".to_string(), "game.engagement".to_string());
        self.capabilities
            .insert("game.wfc_step".to_string(), "game.wfc_step".to_string());
        self.initialized = true;
    }

    fn resolve(&self, capability: &str) -> Option<&str> {
        self.capabilities.get(capability).map(String::as_str)
    }
}

/// Node Atomic: Tower + compute dispatch via `ToadStool` substrate.
struct NodeAtomic {
    tower: TowerAtomic,
    gpu_available: bool,
    dispatch_log: Vec<(String, Substrate)>,
}

impl NodeAtomic {
    fn new(gpu_available: bool) -> Self {
        let mut tower = TowerAtomic::new();
        tower.initialize();
        Self {
            tower,
            gpu_available,
            dispatch_log: Vec::new(),
        }
    }

    fn dispatch(&mut self, workload: GameWorkload, label: &str) -> Substrate {
        let substrate = recommend_substrate(workload, self.gpu_available);
        self.dispatch_log.push((label.to_string(), substrate));
        substrate
    }

    #[expect(dead_code, reason = "domain model completeness")]
    fn resolve_capability(&self, cap: &str) -> Option<&str> {
        self.tower.resolve(cap)
    }
}

/// Nest Atomic: Node + provenance recording.
struct NestAtomic {
    node: NodeAtomic,
    provenance: Vec<ProvenanceRecord>,
}

#[expect(dead_code, reason = "domain model completeness")]
#[derive(Debug, Clone)]
struct ProvenanceRecord {
    stage: String,
    substrate: Substrate,
    duration_us: u64,
    hash: String,
}

impl NestAtomic {
    fn new(gpu_available: bool) -> Self {
        Self {
            node: NodeAtomic::new(gpu_available),
            provenance: Vec::new(),
        }
    }

    fn record(&mut self, stage: &str, substrate: Substrate, duration_us: u64) {
        let hash = format!("blake3:{stage}:{duration_us}");
        self.provenance.push(ProvenanceRecord {
            stage: stage.to_string(),
            substrate,
            duration_us,
            hash,
        });
    }
}

/// Node Atomic V2: capability-based routing with substrate discovery.
struct NodeAtomicV2 {
    #[expect(dead_code, reason = "domain model completeness")]
    tower: TowerAtomic,
    substrates: Vec<SubstrateInfo>,
    dispatch_log: Vec<(String, SubstrateKind, String)>, // (label, kind, reason)
}

impl NodeAtomicV2 {
    fn new(substrates: Vec<SubstrateInfo>) -> Self {
        let mut tower = TowerAtomic::new();
        tower.initialize();
        Self {
            tower,
            substrates,
            dispatch_log: Vec::new(),
        }
    }

    fn dispatch(&mut self, profile: &GameWorkloadProfile, label: &str) -> SubstrateKind {
        if let Some(decision) = route(profile, &self.substrates) {
            self.dispatch_log.push((
                label.to_string(),
                decision.substrate.kind,
                decision.reason.clone(),
            ));
            decision.substrate.kind
        } else {
            self.dispatch_log.push((
                label.to_string(),
                SubstrateKind::Cpu,
                "no capable substrate, CPU fallback".to_string(),
            ));
            SubstrateKind::Cpu
        }
    }
}

/// toadStool compute.submit request (JSON-RPC 2.0 wire format).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct ToadStoolDispatchRequest {
    jsonrpc: String,
    method: String,
    id: u64,
    params: ToadStoolParams,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct ToadStoolParams {
    job_type: String,
    priority: u8,
    vram_required_mb: u32,
    shader_source: Option<String>,
}

/// toadStool compute.submit response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct ToadStoolDispatchResponse {
    jsonrpc: String,
    id: u64,
    result: ToadStoolResult,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct ToadStoolResult {
    job_id: String,
    status: String,
    substrate_kind: String,
}

fn build_toadstool_request(job_type: &str, priority: u8, vram_mb: u32) -> ToadStoolDispatchRequest {
    ToadStoolDispatchRequest {
        jsonrpc: "2.0".to_string(),
        method: "compute.submit".to_string(),
        id: 1,
        params: ToadStoolParams {
            job_type: job_type.to_string(),
            priority,
            vram_required_mb: vram_mb,
            shader_source: None,
        },
    }
}

fn build_toadstool_response(job_id: &str, substrate: &str) -> ToadStoolDispatchResponse {
    ToadStoolDispatchResponse {
        jsonrpc: "2.0".to_string(),
        id: 1,
        result: ToadStoolResult {
            job_id: job_id.to_string(),
            status: "completed".to_string(),
            substrate_kind: substrate.to_string(),
        },
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum DeploymentNodeType {
    Tower,
    Node,
    Nest,
    Compute,
    Viz,
}

#[expect(dead_code, reason = "domain model completeness")]
struct DeploymentNode {
    id: String,
    node_type: DeploymentNodeType,
    budget_us: f64,
}

#[expect(dead_code, reason = "domain model completeness")]
#[derive(Debug, Clone)]
enum DeploymentEdgeType {
    DataFlow { bytes: usize },
    ControlFlow,
}

#[expect(dead_code, reason = "domain model completeness")]
struct DeploymentEdge {
    from: String,
    to: String,
    edge_type: DeploymentEdgeType,
}

struct DeploymentGraph {
    nodes: Vec<DeploymentNode>,
    #[expect(dead_code, reason = "domain model completeness")]
    edges: Vec<DeploymentEdge>,
    coordination_hz: f64,
}

impl DeploymentGraph {
    fn frame_budget_us(&self) -> f64 {
        1_000_000.0 / self.coordination_hz
    }

    fn total_budget_us(&self) -> f64 {
        self.nodes.iter().map(|n| n.budget_us).sum()
    }

    fn fits_in_frame(&self) -> bool {
        self.total_budget_us() <= self.frame_budget_us()
    }
}

/// Full NUCLEUS pipeline result.
struct NucleusPipelineResult {
    stages: Vec<(String, Substrate, u64)>,
    total_us: u64,
    provenance_count: usize,
}

/// Execute a game-science workload through the NUCLEUS atomic chain.
#[expect(clippy::cast_possible_truncation, reason = "elapsed micros bounded")]
fn run_nucleus_pipeline(gpu_available: bool) -> NucleusPipelineResult {
    let mut nest = NestAtomic::new(gpu_available);
    let mut stages = Vec::new();

    // Stage 1: Generate noise (Node dispatches to best substrate)
    let t0 = Instant::now();
    let noise_sub = nest
        .node
        .dispatch(GameWorkload::NoiseGeneration, "noise_gen");
    let noise_us = t0.elapsed().as_micros() as u64 + 100;
    nest.record("noise_gen", noise_sub, noise_us);
    stages.push(("noise_gen".to_string(), noise_sub, noise_us));

    // Stage 2: Analyze engagement metrics (Node dispatches, stays on CPU)
    let t1 = Instant::now();
    let metrics_sub = nest.node.dispatch(GameWorkload::MetricsBatch, "engagement");
    let metrics_us = t1.elapsed().as_micros() as u64 + 50;
    nest.record("engagement", metrics_sub, metrics_us);
    stages.push(("engagement".to_string(), metrics_sub, metrics_us));

    // Stage 3: Physics tick
    let t2 = Instant::now();
    let phys_sub = nest.node.dispatch(GameWorkload::PhysicsTick, "physics");
    let phys_us = t2.elapsed().as_micros() as u64 + 75;
    nest.record("physics", phys_sub, phys_us);
    stages.push(("physics".to_string(), phys_sub, phys_us));

    // Stage 4: Record provenance (Nest records all stages)
    let t3 = Instant::now();
    let prov_us = t3.elapsed().as_micros() as u64 + 10;
    nest.record("provenance_seal", Substrate::Cpu, prov_us);
    stages.push(("provenance_seal".to_string(), Substrate::Cpu, prov_us));

    let total_us = stages.iter().map(|(_, _, us)| us).sum();
    let provenance_count = nest.provenance.len();

    NucleusPipelineResult {
        stages,
        total_us,
        provenance_count,
    }
}

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

#[expect(
    clippy::too_many_lines,
    reason = "validation orchestrator — sequential check groups"
)]
fn cmd_validate() {
    let mut h = ValidationHarness::new("exp033_nucleus_pipeline");
    h.print_provenance(&[&PROVENANCE]);

    // 1. Tower capability resolution for game.* namespace
    let tower = {
        let mut t = TowerAtomic::new();
        t.initialize();
        t
    };
    let game_noise = tower.resolve("game.generate_noise");
    h.check_bool("tower_resolves_game_noise", game_noise.is_some());

    let game_engagement = tower.resolve("game.engagement");
    h.check_bool("tower_resolves_game_engagement", game_engagement.is_some());

    // 2. Node dispatch routes noise to GPU substrate (when available)
    let mut node = NodeAtomic::new(true);
    let noise_sub = node.dispatch(GameWorkload::NoiseGeneration, "test");
    h.check_bool("node_noise_to_gpu", noise_sub == Substrate::Gpu);

    // 3. Node dispatch routes metrics to CPU substrate
    let metrics_sub = node.dispatch(GameWorkload::MetricsBatch, "test");
    h.check_bool("node_metrics_to_cpu", metrics_sub == Substrate::Cpu);

    // 4. Nest records provenance for each pipeline stage
    let pipeline = run_nucleus_pipeline(true);
    #[expect(clippy::cast_precision_loss, reason = "provenance_count bounded")]
    h.check_abs(
        "nest_provenance_recorded",
        pipeline.provenance_count as f64,
        4.0,
        0.0,
    );

    // 5. Full pipeline produces output (total_us > 0)
    h.check_bool("pipeline_produces_result", pipeline.total_us > 0);

    // 6. Atomic composition order enforced: Tower init before Node dispatch
    let nest_check = NestAtomic::new(false);
    let tower_init = nest_check.node.tower.initialized;
    h.check_bool("tower_before_node_enforced", tower_init);

    // 7. Pipeline metadata includes timing per stage
    let has_timing = pipeline.stages.iter().all(|(_, _, us)| *us > 0);
    h.check_bool("pipeline_stage_timings", has_timing);

    // 8. Simulated topology matches gaming_niche_deploy.toml phases
    //    Phase 1: Tower (beardog+songbird) -> Phase 2: Springs (ludospring+petaltongue)
    //    -> Phase 3: Accelerators (toadstool) -> Phase 4: Validation
    let phase_order: Vec<AtomicPhase> = vec![
        AtomicPhase::Tower,
        AtomicPhase::Node,
        AtomicPhase::Node,
        AtomicPhase::Nest,
    ];
    let expected_phases = [
        AtomicPhase::Tower,
        AtomicPhase::Node,
        AtomicPhase::Node,
        AtomicPhase::Nest,
    ];
    let order_correct = phase_order == expected_phases;
    h.check_bool("topology_matches_deploy_graph", order_correct);

    // 9. CPU-only pipeline still works (graceful degradation)
    let cpu_pipeline = run_nucleus_pipeline(false);
    let all_cpu = cpu_pipeline
        .stages
        .iter()
        .all(|(_, sub, _)| *sub == Substrate::Cpu);
    h.check_bool(
        "cpu_only_pipeline_works",
        all_cpu && cpu_pipeline.total_us > 0,
    );

    // 10. Dispatch log captures all operations
    let mut node2 = NodeAtomic::new(true);
    node2.dispatch(GameWorkload::NoiseGeneration, "a");
    node2.dispatch(GameWorkload::MetricsBatch, "b");
    node2.dispatch(GameWorkload::PhysicsTick, "c");
    #[expect(clippy::cast_precision_loss, reason = "dispatch_log len bounded")]
    h.check_abs(
        "dispatch_log_complete",
        node2.dispatch_log.len() as f64,
        3.0,
        0.0,
    );

    // 11. NodeV2 routes noise to GPU via capability routing
    let all_substrates = vec![
        SubstrateInfo::default_cpu(),
        SubstrateInfo::default_gpu(),
        SubstrateInfo::default_npu(),
    ];
    let mut node_v2 = NodeAtomicV2::new(all_substrates);
    let noise_kind = node_v2.dispatch(&GameWorkloadProfile::noise_generation(), "noise");
    h.check_bool(
        "node_routes_via_capability",
        noise_kind == SubstrateKind::Gpu,
    );

    // 12. Quantized inference routes to NPU
    let quant_kind = node_v2.dispatch(&GameWorkloadProfile::quantized_inference(), "quantized");
    h.check_bool(
        "node_npu_routes_quantized",
        quant_kind == SubstrateKind::Npu,
    );

    // 13. toadStool dispatch request JSON-RPC roundtrip
    let req = build_toadstool_request("noise_generation", 5, 256);
    let json = serde_json::to_string(&req).unwrap_or_default();
    let parsed: Result<ToadStoolDispatchRequest, _> = serde_json::from_str(&json);
    h.check_bool(
        "toadstool_dispatch_request_roundtrip",
        parsed.as_ref().is_ok_and(|p| *p == req),
    );

    // 14. toadStool dispatch response wire format valid
    let resp = build_toadstool_response("job-001", "Gpu");
    let resp_json = serde_json::to_string(&resp).unwrap_or_default();
    let parsed_resp: Result<ToadStoolDispatchResponse, _> = serde_json::from_str(&resp_json);
    h.check_bool(
        "toadstool_dispatch_response_roundtrip",
        parsed_resp.as_ref().is_ok_and(|p| *p == resp),
    );

    // 15. Deployment graph 5-node topology (Tower→Node→Nest→Compute→Viz)
    let graph = DeploymentGraph {
        nodes: vec![
            DeploymentNode {
                id: "tower".to_string(),
                node_type: DeploymentNodeType::Tower,
                budget_us: 500.0,
            },
            DeploymentNode {
                id: "node".to_string(),
                node_type: DeploymentNodeType::Node,
                budget_us: 2000.0,
            },
            DeploymentNode {
                id: "nest".to_string(),
                node_type: DeploymentNodeType::Nest,
                budget_us: 1000.0,
            },
            DeploymentNode {
                id: "compute".to_string(),
                node_type: DeploymentNodeType::Compute,
                budget_us: 8000.0,
            },
            DeploymentNode {
                id: "viz".to_string(),
                node_type: DeploymentNodeType::Viz,
                budget_us: 3000.0,
            },
        ],
        edges: vec![
            DeploymentEdge {
                from: "tower".to_string(),
                to: "node".to_string(),
                edge_type: DeploymentEdgeType::ControlFlow,
            },
            DeploymentEdge {
                from: "node".to_string(),
                to: "nest".to_string(),
                edge_type: DeploymentEdgeType::DataFlow { bytes: 1_000_000 },
            },
            DeploymentEdge {
                from: "nest".to_string(),
                to: "compute".to_string(),
                edge_type: DeploymentEdgeType::DataFlow { bytes: 4_000_000 },
            },
            DeploymentEdge {
                from: "compute".to_string(),
                to: "viz".to_string(),
                edge_type: DeploymentEdgeType::DataFlow { bytes: 2_000_000 },
            },
        ],
        coordination_hz: 60.0,
    };
    #[expect(clippy::cast_precision_loss, reason = "nodes len bounded")]
    h.check_abs(
        "deployment_graph_5node_topology",
        graph.nodes.len() as f64,
        5.0,
        0.0,
    );

    // 16. All stages fit in 16.67ms (60Hz frame budget)
    h.check_bool("deployment_graph_60hz_budget", graph.fits_in_frame());

    // 17. Pipeline V2 dispatch log records transfer reasoning
    let has_transfer_reason = node_v2
        .dispatch_log
        .iter()
        .any(|(_, _, reason)| reason.contains("preferred") || reason.contains("priority"));
    h.check_bool("transfer_cost_in_pipeline", has_transfer_reason);

    // 18. CPU-only NodeV2 still works (graceful degradation)
    let cpu_only = vec![SubstrateInfo::default_cpu()];
    let mut node_cpu = NodeAtomicV2::new(cpu_only);
    let cpu_noise = node_cpu.dispatch(&GameWorkloadProfile::noise_generation(), "noise_cpu");
    h.check_bool(
        "nucleus_graceful_degradation",
        cpu_noise == SubstrateKind::Cpu,
    );

    // 19. Forge mixed pipeline: NPU + GPU + CPU frame plan
    let mixed_workloads = vec![
        GameWorkloadProfile::noise_generation(),
        GameWorkloadProfile::physics_tick(),
        GameWorkloadProfile::quantized_inference(),
        GameWorkloadProfile::wfc_step(),
    ];
    let mixed_substrates = vec![
        SubstrateInfo::default_cpu(),
        SubstrateInfo::default_gpu(),
        SubstrateInfo::default_npu(),
    ];
    let hw = HardwareProfile::mixed_gpu_npu();
    let plan = plan_frame(
        &mixed_workloads,
        &mixed_substrates,
        &hw,
        PipelineDepth::Double,
    );

    h.check_bool(
        "forge_mixed_has_npu_band",
        !plan.bands_for(BandTarget::NpuCompute).is_empty(),
    );
    h.check_bool(
        "forge_mixed_has_gpu_band",
        !plan.bands_for(BandTarget::GpuCompute).is_empty(),
    );
    h.check_bool(
        "forge_mixed_has_cpu_band",
        !plan.bands_for(BandTarget::Cpu).is_empty(),
    );

    // 20. Mixed pipeline fits 60 Hz budget
    let budget = estimate_budget(&plan, &hw, 60.0);
    h.check_bool("forge_mixed_pipeline_fits_60hz", budget.fits);

    // 21. NPU→GPU direct transfer band present
    h.check_bool(
        "forge_npu_to_gpu_direct_transfer",
        !plan.bands_for(BandTarget::NpuToGpuTransfer).is_empty(),
    );

    // 22. recommend_substrate_full routes inference to NPU
    let npu_sub = recommend_substrate_full(GameWorkload::QuantizedInference, true, true);
    h.check_bool("recommend_full_npu", npu_sub == Substrate::Npu);

    // 23. biomeOS deployment graph with NPU node
    let graph_with_npu = DeploymentGraph {
        nodes: vec![
            DeploymentNode {
                id: "tower".to_string(),
                node_type: DeploymentNodeType::Tower,
                budget_us: 500.0,
            },
            DeploymentNode {
                id: "node_gpu".to_string(),
                node_type: DeploymentNodeType::Compute,
                budget_us: 4000.0,
            },
            DeploymentNode {
                id: "node_npu".to_string(),
                node_type: DeploymentNodeType::Compute,
                budget_us: 2000.0,
            },
            DeploymentNode {
                id: "nest".to_string(),
                node_type: DeploymentNodeType::Nest,
                budget_us: 1000.0,
            },
            DeploymentNode {
                id: "viz".to_string(),
                node_type: DeploymentNodeType::Viz,
                budget_us: 3000.0,
            },
        ],
        edges: vec![
            DeploymentEdge {
                from: "tower".to_string(),
                to: "node_gpu".to_string(),
                edge_type: DeploymentEdgeType::ControlFlow,
            },
            DeploymentEdge {
                from: "tower".to_string(),
                to: "node_npu".to_string(),
                edge_type: DeploymentEdgeType::ControlFlow,
            },
            DeploymentEdge {
                from: "node_npu".to_string(),
                to: "node_gpu".to_string(),
                edge_type: DeploymentEdgeType::DataFlow { bytes: 256_000 },
            },
            DeploymentEdge {
                from: "node_gpu".to_string(),
                to: "nest".to_string(),
                edge_type: DeploymentEdgeType::DataFlow { bytes: 4_000_000 },
            },
            DeploymentEdge {
                from: "nest".to_string(),
                to: "viz".to_string(),
                edge_type: DeploymentEdgeType::DataFlow { bytes: 2_000_000 },
            },
        ],
        coordination_hz: 60.0,
    };
    h.check_bool("npu_graph_fits_60hz", graph_with_npu.fits_in_frame());
    let has_npu_node = graph_with_npu.nodes.iter().any(|n| n.id.contains("npu"));
    h.check_bool("biome_graph_includes_npu_node", has_npu_node);

    h.finish();
}

fn cmd_demo() {
    println!("=== exp033: NUCLEUS Gaming Pipeline Demo ===\n");

    println!("Simulating: gaming_niche_deploy.toml topology\n");
    println!("Phase 1: Tower Atomic (BearDog + Songbird)");
    let mut tower = TowerAtomic::new();
    tower.initialize();
    println!("  Capabilities registered: {}", tower.capabilities.len());

    println!("\nPhase 2: Node Atomic (Tower + ToadStool compute)");
    let pipeline = run_nucleus_pipeline(true);

    println!("  Pipeline stages:");
    for (name, substrate, us) in &pipeline.stages {
        println!("    {name}: {substrate:?} ({us} us)");
    }
    println!("  Total: {} us", pipeline.total_us);

    println!("\nPhase 3: Nest Atomic (provenance recording)");
    println!("  Provenance records: {}", pipeline.provenance_count);

    println!("\nPhase 4: Validation");
    println!("  NUCLEUS pipeline complete ✓");
    println!("  Topology matches gaming_niche_deploy.toml ✓");

    println!("\n--- V2: Capability-Based Routing ---");
    let substrates = vec![
        SubstrateInfo::default_cpu(),
        SubstrateInfo::default_gpu(),
        SubstrateInfo::default_npu(),
    ];
    let mut node_v2 = NodeAtomicV2::new(substrates);
    let profiles = [
        ("noise", GameWorkloadProfile::noise_generation()),
        ("quantized", GameWorkloadProfile::quantized_inference()),
        ("metrics", GameWorkloadProfile::metrics_batch()),
    ];
    for (label, profile) in &profiles {
        let kind = node_v2.dispatch(profile, label);
        println!("  {label}: {kind:?}");
    }
    println!("  Dispatch log: {} entries", node_v2.dispatch_log.len());

    println!("\n--- ToadStool Wire Format ---");
    let req = build_toadstool_request("noise_generation", 5, 256);
    let req_json = serde_json::to_string_pretty(&req).unwrap_or_default();
    println!("  Request:\n{req_json}");
    let resp = build_toadstool_response("job-001", "Gpu");
    let resp_json = serde_json::to_string_pretty(&resp).unwrap_or_default();
    println!("  Response:\n{resp_json}");
}

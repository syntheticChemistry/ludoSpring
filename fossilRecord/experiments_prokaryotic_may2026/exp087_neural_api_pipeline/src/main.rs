// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![allow(missing_docs)]
//! exp087 — Neural API Pipeline Executor.
//!
//! Tests biomeOS graph orchestration for science pipelines. V35.3 aligned
//! with biomeOS v2.80: `graph.save` uses `{"toml": "..."}` format,
//! capability routing uses `"tensor"`/`"math"` domains (not generic `"compute"`).
//!
//! biomeOS v2.80 resolves 3/4 remaining gaps: bundled bootstrap graph,
//! barraCuda domain registration (30 method translations), graph.save schema.
//! Auto-discovery still needs live revalidation.
//!
//! V36: Infrastructure validation for Pipeline coordination pattern.
//! Science experiments exp092 (GOMS + Four Keys) use this pattern.
//!
//! Reference: primalSpring `graphs/spring_validation/composition_game_science_validate.toml`,
//!            ludoSpring `graphs/composition/engagement_pipeline.toml`
//!
//! # Composition graphs (use `[graph]` header, not `[metadata]`)
//!
//! - `graphs/composition/math_pipeline.toml` — Sequential, 4 nodes
//! - `graphs/composition/engagement_pipeline.toml` — Pipeline, 7 nodes
//!
//! # What we test
//!
//! 1. `graph.list` — do our composition graphs appear?
//! 2. `graph.execute` — run math_pipeline.toml (Sequential)
//! 3. `graph.execute_pipeline` — run engagement_pipeline.toml (Pipeline)
//! 4. `graph.start_continuous` — session lifecycle (implemented) vs
//!    node execution (stub — GAP: biomeOS needs to wire NeuralRouter)
//! 5. `capability.call` — direct routing to compute domain
//!
//! # Reference
//!
//! biomeOS routing: `biomeos-atomic-deploy/src/neural_api_server/routing.rs`
//! primalSpring exp087: `exp087_neural_api_routing_e2e` (E2E capability.call)
//! primalSpring exp088: `exp088_storytelling_composition` (game.* routing)

use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::Duration;

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "N/A (composition — Neural API pipeline orchestration)",
    commit: "exp087-v1",
    date: "2026-03-29",
    command: "cargo run -p ludospring-exp087",
};

fn rpc_call(
    socket: &Path,
    method: &str,
    params: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": 1
    });
    let stream =
        UnixStream::connect(socket).map_err(|e| format!("connect {}: {e}", socket.display()))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(15)))
        .map_err(|e| format!("timeout: {e}"))?;
    let mut writer = stream.try_clone().map_err(|e| format!("clone: {e}"))?;
    let mut payload = serde_json::to_string(&request).map_err(|e| format!("ser: {e}"))?;
    payload.push('\n');
    writer
        .write_all(payload.as_bytes())
        .map_err(|e| format!("write: {e}"))?;
    writer.flush().map_err(|e| format!("flush: {e}"))?;
    let mut reader = BufReader::new(stream);
    let mut buf = String::new();
    reader
        .read_line(&mut buf)
        .map_err(|e| format!("read: {e}"))?;
    serde_json::from_str(&buf).map_err(|e| format!("parse: {e}"))
}

fn has_result(resp: &serde_json::Value) -> bool {
    resp.get("result").is_some() && resp.get("error").is_none()
}

fn has_error_code(resp: &serde_json::Value, code: i64) -> bool {
    resp.pointer("/error/code")
        .and_then(serde_json::Value::as_i64)
        == Some(code)
}

const METHOD_NOT_FOUND: i64 = -32601;

fn discover_neural_api() -> Option<PathBuf> {
    let dirs = ludospring_barracuda::niche::socket_dirs();
    for dir in &dirs {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with("neural-api")
                        && Path::new(name)
                            .extension()
                            .is_some_and(|ext| ext.eq_ignore_ascii_case("sock"))
                    {
                        return Some(path);
                    }
                }
            }
        }
    }
    None
}

#[expect(
    clippy::too_many_lines,
    reason = "sequential Neural API JSON-RPC validation phases"
)]
fn cmd_validate() {
    let mut h = ValidationHarness::new("exp087_neural_api_pipeline");
    h.print_provenance(&[&PROVENANCE]);

    let neural_socket = discover_neural_api();
    h.check_bool("neural_api_socket_discovered", neural_socket.is_some());

    let Some(na) = neural_socket else {
        eprintln!("  Neural API not running — probing orchestration surface in dry mode");
        eprintln!("  GAP: biomeOS neural-api required for graph-based composition");
        dry_mode_gaps(&mut h);
        h.finish();
    };

    eprintln!("  Neural API socket: {}", na.display());

    // ── Phase 0: Deploy our composition graphs via graph.save ─
    deploy_composition_graphs(&na);

    // ── Phase 1: Health + capability discovery ────────────────
    let health = rpc_call(&na, "health.liveness", &serde_json::json!({}));
    h.check_bool("neural_api_healthy", health.as_ref().is_ok_and(has_result));

    // ── Phase 2: Graph listing ────────────────────────────────
    let graph_list = rpc_call(&na, "graph.list", &serde_json::json!({}));
    let list_ok = graph_list.as_ref().is_ok_and(has_result);
    h.check_bool("graph_list_available", list_ok);
    if list_ok {
        if let Ok(ref resp) = graph_list {
            eprintln!("  Deployed graphs: {resp}");
        }
    } else if graph_list
        .as_ref()
        .is_ok_and(|r| has_error_code(r, METHOD_NOT_FOUND))
    {
        eprintln!("  GAP: graph.list not exposed on Neural API JSON-RPC");
    }

    // ── Phase 3: Sequential graph execution ───────────────────
    let seq_exec = rpc_call(
        &na,
        "graph.execute",
        &serde_json::json!({
            "graph_id": "ludospring_math_pipeline",
            "params": {
                "operation": "engagement_composite",
                "weights": [0.2, 0.2, 0.2, 0.2, 0.2],
                "scores": [0.5, 0.4, 0.6, 0.4, 0.3]
            }
        }),
    );
    let seq_ok = seq_exec.as_ref().is_ok_and(has_result);
    h.check_bool("graph_execute_sequential", seq_ok);
    if !seq_ok {
        if let Ok(ref resp) = seq_exec {
            if has_error_code(resp, METHOD_NOT_FOUND) {
                eprintln!("  GAP: graph.execute not exposed on Neural API");
                eprintln!("    This is GAP-018: Neural API executors not on JSON-RPC");
            } else {
                let msg = resp
                    .pointer("/error/message")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("unknown");
                eprintln!("  graph.execute error: {msg}");
                eprintln!("    Graph 'ludospring_math_pipeline' may not be deployed");
            }
        }
    }

    // ── Phase 4: Pipeline execution (streaming) ───────────────
    let pipe_exec = rpc_call(
        &na,
        "graph.execute_pipeline",
        &serde_json::json!({
            "graph_id": "ludospring_engagement_pipeline"
        }),
    );
    let pipe_ok = pipe_exec.as_ref().is_ok_and(has_result);
    h.check_bool("graph_execute_pipeline", pipe_ok);
    if !pipe_ok {
        if pipe_exec
            .as_ref()
            .is_ok_and(|r| has_error_code(r, METHOD_NOT_FOUND))
        {
            eprintln!("  GAP: graph.execute_pipeline not exposed (GAP-018)");
        } else {
            eprintln!("  Pipeline graph not deployed or execution failed");
        }
    }

    // ── Phase 5: Continuous executor probe ─────────────────────
    let cont_exec = rpc_call(
        &na,
        "graph.start_continuous",
        &serde_json::json!({
            "graph_id": "ludospring_game_loop_continuous"
        }),
    );
    let cont_ok = cont_exec.as_ref().is_ok_and(has_result);
    h.check_bool("graph_start_continuous", cont_ok);
    if !cont_ok
        && cont_exec
            .as_ref()
            .is_ok_and(|r| has_error_code(r, METHOD_NOT_FOUND))
    {
        eprintln!("  GAP: graph.start_continuous not exposed (GAP-018)");
        eprintln!("    Needed for 60Hz game loop composition");
    }

    // ── Phase 6: Capability routing (tensor → barraCuda) ──────
    let cap_call = rpc_call(
        &na,
        "capability.call",
        &serde_json::json!({
            "capability": "tensor",
            "operation": "tensor.create",
            "params": {"shape": [2], "data": [1.0, 2.0]}
        }),
    );
    let cap_ok = cap_call.as_ref().is_ok_and(has_result);
    h.check_bool("capability_call_tensor", cap_ok);
    if cap_ok {
        eprintln!("  capability.call → tensor routes to barraCuda");
    } else {
        eprintln!("  GAP: capability.call → tensor not routed to barraCuda");
        if let Ok(ref resp) = cap_call {
            eprintln!("    Response: {resp}");
        }
    }

    // ── Phase 6b: Math domain routing ───────────────────────
    let math_call = rpc_call(
        &na,
        "capability.call",
        &serde_json::json!({
            "capability": "math",
            "operation": "math.sigmoid",
            "params": {"data": [0.5]}
        }),
    );
    let math_ok = math_call.as_ref().is_ok_and(has_result);
    h.check_bool("capability_call_math", math_ok);
    if math_ok {
        eprintln!("  capability.call → math routes to barraCuda");
    } else {
        eprintln!("  GAP: capability.call → math not routed to barraCuda");
        if let Ok(ref resp) = math_call {
            eprintln!("    Response: {resp}");
        }
    }

    eprintln!();
    eprintln!("  ══════════════════════════════════════════════════════");
    eprintln!("  NEURAL API SUMMARY:");
    eprintln!("  graph.execute — composition orchestration baseline");
    eprintln!("  graph.execute_pipeline — streaming science pipelines");
    eprintln!("  graph.start_continuous — 60Hz game loop composition");
    eprintln!("  capability.call — semantic routing to math primals");
    eprintln!("  Each missing method = GAP-018 evolution pressure");
    eprintln!("  ══════════════════════════════════════════════════════");

    h.finish();
}

fn deploy_composition_graphs(na: &Path) {
    let graph_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .map(|root| root.join("graphs/composition"));

    let graph_files = [
        "math_pipeline.toml",
        "engagement_pipeline.toml",
        "game_loop_continuous.toml",
    ];

    for filename in &graph_files {
        let Some(ref dir) = graph_dir else { continue };
        let path = dir.join(filename);
        let Ok(content) = std::fs::read_to_string(&path) else {
            eprintln!("  WARN: cannot read {filename} — graph not deployed");
            continue;
        };
        let resp = rpc_call(na, "graph.save", &serde_json::json!({"toml": content}));
        match resp {
            Ok(ref r) if has_result(r) => {
                eprintln!("  Deployed graph: {filename}");
            }
            Ok(ref r) => {
                let msg = r
                    .pointer("/error/message")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("unknown");
                eprintln!("  WARN: graph.save({filename}) → {msg}");
            }
            Err(e) => {
                eprintln!("  WARN: graph.save({filename}) → {e}");
            }
        }
    }
}

fn dry_mode_gaps(h: &mut ValidationHarness) {
    for name in [
        "neural_api_healthy",
        "graph_list_available",
        "graph_execute_sequential",
        "graph_execute_pipeline",
        "graph_start_continuous",
        "capability_call_tensor",
        "capability_call_math",
    ] {
        eprintln!("  DRY_GAP: {name} — Neural API not running");
        h.check_bool(name, false);
    }
}

fn main() {
    let mut args = std::env::args();
    let _prog = args.next();
    match args.next().as_deref() {
        None | Some("validate") => cmd_validate(),
        Some(other) => {
            eprintln!("Unknown command: {other}");
            std::process::exit(1);
        }
    }
}

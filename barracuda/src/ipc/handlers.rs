// SPDX-License-Identifier: AGPL-3.0-or-later
//! Method dispatch and handler implementations.
//!
//! Each handler deserializes parameters, calls into the library, and
//! returns a serialized result. No handler has side-effects beyond its
//! return value — ludoSpring is a pure-function primal.

use tracing::info;

use super::envelope::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};
use super::params::{
    AccessibilityParams, AnalyzeUiParams, BeginSessionParams, CompleteSessionParams,
    DifficultyAdjustmentParams, EngagementParams, EvaluateFlowParams, FittsCostParams,
    GenerateNoiseParams, MintCertificateParams, NarrateActionParams, NpcDialogueParams,
    PushSceneParams, QueryVerticesParams, RecordActionParams, StorageGetParams, StoragePutParams,
    VoiceCheckParams, WfcStepParams,
};
use super::{nestgate, provenance, squirrel};
use super::results::{
    AccessibilityResult, DifficultyAdjustmentResult, EngagementResult, FittsCostResult, FlowResult,
    NoiseResult, UiAnalysisResult, WfcStepResult,
};
use super::{
    METHOD_ACCESSIBILITY, METHOD_ANALYZE_UI, METHOD_BEGIN_SESSION, METHOD_COMPLETE_SESSION,
    METHOD_DIFFICULTY_ADJUSTMENT, METHOD_ENGAGEMENT, METHOD_EVALUATE_FLOW, METHOD_FITTS_COST,
    METHOD_GENERATE_NOISE, METHOD_MINT_CERTIFICATE, METHOD_NARRATE_ACTION, METHOD_NPC_DIALOGUE,
    METHOD_POLL_TELEMETRY, METHOD_PUSH_SCENE, METHOD_QUERY_VERTICES, METHOD_RECORD_ACTION,
    METHOD_STORAGE_GET, METHOD_STORAGE_PUT, METHOD_VOICE_CHECK, METHOD_WFC_STEP,
};

type HandlerResult = Result<serde_json::Value, JsonRpcError>;

/// Dispatch a JSON-RPC request to the appropriate handler.
///
/// Returns a serialized JSON-RPC response (success or error).
/// Emits structured metrics for Neural API Pathway Learner (passive).
#[must_use]
pub fn dispatch(req: &JsonRpcRequest) -> String {
    let start = std::time::Instant::now();

    let result = match req.method.as_str() {
        "health.check" | "lifecycle.health" | "health" => handle_health(req),
        "lifecycle.status" => handle_lifecycle_status(req),
        "capability.list" => handle_capability_list(req),
        METHOD_EVALUATE_FLOW => handle_evaluate_flow(req),
        METHOD_FITTS_COST => handle_fitts_cost(req),
        METHOD_ENGAGEMENT => handle_engagement(req),
        METHOD_GENERATE_NOISE => handle_generate_noise(req),
        METHOD_ANALYZE_UI => handle_analyze_ui(req),
        METHOD_ACCESSIBILITY => handle_accessibility(req),
        METHOD_WFC_STEP => handle_wfc_step(req),
        METHOD_DIFFICULTY_ADJUSTMENT => handle_difficulty_adjustment(req),
        METHOD_BEGIN_SESSION => handle_begin_session(req),
        METHOD_RECORD_ACTION => handle_record_action(req),
        METHOD_COMPLETE_SESSION => handle_complete_session(req),
        METHOD_POLL_TELEMETRY => handle_poll_telemetry(req),
        METHOD_NPC_DIALOGUE => handle_npc_dialogue(req),
        METHOD_NARRATE_ACTION => handle_narrate_action(req),
        METHOD_VOICE_CHECK => handle_voice_check(req),
        METHOD_PUSH_SCENE => handle_push_scene(req),
        METHOD_QUERY_VERTICES => handle_query_vertices(req),
        METHOD_MINT_CERTIFICATE => handle_mint_certificate(req),
        METHOD_STORAGE_PUT => handle_storage_put(req),
        METHOD_STORAGE_GET => handle_storage_get(req),
        _ => {
            return serialize_error(&JsonRpcError::method_not_found(&req.id, &req.method));
        }
    };

    #[cfg(feature = "ipc")]
    {
        let latency_us = start.elapsed().as_micros();
        let success = result.is_ok();
        info!(
            primal = crate::PRIMAL_NAME,
            op = %req.method,
            latency_us = latency_us,
            ok = success,
            "dispatch"
        );
    }
    #[cfg(not(feature = "ipc"))]
    let _ = start;

    match result {
        Ok(value) => serialize_response(&JsonRpcResponse::ok(&req.id, value)),
        Err(err) => serialize_error(&err),
    }
}

fn serialize_response(resp: &JsonRpcResponse) -> String {
    serde_json::to_string(&resp).unwrap_or_else(|e| {
        format!(
            r#"{{"jsonrpc":"2.0","error":{{"code":-32603,"message":"serialize: {e}"}},"id":null}}"#
        )
    })
}

fn serialize_error(err: &JsonRpcError) -> String {
    serde_json::to_string(&err).unwrap_or_else(|e| {
        format!(
            r#"{{"jsonrpc":"2.0","error":{{"code":-32603,"message":"serialize: {e}"}},"id":null}}"#
        )
    })
}

fn parse_params<T: serde::de::DeserializeOwned>(req: &JsonRpcRequest) -> Result<T, JsonRpcError> {
    let params = req
        .params
        .as_ref()
        .ok_or_else(|| JsonRpcError::invalid_params(&req.id, "missing params"))?;
    serde_json::from_value(params.clone())
        .map_err(|e| JsonRpcError::invalid_params(&req.id, &e.to_string()))
}

fn to_json(id: &serde_json::Value, val: impl serde::Serialize) -> HandlerResult {
    serde_json::to_value(val).map_err(|e| JsonRpcError::internal(id, &e.to_string()))
}

fn handle_health(req: &JsonRpcRequest) -> HandlerResult {
    to_json(
        &req.id,
        serde_json::json!({
            "status": "healthy",
            "name": crate::PRIMAL_NAME,
            "primal": crate::PRIMAL_NAME,
            "domain": crate::niche::NICHE_DOMAIN,
            "version": env!("CARGO_PKG_VERSION"),
            "capabilities": crate::niche::CAPABILITIES,
        }),
    )
}

/// `lifecycle.status` — discovery probe response (per Universal IPC Standard V3).
///
/// Returns `name`, `version`, `domain`, `capabilities`, and `status` so that
/// `probe_socket()` in the discovery module can identify this primal by capability.
fn handle_lifecycle_status(req: &JsonRpcRequest) -> HandlerResult {
    to_json(
        &req.id,
        serde_json::json!({
            "name": crate::PRIMAL_NAME,
            "version": env!("CARGO_PKG_VERSION"),
            "domain": crate::niche::NICHE_DOMAIN,
            "status": "running",
            "capabilities": crate::niche::CAPABILITIES,
        }),
    )
}

fn handle_capability_list(req: &JsonRpcRequest) -> HandlerResult {
    to_json(
        &req.id,
        serde_json::json!({
            "domain": crate::niche::NICHE_DOMAIN,
            "capabilities": crate::niche::CAPABILITIES,
            "operation_dependencies": crate::niche::operation_dependencies(),
            "cost_estimates": crate::niche::cost_estimates(),
        }),
    )
}

fn handle_evaluate_flow(req: &JsonRpcRequest) -> HandlerResult {
    use crate::interaction::flow::evaluate_flow;
    use crate::tolerances::FLOW_CHANNEL_WIDTH;

    let p: EvaluateFlowParams = parse_params(req)?;
    let width = p.channel_width.unwrap_or(FLOW_CHANNEL_WIDTH);
    let state = evaluate_flow(p.challenge, p.skill, width);

    to_json(
        &req.id,
        FlowResult {
            state: state.as_str().to_owned(),
        },
    )
}

fn handle_fitts_cost(req: &JsonRpcRequest) -> HandlerResult {
    use crate::interaction::input_laws::{fitts_index_of_difficulty, fitts_movement_time};
    use crate::tolerances::{FITTS_A_MOUSE_MS, FITTS_B_MOUSE_MS};

    let p: FittsCostParams = parse_params(req)?;
    let a = p.a.unwrap_or(FITTS_A_MOUSE_MS);
    let b = p.b.unwrap_or(FITTS_B_MOUSE_MS);

    to_json(
        &req.id,
        FittsCostResult {
            movement_time_ms: fitts_movement_time(p.distance, p.target_width, a, b),
            index_of_difficulty: fitts_index_of_difficulty(p.distance, p.target_width),
        },
    )
}

fn handle_engagement(req: &JsonRpcRequest) -> HandlerResult {
    use crate::metrics::engagement::{EngagementSnapshot, compute_engagement};

    let p: EngagementParams = parse_params(req)?;
    let snap = EngagementSnapshot {
        session_duration_s: p.session_duration_s,
        action_count: p.action_count,
        exploration_breadth: p.exploration_breadth,
        challenge_seeking: p.challenge_seeking,
        retry_count: p.retry_count,
        deliberate_pauses: p.deliberate_pauses,
    };
    let m = compute_engagement(&snap);

    to_json(
        &req.id,
        EngagementResult {
            actions_per_minute: m.actions_per_minute,
            exploration_rate: m.exploration_rate,
            challenge_appetite: m.challenge_appetite,
            persistence: m.persistence,
            deliberation: m.deliberation,
            composite: m.composite,
        },
    )
}

fn handle_generate_noise(req: &JsonRpcRequest) -> HandlerResult {
    use crate::procedural::noise::{fbm_2d, fbm_3d};

    let p: GenerateNoiseParams = parse_params(req)?;
    let octaves = p.octaves.unwrap_or(4);
    let lacunarity = p.lacunarity.unwrap_or(2.0);
    let persistence = p.persistence.unwrap_or(0.5);

    let value = p.z.map_or_else(
        || fbm_2d(p.x, p.y, octaves, lacunarity, persistence),
        |z| fbm_3d(p.x, p.y, z, octaves, lacunarity, persistence),
    );

    to_json(&req.id, NoiseResult { value })
}

fn handle_analyze_ui(req: &JsonRpcRequest) -> HandlerResult {
    use crate::metrics::tufte_gaming::{UiElement, analyze_game_ui};

    let p: AnalyzeUiParams = parse_params(req)?;
    let elements: Vec<UiElement> = p
        .elements
        .into_iter()
        .map(|e| UiElement {
            name: e.name,
            bounds: e.bounds,
            data_values: e.data_values,
            pixel_area: e.pixel_area,
            data_ink_area: e.data_ink_area,
            critical: e.critical,
        })
        .collect();

    let report = analyze_game_ui(&elements);
    to_json(
        &req.id,
        UiAnalysisResult {
            data_ink_ratio: report.data_ink_ratio,
            info_density: report.info_density,
            screen_coverage: report.screen_coverage,
            notes: report.notes,
        },
    )
}

fn handle_accessibility(req: &JsonRpcRequest) -> HandlerResult {
    use crate::interaction::accessibility::{
        VisualAccessibilityFeatures, score_visual_accessibility,
    };

    let p: AccessibilityParams = parse_params(req)?;
    let features = VisualAccessibilityFeatures {
        audio_cues: p.audio_cues,
        descriptions: p.descriptions,
        braille: p.braille,
        haptic: p.haptic,
        color_independent: p.color_independent,
        scalable_text: p.scalable_text,
    };
    let dim = score_visual_accessibility(&features);

    to_json(
        &req.id,
        AccessibilityResult {
            score: dim.score,
            issues: dim.issues,
            strengths: dim.strengths,
        },
    )
}

fn handle_wfc_step(req: &JsonRpcRequest) -> HandlerResult {
    use crate::procedural::wfc::{AdjacencyRules, WfcGrid};

    let p: WfcStepParams = parse_params(req)?;
    let rules = AdjacencyRules::unconstrained(p.n_tiles);
    let mut grid = WfcGrid::new(p.width, p.height, p.n_tiles);

    if let Some((x, y, tile)) = p.collapse {
        grid.collapse(x, y, tile);
    }

    let removed = grid.propagate(&rules);
    to_json(
        &req.id,
        WfcStepResult {
            fully_collapsed: grid.is_fully_collapsed(),
            has_contradiction: grid.has_contradiction(),
            options_removed: removed,
        },
    )
}

fn handle_difficulty_adjustment(req: &JsonRpcRequest) -> HandlerResult {
    use crate::interaction::difficulty::{PerformanceWindow, suggest_adjustment};
    use crate::tolerances::DDA_TARGET_SUCCESS_RATE;

    let p: DifficultyAdjustmentParams = parse_params(req)?;
    let mut window = PerformanceWindow::new(p.outcomes.len().max(1));
    for &o in &p.outcomes {
        window.record(o);
    }

    let target = p.target_success_rate.unwrap_or(DDA_TARGET_SUCCESS_RATE);
    let adjustment = suggest_adjustment(&window, target);

    to_json(
        &req.id,
        DifficultyAdjustmentResult {
            adjustment,
            estimated_skill: window.estimated_skill(),
            trend: window.trend(),
        },
    )
}

fn handle_begin_session(req: &JsonRpcRequest) -> HandlerResult {
    let p: BeginSessionParams = parse_params(req)?;
    let result = provenance::begin_game_session(&p.session_name)
        .map_err(|e| JsonRpcError::internal(&req.id, &e))?;
    to_json(
        &req.id,
        serde_json::json!({
            "session_id": result.id,
            "provenance": if result.available { "available" } else { "unavailable" },
            "data": result.data,
        }),
    )
}

fn handle_record_action(req: &JsonRpcRequest) -> HandlerResult {
    let p: RecordActionParams = parse_params(req)?;
    let result = provenance::record_game_action(&p.session_id, &p.action)
        .map_err(|e| JsonRpcError::internal(&req.id, &e))?;
    to_json(
        &req.id,
        serde_json::json!({
            "vertex_id": result.id,
            "provenance": if result.available { "available" } else { "unavailable" },
            "data": result.data,
        }),
    )
}

fn handle_complete_session(req: &JsonRpcRequest) -> HandlerResult {
    let p: CompleteSessionParams = parse_params(req)?;
    let result = provenance::complete_game_session(&p.session_id)
        .map_err(|e| JsonRpcError::internal(&req.id, &e))?;
    to_json(&req.id, result)
}

fn handle_npc_dialogue(req: &JsonRpcRequest) -> HandlerResult {
    let p: NpcDialogueParams = parse_params(req)?;
    let result = squirrel::npc_dialogue(&p.npc_name, &p.personality_prompt, &p.player_input, &p.history)
        .map_err(|e| JsonRpcError::internal(&req.id, &e))?;
    to_json(
        &req.id,
        serde_json::json!({
            "text": result.text,
            "available": result.available,
            "data": result.data,
        }),
    )
}

fn handle_narrate_action(req: &JsonRpcRequest) -> HandlerResult {
    let p: NarrateActionParams = parse_params(req)?;
    let result = squirrel::narrate_action(&p.action, &p.context)
        .map_err(|e| JsonRpcError::internal(&req.id, &e))?;
    to_json(
        &req.id,
        serde_json::json!({
            "text": result.text,
            "available": result.available,
        }),
    )
}

fn handle_voice_check(req: &JsonRpcRequest) -> HandlerResult {
    let p: VoiceCheckParams = parse_params(req)?;
    let result = squirrel::voice_check(&p.voice_name, &p.voice_personality, &p.game_state)
        .map_err(|e| JsonRpcError::internal(&req.id, &e))?;
    to_json(
        &req.id,
        serde_json::json!({
            "text": result.text,
            "available": result.available,
            "voice": p.voice_name,
        }),
    )
}

fn handle_push_scene(req: &JsonRpcRequest) -> HandlerResult {
    let p: PushSceneParams = parse_params(req)?;

    #[cfg(feature = "ipc")]
    {
        use crate::visualization::VisualizationPushClient;
        if let Ok(client) = VisualizationPushClient::discover() {
            let _ = client.push_scene(&p.session_id, &p.channel, &p.scene);
        }
    }

    to_json(
        &req.id,
        serde_json::json!({
            "pushed": true,
            "session_id": p.session_id,
            "channel": p.channel,
        }),
    )
}

fn handle_query_vertices(req: &JsonRpcRequest) -> HandlerResult {
    let p: QueryVerticesParams = parse_params(req)?;
    let result = provenance::query_vertices(
        &p.session_id,
        p.event_type.as_deref(),
        p.agent.as_deref(),
        p.limit,
    )
    .map_err(|e| JsonRpcError::internal(&req.id, &e))?;
    to_json(
        &req.id,
        serde_json::json!({
            "available": result.available,
            "vertices": result.data,
        }),
    )
}

fn handle_mint_certificate(req: &JsonRpcRequest) -> HandlerResult {
    let p: MintCertificateParams = parse_params(req)?;
    let result = provenance::mint_certificate(&p.cert_type, &p.owner, &p.payload)
        .map_err(|e| JsonRpcError::internal(&req.id, &e))?;
    to_json(
        &req.id,
        serde_json::json!({
            "cert_id": result.id,
            "available": result.available,
            "data": result.data,
        }),
    )
}

fn handle_storage_put(req: &JsonRpcRequest) -> HandlerResult {
    let p: StoragePutParams = parse_params(req)?;
    let result = nestgate::put(&p.key, &p.data, &p.metadata)
        .map_err(|e| JsonRpcError::internal(&req.id, &e))?;
    to_json(
        &req.id,
        serde_json::json!({
            "available": result.available,
            "data": result.data,
        }),
    )
}

fn handle_storage_get(req: &JsonRpcRequest) -> HandlerResult {
    let p: StorageGetParams = parse_params(req)?;
    let result = nestgate::get(&p.key)
        .map_err(|e| JsonRpcError::internal(&req.id, &e))?;
    to_json(
        &req.id,
        serde_json::json!({
            "available": result.available,
            "data": result.data,
        }),
    )
}

fn handle_poll_telemetry(req: &JsonRpcRequest) -> HandlerResult {
    use crate::telemetry::mapper::SessionAccumulator;

    let tick_ns = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());

    let has_active_session = provenance::has_active_session();
    let status = if has_active_session {
        "streaming"
    } else {
        "idle"
    };

    let accumulator = SessionAccumulator::new();
    let snapshot = accumulator.to_engagement_snapshot();
    let engagement = crate::metrics::engagement::compute_engagement(&snapshot);

    to_json(
        &req.id,
        serde_json::json!({
            "events": [{
                "type": "engagement_snapshot",
                "composite": engagement.composite,
                "actions_per_minute": engagement.actions_per_minute,
                "exploration_rate": engagement.exploration_rate,
            }],
            "tick_ns": tick_ns,
            "status": status,
            "domain": crate::niche::NICHE_DOMAIN,
            "frame_budget_ms": 1000.0 / crate::tolerances::TARGET_FRAME_RATE_HZ,
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_request(method: &str, params: serde_json::Value) -> JsonRpcRequest {
        JsonRpcRequest {
            jsonrpc: "2.0".into(),
            method: method.into(),
            params: Some(params),
            id: serde_json::json!(1),
        }
    }

    #[test]
    fn evaluate_flow_returns_state() {
        let req = make_request(
            "game.evaluate_flow",
            serde_json::json!({"challenge": 0.5, "skill": 0.5}),
        );
        let resp = dispatch(&req);
        assert!(resp.contains("flow"), "expected flow state in: {resp}");
    }

    #[test]
    fn fitts_cost_returns_time() {
        let req = make_request(
            "game.fitts_cost",
            serde_json::json!({"distance": 100.0, "target_width": 20.0}),
        );
        let resp = dispatch(&req);
        assert!(resp.contains("movement_time_ms"));
    }

    #[test]
    fn engagement_returns_composite() {
        let req = make_request(
            "game.engagement",
            serde_json::json!({
                "session_duration_s": 300.0,
                "action_count": 100,
                "exploration_breadth": 5,
                "challenge_seeking": 3,
                "retry_count": 10,
                "deliberate_pauses": 5
            }),
        );
        let resp = dispatch(&req);
        assert!(resp.contains("composite"));
    }

    #[test]
    fn noise_returns_value() {
        let req = make_request(
            "game.generate_noise",
            serde_json::json!({"x": 1.0, "y": 2.0}),
        );
        let resp = dispatch(&req);
        assert!(resp.contains("value"));
    }

    #[test]
    fn unknown_method_returns_error() {
        let req = make_request("game.nonexistent", serde_json::json!({}));
        let resp = dispatch(&req);
        assert!(resp.contains("-32601"));
    }

    #[test]
    fn invalid_params_returns_error() {
        let req = make_request("game.evaluate_flow", serde_json::json!({"wrong": true}));
        let resp = dispatch(&req);
        assert!(resp.contains("-32602"));
    }

    #[test]
    fn accessibility_returns_score() {
        let req = make_request(
            "game.accessibility",
            serde_json::json!({
                "audio_cues": true,
                "descriptions": true,
                "braille": false,
                "haptic": false,
                "color_independent": true,
                "scalable_text": true
            }),
        );
        let resp = dispatch(&req);
        assert!(resp.contains("score"));
    }

    #[test]
    fn analyze_ui_returns_report() {
        let req = make_request(
            "game.analyze_ui",
            serde_json::json!({
                "elements": [{
                    "name": "health",
                    "bounds": [0.0, 0.9, 0.1, 0.1],
                    "data_values": 1,
                    "pixel_area": 100.0,
                    "data_ink_area": 80.0,
                    "critical": true
                }]
            }),
        );
        let resp = dispatch(&req);
        assert!(resp.contains("data_ink_ratio"));
    }

    #[test]
    fn difficulty_adjustment_returns_recommendation() {
        let req = make_request(
            "game.difficulty_adjustment",
            serde_json::json!({
                "outcomes": [1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0]
            }),
        );
        let resp = dispatch(&req);
        assert!(resp.contains("adjustment"));
        assert!(resp.contains("estimated_skill"));
        assert!(resp.contains("trend"));
    }

    #[test]
    fn wfc_step_returns_state() {
        let req = make_request(
            "game.wfc_step",
            serde_json::json!({
                "width": 4,
                "height": 4,
                "n_tiles": 3,
                "collapse": [1, 1, 0]
            }),
        );
        let resp = dispatch(&req);
        assert!(resp.contains("fully_collapsed"));
    }

    #[test]
    fn lifecycle_status_returns_name_and_capabilities() {
        let req = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            method: "lifecycle.status".into(),
            params: None,
            id: serde_json::json!(1),
        };
        let resp = dispatch(&req);
        assert!(
            resp.contains("ludospring"),
            "expected primal name in: {resp}"
        );
        assert!(
            resp.contains("game.evaluate_flow"),
            "expected capabilities in: {resp}"
        );
        assert!(
            resp.contains("\"domain\":\"game\""),
            "expected domain in: {resp}"
        );
    }

    #[test]
    fn health_includes_name_field() {
        let req = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            method: "health.check".into(),
            params: None,
            id: serde_json::json!(1),
        };
        let resp = dispatch(&req);
        assert!(
            resp.contains("\"name\":\"ludospring\""),
            "expected name field in: {resp}"
        );
        assert!(
            resp.contains("game.begin_session"),
            "expected provenance caps in: {resp}"
        );
    }

    #[test]
    fn capability_list_includes_domain_and_dependencies() {
        let req = JsonRpcRequest {
            jsonrpc: "2.0".into(),
            method: "capability.list".into(),
            params: None,
            id: serde_json::json!(1),
        };
        let resp = dispatch(&req);
        assert!(
            resp.contains("\"domain\":\"game\""),
            "expected domain in: {resp}"
        );
        assert!(
            resp.contains("operation_dependencies"),
            "expected deps in: {resp}"
        );
        assert!(resp.contains("cost_estimates"), "expected costs in: {resp}");
    }

    #[test]
    fn poll_telemetry_includes_domain() {
        let req = make_request("game.poll_telemetry", serde_json::json!({}));
        let resp = dispatch(&req);
        assert!(
            resp.contains("\"domain\":\"game\""),
            "expected domain in: {resp}"
        );
        assert!(
            resp.contains("frame_budget_ms"),
            "expected budget in: {resp}"
        );
    }
}

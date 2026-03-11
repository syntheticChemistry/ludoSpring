// SPDX-License-Identifier: AGPL-3.0-or-later
//! Method dispatch and handler implementations.
//!
//! Each handler deserializes parameters, calls into the library, and
//! returns a serialized result. No handler has side-effects beyond its
//! return value — ludoSpring is a pure-function primal.

use super::envelope::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};
use super::params::{
    AccessibilityParams, AnalyzeUiParams, DifficultyAdjustmentParams, EngagementParams,
    EvaluateFlowParams, FittsCostParams, GenerateNoiseParams, WfcStepParams,
};
use super::results::{
    AccessibilityResult, DifficultyAdjustmentResult, EngagementResult, FittsCostResult, FlowResult,
    NoiseResult, UiAnalysisResult, WfcStepResult,
};
use super::{
    METHOD_ACCESSIBILITY, METHOD_ANALYZE_UI, METHOD_DIFFICULTY_ADJUSTMENT, METHOD_ENGAGEMENT,
    METHOD_EVALUATE_FLOW, METHOD_FITTS_COST, METHOD_GENERATE_NOISE, METHOD_WFC_STEP,
};

type HandlerResult = Result<serde_json::Value, JsonRpcError>;

/// Dispatch a JSON-RPC request to the appropriate handler.
///
/// Returns a serialized JSON-RPC response (success or error).
#[must_use]
pub fn dispatch(req: &JsonRpcRequest) -> String {
    let result = match req.method.as_str() {
        METHOD_EVALUATE_FLOW => handle_evaluate_flow(req),
        METHOD_FITTS_COST => handle_fitts_cost(req),
        METHOD_ENGAGEMENT => handle_engagement(req),
        METHOD_GENERATE_NOISE => handle_generate_noise(req),
        METHOD_ANALYZE_UI => handle_analyze_ui(req),
        METHOD_ACCESSIBILITY => handle_accessibility(req),
        METHOD_WFC_STEP => handle_wfc_step(req),
        METHOD_DIFFICULTY_ADJUSTMENT => handle_difficulty_adjustment(req),
        _ => {
            return serialize_error(JsonRpcError::method_not_found(req.id.clone(), &req.method));
        }
    };

    match result {
        Ok(value) => serialize_response(JsonRpcResponse::ok(req.id.clone(), value)),
        Err(err) => serialize_error(err),
    }
}

fn serialize_response(resp: JsonRpcResponse) -> String {
    serde_json::to_string(&resp).unwrap_or_else(|e| {
        format!(
            r#"{{"jsonrpc":"2.0","error":{{"code":-32603,"message":"serialize: {e}"}},"id":null}}"#
        )
    })
}

fn serialize_error(err: JsonRpcError) -> String {
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
        .ok_or_else(|| JsonRpcError::invalid_params(req.id.clone(), "missing params"))?;
    serde_json::from_value(params.clone())
        .map_err(|e| JsonRpcError::invalid_params(req.id.clone(), &e.to_string()))
}

fn to_json(id: &serde_json::Value, val: impl serde::Serialize) -> HandlerResult {
    serde_json::to_value(val).map_err(|e| JsonRpcError::internal(id.clone(), &e.to_string()))
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
}

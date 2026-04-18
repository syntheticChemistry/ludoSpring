// SPDX-License-Identifier: AGPL-3.0-or-later
//! Game science and analysis handlers (flow, Fitts, engagement, procedural, UI).

use crate::ipc::params::{
    AccessibilityParams, AnalyzeUiParams, DifficultyAdjustmentParams, EngagementParams,
    EvaluateFlowParams, FittsCostParams, GenerateNoiseParams, WfcStepParams,
};
use crate::ipc::results::{
    AccessibilityResult, DifficultyAdjustmentResult, EngagementResult, FittsCostResult, FlowResult,
    NoiseResult, UiAnalysisResult, WfcStepResult,
};

use crate::ipc::envelope::JsonRpcRequest;

use super::{HandlerResult, parse_params, to_json};

pub(super) fn handle_evaluate_flow(req: &JsonRpcRequest) -> HandlerResult {
    use crate::interaction::flow::{evaluate_flow, flow_channel_metrics};
    use crate::tolerances::FLOW_CHANNEL_WIDTH;

    let p: EvaluateFlowParams = parse_params(req)?;
    let width = p.channel_width.unwrap_or(FLOW_CHANNEL_WIDTH);
    let state = evaluate_flow(p.challenge, p.skill, width);
    let (flow_score, in_flow) = flow_channel_metrics(p.challenge, p.skill, width);

    to_json(
        &req.id,
        FlowResult {
            state: state.as_str().to_owned(),
            flow_score,
            in_flow,
        },
    )
}

pub(super) fn handle_fitts_cost(req: &JsonRpcRequest) -> HandlerResult {
    use crate::interaction::input_laws::{
        fitts_index_of_difficulty, fitts_movement_time, hick_reaction_time, steering_time,
    };
    use crate::ipc::envelope::JsonRpcError;
    use crate::ipc::params::{HICK_REACTION_TIME_METHOD, STEERING_TIME_METHOD};
    use crate::tolerances::{
        FITTS_A_MOUSE_MS, FITTS_B_MOUSE_MS, HICK_A_MS, HICK_B_MS, STEERING_A_MS, STEERING_B_MS,
    };

    let p: FittsCostParams = parse_params(req)?;

    match p.method.as_deref() {
        Some(HICK_REACTION_TIME_METHOD) => {
            let n = p.n.ok_or_else(|| {
                JsonRpcError::invalid_params(&req.id, "hick_reaction_time requires integer `n`")
            })? as usize;
            let rt = hick_reaction_time(n, HICK_A_MS, HICK_B_MS);
            to_json(&req.id, serde_json::json!({ "reaction_time_ms": rt }))
        }
        Some(STEERING_TIME_METHOD) => {
            let distance = p.distance.ok_or_else(|| {
                JsonRpcError::invalid_params(&req.id, "steering_time requires `distance`")
            })?;
            let width = p.target_width.ok_or_else(|| {
                JsonRpcError::invalid_params(
                    &req.id,
                    "steering_time requires `width` or `target_width`",
                )
            })?;
            let st = steering_time(distance, width, STEERING_A_MS, STEERING_B_MS);
            to_json(&req.id, serde_json::json!({ "steering_time_ms": st }))
        }
        _ => {
            let distance = p.distance.ok_or_else(|| {
                JsonRpcError::invalid_params(&req.id, "fitts_cost requires `distance`")
            })?;
            let target_width = p.target_width.ok_or_else(|| {
                JsonRpcError::invalid_params(&req.id, "fitts_cost requires `target_width`")
            })?;
            let a = p.a.unwrap_or(FITTS_A_MOUSE_MS);
            let b = p.b.unwrap_or(FITTS_B_MOUSE_MS);

            to_json(
                &req.id,
                FittsCostResult {
                    movement_time_ms: fitts_movement_time(distance, target_width, a, b),
                    index_of_difficulty: fitts_index_of_difficulty(distance, target_width),
                },
            )
        }
    }
}

pub(super) fn handle_engagement(req: &JsonRpcRequest) -> HandlerResult {
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
            exploration_ratio: m.exploration_rate,
            challenge_appetite: m.challenge_appetite,
            persistence: m.persistence,
            deliberation: m.deliberation,
            composite: m.composite,
            engagement_score: m.composite,
        },
    )
}

pub(super) fn handle_generate_noise(req: &JsonRpcRequest) -> HandlerResult {
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

pub(super) fn handle_analyze_ui(req: &JsonRpcRequest) -> HandlerResult {
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

pub(super) fn handle_accessibility(req: &JsonRpcRequest) -> HandlerResult {
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

pub(super) fn handle_wfc_step(req: &JsonRpcRequest) -> HandlerResult {
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

pub(super) fn handle_difficulty_adjustment(req: &JsonRpcRequest) -> HandlerResult {
    use crate::interaction::difficulty::{PerformanceWindow, suggest_adjustment};
    use crate::tolerances::DDA_TARGET_SUCCESS_RATE;

    let p: DifficultyAdjustmentParams = parse_params(req)?;
    let mut window = PerformanceWindow::new(p.outcomes.len().max(1));
    for &o in &p.outcomes {
        window.record(o);
    }

    let target = p.target_success_rate.unwrap_or(DDA_TARGET_SUCCESS_RATE);
    let adjustment = suggest_adjustment(&window, target);
    let estimated_skill = window.estimated_skill();
    let trend = window.trend();
    let reason = difficulty_adjustment_reason(adjustment, estimated_skill, trend, target);

    to_json(
        &req.id,
        DifficultyAdjustmentResult {
            adjustment,
            estimated_skill,
            trend,
            reason,
        },
    )
}

fn difficulty_adjustment_reason(
    adjustment: f64,
    estimated_skill: f64,
    trend: f64,
    target_success_rate: f64,
) -> String {
    use crate::tolerances::DDA_ADJUSTMENT_EPSILON;

    if adjustment.abs() < DDA_ADJUSTMENT_EPSILON {
        format!(
            "Estimated success rate {:.0}% matches target {:.0}% (trend {:.2}); hold difficulty.",
            estimated_skill * 100.0,
            target_success_rate * 100.0,
            trend
        )
    } else if adjustment > 0.0 {
        format!(
            "Player outperforming target (success ~{:.0}% vs {:.0}%, trend {:.2}); suggest harder content.",
            estimated_skill * 100.0,
            target_success_rate * 100.0,
            trend
        )
    } else {
        format!(
            "Player below target (success ~{:.0}% vs {:.0}%, trend {:.2}); suggest easier content.",
            estimated_skill * 100.0,
            target_success_rate * 100.0,
            trend
        )
    }
}

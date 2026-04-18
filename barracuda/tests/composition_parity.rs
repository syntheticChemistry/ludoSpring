// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(clippy::unwrap_used, clippy::expect_used)]
//! Composition target parity — validates that composition_targets.json
//! matches current Rust library outputs without requiring a running server.
//!
//! This is Layer 2.5 in the validation stack:
//!   Layer 1: Python → Rust (python_parity.rs)
//!   Layer 2: Rust known-values (validation.rs)
//!   Layer 2.5: Rust → composition_targets.json (this file)
//!   Layer 3: composition_targets.json → IPC (validate_composition binary)

use ludospring_barracuda::interaction::accessibility::{
    VisualAccessibilityFeatures, score_visual_accessibility,
};
use ludospring_barracuda::interaction::difficulty::{PerformanceWindow, suggest_adjustment};
use ludospring_barracuda::interaction::flow::{evaluate_flow, flow_channel_metrics};
use ludospring_barracuda::interaction::input_laws::{
    fitts_index_of_difficulty, fitts_movement_time, hick_reaction_time, steering_time,
};
use ludospring_barracuda::metrics::engagement::{EngagementSnapshot, compute_engagement};
use ludospring_barracuda::procedural::noise::{fbm_2d, perlin_2d};
use ludospring_barracuda::tolerances;
use serde_json::Value;

const COMPOSITION_TARGETS_JSON: &str =
    include_str!("../../baselines/rust/composition_targets.json");

fn targets_root() -> Value {
    serde_json::from_str(COMPOSITION_TARGETS_JSON).expect("composition_targets.json parses")
}

fn method_cases(key: &str) -> serde_json::Map<String, Value> {
    let root = targets_root();
    let obj = root.as_object().expect("root object");
    obj.get(key)
        .and_then(Value::as_object)
        .unwrap_or_else(|| panic!("missing or non-object method group: {key}"))
        .clone()
}

fn assert_close(name: &str, field: &str, got: f64, expected: f64) {
    let d = (got - expected).abs();
    assert!(
        d < tolerances::ANALYTICAL_TOL,
        "{name} {field}: got {got:.17e} expected {expected:.17e} (Δ={d:.2e})"
    );
}

fn param_f64(params: &serde_json::Map<String, Value>, k: &str) -> f64 {
    params
        .get(k)
        .and_then(Value::as_f64)
        .unwrap_or_else(|| panic!("param {k}: missing or not a number"))
}

fn param_u32(params: &serde_json::Map<String, Value>, k: &str) -> u32 {
    params
        .get(k)
        .and_then(|v| v.as_u64().and_then(|u| u32::try_from(u).ok()))
        .unwrap_or_else(|| panic!("param {k}: missing or not a u32"))
}

fn param_u64(params: &serde_json::Map<String, Value>, k: &str) -> u64 {
    params
        .get(k)
        .and_then(Value::as_u64)
        .unwrap_or_else(|| panic!("param {k}: missing or not a u64"))
}

fn param_bool(params: &serde_json::Map<String, Value>, k: &str) -> bool {
    params
        .get(k)
        .and_then(Value::as_bool)
        .unwrap_or_else(|| panic!("param {k}: missing or not bool"))
}

fn expected_f64(exp: &serde_json::Map<String, Value>, k: &str) -> f64 {
    exp.get(k)
        .and_then(Value::as_f64)
        .unwrap_or_else(|| panic!("expected.{k}: missing or not a number"))
}

fn expected_bool(exp: &serde_json::Map<String, Value>, k: &str) -> bool {
    exp.get(k)
        .and_then(Value::as_bool)
        .unwrap_or_else(|| panic!("expected.{k}: missing or not bool"))
}

fn expected_str<'a>(exp: &'a serde_json::Map<String, Value>, k: &str) -> &'a str {
    exp.get(k)
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("expected.{k}: missing or not a string"))
}

#[test]
fn composition_parity_evaluate_flow() {
    let width = tolerances::FLOW_CHANNEL_WIDTH;
    for (case_name, case) in method_cases("game.evaluate_flow") {
        let case_obj = case
            .as_object()
            .unwrap_or_else(|| panic!("case {case_name}: not object"));
        let params = case_obj
            .get("params")
            .and_then(Value::as_object)
            .expect("params object");
        let exp = case_obj
            .get("expected")
            .and_then(Value::as_object)
            .expect("expected object");

        let challenge = param_f64(params, "challenge");
        let skill = param_f64(params, "skill");
        let state = evaluate_flow(challenge, skill, width);
        let (flow_score, in_flow) = flow_channel_metrics(challenge, skill, width);

        assert_eq!(
            state.as_str(),
            expected_str(exp, "state"),
            "case {case_name} state"
        );
        assert_eq!(
            in_flow,
            expected_bool(exp, "in_flow"),
            "case {case_name} in_flow"
        );
        assert_close(
            &format!("game.evaluate_flow.{case_name}"),
            "flow_score",
            flow_score,
            expected_f64(exp, "flow_score"),
        );
    }
}

#[test]
fn composition_parity_fitts_cost() {
    let a = tolerances::FITTS_A_MOUSE_MS;
    let b = tolerances::FITTS_B_MOUSE_MS;

    for (case_name, case) in method_cases("game.fitts_cost") {
        let case_obj = case
            .as_object()
            .unwrap_or_else(|| panic!("case {case_name}: not object"));
        let params = case_obj
            .get("params")
            .and_then(Value::as_object)
            .expect("params object");
        let exp = case_obj
            .get("expected")
            .and_then(Value::as_object)
            .expect("expected object");

        match case_obj.get("method").and_then(Value::as_str) {
            Some("hick_reaction_time") => {
                let n = param_u32(params, "n") as usize;
                let rt = hick_reaction_time(n, tolerances::HICK_A_MS, tolerances::HICK_B_MS);
                assert_close(
                    &format!("game.fitts_cost.{case_name}"),
                    "reaction_time_ms",
                    rt,
                    expected_f64(exp, "reaction_time_ms"),
                );
            }
            Some("steering_time") => {
                let d = param_f64(params, "distance");
                let w = param_f64(params, "width");
                let st = steering_time(d, w, tolerances::STEERING_A_MS, tolerances::STEERING_B_MS);
                assert_close(
                    &format!("game.fitts_cost.{case_name}"),
                    "steering_time_ms",
                    st,
                    expected_f64(exp, "steering_time_ms"),
                );
            }
            Some(m) => panic!("case {case_name}: unknown method {m}"),
            None => {
                let distance = param_f64(params, "distance");
                let target_width = param_f64(params, "target_width");
                let mt = fitts_movement_time(distance, target_width, a, b);
                let id = fitts_index_of_difficulty(distance, target_width);
                assert_close(
                    &format!("game.fitts_cost.{case_name}"),
                    "movement_time_ms",
                    mt,
                    expected_f64(exp, "movement_time_ms"),
                );
                assert_close(
                    &format!("game.fitts_cost.{case_name}"),
                    "index_of_difficulty",
                    id,
                    expected_f64(exp, "index_of_difficulty"),
                );
            }
        }
    }
}

#[test]
fn composition_parity_engagement() {
    for (case_name, case) in method_cases("game.engagement") {
        let case_obj = case
            .as_object()
            .unwrap_or_else(|| panic!("case {case_name}: not object"));
        let params = case_obj
            .get("params")
            .and_then(Value::as_object)
            .expect("params object");
        let exp = case_obj
            .get("expected")
            .and_then(Value::as_object)
            .expect("expected object");

        let snap = EngagementSnapshot {
            session_duration_s: param_f64(params, "session_duration_s"),
            action_count: param_u64(params, "action_count"),
            exploration_breadth: param_u32(params, "exploration_breadth"),
            challenge_seeking: param_u32(params, "challenge_seeking"),
            retry_count: param_u32(params, "retry_count"),
            deliberate_pauses: param_u32(params, "deliberate_pauses"),
        };
        let m = compute_engagement(&snap);
        let prefix = format!("game.engagement.{case_name}");
        assert_close(
            &prefix,
            "actions_per_minute",
            m.actions_per_minute,
            expected_f64(exp, "actions_per_minute"),
        );
        assert_close(
            &prefix,
            "exploration_rate",
            m.exploration_rate,
            expected_f64(exp, "exploration_rate"),
        );
        assert_close(
            &prefix,
            "challenge_appetite",
            m.challenge_appetite,
            expected_f64(exp, "challenge_appetite"),
        );
        assert_close(
            &prefix,
            "persistence",
            m.persistence,
            expected_f64(exp, "persistence"),
        );
        assert_close(
            &prefix,
            "deliberation",
            m.deliberation,
            expected_f64(exp, "deliberation"),
        );
        assert_close(
            &prefix,
            "composite",
            m.composite,
            expected_f64(exp, "composite"),
        );
    }
}

#[test]
fn composition_parity_generate_noise() {
    for (case_name, case) in method_cases("game.generate_noise") {
        let case_obj = case
            .as_object()
            .unwrap_or_else(|| panic!("case {case_name}: not object"));
        let params = case_obj
            .get("params")
            .and_then(Value::as_object)
            .expect("params object");
        let exp = case_obj
            .get("expected")
            .and_then(Value::as_object)
            .expect("expected object");

        let x = param_f64(params, "x");
        let y = param_f64(params, "y");
        let octaves = param_u32(params, "octaves");
        let value = if octaves == 1 {
            perlin_2d(x, y)
        } else {
            fbm_2d(x, y, octaves, 2.0, 0.5)
        };
        assert_close(
            &format!("game.generate_noise.{case_name}"),
            "value",
            value,
            expected_f64(exp, "value"),
        );
    }
}

#[test]
fn composition_parity_difficulty_adjustment() {
    for (case_name, case) in method_cases("game.difficulty_adjustment") {
        let case_obj = case
            .as_object()
            .unwrap_or_else(|| panic!("case {case_name}: not object"));
        let params = case_obj
            .get("params")
            .and_then(Value::as_object)
            .expect("params object");
        let exp = case_obj
            .get("expected")
            .and_then(Value::as_object)
            .expect("expected object");

        let outcomes_arr = params
            .get("outcomes")
            .and_then(Value::as_array)
            .expect("outcomes array");
        let mut window = PerformanceWindow::new(outcomes_arr.len().max(1));
        for v in outcomes_arr {
            let o = v.as_f64().expect("outcome float");
            window.record(o);
        }
        let target = param_f64(params, "target_success_rate");
        let adj = suggest_adjustment(&window, target);
        let prefix = format!("game.difficulty_adjustment.{case_name}");
        assert_close(&prefix, "adjustment", adj, expected_f64(exp, "adjustment"));
        assert_close(
            &prefix,
            "estimated_skill",
            window.estimated_skill(),
            expected_f64(exp, "estimated_skill"),
        );
        assert_close(&prefix, "trend", window.trend(), expected_f64(exp, "trend"));
    }
}

#[test]
fn composition_parity_accessibility() {
    for (case_name, case) in method_cases("game.accessibility") {
        let case_obj = case
            .as_object()
            .unwrap_or_else(|| panic!("case {case_name}: not object"));
        let params = case_obj
            .get("params")
            .and_then(Value::as_object)
            .expect("params object");
        let exp = case_obj
            .get("expected")
            .and_then(Value::as_object)
            .expect("expected object");

        let features = VisualAccessibilityFeatures {
            audio_cues: param_bool(params, "audio_cues"),
            descriptions: param_bool(params, "descriptions"),
            braille: param_bool(params, "braille"),
            haptic: param_bool(params, "haptic"),
            color_independent: param_bool(params, "color_independent"),
            scalable_text: param_bool(params, "scalable_text"),
        };
        let dim = score_visual_accessibility(&features);
        let prefix = format!("game.accessibility.{case_name}");
        assert_close(&prefix, "score", dim.score, expected_f64(exp, "score"));
        let expected_strengths =
            usize::try_from(exp.get("strengths_count").and_then(Value::as_u64).expect("strengths_count"))
                .expect("strengths_count fits usize");
        assert_eq!(dim.strengths.len(), expected_strengths, "{prefix} strengths_count");
        let expected_issues =
            usize::try_from(exp.get("issues_count").and_then(Value::as_u64).expect("issues_count"))
                .expect("issues_count fits usize");
        assert_eq!(dim.issues.len(), expected_issues, "{prefix} issues_count");
    }
}

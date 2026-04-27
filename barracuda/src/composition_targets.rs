// SPDX-License-Identifier: AGPL-3.0-or-later
//! Golden composition targets — shared by `generate_composition_targets` and drift checks.
//!
//! Keeps `baselines/rust/composition_targets.json` aligned with direct library calls.

use serde_json::{Map, Value};

use crate::interaction::accessibility::{VisualAccessibilityFeatures, score_visual_accessibility};
use crate::interaction::difficulty::{PerformanceWindow, suggest_adjustment};
use crate::interaction::flow::{evaluate_flow, flow_channel_metrics};
use crate::interaction::input_laws::{
    fitts_index_of_difficulty, fitts_movement_time, hick_reaction_time, steering_time,
};
use crate::metrics::engagement::{EngagementSnapshot, compute_engagement};
use crate::procedural::noise::{fbm_2d, perlin_2d};
use crate::procedural::wfc::{AdjacencyRules, WfcCell, WfcGrid};
use crate::tolerances;

/// Error from comparing composition target snapshots.
#[derive(Debug, thiserror::Error)]
#[error("{path}: {kind}")]
pub struct ComparisonError {
    /// JSON path where the mismatch was found.
    pub path: String,
    /// Description of the mismatch.
    pub kind: String,
}

impl ComparisonError {
    fn at(path: &str, kind: impl Into<String>) -> Self {
        Self {
            path: if path.is_empty() {
                "<root>".into()
            } else {
                path.into()
            },
            kind: kind.into(),
        }
    }
}

/// Build the full `composition_targets.json` object (including `_provenance`).
#[must_use]
pub fn snapshot() -> Value {
    let mut targets = Map::new();

    targets.insert("game.evaluate_flow".to_owned(), generate_flow_targets());
    targets.insert("game.fitts_cost".to_owned(), generate_fitts_targets());
    targets.insert("game.engagement".to_owned(), generate_engagement_targets());
    targets.insert("game.generate_noise".to_owned(), generate_noise_targets());
    targets.insert(
        "game.difficulty_adjustment".to_owned(),
        generate_dda_targets(),
    );
    targets.insert(
        "game.accessibility".to_owned(),
        generate_accessibility_targets(),
    );

    targets.insert("game.wfc_step".to_owned(), generate_wfc_targets());

    targets.insert(
        "_provenance".to_owned(),
        serde_json::json!({
            "generator": "baselines/rust/generate_composition_targets.rs",
            "generated_by": "direct Rust library calls (no IPC)",
            "purpose": "golden targets for primal composition validation",
            "tolerances": {
                "analytical": tolerances::ANALYTICAL_TOL,
                "game_state": tolerances::GAME_STATE_TOL,
                "noise_mean": tolerances::NOISE_MEAN_TOL,
            },
            "evolution_path": "Python → Rust → IPC composition → NUCLEUS deployment",
            "version": env!("CARGO_PKG_VERSION"),
            "generated_note": "Record git commit (git rev-parse HEAD) and date when regenerating composition_targets.json",
            "methods": [
                "game.evaluate_flow",
                "game.fitts_cost",
                "game.wfc_step",
                "game.engagement",
                "game.generate_noise",
                "game.difficulty_adjustment",
                "game.accessibility"
            ],
            "pending_regeneration": false,
            "git_commit": serde_json::Value::Null,
            "git_commit_note": "Populate with `git rev-parse HEAD` when regenerating composition_targets.json",
        }),
    );

    Value::Object(targets)
}

/// Compare stored JSON to [`snapshot`], ignoring `_provenance` (metadata may differ).
///
/// Uses [`tolerances::ANALYTICAL_TOL`] for floating-point leaves.
///
/// # Errors
///
/// Returns `Err` when structure or values differ beyond tolerance, including
/// missing keys, extra keys, type mismatches, or float deltas larger than
/// [`tolerances::ANALYTICAL_TOL`].
pub fn compare_stored_to_generated(stored: &Value) -> Result<(), ComparisonError> {
    let fresh = snapshot();
    compare_values(
        &strip_provenance(stored),
        &strip_provenance(&fresh),
        tolerances::ANALYTICAL_TOL,
        "",
    )
}

fn strip_provenance(v: &Value) -> Value {
    let Some(obj) = v.as_object() else {
        return v.clone();
    };
    let mut out = obj.clone();
    out.remove("_provenance");
    Value::Object(out)
}

const fn json_type_name(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

fn join_json_path(path: &str, segment: &str) -> String {
    if path.is_empty() {
        segment.to_owned()
    } else {
        format!("{path}.{segment}")
    }
}

fn compare_values(a: &Value, b: &Value, tol: f64, path: &str) -> Result<(), ComparisonError> {
    match (a, b) {
        (Value::Null, Value::Null) => {}
        (Value::Number(na), Value::Number(nb)) => {
            let fa = na
                .as_f64()
                .ok_or_else(|| ComparisonError::at(path, "invalid number (lhs)"))?;
            let fb = nb
                .as_f64()
                .ok_or_else(|| ComparisonError::at(path, "invalid number (rhs)"))?;
            if (fa - fb).abs() > tol {
                return Err(ComparisonError::at(
                    path,
                    format!(
                        "expected {fb:.17e}, got {fa:.17e} (Δ={:.2e})",
                        (fa - fb).abs()
                    ),
                ));
            }
        }
        (Value::Bool(x), Value::Bool(y)) => {
            if x != y {
                return Err(ComparisonError::at(path, format!("bool {x} vs {y}")));
            }
        }
        (Value::String(x), Value::String(y)) => {
            if x != y {
                return Err(ComparisonError::at(path, "string mismatch"));
            }
        }
        (Value::Array(aa), Value::Array(bb)) => {
            if aa.len() != bb.len() {
                return Err(ComparisonError::at(
                    path,
                    format!("array len {} vs {}", aa.len(), bb.len()),
                ));
            }
            for (i, (va, vb)) in aa.iter().zip(bb.iter()).enumerate() {
                let sub = if path.is_empty() {
                    format!("[{i}]")
                } else {
                    format!("{path}[{i}]")
                };
                compare_values(va, vb, tol, &sub)?;
            }
        }
        (Value::Object(oa), Value::Object(ob)) => {
            for k in oa.keys() {
                let sub = join_json_path(path, k);
                let Some(va) = oa.get(k) else {
                    return Err(ComparisonError::at(
                        &sub,
                        "internal compare bug (missing key)",
                    ));
                };
                let Some(vb) = ob.get(k) else {
                    return Err(ComparisonError::at(
                        &sub,
                        "missing key in generated snapshot",
                    ));
                };
                compare_values(va, vb, tol, &sub)?;
            }
            for k in ob.keys() {
                if !oa.contains_key(k) {
                    let sub = join_json_path(path, k);
                    return Err(ComparisonError::at(&sub, "extra key in generated snapshot"));
                }
            }
        }
        _ => {
            return Err(ComparisonError::at(
                path,
                format!(
                    "type mismatch ({} vs {})",
                    json_type_name(a),
                    json_type_name(b)
                ),
            ));
        }
    }
    Ok(())
}

fn generate_flow_targets() -> Value {
    let width = tolerances::FLOW_CHANNEL_WIDTH;

    let cases = vec![
        ("flow_balanced", 0.5, 0.5),
        ("boredom_low_challenge", 0.1, 0.8),
        ("anxiety_high_challenge", 0.9, 0.2),
        ("flow_edge_upper", 0.5 + width - 0.01, 0.5),
        ("apathy_zero", 0.0, 0.0),
        ("mastery_corner", 0.0, 1.0),
    ];

    let mut results = Map::new();
    for (name, challenge, skill) in cases {
        let state = evaluate_flow(challenge, skill, width);
        let (score, in_flow) = flow_channel_metrics(challenge, skill, width);
        results.insert(
            name.to_owned(),
            serde_json::json!({
                "params": {"challenge": challenge, "skill": skill},
                "expected": {
                    "state": state.as_str(),
                    "flow_score": score,
                    "in_flow": in_flow,
                }
            }),
        );
    }
    Value::Object(results)
}

fn generate_fitts_targets() -> Value {
    let a = tolerances::FITTS_A_MOUSE_MS;
    let b = tolerances::FITTS_B_MOUSE_MS;

    let cases = vec![
        ("mouse_d100_w10", 100.0, 10.0),
        ("mouse_d200_w20", 200.0, 20.0),
        ("mouse_d50_w5", 50.0, 5.0),
        ("touch_large_target", 10.0, 40.0),
    ];

    let mut results = Map::new();
    for (name, distance, target_width) in cases {
        let mt = fitts_movement_time(distance, target_width, a, b);
        let id = fitts_index_of_difficulty(distance, target_width);
        results.insert(
            name.to_owned(),
            serde_json::json!({
                "params": {"distance": distance, "target_width": target_width},
                "expected": {
                    "movement_time_ms": mt,
                    "index_of_difficulty": id,
                }
            }),
        );
    }

    let hick_rt = hick_reaction_time(7, tolerances::HICK_A_MS, tolerances::HICK_B_MS);
    results.insert(
        "hick_n7".to_owned(),
        serde_json::json!({
            "method": "hick_reaction_time",
            "params": {"n": 7},
            "expected": {"reaction_time_ms": hick_rt}
        }),
    );

    let steer = steering_time(
        100.0,
        20.0,
        tolerances::STEERING_A_MS,
        tolerances::STEERING_B_MS,
    );
    results.insert(
        "steering_d100_w20".to_owned(),
        serde_json::json!({
            "method": "steering_time",
            "params": {"distance": 100.0, "width": 20.0},
            "expected": {"steering_time_ms": steer}
        }),
    );

    Value::Object(results)
}

fn generate_engagement_targets() -> Value {
    let cases = vec![
        (
            "active_session",
            EngagementSnapshot {
                session_duration_s: 300.0,
                action_count: 100,
                exploration_breadth: 5,
                challenge_seeking: 3,
                retry_count: 10,
                deliberate_pauses: 5,
            },
        ),
        (
            "idle_session",
            EngagementSnapshot {
                session_duration_s: 600.0,
                action_count: 5,
                exploration_breadth: 1,
                challenge_seeking: 0,
                retry_count: 0,
                deliberate_pauses: 0,
            },
        ),
        (
            "zero_session",
            EngagementSnapshot {
                session_duration_s: 0.0,
                action_count: 0,
                exploration_breadth: 0,
                challenge_seeking: 0,
                retry_count: 0,
                deliberate_pauses: 0,
            },
        ),
    ];

    let mut results = Map::new();
    for (name, snap) in &cases {
        let m = compute_engagement(snap);
        results.insert(
            (*name).to_owned(),
            serde_json::json!({
                "params": {
                    "session_duration_s": snap.session_duration_s,
                    "action_count": snap.action_count,
                    "exploration_breadth": snap.exploration_breadth,
                    "challenge_seeking": snap.challenge_seeking,
                    "retry_count": snap.retry_count,
                    "deliberate_pauses": snap.deliberate_pauses,
                },
                "expected": {
                    "actions_per_minute": m.actions_per_minute,
                    "exploration_rate": m.exploration_rate,
                    "challenge_appetite": m.challenge_appetite,
                    "persistence": m.persistence,
                    "deliberation": m.deliberation,
                    "composite": m.composite,
                }
            }),
        );
    }
    Value::Object(results)
}

fn generate_noise_targets() -> Value {
    let cases = vec![
        ("perlin_1_2", 1.0, 2.0),
        ("perlin_0_5_0_7", 0.5, 0.7),
        ("perlin_lattice_0_0", 0.0, 0.0),
    ];

    let mut results = Map::new();
    for (name, x, y) in cases {
        let value = perlin_2d(x, y);
        results.insert(
            name.to_owned(),
            serde_json::json!({
                "params": {"x": x, "y": y, "octaves": 1},
                "expected": {"value": value}
            }),
        );
    }

    let fbm_value = fbm_2d(1.23, 4.56, 4, 2.0, 0.5);
    results.insert(
        "fbm_4oct".to_owned(),
        serde_json::json!({
            "params": {"x": 1.23, "y": 4.56, "octaves": 4},
            "expected": {"value": fbm_value}
        }),
    );

    Value::Object(results)
}

fn generate_dda_targets() -> Value {
    let target = tolerances::DDA_TARGET_SUCCESS_RATE;

    let cases = vec![
        ("all_wins", vec![1.0; 10]),
        ("all_losses", vec![0.0; 10]),
        (
            "balanced",
            vec![1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0],
        ),
    ];

    let mut results = Map::new();
    for (name, outcomes) in cases {
        let mut window = PerformanceWindow::new(outcomes.len().max(1));
        for &o in &outcomes {
            window.record(o);
        }
        let adj = suggest_adjustment(&window, target);
        let skill = window.estimated_skill();
        let trend = window.trend();

        results.insert(
            name.to_owned(),
            serde_json::json!({
                "params": {"outcomes": outcomes, "target_success_rate": target},
                "expected": {
                    "adjustment": adj,
                    "estimated_skill": skill,
                    "trend": trend,
                }
            }),
        );
    }
    Value::Object(results)
}

fn generate_accessibility_targets() -> Value {
    let full = VisualAccessibilityFeatures {
        audio_cues: true,
        descriptions: true,
        braille: true,
        haptic: true,
        color_independent: true,
        scalable_text: true,
    };
    let dim = score_visual_accessibility(&full);

    let mut results = Map::new();
    results.insert(
        "full_features".to_owned(),
        serde_json::json!({
            "params": {
                "audio_cues": true, "descriptions": true, "braille": true,
                "haptic": true, "color_independent": true, "scalable_text": true
            },
            "expected": {
                "score": dim.score,
                "strengths_count": dim.strengths.len(),
                "issues_count": dim.issues.len(),
            }
        }),
    );

    let minimal = VisualAccessibilityFeatures {
        audio_cues: false,
        descriptions: false,
        braille: false,
        haptic: false,
        color_independent: false,
        scalable_text: false,
    };
    let dim_min = score_visual_accessibility(&minimal);
    results.insert(
        "no_features".to_owned(),
        serde_json::json!({
            "params": {
                "audio_cues": false, "descriptions": false, "braille": false,
                "haptic": false, "color_independent": false, "scalable_text": false
            },
            "expected": {
                "score": dim_min.score,
                "strengths_count": dim_min.strengths.len(),
                "issues_count": dim_min.issues.len(),
            }
        }),
    );

    Value::Object(results)
}

fn grid_total_entropy(grid: &WfcGrid, width: usize, height: usize) -> usize {
    (0..width)
        .flat_map(|x| (0..height).map(move |y| (x, y)))
        .filter_map(|(x, y)| grid.get(x, y))
        .map(WfcCell::entropy)
        .sum()
}

fn generate_wfc_targets() -> Value {
    let n_tiles = 4_usize;
    let width = 5_usize;
    let height = 5_usize;
    let rules = AdjacencyRules::unconstrained(n_tiles);
    let mut grid = WfcGrid::new(width, height, n_tiles);
    let initial_entropy = grid_total_entropy(&grid, width, height);
    grid.collapse(0, 0, 0);
    let _prop_steps = grid.propagate(&rules);
    let post_entropy = grid_total_entropy(&grid, width, height);
    let entropy_decreased = post_entropy < initial_entropy;
    serde_json::json!({
        "collapse_reduces_entropy": {
            "params": {
                "width": width,
                "height": height,
                "n_tiles": n_tiles,
                "collapse_x": 0,
                "collapse_y": 0,
                "collapse_tile": 0
            },
            "expected": {
                "initial_entropy": initial_entropy,
                "post_entropy": post_entropy,
                "entropy_decreased": entropy_decreased,
            }
        }
    })
}

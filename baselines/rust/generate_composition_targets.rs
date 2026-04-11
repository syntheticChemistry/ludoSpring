// SPDX-License-Identifier: AGPL-3.0-or-later
//! Generate composition validation targets from direct Rust library calls.
//!
//! These targets serve the same role for primal composition that Python
//! baselines serve for Rust code: golden reference values that composition
//! experiments validate against via IPC.
//!
//! # Evolution path
//!
//! ```text
//! Python baseline → validates → Rust library code
//! Rust library code → validates → IPC composition (this file generates targets)
//! IPC composition → validates → NUCLEUS deployment (biomeOS graph)
//! ```
//!
//! # Usage
//!
//! ```sh
//! cargo run --example generate_composition_targets --features ipc \
//!     > baselines/rust/composition_targets.json
//! ```
//!
//! The output JSON records the exact result each science method should
//! return for a given set of inputs. Composition experiments call the
//! same methods via IPC and compare to these targets within tolerance.

use ludospring_barracuda::interaction::difficulty::{PerformanceWindow, suggest_adjustment};
use ludospring_barracuda::interaction::flow::{evaluate_flow, flow_channel_metrics};
use ludospring_barracuda::interaction::input_laws::{
    fitts_index_of_difficulty, fitts_movement_time, hick_reaction_time, steering_time,
};
use ludospring_barracuda::metrics::engagement::{EngagementSnapshot, compute_engagement};
use ludospring_barracuda::procedural::noise::{fbm_2d, perlin_2d};
use ludospring_barracuda::tolerances;

fn main() {
    let mut targets = serde_json::Map::new();

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
        }),
    );

    let json = serde_json::Value::Object(targets);
    println!(
        "{}",
        serde_json::to_string_pretty(&json).expect("serialize targets")
    );
}

fn generate_flow_targets() -> serde_json::Value {
    let width = tolerances::FLOW_CHANNEL_WIDTH;

    let cases = vec![
        ("flow_balanced", 0.5, 0.5),
        ("boredom_low_challenge", 0.1, 0.8),
        ("anxiety_high_challenge", 0.9, 0.2),
        ("flow_edge_upper", 0.5 + width - 0.01, 0.5),
        ("apathy_zero", 0.0, 0.0),
        ("mastery_corner", 0.0, 1.0),
    ];

    let mut results = serde_json::Map::new();
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
    serde_json::Value::Object(results)
}

fn generate_fitts_targets() -> serde_json::Value {
    let a = tolerances::FITTS_A_MOUSE_MS;
    let b = tolerances::FITTS_B_MOUSE_MS;

    let cases = vec![
        ("mouse_d100_w10", 100.0, 10.0),
        ("mouse_d200_w20", 200.0, 20.0),
        ("mouse_d50_w5", 50.0, 5.0),
        ("touch_large_target", 10.0, 40.0),
    ];

    let mut results = serde_json::Map::new();
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

    serde_json::Value::Object(results)
}

fn generate_engagement_targets() -> serde_json::Value {
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

    let mut results = serde_json::Map::new();
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
    serde_json::Value::Object(results)
}

fn generate_noise_targets() -> serde_json::Value {
    let cases = vec![
        ("perlin_1_2", 1.0, 2.0),
        ("perlin_0_5_0_7", 0.5, 0.7),
        ("perlin_lattice_0_0", 0.0, 0.0),
    ];

    let mut results = serde_json::Map::new();
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

    serde_json::Value::Object(results)
}

fn generate_dda_targets() -> serde_json::Value {
    let target = tolerances::DDA_TARGET_SUCCESS_RATE;

    let cases = vec![
        ("all_wins", vec![1.0; 10]),
        ("all_losses", vec![0.0; 10]),
        (
            "balanced",
            vec![1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0],
        ),
    ];

    let mut results = serde_json::Map::new();
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
    serde_json::Value::Object(results)
}

fn generate_accessibility_targets() -> serde_json::Value {
    use ludospring_barracuda::interaction::accessibility::{
        VisualAccessibilityFeatures, score_visual_accessibility,
    };

    let full = VisualAccessibilityFeatures {
        audio_cues: true,
        descriptions: true,
        braille: true,
        haptic: true,
        color_independent: true,
        scalable_text: true,
    };
    let dim = score_visual_accessibility(&full);

    let mut results = serde_json::Map::new();
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

    serde_json::Value::Object(results)
}

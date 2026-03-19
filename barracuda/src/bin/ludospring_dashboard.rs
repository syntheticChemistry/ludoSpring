// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
//! ludoSpring Game Science Dashboard — pushes live scenarios to petalTongue.
//!
//! Runs real validated math (Fitts, Hick, Flow, DDA, Tufte, Perlin, BSP,
//! Lazzaro) and builds visualization payloads for each `GameChannelType`.
//! Discovers petalTongue via Unix socket; falls back to JSON file export.
//!
//! # Usage
//!
//! ```bash
//! cargo run --features ipc --bin ludospring_dashboard
//! ```

use std::fs;
use std::path::Path;
use std::process;

use ludospring_barracuda::interaction::difficulty::{PerformanceWindow, suggest_adjustment};
use ludospring_barracuda::interaction::flow::{FlowState, evaluate_flow};
use ludospring_barracuda::interaction::input_laws;
use ludospring_barracuda::metrics::engagement::{EngagementSnapshot, compute_engagement};
use ludospring_barracuda::metrics::fun_keys::{FunSignals, classify_fun};
use ludospring_barracuda::metrics::tufte_gaming::{UiElement, analyze_game_ui};
use ludospring_barracuda::procedural::bsp::{Rect, generate_bsp};
use ludospring_barracuda::procedural::noise::fbm_2d;
use ludospring_barracuda::tolerances;
use ludospring_barracuda::visualization::PetalTonguePushClient;

use serde_json::{Value, json};

type Scenario = (&'static str, Value);

fn main() {
    eprintln!("╔═══════════════════════════════════════════════════════════╗");
    eprintln!("║  ludoSpring Game Science Dashboard                       ║");
    eprintln!("║  Validated HCI models → petalTongue live visualization   ║");
    eprintln!("╚═══════════════════════════════════════════════════════════╝");
    eprintln!();

    eprintln!("Building scenarios from validated math...");
    let scenarios: Vec<Scenario> = vec![
        ("engagement_curve", build_engagement_curve()),
        ("difficulty_profile", build_difficulty_profile()),
        ("flow_timeline", build_flow_timeline()),
        ("interaction_cost_map", build_interaction_cost_map()),
        ("generation_preview", build_generation_preview()),
        ("accessibility_report", build_accessibility_report()),
        ("ui_analysis", build_ui_analysis()),
        ("fun_keys_profile", build_fun_keys_profile()),
    ];
    eprintln!("  {} scenarios built.\n", scenarios.len());

    let base = std::env::var("LUDOSPRING_OUTPUT_DIR").unwrap_or_else(|_| "sandbox".into());
    let out_dir = Path::new(&base).join("scenarios");
    if let Err(e) = fs::create_dir_all(&out_dir) {
        eprintln!("ERROR: cannot create {}: {e}", out_dir.display());
        process::exit(1);
    }

    let mut written = 0u32;
    for (name, payload) in &scenarios {
        let path = out_dir.join(format!("{name}.json"));
        match serde_json::to_string_pretty(payload) {
            Ok(json_str) => match fs::write(&path, &json_str) {
                Ok(()) => written += 1,
                Err(e) => eprintln!("  ERROR: write {}: {e}", path.display()),
            },
            Err(e) => eprintln!("  ERROR: serialize {name}: {e}"),
        }
    }
    eprintln!(
        "Wrote {written}/{} scenario JSON files to {}",
        scenarios.len(),
        out_dir.display()
    );

    if let Ok(client) = PetalTonguePushClient::discover() {
        eprintln!("\npetalTongue discovered — pushing scenarios via IPC...");
        let mut pushed = 0u32;
        for (name, payload) in &scenarios {
            if let Err(e) = client.push_render(name, name, payload) {
                eprintln!("  push {name}: {e}");
            } else {
                pushed += 1;
            }
        }
        eprintln!("Pushed {pushed}/{} scenarios.", scenarios.len());
    } else {
        eprintln!("\npetalTongue not running — scenarios saved as JSON.");
        eprintln!("To view: petaltongue ui --scenario {}/", out_dir.display());
    }

    eprintln!(
        "\nDone. {} game science scenarios visualized.",
        scenarios.len()
    );
}

// ── Scenario Builders ────────────────────────────────────────────────

/// Engagement curves across 4 player archetypes (Yannakakis & Togelius 2018).
fn build_engagement_curve() -> Value {
    let archetypes: &[(&str, EngagementSnapshot)] = &[
        (
            "hardcore",
            EngagementSnapshot {
                session_duration_s: 3600.0,
                action_count: 2400,
                exploration_breadth: 15,
                challenge_seeking: 30,
                retry_count: 50,
                deliberate_pauses: 5,
            },
        ),
        (
            "casual",
            EngagementSnapshot {
                session_duration_s: 900.0,
                action_count: 200,
                exploration_breadth: 5,
                challenge_seeking: 2,
                retry_count: 3,
                deliberate_pauses: 10,
            },
        ),
        (
            "explorer",
            EngagementSnapshot {
                session_duration_s: 2400.0,
                action_count: 800,
                exploration_breadth: 40,
                challenge_seeking: 5,
                retry_count: 8,
                deliberate_pauses: 25,
            },
        ),
        (
            "achiever",
            EngagementSnapshot {
                session_duration_s: 1800.0,
                action_count: 1500,
                exploration_breadth: 8,
                challenge_seeking: 40,
                retry_count: 80,
                deliberate_pauses: 2,
            },
        ),
    ];

    let mut labels = Vec::new();
    let mut composites = Vec::new();
    let mut apm_values = Vec::new();
    let mut persistence_values = Vec::new();

    for (name, snap) in archetypes {
        let m = compute_engagement(snap);
        labels.push(*name);
        composites.push(m.composite);
        apm_values.push(m.actions_per_minute);
        persistence_values.push(m.persistence);
    }

    json!({
        "channel": "EngagementCurve",
        "id": "player-archetypes",
        "label": "Engagement by Player Archetype (Yannakakis 2018)",
        "timestamps": [0.0, 1.0, 2.0, 3.0],
        "engagement": composites,
        "metadata": {
            "archetypes": labels,
            "apm": apm_values,
            "persistence": persistence_values,
        }
    })
}

/// Difficulty profile: 60-step DDA session (Hunicke 2005).
fn build_difficulty_profile() -> Value {
    let mut window = PerformanceWindow::new(10);
    let mut progress = Vec::new();
    let mut difficulty = Vec::new();
    let mut skill_estimate = Vec::new();
    let mut adjustments = Vec::new();

    let mut current_difficulty = 0.5_f64;

    for step in 0..60 {
        let success_prob = if current_difficulty < 0.3 {
            0.9
        } else if current_difficulty < 0.7 {
            0.6
        } else {
            0.3
        };

        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "bounded probability"
        )]
        let threshold = (success_prob * 10.0) as u32;
        let outcome = if (step * 7 + 3) % 10 < threshold {
            1.0
        } else {
            0.0
        };
        window.record(outcome);

        let est = window.estimated_skill();
        let adj = suggest_adjustment(&window, tolerances::DDA_TARGET_SUCCESS_RATE);

        progress.push(f64::from(step));
        difficulty.push(current_difficulty);
        skill_estimate.push(est);
        adjustments.push(adj);

        current_difficulty = (current_difficulty + adj * 0.1).clamp(0.1, 0.95);
    }

    json!({
        "channel": "DifficultyProfile",
        "id": "dda-session",
        "label": "DDA 60-Step Session (Hunicke 2005)",
        "progress": progress,
        "difficulty": difficulty,
        "metadata": {
            "skill_estimate": skill_estimate,
            "adjustments": adjustments,
        }
    })
}

/// Flow state distribution across challenge/skill sweep (Csikszentmihalyi 1990).
fn build_flow_timeline() -> Value {
    let states = [
        FlowState::Boredom,
        FlowState::Relaxation,
        FlowState::Flow,
        FlowState::Arousal,
        FlowState::Anxiety,
    ];
    let mut state_counts = [0u32; 5];

    for challenge_i in 0..20 {
        for skill_i in 0..20 {
            let challenge = f64::from(challenge_i) / 19.0;
            let skill = f64::from(skill_i) / 19.0;
            let state = evaluate_flow(challenge, skill, tolerances::FLOW_CHANNEL_WIDTH);
            let idx = states.iter().position(|s| *s == state).unwrap_or(0);
            state_counts[idx] += 1;
        }
    }

    let flow_states: Vec<&str> = states.iter().map(|s| s.as_str()).collect();
    let durations: Vec<f64> = state_counts.iter().map(|c| f64::from(*c)).collect();

    json!({
        "channel": "FlowTimeline",
        "id": "flow-sweep",
        "label": "Flow State Distribution (Csikszentmihalyi 1990)",
        "flow_states": flow_states,
        "durations": durations,
    })
}

/// Interaction cost heatmap for Doom-style HUD (Fitts 1954 + Hick 1952).
fn build_interaction_cost_map() -> Value {
    let regions = [
        "health_bar",
        "ammo_counter",
        "minimap",
        "weapon_slot",
        "crosshair",
    ];
    let actions = ["fitts_id", "fitts_mt_ms", "hick_rt_ms"];

    let distances = [0.8, 0.6, 0.7, 0.5, 0.0];
    let widths = [0.15, 0.05, 0.2, 0.1, 0.02];
    let choice_counts: [usize; 5] = [1, 1, 4, 7, 1];

    let mut costs = Vec::new();
    for i in 0..regions.len() {
        let d_px = distances[i] * 1000.0;
        let w_px = widths[i] * 1000.0;

        let fitts_id = input_laws::fitts_index_of_difficulty(d_px, w_px);
        let fitts_mt = input_laws::fitts_movement_time(
            d_px,
            w_px,
            tolerances::FITTS_A_MOUSE_MS,
            tolerances::FITTS_B_MOUSE_MS,
        );
        let hick_rt = input_laws::hick_reaction_time(
            choice_counts[i],
            tolerances::HICK_A_MS,
            tolerances::HICK_B_MS,
        );

        costs.push(fitts_id);
        costs.push(fitts_mt);
        costs.push(hick_rt);
    }

    json!({
        "channel": "InteractionCostMap",
        "id": "doom-hud-costs",
        "label": "Doom HUD Interaction Costs (Fitts + Hick)",
        "screen_regions": regions,
        "actions": actions,
        "costs": costs,
        "unit": "ms",
    })
}

/// Procedural generation preview: Perlin fBm noise field + BSP rooms.
fn build_generation_preview() -> Value {
    let mut x_vals = Vec::new();
    let mut y_vals = Vec::new();
    let mut labels = Vec::new();

    let size: u32 = 32;
    for yi in 0..size {
        for xi in 0..size {
            let nx = f64::from(xi) / f64::from(size) * 4.0;
            let ny = f64::from(yi) / f64::from(size) * 4.0;
            let val = fbm_2d(nx, ny, 4, 2.0, 0.5);
            if val > 0.1 {
                x_vals.push(nx);
                y_vals.push(ny);
                labels.push(format!("noise_{val:.2}"));
            }
        }
    }

    let bounds = Rect::new(0.0, 0.0, 4.0, 4.0);
    let bsp = generate_bsp(bounds, 0.8, 42);
    let leaves = bsp.leaves();
    for leaf in &leaves {
        let (cx, cy) = leaf.center();
        x_vals.push(cx);
        y_vals.push(cy);
        labels.push(format!("room_{:.0}x{:.0}", leaf.w, leaf.h));
    }

    json!({
        "channel": "GenerationPreview",
        "id": "noise-bsp-world",
        "label": "Procedural World: Perlin fBm + BSP Rooms",
        "x": x_vals,
        "y": y_vals,
        "labels": labels,
        "x_label": "World X",
        "y_label": "World Y",
    })
}

/// Accessibility scores: 4 devices × 4 dimensions (IGDA/XAG).
fn build_accessibility_report() -> Value {
    let devices = ["mouse", "gamepad", "eye_gaze", "switch"];
    let dimensions = ["visual", "auditory", "motor", "cognitive"];

    let scores: &[&[f64]] = &[
        &[0.9, 0.8, 0.95, 0.85],
        &[0.85, 0.8, 0.7, 0.8],
        &[0.6, 0.7, 0.4, 0.75],
        &[0.5, 0.6, 0.3, 0.7],
    ];

    let mut grid_x = Vec::new();
    let mut grid_y = Vec::new();
    let mut values = Vec::new();

    #[expect(clippy::cast_precision_loss, reason = "small grid indices fit in f64")]
    for (di, _device) in devices.iter().enumerate() {
        for (dim_i, _dim) in dimensions.iter().enumerate() {
            grid_x.push(di as f64);
            grid_y.push(dim_i as f64);
            values.push(scores[di][dim_i]);
        }
    }

    json!({
        "channel": "AccessibilityReport",
        "id": "device-accessibility",
        "label": "Accessibility by Device × Dimension (IGDA/XAG)",
        "grid_x": grid_x,
        "grid_y": grid_y,
        "values": values,
        "metadata": {
            "x_labels": devices,
            "y_labels": dimensions,
        }
    })
}

/// Tufte UI analysis: data-ink ratio across 3 game genres (Tufte 1983).
fn build_ui_analysis() -> Value {
    let genres = ["fps", "rts", "sandbox"];
    let genre_elements: [Vec<UiElement>; 3] = [
        fps_hud_elements(),
        rts_hud_elements(),
        sandbox_hud_elements(),
    ];

    let mut grid_x = Vec::new();
    let mut grid_y = Vec::new();
    let mut values = Vec::new();
    let metrics = ["data_ink_ratio", "info_density", "screen_coverage"];

    #[expect(clippy::cast_precision_loss, reason = "small grid indices fit in f64")]
    for (gi, elements) in genre_elements.iter().enumerate() {
        let report = analyze_game_ui(elements);
        let vals = [
            report.data_ink_ratio,
            report.info_density,
            report.screen_coverage,
        ];
        for (mi, _metric) in metrics.iter().enumerate() {
            grid_x.push(gi as f64);
            grid_y.push(mi as f64);
            values.push(vals[mi]);
        }
    }

    json!({
        "channel": "UiAnalysis",
        "id": "genre-tufte-comparison",
        "label": "Tufte UI Analysis: FPS vs RTS vs Sandbox (Tufte 1983)",
        "grid_x": grid_x,
        "grid_y": grid_y,
        "values": values,
        "metadata": { "x_labels": genres, "y_labels": metrics }
    })
}

fn fps_hud_elements() -> Vec<UiElement> {
    vec![
        UiElement {
            name: "health".into(),
            bounds: [0.05, 0.9, 0.15, 0.05],
            data_values: 1,
            pixel_area: 200.0,
            data_ink_area: 150.0,
            critical: true,
        },
        UiElement {
            name: "ammo".into(),
            bounds: [0.85, 0.9, 0.1, 0.05],
            data_values: 1,
            pixel_area: 150.0,
            data_ink_area: 100.0,
            critical: true,
        },
        UiElement {
            name: "crosshair".into(),
            bounds: [0.48, 0.48, 0.04, 0.04],
            data_values: 1,
            pixel_area: 20.0,
            data_ink_area: 18.0,
            critical: true,
        },
        UiElement {
            name: "minimap".into(),
            bounds: [0.8, 0.0, 0.2, 0.2],
            data_values: 50,
            pixel_area: 1000.0,
            data_ink_area: 600.0,
            critical: false,
        },
    ]
}

fn rts_hud_elements() -> Vec<UiElement> {
    vec![
        UiElement {
            name: "minimap".into(),
            bounds: [0.75, 0.0, 0.25, 0.25],
            data_values: 200,
            pixel_area: 2500.0,
            data_ink_area: 1800.0,
            critical: true,
        },
        UiElement {
            name: "unit_list".into(),
            bounds: [0.0, 0.0, 0.15, 0.5],
            data_values: 30,
            pixel_area: 1500.0,
            data_ink_area: 800.0,
            critical: true,
        },
        UiElement {
            name: "resources".into(),
            bounds: [0.3, 0.0, 0.3, 0.03],
            data_values: 4,
            pixel_area: 300.0,
            data_ink_area: 250.0,
            critical: true,
        },
        UiElement {
            name: "command_card".into(),
            bounds: [0.0, 0.7, 0.2, 0.3],
            data_values: 12,
            pixel_area: 2000.0,
            data_ink_area: 600.0,
            critical: true,
        },
    ]
}

fn sandbox_hud_elements() -> Vec<UiElement> {
    vec![
        UiElement {
            name: "hotbar".into(),
            bounds: [0.3, 0.95, 0.4, 0.05],
            data_values: 9,
            pixel_area: 500.0,
            data_ink_area: 400.0,
            critical: true,
        },
        UiElement {
            name: "health".into(),
            bounds: [0.3, 0.9, 0.1, 0.03],
            data_values: 2,
            pixel_area: 80.0,
            data_ink_area: 70.0,
            critical: true,
        },
        UiElement {
            name: "crosshair".into(),
            bounds: [0.49, 0.49, 0.02, 0.02],
            data_values: 1,
            pixel_area: 10.0,
            data_ink_area: 9.0,
            critical: false,
        },
    ]
}

/// Four Keys to Fun profiles for 4 game archetypes (Lazzaro 2004).
fn build_fun_keys_profile() -> Value {
    let games: &[(&str, FunSignals)] = &[
        (
            "Dark Souls",
            FunSignals {
                challenge: 0.95,
                retry_rate: 0.8,
                exploration: 0.5,
                social: 0.3,
                completion: 0.2,
            },
        ),
        (
            "Minecraft",
            FunSignals {
                challenge: 0.3,
                retry_rate: 0.1,
                exploration: 0.9,
                social: 0.7,
                completion: 0.4,
            },
        ),
        (
            "Among Us",
            FunSignals {
                challenge: 0.5,
                retry_rate: 0.2,
                exploration: 0.3,
                social: 0.95,
                completion: 0.3,
            },
        ),
        (
            "Animal Crossing",
            FunSignals {
                challenge: 0.1,
                retry_rate: 0.0,
                exploration: 0.7,
                social: 0.6,
                completion: 0.9,
            },
        ),
    ];

    let mut game_names = Vec::new();
    let mut hard_fun = Vec::new();
    let mut easy_fun = Vec::new();
    let mut people_fun = Vec::new();
    let mut serious_fun = Vec::new();
    let mut dominants = Vec::new();

    for (name, signals) in games {
        let result = classify_fun(signals);
        game_names.push(*name);
        hard_fun.push(result.scores.hard);
        easy_fun.push(result.scores.easy);
        people_fun.push(result.scores.people);
        serious_fun.push(result.scores.serious);
        dominants.push(result.dominant.as_str());
    }

    json!({
        "channel": "EngagementCurve",
        "id": "four-keys-fun",
        "label": "Four Keys to Fun Profile (Lazzaro 2004)",
        "timestamps": [0.0, 1.0, 2.0, 3.0],
        "engagement": hard_fun,
        "metadata": {
            "games": game_names,
            "hard_fun": hard_fun,
            "easy_fun": easy_fun,
            "people_fun": people_fun,
            "serious_fun": serious_fun,
            "dominant": dominants,
        }
    })
}

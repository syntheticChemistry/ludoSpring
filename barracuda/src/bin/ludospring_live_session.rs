// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![deny(clippy::expect_used, clippy::unwrap_used)]
#![warn(clippy::pedantic)]
#![allow(clippy::doc_markdown)]
//! ludoSpring Live Game Session — streaming demo for petalTongue.
//!
//! Simulates a 120-tick game session and pushes incremental updates
//! via `visualization.render.stream`:
//!
//! - **append**: new engagement data points each tick
//! - **set_value**: current difficulty gauge
//! - **replace**: flow state bar chart each phase
//!
//! Proves the streaming path works before biomeOS Continuous mode exists.
//!
//! # Usage
//!
//! ```bash
//! cargo run --features ipc --bin ludospring_live_session
//! ```

use std::fs;
use std::path::Path;
use std::process;

use ludospring_barracuda::interaction::difficulty::{PerformanceWindow, suggest_adjustment};
use ludospring_barracuda::interaction::flow::{FlowState, evaluate_flow};
use ludospring_barracuda::metrics::engagement::{EngagementSnapshot, compute_engagement};
use ludospring_barracuda::tolerances;
use ludospring_barracuda::visualization::PetalTonguePushClient;

use serde_json::{Value, json};

fn main() {
    eprintln!("╔═══════════════════════════════════════════════════════════╗");
    eprintln!("║  ludoSpring Live Game Session                            ║");
    eprintln!("║  Streaming game science to petalTongue tick-by-tick      ║");
    eprintln!("╚═══════════════════════════════════════════════════════════╝");
    eprintln!();

    let client = PetalTonguePushClient::discover().ok();
    if client.is_some() {
        eprintln!("petalTongue discovered — streaming live.");
    } else {
        eprintln!("petalTongue not running — recording session to JSON.");
    }

    let total_ticks = 120u32;
    let (engagement_trace, difficulty_trace, flow_phase_charts) =
        run_session(total_ticks, client.as_ref());

    eprintln!("\nSession complete: {total_ticks} ticks simulated.");
    write_session_json(
        total_ticks,
        &engagement_trace,
        &difficulty_trace,
        &flow_phase_charts,
    );
    eprintln!("Done.");
}

fn run_session(
    total_ticks: u32,
    client: Option<&PetalTonguePushClient>,
) -> (Vec<Value>, Vec<Value>, Vec<Value>) {
    let mut window = PerformanceWindow::new(10);
    let mut current_difficulty = 0.5_f64;
    let mut engagement_trace = Vec::new();
    let mut difficulty_trace = Vec::new();
    let mut flow_phase_charts = Vec::new();
    let phase_len = 30;

    for tick in 0..total_ticks {
        let success_prob = if current_difficulty < 0.3 {
            0.9
        } else if current_difficulty < 0.7 {
            0.6
        } else {
            0.3
        };

        let outcome = if deterministic_roll(tick, success_prob) {
            1.0
        } else {
            0.0
        };
        window.record(outcome);

        let skill = window.estimated_skill();
        let adj = suggest_adjustment(&window, tolerances::DDA_TARGET_SUCCESS_RATE);
        current_difficulty = (current_difficulty + adj * 0.05).clamp(0.1, 0.95);

        let snap = EngagementSnapshot {
            session_duration_s: f64::from(tick + 1) * 0.5,
            action_count: u64::from(tick + 1) * 3,
            exploration_breadth: (tick / 10).min(20),
            challenge_seeking: tick / 15,
            retry_count: if outcome < 0.5 { tick / 5 } else { 0 },
            deliberate_pauses: tick / 20,
        };
        let em = compute_engagement(&snap);

        push_tick_data(client, tick, &em, current_difficulty);

        engagement_trace.push(json!({
            "tick": tick, "engagement": em.composite,
            "difficulty": current_difficulty, "skill": skill,
        }));
        difficulty_trace.push(json!({
            "tick": tick, "difficulty": current_difficulty,
            "skill": skill, "adjustment": adj,
        }));

        if tick > 0 && tick % phase_len == 0 {
            let phase_chart = build_phase_flow_chart(tick, current_difficulty, skill);
            if let Some(c) = client {
                let _ = c.push_stream("live-flow", "replace", &phase_chart);
            }
            flow_phase_charts.push(phase_chart);
            eprintln!(
                "  Phase {}: difficulty={current_difficulty:.3} skill={skill:.3} engagement={:.3}",
                tick / phase_len,
                em.composite,
            );
        }
    }

    (engagement_trace, difficulty_trace, flow_phase_charts)
}

fn push_tick_data(
    client: Option<&PetalTonguePushClient>,
    tick: u32,
    em: &ludospring_barracuda::metrics::engagement::EngagementMetrics,
    difficulty: f64,
) {
    if let Some(c) = client {
        let _ = c.push_stream(
            "live-engagement",
            "append",
            &json!({
                "channel": "EngagementCurve",
                "point": { "x": f64::from(tick), "y": em.composite }
            }),
        );
        let _ = c.push_stream(
            "live-difficulty",
            "set_value",
            &json!({
                "channel": "DifficultyProfile",
                "value": difficulty,
                "label": format!("Tick {tick}: difficulty {difficulty:.3}")
            }),
        );
    }
}

fn write_session_json(
    total_ticks: u32,
    engagement_trace: &[Value],
    difficulty_trace: &[Value],
    flow_phase_charts: &[Value],
) {
    let out_dir = Path::new("sandbox/sessions");
    if let Err(e) = fs::create_dir_all(out_dir) {
        eprintln!("ERROR: cannot create {}: {e}", out_dir.display());
        process::exit(1);
    }

    let session = json!({
        "session_id": "live-game-session-demo",
        "ticks": total_ticks,
        "engagement_trace": engagement_trace,
        "difficulty_trace": difficulty_trace,
        "flow_phase_charts": flow_phase_charts,
    });

    let path = out_dir.join("live_session.json");
    match serde_json::to_string_pretty(&session) {
        Ok(json_str) => match fs::write(&path, &json_str) {
            Ok(()) => eprintln!("Session saved to {}", path.display()),
            Err(e) => eprintln!("ERROR: write {}: {e}", path.display()),
        },
        Err(e) => eprintln!("ERROR: serialize session: {e}"),
    }
}

fn build_phase_flow_chart(tick: u32, difficulty: f64, skill: f64) -> Value {
    let states = [
        FlowState::Boredom,
        FlowState::Relaxation,
        FlowState::Flow,
        FlowState::Arousal,
        FlowState::Anxiety,
    ];
    let mut counts = [0u32; 5];

    for d in 0..10 {
        for s in 0..10 {
            let c = (f64::from(d) - 5.0).mul_add(0.05, difficulty);
            let sk = (f64::from(s) - 5.0).mul_add(0.05, skill);
            let state = evaluate_flow(
                c.clamp(0.0, 1.0),
                sk.clamp(0.0, 1.0),
                tolerances::FLOW_CHANNEL_WIDTH,
            );
            let idx = states.iter().position(|st| *st == state).unwrap_or(0);
            counts[idx] += 1;
        }
    }

    let flow_states: Vec<&str> = states.iter().map(|s| s.as_str()).collect();
    let durations: Vec<f64> = counts.iter().map(|c| f64::from(*c)).collect();

    json!({
        "channel": "FlowTimeline",
        "id": format!("flow-phase-tick-{tick}"),
        "label": format!("Flow Distribution at Tick {tick}"),
        "flow_states": flow_states,
        "durations": durations,
    })
}

/// Deterministic pseudo-random roll using tick as seed.
fn deterministic_roll(tick: u32, success_prob: f64) -> bool {
    let hash = tick.wrapping_mul(2_654_435_761);
    let normalized = f64::from(hash % 1000) / 1000.0;
    normalized < success_prob
}

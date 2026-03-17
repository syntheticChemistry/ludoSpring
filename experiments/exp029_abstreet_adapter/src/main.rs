// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![expect(
    dead_code,
    reason = "wire format types for IPC contract — constructed by remote callers, not locally"
)]
#![expect(
    clippy::cast_possible_truncation,
    reason = "validation harness: small-range numeric conversions"
)]
#![expect(
    clippy::cast_sign_loss,
    reason = "validation harness: non-negative values cast to unsigned"
)]
#![expect(
    clippy::cast_precision_loss,
    reason = "validation harness: counter/timing values within f64 range"
)]
// SPDX-License-Identifier: AGPL-3.0-or-later
//! exp029 — A/B Street Telemetry Adapter
//!
//! Demonstrates translating A/B Street's simulation analytics
//! (Apache-2.0, github.com/a-b-street/abstreet) into ludoSpring telemetry.
//!
//! A/B Street is a transportation planning game with:
//! - Headless mode for programmatic simulation
//! - `sim::analytics` with `SlidingWindow` for event counts
//! - Trip completion, signal changes, throughput metrics
//!
//! Unique angle: simulation-as-game. The "player" is the city planner.
//! Their "moves" are infrastructure changes. "Challenges" are congestion
//! points. "Exploration" is discovering traffic patterns.

use std::io::Write;

use ludospring_barracuda::telemetry::events::{
    ChallengePayload, EventType, ExplorationPayload, PlayerActionPayload, PlayerMovePayload,
    SessionEndPayload, SessionStartPayload, TelemetryEvent,
};
use ludospring_barracuda::telemetry::mapper::SessionAccumulator;
use ludospring_barracuda::telemetry::report::generate_report;
use ludospring_barracuda::validation::ValidationResult;

/// Simulated A/B Street analytics event.
#[derive(Debug, Clone)]
enum AbStreetEvent {
    SimStart {
        map: String,
        scenario: String,
    },
    TripComplete {
        agent_id: u64,
        mode: String,
        duration_s: f64,
        distance_m: f64,
    },
    TripFailed {
        agent_id: u64,
        mode: String,
        reason: String,
    },
    SignalChange {
        intersection_id: u64,
        phase: u32,
    },
    RoadThroughput {
        road_id: u64,
        count: u32,
        period_s: f64,
    },
    InfraChange {
        change_type: String,
        location: String,
    },
    CongestionDetected {
        road_id: u64,
        delay_s: f64,
    },
    SimEnd {
        duration_s: f64,
        trips_completed: u64,
        trips_failed: u64,
    },
}

#[expect(
    clippy::too_many_lines,
    reason = "synthetic event sequence — infrastructure simulation"
)]
fn synthetic_simulation() -> Vec<(u64, AbStreetEvent)> {
    vec![
        (
            0,
            AbStreetEvent::SimStart {
                map: "seattle_downtown".into(),
                scenario: "weekday_commute".into(),
            },
        ),
        (
            1000,
            AbStreetEvent::InfraChange {
                change_type: "add_bike_lane".into(),
                location: "5th_ave".into(),
            },
        ),
        (
            2000,
            AbStreetEvent::TripComplete {
                agent_id: 1,
                mode: "car".into(),
                duration_s: 300.0,
                distance_m: 2000.0,
            },
        ),
        (
            2500,
            AbStreetEvent::TripComplete {
                agent_id: 2,
                mode: "bike".into(),
                duration_s: 400.0,
                distance_m: 1500.0,
            },
        ),
        (
            3000,
            AbStreetEvent::CongestionDetected {
                road_id: 42,
                delay_s: 120.0,
            },
        ),
        (
            3500,
            AbStreetEvent::RoadThroughput {
                road_id: 42,
                count: 50,
                period_s: 300.0,
            },
        ),
        (
            4000,
            AbStreetEvent::TripFailed {
                agent_id: 3,
                mode: "bus".into(),
                reason: "no_route".into(),
            },
        ),
        (
            4500,
            AbStreetEvent::InfraChange {
                change_type: "add_bus_stop".into(),
                location: "pike_st".into(),
            },
        ),
        (
            5000,
            AbStreetEvent::TripComplete {
                agent_id: 4,
                mode: "bus".into(),
                duration_s: 500.0,
                distance_m: 3000.0,
            },
        ),
        (
            5500,
            AbStreetEvent::SignalChange {
                intersection_id: 7,
                phase: 3,
            },
        ),
        (
            6000,
            AbStreetEvent::TripComplete {
                agent_id: 5,
                mode: "walk".into(),
                duration_s: 600.0,
                distance_m: 500.0,
            },
        ),
        (
            6500,
            AbStreetEvent::CongestionDetected {
                road_id: 15,
                delay_s: 60.0,
            },
        ),
        (
            7000,
            AbStreetEvent::RoadThroughput {
                road_id: 15,
                count: 30,
                period_s: 300.0,
            },
        ),
        (
            7500,
            AbStreetEvent::InfraChange {
                change_type: "retime_signal".into(),
                location: "intersection_7".into(),
            },
        ),
        (
            8000,
            AbStreetEvent::TripComplete {
                agent_id: 6,
                mode: "car".into(),
                duration_s: 250.0,
                distance_m: 2500.0,
            },
        ),
        (
            10000,
            AbStreetEvent::SimEnd {
                duration_s: 10.0,
                trips_completed: 5,
                trips_failed: 1,
            },
        ),
    ]
}

#[expect(
    clippy::too_many_lines,
    reason = "event translation — match arms per variant"
)]
fn translate_event(ts_ms: u64, event: &AbStreetEvent, sid: &str) -> Vec<TelemetryEvent> {
    let mut out = Vec::new();
    match event {
        AbStreetEvent::SimStart { map, scenario } => {
            out.push(TelemetryEvent {
                timestamp_ms: ts_ms,
                session_id: sid.into(),
                event_type: EventType::SessionStart,
                payload: serde_json::to_value(SessionStartPayload {
                    game_name: "abstreet".into(),
                    game_version: "0.3".into(),
                    genre: format!("simulation:{map}:{scenario}"),
                })
                .unwrap_or_default(),
            });
        }
        AbStreetEvent::InfraChange {
            change_type,
            location,
        } => {
            out.push(TelemetryEvent {
                timestamp_ms: ts_ms,
                session_id: sid.into(),
                event_type: EventType::PlayerAction,
                payload: serde_json::to_value(PlayerActionPayload {
                    action: change_type.clone(),
                    target: location.clone(),
                    success: true,
                })
                .unwrap_or_default(),
            });
        }
        AbStreetEvent::TripComplete {
            agent_id,
            duration_s,
            distance_m,
            ..
        } => {
            out.push(TelemetryEvent {
                timestamp_ms: ts_ms,
                session_id: sid.into(),
                event_type: EventType::ChallengeComplete,
                payload: serde_json::to_value(ChallengePayload {
                    challenge_id: format!("trip_{agent_id}"),
                    difficulty: (*duration_s / 600.0).min(1.0),
                    challenge_type: "trip_completion".into(),
                })
                .unwrap_or_default(),
            });
            out.push(TelemetryEvent {
                timestamp_ms: ts_ms,
                session_id: sid.into(),
                event_type: EventType::PlayerMove,
                payload: serde_json::to_value(PlayerMovePayload {
                    x: *distance_m,
                    speed: distance_m / duration_s,
                    ..Default::default()
                })
                .unwrap_or_default(),
            });
        }
        AbStreetEvent::TripFailed {
            agent_id, reason, ..
        } => {
            out.push(TelemetryEvent {
                timestamp_ms: ts_ms,
                session_id: sid.into(),
                event_type: EventType::ChallengeFail,
                payload: serde_json::to_value(ChallengePayload {
                    challenge_id: format!("trip_{agent_id}"),
                    difficulty: 0.8,
                    challenge_type: format!("trip_failure:{reason}"),
                })
                .unwrap_or_default(),
            });
        }
        AbStreetEvent::CongestionDetected { road_id, delay_s } => {
            out.push(TelemetryEvent {
                timestamp_ms: ts_ms,
                session_id: sid.into(),
                event_type: EventType::ChallengeEncounter,
                payload: serde_json::to_value(ChallengePayload {
                    challenge_id: format!("congestion_road_{road_id}"),
                    difficulty: (*delay_s / 180.0).min(1.0),
                    challenge_type: "congestion".into(),
                })
                .unwrap_or_default(),
            });
        }
        AbStreetEvent::RoadThroughput { road_id, count, .. } => {
            out.push(TelemetryEvent {
                timestamp_ms: ts_ms,
                session_id: sid.into(),
                event_type: EventType::ExplorationDiscover,
                payload: serde_json::to_value(ExplorationPayload {
                    discovery_id: format!("throughput_road_{road_id}"),
                    category: format!("throughput:{count}"),
                    ..Default::default()
                })
                .unwrap_or_default(),
            });
        }
        AbStreetEvent::SimEnd { duration_s, .. } => {
            out.push(TelemetryEvent {
                timestamp_ms: ts_ms,
                session_id: sid.into(),
                event_type: EventType::SessionEnd,
                payload: serde_json::to_value(SessionEndPayload {
                    duration_s: *duration_s,
                    reason: "simulation_complete".into(),
                })
                .unwrap_or_default(),
            });
        }
        AbStreetEvent::SignalChange { .. } => {}
    }
    out
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("validate") => cmd_validate(),
        Some("demo") => cmd_demo(),
        _ => {
            eprintln!("Usage: exp029_abstreet_adapter <validate|demo>");
            std::process::exit(1);
        }
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "validation orchestrator — sequential check groups"
)]
fn cmd_validate() {
    println!("=== exp029: A/B Street Adapter Validation ===\n");
    let mut results = Vec::new();

    let sim_events = synthetic_simulation();
    let sid = "test";
    let telemetry: Vec<TelemetryEvent> = sim_events
        .iter()
        .flat_map(|(ts, evt)| translate_event(*ts, evt, sid))
        .collect();

    results.push(ValidationResult::check(
        "exp029",
        "events_translated",
        telemetry.len() as f64,
        20.0,
        1.0,
    ));
    results.push(ValidationResult::check(
        "exp029",
        "has_session_start",
        if telemetry
            .iter()
            .any(|e| e.event_type == EventType::SessionStart)
        {
            1.0
        } else {
            0.0
        },
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        "exp029",
        "has_session_end",
        if telemetry
            .iter()
            .any(|e| e.event_type == EventType::SessionEnd)
        {
            1.0
        } else {
            0.0
        },
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        "exp029",
        "has_player_actions",
        telemetry
            .iter()
            .filter(|e| e.event_type == EventType::PlayerAction)
            .count() as f64,
        3.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        "exp029",
        "has_trip_completions",
        telemetry
            .iter()
            .filter(|e| e.event_type == EventType::ChallengeComplete)
            .count() as f64,
        5.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        "exp029",
        "has_congestion_challenges",
        telemetry
            .iter()
            .filter(|e| e.event_type == EventType::ChallengeEncounter)
            .count() as f64,
        2.0,
        0.0,
    ));

    let mut acc = SessionAccumulator::new();
    acc.ingest_all(&telemetry);
    let report = generate_report(&acc);
    results.push(ValidationResult::check(
        "exp029",
        "report_generates",
        if serde_json::to_string(&report).is_ok() {
            1.0
        } else {
            0.0
        },
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        "exp029",
        "engagement_positive",
        if report.engagement.composite > 0.0 {
            1.0
        } else {
            0.0
        },
        1.0,
        0.0,
    ));

    for r in &results {
        let status = if r.passed { "PASS" } else { "FAIL" };
        println!("  [{status}] {}", r.description);
    }
    let pass = results.iter().filter(|r| r.passed).count();
    println!("\nResults: {pass}/{} passed", results.len());
    if pass < results.len() {
        std::process::exit(1);
    }
}

fn cmd_demo() {
    println!("=== exp029: A/B Street Adapter Demo ===\n");
    let sim_events = synthetic_simulation();
    let sid = "abstreet-demo";
    let telemetry: Vec<TelemetryEvent> = sim_events
        .iter()
        .flat_map(|(ts, evt)| translate_event(*ts, evt, sid))
        .collect();

    println!(
        "Translated {} events from synthetic A/B Street simulation\n",
        telemetry.len()
    );

    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    for evt in &telemetry {
        if let Ok(json) = serde_json::to_string(evt) {
            let _ = writeln!(out, "{json}");
        }
    }

    eprintln!("\n--- Analysis ---\n");
    let mut acc = SessionAccumulator::new();
    acc.ingest_all(&telemetry);
    let report = generate_report(&acc);
    let json = serde_json::to_string_pretty(&report).unwrap_or_default();
    eprintln!("{json}");
}

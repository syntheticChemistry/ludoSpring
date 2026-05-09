// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![expect(
    clippy::cast_precision_loss,
    reason = "validation harness: counter/timing values within f64 range"
)]
//! exp028 — Fish Folk / Jumpy Telemetry Adapter
//!
//! Demonstrates the Bevy plugin pattern for telemetry emission.
//! Fish Folk (MIT/Apache-2.0, github.com/fishfolk) uses Bevy + Bones:
//! - Bevy ECS for rendering and audio
//! - Bones framework for deterministic core gameplay
//! - Rollback networking with snapshot/restore
//!
//! This adapter models how a Bevy plugin would hook into `EventReader`
//! and translate game events to ludoSpring telemetry. Since we can't
//! depend on bevy (massive dep tree), we simulate the event flow.
//!
//! The pattern: Bevy System -> `EventReader<GameEvent>` -> `TelemetryWriter`

use std::io::Write;

use ludospring_barracuda::telemetry::events::{
    ChallengePayload, EventType, ExplorationPayload, PlayerActionPayload, PlayerDamagePayload,
    PlayerDeathPayload, PlayerMovePayload, SessionEndPayload, SessionStartPayload, TelemetryEvent,
};
use ludospring_barracuda::telemetry::mapper::SessionAccumulator;
use ludospring_barracuda::telemetry::report::generate_report;
use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "N/A (adapter validation — synthetic Fish Folk match)",
    commit: "19e402c0",
    date: "2026-03-29",
    command: "N/A (translate_event + SessionAccumulator)",
};

/// Simulated Bevy/Bones game event (what `EventReader` would provide).
#[expect(
    dead_code,
    reason = "structural completeness — domain model includes all variants"
)]
#[derive(Debug, Clone)]
enum FishFolkEvent {
    MatchStart {
        map: String,
        players: u32,
    },
    PlayerSpawn {
        id: u32,
        x: f32,
        y: f32,
    },
    PlayerMove {
        id: u32,
        x: f32,
        y: f32,
        vel_x: f32,
        vel_y: f32,
    },
    WeaponPickup {
        player_id: u32,
        weapon: String,
    },
    PlayerHit {
        target_id: u32,
        damage: f32,
        source_id: u32,
        hp_left: f32,
    },
    PlayerKill {
        killer_id: u32,
        victim_id: u32,
    },
    PlayerRespawn {
        id: u32,
        x: f32,
        y: f32,
    },
    MatchEnd {
        winner_id: u32,
        duration_s: f32,
    },
}

#[expect(clippy::too_many_lines, reason = "synthetic match event sequence")]
fn synthetic_match() -> Vec<(u64, FishFolkEvent)> {
    vec![
        (
            0,
            FishFolkEvent::MatchStart {
                map: "coral_reef".into(),
                players: 4,
            },
        ),
        (
            100,
            FishFolkEvent::PlayerSpawn {
                id: 1,
                x: 10.0,
                y: 5.0,
            },
        ),
        (
            100,
            FishFolkEvent::PlayerSpawn {
                id: 2,
                x: 30.0,
                y: 5.0,
            },
        ),
        (
            500,
            FishFolkEvent::PlayerMove {
                id: 1,
                x: 12.0,
                y: 5.0,
                vel_x: 2.0,
                vel_y: 0.0,
            },
        ),
        (
            800,
            FishFolkEvent::WeaponPickup {
                player_id: 1,
                weapon: "blunderbuss".into(),
            },
        ),
        (
            1000,
            FishFolkEvent::PlayerMove {
                id: 1,
                x: 15.0,
                y: 6.0,
                vel_x: 1.5,
                vel_y: 0.5,
            },
        ),
        (
            1200,
            FishFolkEvent::PlayerHit {
                target_id: 2,
                damage: 30.0,
                source_id: 1,
                hp_left: 70.0,
            },
        ),
        (
            1500,
            FishFolkEvent::PlayerMove {
                id: 2,
                x: 28.0,
                y: 7.0,
                vel_x: -1.0,
                vel_y: 1.0,
            },
        ),
        (
            1800,
            FishFolkEvent::WeaponPickup {
                player_id: 2,
                weapon: "sword".into(),
            },
        ),
        (
            2000,
            FishFolkEvent::PlayerHit {
                target_id: 1,
                damage: 20.0,
                source_id: 2,
                hp_left: 80.0,
            },
        ),
        (
            2200,
            FishFolkEvent::PlayerMove {
                id: 1,
                x: 18.0,
                y: 8.0,
                vel_x: 1.0,
                vel_y: 1.0,
            },
        ),
        (
            2500,
            FishFolkEvent::PlayerHit {
                target_id: 2,
                damage: 40.0,
                source_id: 1,
                hp_left: 30.0,
            },
        ),
        (
            2800,
            FishFolkEvent::PlayerHit {
                target_id: 2,
                damage: 30.0,
                source_id: 1,
                hp_left: 0.0,
            },
        ),
        (
            2800,
            FishFolkEvent::PlayerKill {
                killer_id: 1,
                victim_id: 2,
            },
        ),
        (
            3000,
            FishFolkEvent::PlayerRespawn {
                id: 2,
                x: 30.0,
                y: 5.0,
            },
        ),
        (
            3500,
            FishFolkEvent::PlayerMove {
                id: 1,
                x: 22.0,
                y: 10.0,
                vel_x: 1.0,
                vel_y: 0.5,
            },
        ),
        (
            4000,
            FishFolkEvent::PlayerHit {
                target_id: 1,
                damage: 50.0,
                source_id: 2,
                hp_left: 30.0,
            },
        ),
        (
            4200,
            FishFolkEvent::PlayerHit {
                target_id: 1,
                damage: 30.0,
                source_id: 2,
                hp_left: 0.0,
            },
        ),
        (
            4200,
            FishFolkEvent::PlayerKill {
                killer_id: 2,
                victim_id: 1,
            },
        ),
        (
            5000,
            FishFolkEvent::MatchEnd {
                winner_id: 2,
                duration_s: 5.0,
            },
        ),
    ]
}

/// Translate a Fish Folk event into ludoSpring telemetry (for player 1's perspective).
#[expect(
    clippy::too_many_lines,
    reason = "event translation — match arms per variant"
)]
fn translate_event(ts_ms: u64, event: &FishFolkEvent, sid: &str) -> Vec<TelemetryEvent> {
    let mut out = Vec::new();
    match event {
        FishFolkEvent::MatchStart { map, .. } => {
            out.push(TelemetryEvent {
                timestamp_ms: ts_ms,
                session_id: sid.into(),
                event_type: EventType::SessionStart,
                payload: serde_json::to_value(SessionStartPayload {
                    game_name: "fishfolk_jumpy".into(),
                    game_version: "0.10".into(),
                    genre: format!("2d_shooter:{map}"),
                })
                .unwrap_or_default(),
            });
        }
        FishFolkEvent::PlayerSpawn { id, x, y } if *id == 1 => {
            out.push(TelemetryEvent {
                timestamp_ms: ts_ms,
                session_id: sid.into(),
                event_type: EventType::PlayerMove,
                payload: serde_json::to_value(PlayerMovePayload {
                    x: f64::from(*x),
                    y: f64::from(*y),
                    ..Default::default()
                })
                .unwrap_or_default(),
            });
        }
        FishFolkEvent::PlayerMove {
            id,
            x,
            y,
            vel_x,
            vel_y,
        } if *id == 1 => {
            let speed = f64::from(*vel_x).hypot(f64::from(*vel_y));
            out.push(TelemetryEvent {
                timestamp_ms: ts_ms,
                session_id: sid.into(),
                event_type: EventType::PlayerMove,
                payload: serde_json::to_value(PlayerMovePayload {
                    x: f64::from(*x),
                    y: f64::from(*y),
                    speed,
                    ..Default::default()
                })
                .unwrap_or_default(),
            });
        }
        FishFolkEvent::WeaponPickup { player_id, weapon } if *player_id == 1 => {
            out.push(TelemetryEvent {
                timestamp_ms: ts_ms,
                session_id: sid.into(),
                event_type: EventType::ExplorationDiscover,
                payload: serde_json::to_value(ExplorationPayload {
                    discovery_id: weapon.clone(),
                    category: "weapon".into(),
                    ..Default::default()
                })
                .unwrap_or_default(),
            });
        }
        FishFolkEvent::PlayerHit {
            target_id,
            damage,
            hp_left,
            source_id,
        } if *target_id == 1 => {
            out.push(TelemetryEvent {
                timestamp_ms: ts_ms,
                session_id: sid.into(),
                event_type: EventType::PlayerDamage,
                payload: serde_json::to_value(PlayerDamagePayload {
                    amount: f64::from(*damage),
                    source: format!("player_{source_id}"),
                    health_remaining: f64::from(*hp_left),
                })
                .unwrap_or_default(),
            });
        }
        FishFolkEvent::PlayerHit {
            source_id,
            target_id,
            ..
        } if *source_id == 1 => {
            out.push(TelemetryEvent {
                timestamp_ms: ts_ms,
                session_id: sid.into(),
                event_type: EventType::PlayerAction,
                payload: serde_json::to_value(PlayerActionPayload {
                    action: "attack".into(),
                    target: format!("player_{target_id}"),
                    success: true,
                })
                .unwrap_or_default(),
            });
        }
        FishFolkEvent::PlayerKill {
            killer_id,
            victim_id,
        } if *killer_id == 1 => {
            out.push(TelemetryEvent {
                timestamp_ms: ts_ms,
                session_id: sid.into(),
                event_type: EventType::ChallengeComplete,
                payload: serde_json::to_value(ChallengePayload {
                    challenge_id: format!("kill_player_{victim_id}"),
                    difficulty: 0.6,
                    challenge_type: "pvp".into(),
                })
                .unwrap_or_default(),
            });
        }
        FishFolkEvent::PlayerKill { victim_id, .. } if *victim_id == 1 => {
            out.push(TelemetryEvent {
                timestamp_ms: ts_ms,
                session_id: sid.into(),
                event_type: EventType::PlayerDeath,
                payload: serde_json::to_value(PlayerDeathPayload {
                    cause: "killed_in_combat".into(),
                    respawn: true,
                })
                .unwrap_or_default(),
            });
        }
        FishFolkEvent::MatchEnd { duration_s, .. } => {
            out.push(TelemetryEvent {
                timestamp_ms: ts_ms,
                session_id: sid.into(),
                event_type: EventType::SessionEnd,
                payload: serde_json::to_value(SessionEndPayload {
                    duration_s: f64::from(*duration_s),
                    reason: "match_end".into(),
                })
                .unwrap_or_default(),
            });
        }
        _ => {}
    }
    out
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("validate") => cmd_validate(),
        Some("demo") => cmd_demo(),
        _ => {
            eprintln!("Usage: exp028_fishfolk_adapter <validate|demo>");
            std::process::exit(1);
        }
    }
}

fn cmd_validate() {
    let mut h = ValidationHarness::new("exp028_fishfolk_adapter");
    h.print_provenance(&[&PROVENANCE]);

    let match_events = synthetic_match();
    let sid = "test";
    let telemetry: Vec<TelemetryEvent> = match_events
        .iter()
        .flat_map(|(ts, evt)| translate_event(*ts, evt, sid))
        .collect();

    h.check_abs("events_translated", telemetry.len() as f64, 16.0, 1.0);
    h.check_bool(
        "has_session_start",
        telemetry
            .iter()
            .any(|e| e.event_type == EventType::SessionStart),
    );
    h.check_bool(
        "has_session_end",
        telemetry
            .iter()
            .any(|e| e.event_type == EventType::SessionEnd),
    );
    h.check_bool(
        "has_player_death",
        telemetry
            .iter()
            .any(|e| e.event_type == EventType::PlayerDeath),
    );
    h.check_bool(
        "has_challenge_complete",
        telemetry
            .iter()
            .any(|e| e.event_type == EventType::ChallengeComplete),
    );

    let mut acc = SessionAccumulator::new();
    acc.ingest_all(&telemetry);
    let report = generate_report(&acc);
    h.check_bool("report_generates", serde_json::to_string(&report).is_ok());
    h.check_bool("engagement_positive", report.engagement.composite > 0.0);

    h.finish();
}

fn cmd_demo() {
    println!("=== exp028: Fish Folk Adapter Demo ===\n");
    let match_events = synthetic_match();
    let sid = "fishfolk-demo";
    let telemetry: Vec<TelemetryEvent> = match_events
        .iter()
        .flat_map(|(ts, evt)| translate_event(*ts, evt, sid))
        .collect();

    println!(
        "Translated {} events from synthetic Fish Folk match\n",
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

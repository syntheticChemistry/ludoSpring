// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![expect(
    clippy::cast_precision_loss,
    reason = "validation harness: counter/timing values within f64 range"
)]
//! exp027 — Veloren Telemetry Adapter
//!
//! Demonstrates parsing Veloren server log events (SPECS ECS tracing output)
//! and translating them into the ludoSpring telemetry protocol.
//!
//! Veloren (GPL-3.0, gitlab.com/veloren/veloren) uses:
//! - SPECS ECS with `EventBus` for game events
//! - tracing crate for structured logging
//! - WASM plugin system for extensibility
//!
//! This adapter parses tracing-style log lines and maps:
//! - `HealthChangeEvent` -> `player_damage`
//! - Entity position ticks -> `player_move`
//! - `DeleteEvent` (mob death) -> `challenge_complete`
//! - Region enter events -> `exploration_discover`
//!
//! Future: WASM plugin for live in-game telemetry emission.

use std::io::Write;

use regex::Regex;

use ludospring_barracuda::telemetry::events::{
    ChallengePayload, EventType, ExplorationPayload, PlayerDamagePayload, PlayerMovePayload,
    SessionEndPayload, SessionStartPayload, TelemetryEvent,
};
use ludospring_barracuda::telemetry::mapper::SessionAccumulator;
use ludospring_barracuda::telemetry::report::generate_report;
use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "N/A (adapter validation — synthetic Veloren log)",
    commit: "4b683e3e",
    date: "2026-03-29",
    command: "N/A (parse_veloren_line + SessionAccumulator)",
};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("parse") => cmd_parse(&args[2..]),
        Some("validate") => cmd_validate(),
        Some("demo") => cmd_demo(),
        _ => {
            eprintln!("Usage: exp027_veloren_adapter <parse|validate|demo>");
            eprintln!("  parse <veloren.log>  Parse Veloren server logs to telemetry NDJSON");
            eprintln!("  validate             Run adapter validation with synthetic logs");
            eprintln!("  demo                 Generate + analyze synthetic Veloren session");
            std::process::exit(1);
        }
    }
}

/// Synthetic Veloren server log lines (representative of real tracing output).
const fn synthetic_veloren_log() -> &'static str {
    r#"2026-03-11T10:00:00.000Z INFO veloren_server::sys::entity: session_start player="Explorer" class="ranger"
2026-03-11T10:00:01.000Z INFO veloren_server::sys::entity: player_pos x=100.5 y=200.3 z=50.0 player="Explorer"
2026-03-11T10:00:02.000Z INFO veloren_server::sys::entity: player_pos x=102.1 y=201.7 z=50.0 player="Explorer"
2026-03-11T10:00:03.000Z INFO veloren_server::events::entity_manipulation: health_change entity="Explorer" change=-15.0 source="wolf" hp_remaining=85.0
2026-03-11T10:00:04.000Z INFO veloren_server::sys::entity: player_pos x=103.5 y=203.1 z=50.0 player="Explorer"
2026-03-11T10:00:05.000Z INFO veloren_server::events::entity_manipulation: entity_delete entity="wolf_42" cause="killed_by:Explorer"
2026-03-11T10:00:06.000Z INFO veloren_server::sys::entity: player_pos x=110.0 y=210.0 z=52.0 player="Explorer"
2026-03-11T10:00:07.000Z INFO veloren_server::sys::terrain: region_enter player="Explorer" region="grassland_north" biome="grassland"
2026-03-11T10:00:08.000Z INFO veloren_server::sys::entity: player_pos x=115.2 y=215.8 z=53.0 player="Explorer"
2026-03-11T10:00:09.000Z INFO veloren_server::events::entity_manipulation: health_change entity="Explorer" change=-25.0 source="bear" hp_remaining=60.0
2026-03-11T10:00:10.000Z INFO veloren_server::events::entity_manipulation: entity_delete entity="bear_17" cause="killed_by:Explorer"
2026-03-11T10:00:11.000Z INFO veloren_server::sys::entity: player_pos x=120.0 y=220.0 z=55.0 player="Explorer"
2026-03-11T10:00:12.000Z INFO veloren_server::sys::terrain: region_enter player="Explorer" region="mountain_pass" biome="mountain"
2026-03-11T10:00:13.000Z INFO veloren_server::events::entity_manipulation: health_change entity="Explorer" change=-40.0 source="ogre" hp_remaining=20.0
2026-03-11T10:00:14.000Z INFO veloren_server::events::entity_manipulation: entity_delete entity="ogre_3" cause="killed_by:Explorer"
2026-03-11T10:00:15.000Z INFO veloren_server::sys::terrain: region_enter player="Explorer" region="cave_system" biome="underground"
2026-03-11T10:00:16.000Z INFO veloren_server::sys::entity: player_pos x=125.0 y=225.0 z=40.0 player="Explorer"
2026-03-11T10:00:30.000Z INFO veloren_server::sys::entity: session_end player="Explorer" reason="disconnect" duration_s=30.0
"#
}

/// Parse a Veloren log line into a telemetry event.
#[expect(clippy::too_many_lines, reason = "log parsing — regex and match arms")]
fn parse_veloren_line(line: &str, sid: &str) -> Option<TelemetryEvent> {
    let ts_re = Regex::new(r"(\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:(\d{2})\.(\d{3}))").ok()?;
    let ts_cap = ts_re.captures(line)?;
    let secs: u64 = ts_cap.get(2)?.as_str().parse().ok()?;
    let millis: u64 = ts_cap.get(3)?.as_str().parse().ok()?;
    let ts_ms = secs * 1000 + millis;

    if line.contains("session_start") {
        return Some(TelemetryEvent {
            timestamp_ms: ts_ms,
            session_id: sid.into(),
            event_type: EventType::SessionStart,
            payload: serde_json::to_value(SessionStartPayload {
                game_name: "veloren".into(),
                game_version: "0.16".into(),
                genre: "voxel_rpg".into(),
            })
            .unwrap_or_default(),
        });
    }

    if line.contains("session_end") {
        let dur_re = Regex::new(r"duration_s=([0-9.]+)").ok()?;
        let dur = dur_re
            .captures(line)
            .and_then(|c| c.get(1)?.as_str().parse::<f64>().ok())
            .unwrap_or(0.0);
        return Some(TelemetryEvent {
            timestamp_ms: ts_ms,
            session_id: sid.into(),
            event_type: EventType::SessionEnd,
            payload: serde_json::to_value(SessionEndPayload {
                duration_s: dur,
                reason: "disconnect".into(),
            })
            .unwrap_or_default(),
        });
    }

    if line.contains("player_pos") {
        let x_re = Regex::new(r"x=([0-9.-]+)").ok()?;
        let y_re = Regex::new(r"y=([0-9.-]+)").ok()?;
        let z_re = Regex::new(r"z=([0-9.-]+)").ok()?;
        let x = x_re
            .captures(line)
            .and_then(|c| c.get(1)?.as_str().parse().ok())
            .unwrap_or(0.0);
        let y = y_re
            .captures(line)
            .and_then(|c| c.get(1)?.as_str().parse().ok())
            .unwrap_or(0.0);
        let z = z_re
            .captures(line)
            .and_then(|c| c.get(1)?.as_str().parse().ok())
            .unwrap_or(0.0);
        return Some(TelemetryEvent {
            timestamp_ms: ts_ms,
            session_id: sid.into(),
            event_type: EventType::PlayerMove,
            payload: serde_json::to_value(PlayerMovePayload {
                x,
                y,
                z,
                ..Default::default()
            })
            .unwrap_or_default(),
        });
    }

    if line.contains("health_change") {
        let change_re = Regex::new(r"change=(-?[0-9.]+)").ok()?;
        let source_re = Regex::new(r#"source="([^"]+)""#).ok()?;
        let hp_re = Regex::new(r"hp_remaining=([0-9.]+)").ok()?;
        let amount = change_re
            .captures(line)
            .and_then(|c| c.get(1)?.as_str().parse::<f64>().ok())
            .unwrap_or(0.0)
            .abs();
        let source = source_re
            .captures(line)
            .and_then(|c| Some(c.get(1)?.as_str().to_string()))
            .unwrap_or_default();
        let hp = hp_re
            .captures(line)
            .and_then(|c| c.get(1)?.as_str().parse().ok())
            .unwrap_or(0.0);
        return Some(TelemetryEvent {
            timestamp_ms: ts_ms,
            session_id: sid.into(),
            event_type: EventType::PlayerDamage,
            payload: serde_json::to_value(PlayerDamagePayload {
                amount,
                source,
                health_remaining: hp,
            })
            .unwrap_or_default(),
        });
    }

    if line.contains("entity_delete") && line.contains("killed_by") {
        let entity_re = Regex::new(r#"entity="([^"]+)""#).ok()?;
        let entity = entity_re
            .captures(line)
            .and_then(|c| Some(c.get(1)?.as_str().to_string()))
            .unwrap_or_default();
        return Some(TelemetryEvent {
            timestamp_ms: ts_ms,
            session_id: sid.into(),
            event_type: EventType::ChallengeComplete,
            payload: serde_json::to_value(ChallengePayload {
                challenge_id: entity,
                difficulty: 0.5,
                challenge_type: "combat".into(),
            })
            .unwrap_or_default(),
        });
    }

    if line.contains("region_enter") {
        let region_re = Regex::new(r#"region="([^"]+)""#).ok()?;
        let region = region_re
            .captures(line)
            .and_then(|c| Some(c.get(1)?.as_str().to_string()))
            .unwrap_or_default();
        return Some(TelemetryEvent {
            timestamp_ms: ts_ms,
            session_id: sid.into(),
            event_type: EventType::ExplorationDiscover,
            payload: serde_json::to_value(ExplorationPayload {
                discovery_id: region,
                category: "region".into(),
                ..Default::default()
            })
            .unwrap_or_default(),
        });
    }

    None
}

fn cmd_parse(args: &[String]) {
    let input = if args.is_empty() {
        synthetic_veloren_log().to_string()
    } else {
        std::fs::read_to_string(&args[0]).unwrap_or_else(|e| {
            eprintln!("Error reading {}: {e}", args[0]);
            std::process::exit(1);
        })
    };

    let sid = "veloren-session";
    let stdout = std::io::stdout();
    let mut out = stdout.lock();

    let mut count = 0;
    for line in input.lines() {
        if let Some(evt) = parse_veloren_line(line, sid) {
            if let Ok(json) = serde_json::to_string(&evt) {
                let _ = writeln!(out, "{json}");
                count += 1;
            }
        }
    }
    eprintln!("Parsed {count} events from Veloren log");
}

fn cmd_validate() {
    let mut h = ValidationHarness::new("exp027_veloren_adapter");
    h.print_provenance(&[&PROVENANCE]);

    let log = synthetic_veloren_log();
    let sid = "test";
    let events: Vec<TelemetryEvent> = log
        .lines()
        .filter_map(|line| parse_veloren_line(line, sid))
        .collect();

    h.check_abs("parsed_events_count", events.len() as f64, 18.0, 0.0);
    h.check_bool(
        "has_session_start",
        events
            .iter()
            .any(|e| e.event_type == EventType::SessionStart),
    );
    h.check_bool(
        "has_session_end",
        events.iter().any(|e| e.event_type == EventType::SessionEnd),
    );
    h.check_abs(
        "has_player_moves",
        events
            .iter()
            .filter(|e| e.event_type == EventType::PlayerMove)
            .count() as f64,
        7.0,
        0.0,
    );
    h.check_abs(
        "has_damage_events",
        events
            .iter()
            .filter(|e| e.event_type == EventType::PlayerDamage)
            .count() as f64,
        3.0,
        0.0,
    );
    h.check_abs(
        "has_kills",
        events
            .iter()
            .filter(|e| e.event_type == EventType::ChallengeComplete)
            .count() as f64,
        3.0,
        0.0,
    );
    h.check_abs(
        "has_exploration",
        events
            .iter()
            .filter(|e| e.event_type == EventType::ExplorationDiscover)
            .count() as f64,
        3.0,
        0.0,
    );

    let mut acc = SessionAccumulator::new();
    acc.ingest_all(&events);
    let report = generate_report(&acc);
    h.check_bool("report_generates", serde_json::to_string(&report).is_ok());
    h.check_bool("engagement_positive", report.engagement.composite > 0.0);

    h.finish();
}

fn cmd_demo() {
    println!("=== exp027: Veloren Adapter Demo ===\n");
    let log = synthetic_veloren_log();
    let sid = "veloren-demo";
    let events: Vec<TelemetryEvent> = log
        .lines()
        .filter_map(|line| parse_veloren_line(line, sid))
        .collect();

    println!(
        "Parsed {} events from synthetic Veloren log\n",
        events.len()
    );

    let mut acc = SessionAccumulator::new();
    acc.ingest_all(&events);
    let report = generate_report(&acc);
    let json = serde_json::to_string_pretty(&report).unwrap_or_default();
    println!("{json}");
}

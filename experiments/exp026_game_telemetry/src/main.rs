// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
#![expect(
    clippy::cast_precision_loss,
    reason = "validation harness: counter/timing values within f64 range"
)]
//! exp026 — Game Telemetry Protocol CLI
//!
//! Analyzes NDJSON telemetry files and produces gameplay analysis reports.
//! Also generates synthetic telemetry for protocol validation.

use std::io::{self, Write};
use std::path::PathBuf;

use ludospring_barracuda::telemetry::events::{
    ChallengePayload, EventType, ExplorationPayload, PlayerActionPayload, PlayerDamagePayload,
    PlayerDeathPayload, PlayerMovePayload, SessionEndPayload, SessionStartPayload, TelemetryEvent,
};
use ludospring_barracuda::telemetry::mapper::SessionAccumulator;
use ludospring_barracuda::telemetry::report::generate_report;
use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "N/A (protocol validation — synthetic telemetry)",
    commit: "N/A",
    date: "N/A",
    command: "N/A (NDJSON roundtrip + SessionAccumulator)",
};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        print_usage();
        std::process::exit(1);
    }

    match args[1].as_str() {
        "analyze" => cmd_analyze(&args[2..]),
        "generate" => cmd_generate(&args[2..]),
        "validate" => cmd_validate(),
        "schema" => cmd_schema(),
        _ => {
            print_usage();
            std::process::exit(1);
        }
    }
}

fn print_usage() {
    eprintln!("Usage: exp026_game_telemetry <command> [args]");
    eprintln!();
    eprintln!("Commands:");
    eprintln!("  analyze <file.ndjson>   Analyze a telemetry file (or - for stdin)");
    eprintln!("  generate <file.ndjson>  Generate synthetic telemetry to file (or - for stdout)");
    eprintln!("  validate               Run protocol validation checks");
    eprintln!("  schema                 Print the event type schema");
}

// ── analyze ────────────────────────────────────────────────────────

fn cmd_analyze(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: exp026_game_telemetry analyze <file.ndjson | ->");
        std::process::exit(1);
    }

    let (events, errors) = if args[0] == "-" {
        let stdin = io::stdin();
        ludospring_barracuda::telemetry::parse_ndjson_reader(stdin.lock())
    } else {
        let path = PathBuf::from(&args[0]);
        let content = std::fs::read_to_string(&path).unwrap_or_else(|e| {
            eprintln!("Error reading {}: {e}", path.display());
            std::process::exit(1);
        });
        ludospring_barracuda::telemetry::parse_ndjson(&content)
    };

    eprintln!("Parsed {} events ({errors} errors)", events.len());

    let mut acc = SessionAccumulator::new();
    acc.ingest_all(&events);

    let report = generate_report(&acc);
    let json = serde_json::to_string_pretty(&report).unwrap_or_else(|e| {
        eprintln!("Serialization error: {e}");
        std::process::exit(1);
    });
    println!("{json}");
}

// ── generate ───────────────────────────────────────────────────────

fn cmd_generate(args: &[String]) {
    let events = generate_synthetic_session();

    if args.is_empty() || args[0] == "-" {
        let stdout = io::stdout();
        let mut out = stdout.lock();
        for evt in &events {
            let json = serde_json::to_string(evt).unwrap_or_default();
            let _ = writeln!(out, "{json}");
        }
    } else {
        let path = PathBuf::from(&args[0]);
        let mut file = std::fs::File::create(&path).unwrap_or_else(|e| {
            eprintln!("Error creating {}: {e}", path.display());
            std::process::exit(1);
        });
        for evt in &events {
            let json = serde_json::to_string(evt).unwrap_or_default();
            let _ = writeln!(file, "{json}");
        }
        eprintln!("Wrote {} events to {}", events.len(), path.display());
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "synthetic session builder — event sequence"
)]
fn generate_synthetic_session() -> Vec<TelemetryEvent> {
    let sid = "synthetic-001";
    let mut events = Vec::new();
    let mut ts: u64 = 0;

    events.push(TelemetryEvent {
        timestamp_ms: ts,
        session_id: sid.into(),
        event_type: EventType::SessionStart,
        payload: serde_json::to_value(SessionStartPayload {
            game_name: "synthetic_roguelike".into(),
            game_version: "0.1.0".into(),
            genre: "roguelike".into(),
        })
        .unwrap_or_default(),
    });

    for tick in 1..=300 {
        ts = tick * 200;
        let x = (tick as f64 * 0.1).sin() * 10.0;
        let y = (tick as f64 * 0.07).cos() * 10.0;

        events.push(TelemetryEvent {
            timestamp_ms: ts,
            session_id: sid.into(),
            event_type: EventType::PlayerMove,
            payload: serde_json::to_value(PlayerMovePayload {
                x,
                y,
                angle: tick as f64 * 0.05,
                speed: 2.0,
                ..Default::default()
            })
            .unwrap_or_default(),
        });

        if tick % 5 == 0 {
            events.push(TelemetryEvent {
                timestamp_ms: ts + 50,
                session_id: sid.into(),
                event_type: EventType::PlayerAction,
                payload: serde_json::to_value(PlayerActionPayload {
                    action: "attack".into(),
                    success: tick % 3 != 0,
                    ..Default::default()
                })
                .unwrap_or_default(),
            });
        }

        if tick % 30 == 0 {
            let floor = tick / 30;
            events.push(TelemetryEvent {
                timestamp_ms: ts + 100,
                session_id: sid.into(),
                event_type: EventType::ExplorationDiscover,
                payload: serde_json::to_value(ExplorationPayload {
                    discovery_id: format!("floor_{floor}"),
                    category: "area".into(),
                    x,
                    y,
                })
                .unwrap_or_default(),
            });
        }

        if tick % 20 == 0 {
            let difficulty = (tick as f64 / 300.0).min(1.0);
            events.push(TelemetryEvent {
                timestamp_ms: ts + 150,
                session_id: sid.into(),
                event_type: EventType::ChallengeEncounter,
                payload: serde_json::to_value(ChallengePayload {
                    challenge_id: format!("combat_{tick}"),
                    difficulty,
                    challenge_type: "combat".into(),
                })
                .unwrap_or_default(),
            });

            if tick % 40 == 0 {
                events.push(TelemetryEvent {
                    timestamp_ms: ts + 200,
                    session_id: sid.into(),
                    event_type: EventType::ChallengeFail,
                    payload: serde_json::to_value(ChallengePayload {
                        challenge_id: format!("combat_{tick}"),
                        difficulty,
                        challenge_type: "combat".into(),
                    })
                    .unwrap_or_default(),
                });
                events.push(TelemetryEvent {
                    timestamp_ms: ts + 250,
                    session_id: sid.into(),
                    event_type: EventType::PlayerDamage,
                    payload: serde_json::to_value(PlayerDamagePayload {
                        amount: 15.0,
                        source: "enemy".into(),
                        health_remaining: (tick as f64).mul_add(-0.1, 85.0),
                    })
                    .unwrap_or_default(),
                });
            } else {
                events.push(TelemetryEvent {
                    timestamp_ms: ts + 200,
                    session_id: sid.into(),
                    event_type: EventType::ChallengeComplete,
                    payload: serde_json::to_value(ChallengePayload {
                        challenge_id: format!("combat_{tick}"),
                        difficulty,
                        challenge_type: "combat".into(),
                    })
                    .unwrap_or_default(),
                });
            }
        }

        if tick == 200 {
            events.push(TelemetryEvent {
                timestamp_ms: ts + 300,
                session_id: sid.into(),
                event_type: EventType::PlayerDeath,
                payload: serde_json::to_value(PlayerDeathPayload {
                    cause: "boss_fight".into(),
                    respawn: true,
                })
                .unwrap_or_default(),
            });
        }
    }

    let duration_s = ts as f64 / 1000.0;
    events.push(TelemetryEvent {
        timestamp_ms: ts + 500,
        session_id: sid.into(),
        event_type: EventType::SessionEnd,
        payload: serde_json::to_value(SessionEndPayload {
            duration_s,
            reason: "completed".into(),
        })
        .unwrap_or_default(),
    });

    events
}

// ── validate ───────────────────────────────────────────────────────

fn cmd_validate() {
    let mut h = ValidationHarness::new("exp026_game_telemetry");
    h.print_provenance(&[&PROVENANCE]);

    validate_roundtrip(&mut h);
    validate_ndjson_parsing(&mut h);
    validate_synthetic_analysis(&mut h);
    validate_empty_session(&mut h);
    validate_all_event_types(&mut h);

    h.finish();
}

fn validate_roundtrip(h: &mut ValidationHarness) {
    let events = generate_synthetic_session();
    let mut ndjson = String::new();
    for evt in &events {
        ndjson.push_str(&serde_json::to_string(evt).unwrap_or_default());
        ndjson.push('\n');
    }
    let (parsed, errors) = ludospring_barracuda::telemetry::parse_ndjson(&ndjson);

    h.check_abs("roundtrip_no_errors", errors as f64, 0.0, 0.0);
    h.check_abs(
        "roundtrip_event_count",
        parsed.len() as f64,
        events.len() as f64,
        0.0,
    );
}

fn validate_ndjson_parsing(h: &mut ValidationHarness) {
    let input = r#"{"timestamp_ms":0,"session_id":"s","event_type":"session_start","payload":{}}
not json
{"timestamp_ms":1000,"session_id":"s","event_type":"player_move","payload":{"x":1.0,"y":2.0}}
"#;
    let (events, errors) = ludospring_barracuda::telemetry::parse_ndjson(input);
    h.check_abs("ndjson_valid_count", events.len() as f64, 2.0, 0.0);
    h.check_abs("ndjson_error_count", errors as f64, 1.0, 0.0);
}

fn validate_synthetic_analysis(h: &mut ValidationHarness) {
    let events = generate_synthetic_session();
    let mut acc = SessionAccumulator::new();
    acc.ingest_all(&events);
    let report = generate_report(&acc);

    h.check_bool(
        "game_name_correct",
        report.session.game_name == "synthetic_roguelike",
    );
    h.check_bool("engagement_positive", report.engagement.composite > 0.0);
    h.check_bool("has_flow_samples", !report.flow.timeline.is_empty());
    h.check_bool("fun_classified", !report.fun.dominant.is_empty());
    h.check_abs("deaths_counted", f64::from(report.session.deaths), 1.0, 0.0);
    h.check_bool("discoveries_positive", report.session.total_discoveries > 0);
    h.check_bool("report_serializes", serde_json::to_string(&report).is_ok());
}

fn validate_empty_session(h: &mut ValidationHarness) {
    let acc = SessionAccumulator::new();
    let report = generate_report(&acc);
    h.check_bool(
        "empty_session_no_panic",
        report.engagement.composite.is_finite(),
    );
}

fn validate_all_event_types(h: &mut ValidationHarness) {
    let types = [
        EventType::SessionStart,
        EventType::SessionEnd,
        EventType::PlayerMove,
        EventType::PlayerAction,
        EventType::PlayerDamage,
        EventType::PlayerDeath,
        EventType::ChallengeEncounter,
        EventType::ChallengeComplete,
        EventType::ChallengeFail,
        EventType::ExplorationDiscover,
        EventType::UiInteract,
        EventType::UiLayout,
        EventType::InputRaw,
    ];
    let mut all_pass = true;
    for et in &types {
        let evt = TelemetryEvent {
            timestamp_ms: 0,
            session_id: "test".into(),
            event_type: et.clone(),
            payload: serde_json::json!({}),
        };
        let json = serde_json::to_string(&evt).unwrap_or_default();
        let back: Result<TelemetryEvent, _> = serde_json::from_str(&json);
        if back.is_err() {
            all_pass = false;
        }
    }
    h.check_bool("all_event_types_roundtrip", all_pass);
}

// ── schema ─────────────────────────────────────────────────────────

fn cmd_schema() {
    println!("ludoSpring Game Telemetry Protocol v1");
    println!("=====================================\n");
    println!("Wire format: NDJSON (one JSON object per line)");
    println!("Transport: file, stdin, Unix socket (JSON-RPC), HTTP POST\n");
    println!("Event envelope:");
    println!("  {{");
    println!("    \"timestamp_ms\": <u64>,");
    println!("    \"session_id\": \"<string>\",");
    println!("    \"event_type\": \"<snake_case_type>\",");
    println!("    \"payload\": {{ ... }}");
    println!("  }}\n");
    println!("Event types:");
    let types = [
        ("session_start", "Game session started"),
        ("session_end", "Game session ended"),
        ("player_move", "Player position update"),
        ("player_action", "Discrete action (attack, use, etc.)"),
        ("player_damage", "Player took damage"),
        ("player_death", "Player died"),
        ("challenge_encounter", "Challenge started"),
        ("challenge_complete", "Challenge succeeded"),
        ("challenge_fail", "Challenge failed"),
        ("exploration_discover", "New area/item discovered"),
        ("ui_interact", "UI element interaction"),
        ("ui_layout", "UI layout snapshot"),
        ("input_raw", "Raw input event"),
    ];
    for (name, desc) in &types {
        println!("  {name:25} -- {desc}");
    }
    println!("\nPayload schemas: see ludospring_barracuda::telemetry::events");
    println!("\nPortability:");
    println!("  Any language that can write JSON can produce these events.");
    println!("  Unity (C#), Godot (GDScript), web (JS) -- all compatible.");
}

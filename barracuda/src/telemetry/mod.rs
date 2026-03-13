// SPDX-License-Identifier: AGPL-3.0-or-later
//! Portable game telemetry — engine-agnostic event protocol and analysis.
//!
//! # Architecture
//!
//! ```text
//! Any game ──> NDJSON events ──> SessionAccumulator ──> Analysis engines ──> Report
//! ```
//!
//! The telemetry module defines a standard event format that any game engine
//! can emit (Rust, Unity, Godot, web). Events are ingested into a
//! `SessionAccumulator`, which builds the inputs for ludoSpring's existing
//! analysis functions (engagement, flow, DDA, fun classification, Tufte UI,
//! interaction costs). The result is a `GameplayAnalysisReport`.
//!
//! # Protocol
//!
//! Events are newline-delimited JSON (NDJSON). Each line is a
//! `TelemetryEvent` with a `timestamp_ms`, `session_id`, `event_type`,
//! and `payload`.
//!
//! # Usage
//!
//! ```rust
//! use ludospring_barracuda::telemetry::events::{TelemetryEvent, EventType};
//! use ludospring_barracuda::telemetry::mapper::SessionAccumulator;
//! use ludospring_barracuda::telemetry::report::generate_report;
//!
//! let mut acc = SessionAccumulator::new();
//! // ... ingest events ...
//! let report = generate_report(&acc);
//! ```

/// Telemetry event types and payloads.
pub mod events;
/// Event-to-metric mapping.
pub mod mapper;
/// Report generation.
pub mod report;

/// Parse NDJSON telemetry from a string.
///
/// Skips malformed lines with a warning count.
#[must_use]
pub fn parse_ndjson(input: &str) -> (Vec<events::TelemetryEvent>, usize) {
    let mut events = Vec::new();
    let mut errors = 0;
    for line in input.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        match serde_json::from_str::<events::TelemetryEvent>(trimmed) {
            Ok(evt) => events.push(evt),
            Err(_) => errors += 1,
        }
    }
    (events, errors)
}

/// Streaming NDJSON parser — yields events one at a time without buffering.
///
/// Errors are silently skipped (same as `parse_ndjson_reader`). For error
/// reporting, callers can chain `.inspect()` on the reader.
#[must_use]
pub const fn iter_ndjson<R: std::io::BufRead>(reader: R) -> NdjsonIter<R> {
    NdjsonIter { reader }
}

/// Streaming iterator over NDJSON telemetry events.
pub struct NdjsonIter<R> {
    reader: R,
}

impl<R: std::io::BufRead> Iterator for NdjsonIter<R> {
    type Item = events::TelemetryEvent;

    fn next(&mut self) -> Option<Self::Item> {
        let mut line = String::new();
        loop {
            line.clear();
            let bytes_read = self.reader.read_line(&mut line).ok()?;
            if bytes_read == 0 {
                return None; // EOF
            }
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            if let Ok(evt) = serde_json::from_str::<events::TelemetryEvent>(trimmed) {
                return Some(evt);
            }
        }
    }
}

/// Parse NDJSON from a reader (file, stdin, etc.).
pub fn parse_ndjson_reader<R: std::io::BufRead>(reader: R) -> (Vec<events::TelemetryEvent>, usize) {
    let mut events = Vec::new();
    let mut errors = 0;
    for line in reader.lines() {
        match line {
            Ok(l) => {
                let trimmed = l.trim().to_string();
                if trimmed.is_empty() {
                    continue;
                }
                match serde_json::from_str::<events::TelemetryEvent>(&trimmed) {
                    Ok(evt) => events.push(evt),
                    Err(_) => errors += 1,
                }
            }
            Err(_) => errors += 1,
        }
    }
    (events, errors)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ndjson_basic() {
        let input = r#"{"timestamp_ms":0,"session_id":"s","event_type":"session_start","payload":{}}
{"timestamp_ms":1000,"session_id":"s","event_type":"player_move","payload":{"x":1.0,"y":2.0}}
this is not json
{"timestamp_ms":2000,"session_id":"s","event_type":"session_end","payload":{"duration_s":2.0}}
"#;
        let (events, errors) = parse_ndjson(input);
        assert_eq!(events.len(), 3);
        assert_eq!(errors, 1);
    }

    #[test]
    fn iter_ndjson_streaming() {
        let input = r#"{"timestamp_ms":0,"session_id":"s","event_type":"session_start","payload":{}}
{"timestamp_ms":1000,"session_id":"s","event_type":"player_move","payload":{"x":1.0,"y":2.0}}
bad line
{"timestamp_ms":2000,"session_id":"s","event_type":"session_end","payload":{"duration_s":2.0}}
"#;
        let reader = std::io::Cursor::new(input);
        let events: Vec<_> = iter_ndjson(reader).collect();
        assert_eq!(events.len(), 3, "malformed line should be skipped");
        assert_eq!(events[0].event_type, events::EventType::SessionStart);
        assert_eq!(events[1].event_type, events::EventType::PlayerMove);
        assert_eq!(events[2].event_type, events::EventType::SessionEnd);
    }

    #[test]
    fn end_to_end_ndjson_to_report() {
        let input = r#"{"timestamp_ms":0,"session_id":"s","event_type":"session_start","payload":{"game_name":"demo","genre":"action"}}
{"timestamp_ms":1000,"session_id":"s","event_type":"player_action","payload":{"action":"attack","success":true}}
{"timestamp_ms":2000,"session_id":"s","event_type":"challenge_encounter","payload":{"difficulty":0.5,"challenge_type":"combat"}}
{"timestamp_ms":3000,"session_id":"s","event_type":"challenge_complete","payload":{}}
{"timestamp_ms":4000,"session_id":"s","event_type":"exploration_discover","payload":{"discovery_id":"cave_1"}}
{"timestamp_ms":5000,"session_id":"s","event_type":"session_end","payload":{"duration_s":5.0}}
"#;
        let (events, errors) = parse_ndjson(input);
        assert_eq!(errors, 0);

        let mut acc = mapper::SessionAccumulator::new();
        acc.ingest_all(&events);

        let report = report::generate_report(&acc);
        assert_eq!(report.session.game_name, "demo");
        assert!(report.engagement.composite > 0.0);
    }
}

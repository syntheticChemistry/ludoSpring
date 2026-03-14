// SPDX-License-Identifier: AGPL-3.0-or-later
//! Event-to-metric mapper — accumulates telemetry events into ludoSpring
//! analysis inputs.
//!
//! The mapper is engine-agnostic: it consumes `TelemetryEvent` streams
//! (from any producer) and builds the structs that ludoSpring's existing
//! analysis functions expect.

use std::collections::HashSet;

use crate::interaction::difficulty::PerformanceWindow;
use crate::interaction::flow::FlowState;
use crate::metrics::engagement::EngagementSnapshot;
use crate::metrics::fun_keys::FunSignals;
use crate::metrics::tufte_gaming::UiElement;

use super::events::{
    ChallengePayload, EventType, ExplorationPayload, PlayerActionPayload, PlayerDamagePayload,
    PlayerMovePayload, SessionEndPayload, SessionStartPayload, TelemetryEvent, UiElementDescriptor,
    UiInteractPayload, UiLayoutPayload,
};

/// Accumulated state from an event stream, ready for analysis.
#[derive(Debug, Clone)]
pub struct SessionAccumulator {
    /// Session ID from the first telemetry event.
    pub session_id: String,
    /// Game metadata from `session_start`.
    pub game_name: String,
    /// Genre hint.
    pub genre: String,
    /// Session duration in seconds (from `session_end` or last event timestamp).
    pub duration_s: f64,
    /// First event timestamp.
    pub first_timestamp_ms: Option<u64>,
    /// Last event timestamp.
    pub last_timestamp_ms: Option<u64>,
    /// Total action count.
    pub action_count: u64,
    /// Distinct areas/items discovered.
    pub discoveries: HashSet<String>,
    /// Challenge encounters.
    pub challenge_count: u32,
    /// Challenge completions.
    pub challenge_completions: u32,
    /// Challenge failures.
    pub challenge_failures: u32,
    /// Retry indicators (deaths followed by continued play).
    pub retry_count: u32,
    /// Total damage taken.
    pub total_damage: f64,
    /// Death count.
    pub death_count: u32,
    /// Performance window for DDA.
    pub performance: PerformanceWindow,
    /// Flow state samples: (`timestamp_ms`, challenge, skill).
    pub flow_samples: Vec<(u64, f64, f64)>,
    /// UI layout snapshots.
    pub ui_layouts: Vec<Vec<UiElementDescriptor>>,
    /// UI interaction records for Fitts/Hick analysis.
    pub ui_interactions: Vec<UiInteractPayload>,
    /// Player positions for movement analysis.
    pub positions: Vec<(u64, f64, f64)>,
    /// Input events for GOMS analysis.
    pub input_actions: Vec<(u64, String)>,
    /// Social/cooperative signals.
    pub social_actions: u32,
    /// Completion/collection signals.
    pub completion_actions: u32,
    /// Deliberate pauses detected.
    pub deliberate_pauses: u32,
    /// Whether session has ended.
    pub ended: bool,
}

impl Default for SessionAccumulator {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionAccumulator {
    /// Create a new empty accumulator.
    #[must_use]
    pub fn new() -> Self {
        Self {
            session_id: String::new(),
            game_name: String::new(),
            genre: String::new(),
            duration_s: 0.0,
            first_timestamp_ms: None,
            last_timestamp_ms: None,
            action_count: 0,
            discoveries: HashSet::new(),
            challenge_count: 0,
            challenge_completions: 0,
            challenge_failures: 0,
            retry_count: 0,
            total_damage: 0.0,
            death_count: 0,
            // 50-event sliding window balances responsiveness with stability.
            // Typical action rate is ~1/s; 50 events ≈ ~50s of recent history.
            performance: PerformanceWindow::new(50),
            flow_samples: Vec::new(),
            ui_layouts: Vec::new(),
            ui_interactions: Vec::new(),
            positions: Vec::new(),
            input_actions: Vec::new(),
            social_actions: 0,
            completion_actions: 0,
            deliberate_pauses: 0,
            ended: false,
        }
    }

    /// Ingest a single telemetry event.
    pub fn ingest(&mut self, event: &TelemetryEvent) {
        if self.first_timestamp_ms.is_none() {
            self.first_timestamp_ms = Some(event.timestamp_ms);
            self.session_id.clone_from(&event.session_id);
        }
        self.last_timestamp_ms = Some(event.timestamp_ms);

        match &event.event_type {
            EventType::SessionStart => self.handle_session_start(event),
            EventType::SessionEnd => self.handle_session_end(event),
            EventType::PlayerMove => self.handle_player_move(event),
            EventType::PlayerAction => self.handle_player_action(event),
            EventType::PlayerDamage => self.handle_player_damage(event),
            EventType::PlayerDeath => self.handle_player_death(event),
            EventType::ChallengeEncounter => self.handle_challenge_encounter(event),
            EventType::ChallengeComplete => self.handle_challenge_complete(event),
            EventType::ChallengeFail => self.handle_challenge_fail(event),
            EventType::ExplorationDiscover => self.handle_exploration(event),
            EventType::UiInteract => self.handle_ui_interact(event),
            EventType::UiLayout => self.handle_ui_layout(event),
            EventType::InputRaw => self.handle_input_raw(event),
            EventType::Custom(_) => {}
        }
    }

    /// Ingest a batch of events.
    pub fn ingest_all(&mut self, events: &[TelemetryEvent]) {
        for event in events {
            self.ingest(event);
        }
    }

    // ── Handlers ───────────────────────────────────────────────────

    fn handle_session_start(&mut self, event: &TelemetryEvent) {
        if let Some(p) = event.parse_payload::<SessionStartPayload>() {
            self.game_name = p.game_name;
            self.genre = p.genre;
        }
    }

    fn handle_session_end(&mut self, event: &TelemetryEvent) {
        self.ended = true;
        if let Some(p) = event.parse_payload::<SessionEndPayload>() {
            if p.duration_s > 0.0 {
                self.duration_s = p.duration_s;
            }
        }
    }

    fn handle_player_move(&mut self, event: &TelemetryEvent) {
        if let Some(p) = event.parse_payload::<PlayerMovePayload>() {
            self.positions.push((event.timestamp_ms, p.x, p.y));
            self.detect_pause(event.timestamp_ms);
        }
    }

    fn handle_player_action(&mut self, event: &TelemetryEvent) {
        self.action_count += 1;
        if let Some(p) = event.parse_payload::<PlayerActionPayload>() {
            self.input_actions
                .push((event.timestamp_ms, p.action.clone()));
            if is_social_action(&p.action) {
                self.social_actions += 1;
            }
            if is_completion_action(&p.action) {
                self.completion_actions += 1;
            }
        }
    }

    fn handle_player_damage(&mut self, event: &TelemetryEvent) {
        if let Some(p) = event.parse_payload::<PlayerDamagePayload>() {
            self.total_damage += p.amount;
        }
    }

    const fn handle_player_death(&mut self, event: &TelemetryEvent) {
        let _ = event;
        self.death_count += 1;
        self.retry_count += 1;
    }

    fn handle_challenge_encounter(&mut self, event: &TelemetryEvent) {
        self.challenge_count += 1;
        if let Some(p) = event.parse_payload::<ChallengePayload>() {
            let skill = self.performance.estimated_skill();
            self.flow_samples
                .push((event.timestamp_ms, p.difficulty, skill));
        }
    }

    fn handle_challenge_complete(&mut self, event: &TelemetryEvent) {
        let _ = event;
        self.challenge_completions += 1;
        self.performance.record(1.0);
        self.action_count += 1;
    }

    fn handle_challenge_fail(&mut self, event: &TelemetryEvent) {
        let _ = event;
        self.challenge_failures += 1;
        self.performance.record(0.0);
        self.action_count += 1;
    }

    fn handle_exploration(&mut self, event: &TelemetryEvent) {
        self.action_count += 1;
        if let Some(p) = event.parse_payload::<ExplorationPayload>() {
            let id = if p.discovery_id.is_empty() {
                format!("area_{:.0}_{:.0}", p.x, p.y)
            } else {
                p.discovery_id
            };
            self.discoveries.insert(id);
        } else {
            self.discoveries
                .insert(format!("discovery_{}", self.discoveries.len()));
        }
    }

    fn handle_ui_interact(&mut self, event: &TelemetryEvent) {
        if let Some(p) = event.parse_payload::<UiInteractPayload>() {
            self.ui_interactions.push(p);
        }
    }

    fn handle_ui_layout(&mut self, event: &TelemetryEvent) {
        if let Some(p) = event.parse_payload::<UiLayoutPayload>() {
            self.ui_layouts.push(p.elements);
        }
    }

    fn handle_input_raw(&mut self, event: &TelemetryEvent) {
        self.input_actions
            .push((event.timestamp_ms, "keystroke".into()));
    }

    fn detect_pause(&mut self, timestamp_ms: u64) {
        /// Milliseconds of inactivity before a pause is counted.
        ///
        /// Provenance: 3 seconds aligns with typical "deliberate pause" in game UX.
        /// Source: Zhu & Fang (2012), "Visualizing Game Telemetry Data," GDC Vault.
        const PAUSE_THRESHOLD_MS: u64 = 3000;
        if let Some(prev) = self.positions.iter().rev().nth(1).map(|(t, _, _)| *t) {
            if timestamp_ms.saturating_sub(prev) > PAUSE_THRESHOLD_MS {
                self.deliberate_pauses += 1;
            }
        }
    }

    // ── Metric Extraction ──────────────────────────────────────────

    /// Compute effective session duration (seconds).
    #[must_use]
    #[expect(
        clippy::cast_precision_loss,
        reason = "timestamp deltas fit in f64 mantissa"
    )]
    pub fn effective_duration_s(&self) -> f64 {
        if self.duration_s > 0.0 {
            return self.duration_s;
        }
        match (self.first_timestamp_ms, self.last_timestamp_ms) {
            (Some(first), Some(last)) => (last.saturating_sub(first)) as f64 / 1000.0,
            _ => 0.0,
        }
    }

    /// Build an `EngagementSnapshot` from accumulated events.
    #[must_use]
    #[expect(
        clippy::cast_possible_truncation,
        reason = "discovery/action counts are small"
    )]
    pub fn to_engagement_snapshot(&self) -> EngagementSnapshot {
        EngagementSnapshot {
            session_duration_s: self.effective_duration_s(),
            action_count: self.action_count,
            exploration_breadth: self.discoveries.len() as u32,
            challenge_seeking: self.challenge_count,
            retry_count: self.retry_count,
            deliberate_pauses: self.deliberate_pauses,
        }
    }

    /// Sample flow states from challenge encounters.
    #[must_use]
    pub fn to_flow_timeline(&self) -> Vec<(u64, FlowState)> {
        self.flow_samples
            .iter()
            .map(|(t, challenge, skill)| {
                let state = crate::interaction::flow::evaluate_flow(
                    *challenge,
                    *skill,
                    crate::tolerances::FLOW_CHANNEL_WIDTH,
                );
                (*t, state)
            })
            .collect()
    }

    /// Build `FunSignals` from accumulated behavioral data.
    #[must_use]
    #[expect(clippy::cast_precision_loss, reason = "counts are small")]
    pub fn to_fun_signals(&self) -> FunSignals {
        let total = (self.action_count.max(1)) as f64;
        FunSignals {
            challenge: (f64::from(self.challenge_count) / total).min(1.0),
            exploration: (self.discoveries.len() as f64 / total).min(1.0),
            social: (f64::from(self.social_actions) / total).min(1.0),
            completion: (f64::from(self.completion_actions) / total).min(1.0),
            retry_rate: (f64::from(self.retry_count) / total).min(1.0),
        }
    }

    /// Convert the latest UI layout to `UiElement` list for Tufte analysis.
    #[must_use]
    pub fn to_ui_elements(&self) -> Vec<UiElement> {
        self.ui_layouts
            .last()
            .map(|layout| {
                layout
                    .iter()
                    .map(|desc| UiElement {
                        name: desc.name.clone(),
                        bounds: desc.bounds,
                        data_values: desc.data_values,
                        pixel_area: desc.pixel_area,
                        data_ink_area: desc.data_ink_area,
                        critical: desc.critical,
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get the performance window reference for DDA analysis.
    #[must_use]
    pub const fn performance_window(&self) -> &PerformanceWindow {
        &self.performance
    }

    /// Complete the provenance trio workflow for this session.
    ///
    /// Triggers dehydration (rhizoCrypt), commit (LoamSpine), and attribution
    /// (sweetGrass) via biomeOS capability routing. Returns `None` if the
    /// session hasn't ended or the IPC feature is disabled.
    #[cfg(feature = "ipc")]
    pub fn complete_provenance(&self) -> Option<serde_json::Value> {
        if !self.ended || self.session_id.is_empty() {
            return None;
        }
        crate::ipc::provenance::complete_game_session(&self.session_id).ok()
    }
}

fn is_social_action(action: &str) -> bool {
    matches!(
        action,
        "chat" | "emote" | "trade" | "invite" | "cooperate" | "group" | "wave" | "ping"
    )
}

fn is_completion_action(action: &str) -> bool {
    matches!(
        action,
        "collect" | "complete" | "achieve" | "unlock" | "finish" | "craft" | "build"
    )
}

#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;
    use crate::metrics::engagement::compute_engagement;
    use crate::metrics::fun_keys::classify_fun;

    fn make_event(ts: u64, event_type: EventType, payload: serde_json::Value) -> TelemetryEvent {
        TelemetryEvent {
            timestamp_ms: ts,
            session_id: "test".into(),
            event_type,
            payload,
        }
    }

    #[test]
    fn accumulator_builds_engagement() {
        let mut acc = SessionAccumulator::new();
        acc.ingest(&make_event(
            0,
            EventType::SessionStart,
            serde_json::json!({"game_name": "test_game"}),
        ));
        for i in 1..=100 {
            acc.ingest(&make_event(
                i * 1000,
                EventType::PlayerAction,
                serde_json::json!({"action": "move"}),
            ));
        }
        for i in 0..5 {
            acc.ingest(&make_event(
                (i + 101) * 1000,
                EventType::ExplorationDiscover,
                serde_json::json!({"discovery_id": format!("area_{i}")}),
            ));
        }
        acc.ingest(&make_event(
            120_000,
            EventType::SessionEnd,
            serde_json::json!({"duration_s": 120.0}),
        ));

        let snap = acc.to_engagement_snapshot();
        let metrics = compute_engagement(&snap);
        assert!(metrics.composite > 0.0);
        assert!(metrics.actions_per_minute > 0.0);
        assert_eq!(snap.exploration_breadth, 5);
    }

    #[test]
    fn accumulator_builds_flow_timeline() {
        let mut acc = SessionAccumulator::new();
        for i in 0..5 {
            acc.ingest(&make_event(
                i * 1000,
                EventType::ChallengeComplete,
                serde_json::json!({}),
            ));
        }
        acc.ingest(&make_event(
            6000,
            EventType::ChallengeEncounter,
            serde_json::json!({"difficulty": 0.5, "challenge_type": "combat"}),
        ));
        let timeline = acc.to_flow_timeline();
        assert_eq!(timeline.len(), 1);
    }

    #[test]
    fn accumulator_builds_fun_signals() {
        let mut acc = SessionAccumulator::new();
        for _ in 0..10 {
            acc.ingest(&make_event(
                0,
                EventType::PlayerAction,
                serde_json::json!({"action": "attack"}),
            ));
        }
        for _ in 0..3 {
            acc.ingest(&make_event(
                0,
                EventType::ChallengeEncounter,
                serde_json::json!({"difficulty": 0.8}),
            ));
        }
        for _ in 0..2 {
            acc.ingest(&make_event(
                0,
                EventType::ChallengeFail,
                serde_json::json!({}),
            ));
        }
        let signals = acc.to_fun_signals();
        let classification = classify_fun(&signals);
        assert!(classification.scores.hard > 0.0);
    }

    #[test]
    fn empty_events_ingest_no_op() {
        let mut acc = SessionAccumulator::new();
        acc.ingest_all(&[]);
        assert_eq!(acc.action_count, 0);
        assert!(acc.first_timestamp_ms.is_none());
        assert!(acc.last_timestamp_ms.is_none());
    }

    #[test]
    fn custom_event_type_ignored() {
        let mut acc = SessionAccumulator::new();
        acc.ingest(&make_event(
            1000,
            EventType::Custom("unknown_event".into()),
            serde_json::json!({}),
        ));
        assert_eq!(acc.action_count, 0);
        assert_eq!(acc.first_timestamp_ms, Some(1000));
        assert_eq!(acc.last_timestamp_ms, Some(1000));
    }

    #[test]
    fn player_death_increments_count() {
        let mut acc = SessionAccumulator::new();
        acc.ingest(&make_event(
            0,
            EventType::PlayerDeath,
            serde_json::json!({}),
        ));
        acc.ingest(&make_event(
            1000,
            EventType::PlayerDeath,
            serde_json::json!({}),
        ));
        assert_eq!(acc.death_count, 2);
        assert_eq!(acc.retry_count, 2);
    }

    #[test]
    fn player_damage_accumulates() {
        let mut acc = SessionAccumulator::new();
        acc.ingest(&make_event(
            0,
            EventType::PlayerDamage,
            serde_json::json!({"amount": 10.0}),
        ));
        acc.ingest(&make_event(
            1000,
            EventType::PlayerDamage,
            serde_json::json!({"amount": 25.5}),
        ));
        assert!((acc.total_damage - 35.5).abs() < f64::EPSILON);
    }

    #[test]
    fn exploration_empty_discovery_id_uses_coords() {
        let mut acc = SessionAccumulator::new();
        acc.ingest(&make_event(
            0,
            EventType::ExplorationDiscover,
            serde_json::json!({"x": 42.0, "y": 17.0}),
        ));
        assert!(acc.discoveries.contains("area_42_17"));
    }

    #[test]
    fn exploration_parse_failure_uses_fallback_id() {
        let mut acc = SessionAccumulator::new();
        acc.ingest(&make_event(
            0,
            EventType::ExplorationDiscover,
            serde_json::json!("invalid"),
        ));
        assert!(acc.discoveries.contains("discovery_0"));
    }

    #[test]
    fn session_end_zero_duration_keeps_timestamp_delta() {
        let mut acc = SessionAccumulator::new();
        acc.ingest(&make_event(
            1000,
            EventType::SessionStart,
            serde_json::json!({"game_name": "x"}),
        ));
        acc.ingest(&make_event(
            5000,
            EventType::SessionEnd,
            serde_json::json!({"duration_s": 0.0}),
        ));
        assert!(acc.ended);
        assert!((acc.effective_duration_s() - 4.0).abs() < 0.001);
    }

    #[test]
    fn effective_duration_empty_returns_zero() {
        let acc = SessionAccumulator::new();
        assert!(acc.effective_duration_s().abs() < f64::EPSILON);
    }

    #[test]
    fn ui_layout_and_interact_populate_report_inputs() {
        let mut acc = SessionAccumulator::new();
        acc.ingest(&make_event(
            0,
            EventType::UiLayout,
            serde_json::json!({
                "elements": [{"name": "btn", "bounds": [0.1, 0.1, 0.2, 0.1]}]
            }),
        ));
        acc.ingest(&make_event(
            1000,
            EventType::UiInteract,
            serde_json::json!({
                "element": "btn",
                "distance_px": 100.0,
                "target_width_px": 50.0,
                "n_options": 4
            }),
        ));
        let elements = acc.to_ui_elements();
        assert_eq!(elements.len(), 1);
        assert_eq!(elements[0].name, "btn");
        assert_eq!(acc.ui_interactions.len(), 1);
    }

    #[test]
    fn input_raw_adds_keystroke() {
        let mut acc = SessionAccumulator::new();
        acc.ingest(&make_event(0, EventType::InputRaw, serde_json::json!({})));
        assert_eq!(acc.input_actions.len(), 1);
        assert_eq!(acc.input_actions[0].1, "keystroke");
    }

    #[test]
    fn social_and_completion_actions_counted() {
        let mut acc = SessionAccumulator::new();
        acc.ingest(&make_event(
            0,
            EventType::PlayerAction,
            serde_json::json!({"action": "chat"}),
        ));
        acc.ingest(&make_event(
            1000,
            EventType::PlayerAction,
            serde_json::json!({"action": "collect"}),
        ));
        assert_eq!(acc.social_actions, 1);
        assert_eq!(acc.completion_actions, 1);
    }

    #[test]
    fn detect_pause_on_gap() {
        let mut acc = SessionAccumulator::new();
        acc.ingest(&make_event(
            0,
            EventType::PlayerMove,
            serde_json::json!({"x": 1.0, "y": 1.0}),
        ));
        acc.ingest(&make_event(
            5000,
            EventType::PlayerMove,
            serde_json::json!({"x": 2.0, "y": 2.0}),
        ));
        assert_eq!(acc.deliberate_pauses, 1);
    }

    #[test]
    fn to_ui_elements_empty_when_no_layout() {
        let acc = SessionAccumulator::new();
        assert!(acc.to_ui_elements().is_empty());
    }

    #[test]
    fn performance_window_accessible() {
        let acc = SessionAccumulator::new();
        let _ = acc.performance_window();
    }

    #[test]
    fn default_accumulator_matches_new() {
        let def: SessionAccumulator = SessionAccumulator::default();
        let new = SessionAccumulator::new();
        assert_eq!(def.action_count, new.action_count);
        assert_eq!(def.ended, new.ended);
    }
}

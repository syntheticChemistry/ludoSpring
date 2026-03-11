// SPDX-License-Identifier: AGPL-3.0-or-later
//! Gameplay analysis report — the output of telemetry analysis.
//!
//! Takes accumulated session data and runs it through all applicable
//! ludoSpring analysis engines to produce a comprehensive report.

use serde::Serialize;

use crate::interaction::difficulty::suggest_adjustment;
use crate::interaction::flow::FlowState;
use crate::interaction::input_laws;
use crate::metrics::engagement::{compute_engagement, EngagementMetrics};
use crate::metrics::fun_keys::{classify_fun, FunClassification};
use crate::metrics::tufte_gaming::{analyze_game_ui, GameUiTufteReport};
use crate::tolerances;

use super::mapper::SessionAccumulator;

/// Complete gameplay analysis report.
#[derive(Debug, Clone, Serialize)]
pub struct GameplayAnalysisReport {
    /// Game metadata.
    pub session: SessionSummary,
    /// Engagement analysis.
    pub engagement: EngagementReport,
    /// Flow state timeline.
    pub flow: FlowReport,
    /// Dynamic difficulty adjustment analysis.
    pub difficulty: DifficultyReport,
    /// Fun classification (Lazzaro Four Keys).
    pub fun: FunReport,
    /// UI Tufte analysis (if UI events were present).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ui_tufte: Option<TufteReport>,
    /// Interaction cost analysis (if UI interactions were present).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interaction_costs: Option<InteractionCostReport>,
}

/// Session metadata summary.
#[derive(Debug, Clone, Serialize)]
pub struct SessionSummary {
    /// Game name.
    pub game_name: String,
    /// Genre hint.
    pub genre: String,
    /// Total duration in seconds.
    pub duration_s: f64,
    /// Total actions.
    pub total_actions: u64,
    /// Total discoveries.
    pub total_discoveries: usize,
    /// Total challenges encountered.
    pub challenges_encountered: u32,
    /// Challenge success rate.
    pub challenge_success_rate: f64,
    /// Death count.
    pub deaths: u32,
}

/// Engagement metrics report.
#[derive(Debug, Clone, Serialize)]
pub struct EngagementReport {
    /// Composite score (0.0–1.0).
    pub composite: f64,
    /// Actions per minute.
    pub actions_per_minute: f64,
    /// Exploration rate (discoveries/min).
    pub exploration_rate: f64,
    /// Challenge appetite.
    pub challenge_appetite: f64,
    /// Persistence (retry rate).
    pub persistence: f64,
    /// Deliberation rate.
    pub deliberation: f64,
    /// Human-readable interpretation.
    pub interpretation: String,
}

/// Flow state timeline report.
#[derive(Debug, Clone, Serialize)]
pub struct FlowReport {
    /// Time spent in each flow state (seconds).
    pub time_in_flow: f64,
    /// Time spent in boredom.
    pub time_in_boredom: f64,
    /// Time spent in anxiety.
    pub time_in_anxiety: f64,
    /// Flow state at each sample point.
    pub timeline: Vec<FlowSample>,
    /// Dominant flow state.
    pub dominant_state: String,
    /// Human-readable interpretation.
    pub interpretation: String,
}

/// A single flow state sample.
#[derive(Debug, Clone, Serialize)]
pub struct FlowSample {
    /// Timestamp (ms).
    pub timestamp_ms: u64,
    /// Flow state name.
    pub state: String,
}

/// Difficulty adjustment report.
#[derive(Debug, Clone, Serialize)]
pub struct DifficultyReport {
    /// Current estimated skill.
    pub estimated_skill: f64,
    /// Performance trend (positive = improving).
    pub trend: f64,
    /// Suggested adjustment (-1.0 to 1.0).
    pub suggested_adjustment: f64,
    /// Human-readable recommendation.
    pub recommendation: String,
}

/// Fun classification report.
#[derive(Debug, Clone, Serialize)]
pub struct FunReport {
    /// Dominant fun type.
    pub dominant: String,
    /// Per-key scores.
    pub hard_fun: f64,
    /// Easy fun score.
    pub easy_fun: f64,
    /// People fun score.
    pub people_fun: f64,
    /// Serious fun score.
    pub serious_fun: f64,
    /// Human-readable interpretation.
    pub interpretation: String,
}

/// Tufte UI analysis report.
#[derive(Debug, Clone, Serialize)]
pub struct TufteReport {
    /// Overall data-ink ratio.
    pub data_ink_ratio: f64,
    /// Information density.
    pub info_density: f64,
    /// Screen coverage.
    pub screen_coverage: f64,
    /// Notes and recommendations.
    pub notes: Vec<String>,
    /// Per-element analysis count.
    pub elements_analyzed: usize,
}

/// Interaction cost analysis report.
#[derive(Debug, Clone, Serialize)]
pub struct InteractionCostReport {
    /// Average Fitts acquisition time (ms).
    pub avg_fitts_time_ms: f64,
    /// Average Hick decision time (ms).
    pub avg_hick_time_ms: f64,
    /// Total interactions analyzed.
    pub interactions_analyzed: usize,
    /// Costliest element.
    pub costliest_element: String,
    /// Costliest element time (ms).
    pub costliest_time_ms: f64,
}

/// Generate a complete gameplay analysis report from a session accumulator.
#[must_use]
pub fn generate_report(acc: &SessionAccumulator) -> GameplayAnalysisReport {
    let session = build_session_summary(acc);
    let engagement = build_engagement_report(acc);
    let flow = build_flow_report(acc);
    let difficulty = build_difficulty_report(acc);
    let fun = build_fun_report(acc);
    let ui_tufte = build_tufte_report(acc);
    let interaction_costs = build_interaction_cost_report(acc);

    GameplayAnalysisReport {
        session,
        engagement,
        flow,
        difficulty,
        fun,
        ui_tufte,
        interaction_costs,
    }
}

fn build_session_summary(acc: &SessionAccumulator) -> SessionSummary {
    let total_challenges = acc.challenge_completions + acc.challenge_failures;
    let success_rate = if total_challenges > 0 {
        f64::from(acc.challenge_completions) / f64::from(total_challenges)
    } else {
        0.0
    };

    SessionSummary {
        game_name: acc.game_name.clone(),
        genre: acc.genre.clone(),
        duration_s: acc.effective_duration_s(),
        total_actions: acc.action_count,
        total_discoveries: acc.discoveries.len(),
        challenges_encountered: acc.challenge_count,
        challenge_success_rate: success_rate,
        deaths: acc.death_count,
    }
}

fn build_engagement_report(acc: &SessionAccumulator) -> EngagementReport {
    let snap = acc.to_engagement_snapshot();
    let m: EngagementMetrics = compute_engagement(&snap);

    let interpretation = if m.composite > 0.7 {
        "Highly engaged — strong activity, exploration, and persistence."
    } else if m.composite > 0.4 {
        "Moderately engaged — steady play with room for deeper hooks."
    } else if m.composite > 0.15 {
        "Low engagement — consider pacing, challenge, or novelty changes."
    } else {
        "Minimal engagement — player may be disengaged or tutorial phase."
    };

    EngagementReport {
        composite: m.composite,
        actions_per_minute: m.actions_per_minute,
        exploration_rate: m.exploration_rate,
        challenge_appetite: m.challenge_appetite,
        persistence: m.persistence,
        deliberation: m.deliberation,
        interpretation: interpretation.into(),
    }
}

fn build_flow_report(acc: &SessionAccumulator) -> FlowReport {
    let timeline = acc.to_flow_timeline();
    let mut flow_count = 0u32;
    let mut boredom_count = 0u32;
    let mut anxiety_count = 0u32;
    let mut relaxation_count = 0u32;
    let mut arousal_count = 0u32;

    for (_, state) in &timeline {
        match state {
            FlowState::Flow => flow_count += 1,
            FlowState::Boredom => boredom_count += 1,
            FlowState::Anxiety => anxiety_count += 1,
            FlowState::Relaxation => relaxation_count += 1,
            FlowState::Arousal => arousal_count += 1,
        }
    }

    #[expect(clippy::cast_precision_loss, reason = "timeline samples are small")]
    let total = timeline.len().max(1) as f64;
    let duration = acc.effective_duration_s();

    let dominant = if flow_count >= boredom_count
        && flow_count >= anxiety_count
        && flow_count >= relaxation_count
        && flow_count >= arousal_count
    {
        "flow"
    } else if anxiety_count >= boredom_count
        && anxiety_count >= relaxation_count
        && anxiety_count >= arousal_count
    {
        "anxiety"
    } else if boredom_count >= relaxation_count && boredom_count >= arousal_count {
        "boredom"
    } else if arousal_count >= relaxation_count {
        "arousal"
    } else {
        "relaxation"
    };

    let interpretation = match dominant {
        "flow" => "Player spends most time in flow — difficulty is well-calibrated.",
        "anxiety" => "Player frequently in anxiety — challenge exceeds skill. Consider easing.",
        "boredom" => "Player frequently bored — challenge too low. Increase difficulty.",
        "arousal" => "Player in arousal zone — stimulating but tiring. Monitor for frustration.",
        _ => "Player relaxed — consider adding engagement hooks.",
    };

    FlowReport {
        time_in_flow: f64::from(flow_count) / total * duration,
        time_in_boredom: f64::from(boredom_count) / total * duration,
        time_in_anxiety: f64::from(anxiety_count) / total * duration,
        timeline: timeline
            .iter()
            .map(|(t, s)| FlowSample {
                timestamp_ms: *t,
                state: s.as_str().into(),
            })
            .collect(),
        dominant_state: dominant.into(),
        interpretation: interpretation.into(),
    }
}

fn build_difficulty_report(acc: &SessionAccumulator) -> DifficultyReport {
    let window = acc.performance_window();
    let skill = window.estimated_skill();
    let trend = window.trend();
    let adj = suggest_adjustment(window, tolerances::DDA_TARGET_SUCCESS_RATE);

    let recommendation = if adj > 0.3 {
        "Player performing well above target — increase difficulty significantly."
    } else if adj > 0.1 {
        "Player slightly above target — mild difficulty increase suggested."
    } else if adj < -0.3 {
        "Player struggling — reduce difficulty significantly."
    } else if adj < -0.1 {
        "Player slightly below target — mild difficulty reduction suggested."
    } else {
        "Difficulty well-matched to player skill. No adjustment needed."
    };

    DifficultyReport {
        estimated_skill: skill,
        trend,
        suggested_adjustment: adj,
        recommendation: recommendation.into(),
    }
}

fn build_fun_report(acc: &SessionAccumulator) -> FunReport {
    let signals = acc.to_fun_signals();
    let c: FunClassification = classify_fun(&signals);

    let interpretation = match c.dominant.as_str() {
        "hard_fun" => "Dominated by challenge and mastery — fiero-seeking behavior.",
        "easy_fun" => "Dominated by exploration and curiosity — discovery-driven play.",
        "people_fun" => "Dominated by social interaction — cooperative/competitive play.",
        "serious_fun" => "Dominated by completion and collection — steady-progress play.",
        _ => "Mixed fun profile.",
    };

    FunReport {
        dominant: c.dominant.to_string(),
        hard_fun: c.scores.hard,
        easy_fun: c.scores.easy,
        people_fun: c.scores.people,
        serious_fun: c.scores.serious,
        interpretation: interpretation.into(),
    }
}

fn build_tufte_report(acc: &SessionAccumulator) -> Option<TufteReport> {
    let elements = acc.to_ui_elements();
    if elements.is_empty() {
        return None;
    }
    let report: GameUiTufteReport = analyze_game_ui(&elements);
    Some(TufteReport {
        data_ink_ratio: report.data_ink_ratio,
        info_density: report.info_density,
        screen_coverage: report.screen_coverage,
        notes: report.notes,
        elements_analyzed: elements.len(),
    })
}

#[expect(clippy::cast_precision_loss, reason = "interaction counts are small")]
fn build_interaction_cost_report(acc: &SessionAccumulator) -> Option<InteractionCostReport> {
    if acc.ui_interactions.is_empty() {
        return None;
    }

    let mut total_fitts = 0.0;
    let mut total_hick = 0.0;
    let mut costliest_name = String::new();
    let mut costliest_time = 0.0_f64;

    for ui in &acc.ui_interactions {
        let fitts = input_laws::fitts_movement_time(
            ui.distance_px,
            ui.target_width_px,
            tolerances::FITTS_A_MOUSE_MS,
            tolerances::FITTS_B_MOUSE_MS,
        );
        let hick = input_laws::hick_reaction_time(
            ui.n_options,
            tolerances::HICK_A_MS,
            tolerances::HICK_B_MS,
        );
        let cost = fitts + hick;
        total_fitts += fitts;
        total_hick += hick;

        if cost > costliest_time {
            costliest_time = cost;
            costliest_name.clone_from(&ui.element);
        }
    }

    let n = acc.ui_interactions.len() as f64;
    Some(InteractionCostReport {
        avg_fitts_time_ms: total_fitts / n,
        avg_hick_time_ms: total_hick / n,
        interactions_analyzed: acc.ui_interactions.len(),
        costliest_element: costliest_name,
        costliest_time_ms: costliest_time,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::telemetry::events::{EventType, TelemetryEvent};

    fn make_event(ts: u64, event_type: EventType, payload: serde_json::Value) -> TelemetryEvent {
        TelemetryEvent {
            timestamp_ms: ts,
            session_id: "test".into(),
            event_type,
            payload,
        }
    }

    #[test]
    fn full_report_from_synthetic_session() {
        let mut acc = SessionAccumulator::new();

        acc.ingest(&make_event(
            0,
            EventType::SessionStart,
            serde_json::json!({"game_name": "test_game", "genre": "roguelike"}),
        ));

        for i in 1..=50 {
            acc.ingest(&make_event(
                i * 2000,
                EventType::PlayerAction,
                serde_json::json!({"action": "move"}),
            ));
            if i % 10 == 0 {
                acc.ingest(&make_event(
                    i * 2000 + 500,
                    EventType::ExplorationDiscover,
                    serde_json::json!({"discovery_id": format!("room_{i}")}),
                ));
            }
            if i % 15 == 0 {
                acc.ingest(&make_event(
                    i * 2000 + 800,
                    EventType::ChallengeEncounter,
                    serde_json::json!({"difficulty": 0.5, "challenge_type": "combat"}),
                ));
                acc.ingest(&make_event(
                    i * 2000 + 1000,
                    EventType::ChallengeComplete,
                    serde_json::json!({}),
                ));
            }
        }

        acc.ingest(&make_event(
            120_000,
            EventType::SessionEnd,
            serde_json::json!({"duration_s": 120.0}),
        ));

        let report = generate_report(&acc);

        assert_eq!(report.session.game_name, "test_game");
        assert!(report.engagement.composite > 0.0);
        assert!(!report.fun.dominant.is_empty());
        assert!(report.ui_tufte.is_none());
        assert!(report.interaction_costs.is_none());

        let json = serde_json::to_string_pretty(&report).expect("serialize report");
        assert!(json.contains("engagement"));
        assert!(json.contains("flow"));
        assert!(json.contains("difficulty"));
    }
}

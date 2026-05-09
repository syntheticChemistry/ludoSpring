// SPDX-License-Identifier: AGPL-3.0-or-later
//! Scenario: Engagement Metrics — composite scoring, classification.
//! Absorbed from exp015_fun_keys + exp016_tufte_metrics.

use super::registry::{Scenario, ScenarioMeta, Tier, Track};
use crate::metrics::engagement::{EngagementSnapshot, compute_engagement};
use crate::validation::{BaselineProvenance, ValidationHarness};

pub const SCENARIO: Scenario = Scenario {
    meta: ScenarioMeta {
        id: "engagement_metrics",
        track: Track::EngagementMetrics,
        tier: Tier::Rust,
        provenance_crate: "exp015_fun_keys",
        provenance_date: "2026-04-11",
        description: "Validate engagement composite scoring",
    },
    run: run_engagement,
};

fn run_engagement(h: &mut ValidationHarness) {
    let prov = BaselineProvenance {
        script: "baselines/python/fun_keys_model.py",
        commit: "231928a",
        date: "2026-04-17",
        command: "python3 baselines/python/fun_keys_model.py",
    };
    h.print_provenance(&[&prov]);

    let snap = EngagementSnapshot {
        session_duration_s: 600.0,
        action_count: 300,
        exploration_breadth: 12,
        challenge_seeking: 8,
        retry_count: 5,
        deliberate_pauses: 3,
    };

    let metrics = compute_engagement(&snap);

    h.check_bool(
        "Engagement composite in [0,1]",
        (0.0..=1.0).contains(&metrics.composite),
    );
    h.check_bool(
        "APM positive for active session",
        metrics.actions_per_minute > 0.0,
    );
    h.check_bool("Exploration rate positive", metrics.exploration_rate > 0.0);
    h.check_bool(
        "Challenge appetite in [0,1]",
        (0.0..=1.0).contains(&metrics.challenge_appetite),
    );
}

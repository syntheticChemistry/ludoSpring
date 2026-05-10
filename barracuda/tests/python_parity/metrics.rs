// SPDX-License-Identifier: AGPL-3.0-or-later

use ludospring_barracuda::interaction::flow::{FlowState, evaluate_flow};
use ludospring_barracuda::metrics::engagement::{EngagementSnapshot, compute_engagement};
use ludospring_barracuda::metrics::fun_keys::{FunKey, FunSignals, classify_fun};
use ludospring_barracuda::tolerances;
use std::path::Path;

// ── Four Keys to Fun ──────────────────────────────────────────────
// JSON: fun_keys_model.py

#[test]
fn parity_fun_dark_souls() {
    // fun_keys_model.py.dark_souls_boss.dominant = "hard"
    let c = classify_fun(&FunSignals {
        challenge: 0.95,
        exploration: 0.2,
        social: 0.05,
        completion: 0.3,
        retry_rate: 0.9,
    });
    assert_eq!(c.dominant, FunKey::Hard, "Dark Souls = Hard Fun");
}

#[test]
fn parity_fun_minecraft_creative() {
    // fun_keys_model.py.minecraft_creative.dominant = "easy"
    let c = classify_fun(&FunSignals {
        challenge: 0.1,
        exploration: 0.9,
        social: 0.1,
        completion: 0.3,
        retry_rate: 0.0,
    });
    assert_eq!(c.dominant, FunKey::Easy, "Minecraft Creative = Easy Fun");
}

#[test]
fn parity_fun_among_us() {
    // fun_keys_model.py.among_us.dominant = "people"
    let c = classify_fun(&FunSignals {
        challenge: 0.3,
        exploration: 0.1,
        social: 0.95,
        completion: 0.1,
        retry_rate: 0.1,
    });
    assert_eq!(c.dominant, FunKey::People, "Among Us = People Fun");
}

#[test]
fn parity_fun_animal_crossing() {
    // fun_keys_model.py.animal_crossing.dominant = "serious"
    let c = classify_fun(&FunSignals {
        challenge: 0.05,
        exploration: 0.3,
        social: 0.1,
        completion: 0.9,
        retry_rate: 0.0,
    });
    assert_eq!(c.dominant, FunKey::Serious, "Animal Crossing = Serious Fun");
}

// ── Four Keys: Numeric Scores ────────────────────────────────────
// JSON: fun_keys_model.py — extended: exact score parity per scenario

#[test]
fn parity_fun_dark_souls_scores() {
    // fun_keys_model.py.dark_souls_boss.scores
    let c = classify_fun(&FunSignals {
        challenge: 0.95,
        exploration: 0.2,
        social: 0.05,
        completion: 0.3,
        retry_rate: 0.9,
    });
    assert!((c.scores.hard - 0.93).abs() < tolerances::ANALYTICAL_TOL);
    assert!((c.scores.easy - 0.17).abs() < tolerances::ANALYTICAL_TOL);
    assert!((c.scores.people - 0.05).abs() < tolerances::ANALYTICAL_TOL);
    assert!((c.scores.serious - 0.36).abs() < tolerances::ANALYTICAL_TOL);
}

#[test]
fn parity_fun_minecraft_scores() {
    // fun_keys_model.py.minecraft_creative.scores
    let c = classify_fun(&FunSignals {
        challenge: 0.1,
        exploration: 0.9,
        social: 0.1,
        completion: 0.3,
        retry_rate: 0.0,
    });
    assert!((c.scores.hard - 0.06).abs() < tolerances::ANALYTICAL_TOL);
    assert!((c.scores.easy - 0.9).abs() < tolerances::ANALYTICAL_TOL);
    assert!((c.scores.people - 0.1).abs() < tolerances::ANALYTICAL_TOL);
    assert!((c.scores.serious - 0.48).abs() < tolerances::ANALYTICAL_TOL);
}

#[test]
fn parity_fun_among_us_scores() {
    // fun_keys_model.py.among_us.scores
    let c = classify_fun(&FunSignals {
        challenge: 0.3,
        exploration: 0.1,
        social: 0.95,
        completion: 0.1,
        retry_rate: 0.1,
    });
    assert!((c.scores.hard - 0.22).abs() < tolerances::ANALYTICAL_TOL);
    assert!((c.scores.easy - 0.22).abs() < tolerances::ANALYTICAL_TOL);
    assert!((c.scores.people - 0.95).abs() < tolerances::ANALYTICAL_TOL);
    assert!((c.scores.serious - 0.1825).abs() < tolerances::ANALYTICAL_TOL);
}

#[test]
fn parity_fun_animal_crossing_scores() {
    // fun_keys_model.py.animal_crossing.scores
    let c = classify_fun(&FunSignals {
        challenge: 0.05,
        exploration: 0.3,
        social: 0.1,
        completion: 0.9,
        retry_rate: 0.0,
    });
    assert!((c.scores.hard - 0.03).abs() < tolerances::ANALYTICAL_TOL);
    assert!((c.scores.easy - 0.43).abs() < tolerances::ANALYTICAL_TOL);
    assert!((c.scores.people - 0.1).abs() < tolerances::ANALYTICAL_TOL);
    assert!((c.scores.serious - 0.9075).abs() < tolerances::ANALYTICAL_TOL);
}

#[test]
fn parity_fun_celeste() {
    // fun_keys_model.py.celeste.dominant = "hard", scores
    let c = classify_fun(&FunSignals {
        challenge: 0.9,
        exploration: 0.3,
        social: 0.0,
        completion: 0.4,
        retry_rate: 0.85,
    });
    assert_eq!(c.dominant, FunKey::Hard, "Celeste = Hard Fun");
    assert!((c.scores.hard - 0.88).abs() < tolerances::ANALYTICAL_TOL);
    assert!((c.scores.easy - 0.26).abs() < tolerances::ANALYTICAL_TOL);
    assert!((c.scores.people - 0.0).abs() < tolerances::ANALYTICAL_TOL);
    assert!((c.scores.serious - 0.445).abs() < tolerances::ANALYTICAL_TOL);
}

#[test]
fn parity_fun_no_mans_sky() {
    // fun_keys_model.py.no_mans_sky.dominant = "easy", scores
    let c = classify_fun(&FunSignals {
        challenge: 0.15,
        exploration: 0.85,
        social: 0.15,
        completion: 0.2,
        retry_rate: 0.05,
    });
    assert_eq!(c.dominant, FunKey::Easy, "No Man's Sky = Easy Fun");
    assert!((c.scores.hard - 0.11).abs() < tolerances::ANALYTICAL_TOL);
    assert!((c.scores.easy - 0.85).abs() < tolerances::ANALYTICAL_TOL);
    assert!((c.scores.people - 0.15).abs() < tolerances::ANALYTICAL_TOL);
    assert!((c.scores.serious - 0.395).abs() < tolerances::ANALYTICAL_TOL);
}

// ── Fun Keys: Edge Cases ─────────────────────────────────────────
// JSON: fun_keys_model.py — zero_scores, max_scores

#[test]
fn parity_fun_keys_zero_scores() {
    // fun_keys_model.py.zero_scores: all signals zero
    // Expected: hard=0.0, easy=0.2, people=0, serious=0.3
    let c = classify_fun(&FunSignals {
        challenge: 0.0,
        exploration: 0.0,
        social: 0.0,
        completion: 0.0,
        retry_rate: 0.0,
    });
    assert!(
        c.scores.hard.abs() < tolerances::ANALYTICAL_TOL,
        "zero hard: Rust={}, Python=0.0",
        c.scores.hard
    );
    assert!(
        (c.scores.easy - 0.2).abs() < tolerances::ANALYTICAL_TOL,
        "zero easy: Rust={}, Python=0.2",
        c.scores.easy
    );
    assert!(
        c.scores.people.abs() < tolerances::ANALYTICAL_TOL,
        "zero people: Rust={}, Python=0.0",
        c.scores.people
    );
    assert!(
        (c.scores.serious - 0.3).abs() < tolerances::ANALYTICAL_TOL,
        "zero serious: Rust={}, Python=0.3",
        c.scores.serious
    );
}

#[test]
fn parity_fun_keys_max_scores() {
    // fun_keys_model.py.max_scores: all signals at 1.0
    // Expected: hard=1.0, easy=0.8, people=1, serious=0.7
    let c = classify_fun(&FunSignals {
        challenge: 1.0,
        exploration: 1.0,
        social: 1.0,
        completion: 1.0,
        retry_rate: 1.0,
    });
    assert!(
        (c.scores.hard - 1.0).abs() < tolerances::ANALYTICAL_TOL,
        "max hard: Rust={}, Python=1.0",
        c.scores.hard
    );
    assert!(
        (c.scores.easy - 0.8).abs() < tolerances::ANALYTICAL_TOL,
        "max easy: Rust={}, Python=0.8",
        c.scores.easy
    );
    assert!(
        (c.scores.people - 1.0).abs() < tolerances::ANALYTICAL_TOL,
        "max people: Rust={}, Python=1.0",
        c.scores.people
    );
    assert!(
        (c.scores.serious - 0.7).abs() < tolerances::ANALYTICAL_TOL,
        "max serious: Rust={}, Python=0.7",
        c.scores.serious
    );
}

// ── Flow / Engagement / DDA ──────────────────────────────────────
// JSON: flow_engagement.py

#[test]
fn parity_flow_states() {
    // flow_engagement.py.flow_states — matching Python's evaluate_flow exactly
    let w = tolerances::FLOW_CHANNEL_WIDTH;
    assert_eq!(
        evaluate_flow(0.5, 0.5, w),
        FlowState::Flow,
        "exact_diagonal"
    );
    assert_eq!(
        evaluate_flow(0.5, w.mul_add(-0.9, 0.5), w),
        FlowState::Flow,
        "inside_channel_low"
    );
    assert_eq!(
        evaluate_flow(0.5, w.mul_add(0.9, 0.5), w),
        FlowState::Flow,
        "inside_channel_high"
    );
    assert_eq!(
        evaluate_flow(0.9, 0.1, w),
        FlowState::Anxiety,
        "high_challenge_low_skill"
    );
    assert_eq!(
        evaluate_flow(0.1, 0.9, w),
        FlowState::Boredom,
        "low_challenge_high_skill"
    );
}

#[test]
fn parity_engagement_active() {
    // flow_engagement.py.engagement_active — 300s session, 200 actions, 15 explore, 10 challenge, 20 retry, 15 pauses
    let snap = EngagementSnapshot {
        session_duration_s: 300.0,
        action_count: 200,
        exploration_breadth: 15,
        challenge_seeking: 10,
        retry_count: 20,
        deliberate_pauses: 15,
    };
    let m = compute_engagement(&snap);
    let python_composite = 0.298_333_333_333_333_34;
    assert!(
        (m.composite - python_composite).abs() < tolerances::ANALYTICAL_TOL,
        "active composite: Rust={}, Python={python_composite}",
        m.composite
    );
    assert!((m.actions_per_minute - 40.0).abs() < tolerances::ANALYTICAL_TOL);
    assert!((m.exploration_rate - 3.0).abs() < tolerances::ANALYTICAL_TOL);
    assert!((m.challenge_appetite - 0.05).abs() < tolerances::ANALYTICAL_TOL);
    assert!((m.persistence - 0.1).abs() < tolerances::ANALYTICAL_TOL);
    assert!((m.deliberation - 0.075).abs() < tolerances::ANALYTICAL_TOL);
}

#[test]
fn parity_engagement_idle() {
    // flow_engagement.py.engagement_idle — 300s session, 2 actions
    let snap = EngagementSnapshot {
        session_duration_s: 300.0,
        action_count: 2,
        exploration_breadth: 1,
        challenge_seeking: 0,
        retry_count: 0,
        deliberate_pauses: 0,
    };
    let m = compute_engagement(&snap);
    let python_composite = 0.009_333_333_333_333_334;
    assert!(
        (m.composite - python_composite).abs() < tolerances::ANALYTICAL_TOL,
        "idle composite: Rust={}, Python={python_composite}",
        m.composite
    );
}

#[test]
fn parity_engagement_zero() {
    // flow_engagement.py.engagement_zero — 0s session, 0 actions → 0.0
    let snap = EngagementSnapshot::default();
    let m = compute_engagement(&snap);
    assert!(
        m.composite.abs() < tolerances::ANALYTICAL_TOL,
        "zero composite: Rust={}",
        m.composite
    );
}

// ── DDA session (flow_engagement.py) ───────────────────────────────
// JSON: flow_engagement.py.dda_session — sigmoid difficulty ramp + skill growth

#[test]
fn parity_dda_session_rounds() {
    // flow_engagement.py: DDA session simulation (matches exp004)
    // Skip if combined_baselines.json hasn't been generated yet
    let manifest = env!("CARGO_MANIFEST_DIR");
    let path = Path::new(manifest).join("../baselines/python/combined_baselines.json");
    let Ok(content) = std::fs::read_to_string(&path) else {
        eprintln!(
            "  [SKIP] {}: not found (run: python3 baselines/python/run_all_baselines.py)",
            path.display()
        );
        return;
    };
    let root: serde_json::Value =
        serde_json::from_str(&content).expect("parse combined_baselines.json");
    let arr = root
        .get("flow_engagement.py")
        .and_then(|v| v.get("dda_session"))
        .and_then(serde_json::Value::as_array)
        .expect("flow_engagement.py.dda_session array");

    let w = tolerances::FLOW_CHANNEL_WIDTH;
    let mut skill = 0.3_f64;
    #[expect(clippy::cast_precision_loss, reason = "round indices ≤ 19 fit in f64")]
    for (round_num, entry) in arr.iter().enumerate().take(20) {
        let progress = round_num as f64 / 19.0;
        let x = progress.mul_add(8.0, -4.0);
        let sigmoid = 1.0 / (1.0 + (-x).exp());
        let difficulty = 0.7_f64.mul_add(sigmoid, 0.2);
        let state = evaluate_flow(difficulty, skill, w);
        let exp_diff = entry["difficulty"]
            .as_f64()
            .expect("dda_session[].difficulty");
        let exp_skill = entry["skill"].as_f64().expect("dda_session[].skill");
        let exp_flow = entry["flow"].as_str().expect("dda_session[].flow");

        assert!(
            (difficulty - exp_diff).abs() < tolerances::ANALYTICAL_TOL,
            "round {} difficulty: Rust={difficulty}, JSON={exp_diff}",
            round_num + 1,
        );
        assert!(
            (skill - exp_skill).abs() < tolerances::ANALYTICAL_TOL,
            "round {} skill: Rust={skill}, JSON={exp_skill}",
            round_num + 1,
        );
        assert_eq!(
            state.as_str(),
            exp_flow.to_lowercase(),
            "round {} flow state",
            round_num + 1
        );

        skill = (skill + 0.02).min(0.95);
    }
}

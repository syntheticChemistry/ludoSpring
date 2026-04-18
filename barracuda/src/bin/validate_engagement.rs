// SPDX-License-Identifier: AGPL-3.0-or-later
//! Validates engagement metrics and Four Keys classification against Python baselines.
#![forbid(unsafe_code)]

use ludospring_barracuda::metrics::engagement::{EngagementSnapshot, compute_engagement};
use ludospring_barracuda::metrics::fun_keys::{FunKey, FunSignals, classify_fun};
use ludospring_barracuda::tolerances::ANALYTICAL_TOL;
use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};

fn main() {
    let provenance = BaselineProvenance {
        script: "baselines/python/run_all_baselines.py",
        commit: "19e402c0",
        date: "2026-04-10",
        command: "python3 baselines/python/run_all_baselines.py",
    };

    let mut h = ValidationHarness::new("Engagement & Four Keys (Python parity)");
    h.print_provenance(&[&provenance]);

    let active = EngagementSnapshot {
        session_duration_s: 300.0,
        action_count: 200,
        exploration_breadth: 15,
        challenge_seeking: 10,
        retry_count: 20,
        deliberate_pauses: 15,
    };
    let m_active = compute_engagement(&active);
    h.check_abs(
        "Engagement active composite",
        m_active.composite,
        0.298_333_333_333_333_34,
        ANALYTICAL_TOL,
    );
    h.check_abs(
        "Engagement active APM",
        m_active.actions_per_minute,
        40.0,
        ANALYTICAL_TOL,
    );

    let idle = EngagementSnapshot {
        session_duration_s: 300.0,
        action_count: 2,
        exploration_breadth: 1,
        challenge_seeking: 0,
        retry_count: 0,
        deliberate_pauses: 0,
    };
    let m_idle = compute_engagement(&idle);
    h.check_abs(
        "Engagement idle composite",
        m_idle.composite,
        0.009_333_333_333_333_334,
        ANALYTICAL_TOL,
    );

    h.check_bool(
        "Four Keys: Dark Souls → Hard",
        classify_fun(&FunSignals {
            challenge: 0.95,
            exploration: 0.2,
            social: 0.05,
            completion: 0.3,
            retry_rate: 0.9,
        })
        .dominant
            == FunKey::Hard,
    );
    h.check_bool(
        "Four Keys: Minecraft Creative → Easy",
        classify_fun(&FunSignals {
            challenge: 0.1,
            exploration: 0.9,
            social: 0.1,
            completion: 0.3,
            retry_rate: 0.0,
        })
        .dominant
            == FunKey::Easy,
    );
    h.check_bool(
        "Four Keys: Among Us → People",
        classify_fun(&FunSignals {
            challenge: 0.3,
            exploration: 0.1,
            social: 0.95,
            completion: 0.1,
            retry_rate: 0.1,
        })
        .dominant
            == FunKey::People,
    );
    h.check_bool(
        "Four Keys: Animal Crossing → Serious",
        classify_fun(&FunSignals {
            challenge: 0.05,
            exploration: 0.3,
            social: 0.1,
            completion: 0.9,
            retry_rate: 0.0,
        })
        .dominant
            == FunKey::Serious,
    );

    h.finish();
}

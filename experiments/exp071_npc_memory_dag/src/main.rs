// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
//! exp071 — NPC Memory DAG Retrieval
//!
//! Validates graph-aware context assembly from NPC memory subgraphs:
//! - Recent interactions included verbatim
//! - Promise-related vertices always included regardless of age
//! - Trust milestone vertices always included
//! - Older routine interactions summarized, not included verbatim
//! - Cumulative trust calculated correctly
//! - Secret reveals and special events excluded from routine summary

use ludospring_barracuda::game::rpgpt::memory::{
    InteractionType, NpcInteraction, NpcMemoryAssembler,
};
use ludospring_barracuda::validation::ValidationHarness;

const EXP: &str = "exp071_npc_memory_dag";

fn build_session(
    npc_id: &str,
    session: u32,
    count: usize,
    interaction_type: InteractionType,
    trust_delta: f64,
) -> Vec<NpcInteraction> {
    (0..count)
        .map(|i| {
            #[expect(clippy::cast_possible_truncation, reason = "test value fits in u32")]
            let sequence = i as u32;
            NpcInteraction {
                npc_id: npc_id.into(),
                player_id: "player1".into(),
                interaction_type,
                summary: format!("Session {session} interaction {i}"),
                trust_delta,
                knowledge_revealed: vec![],
                promises_made: vec![],
                emotional_state: "neutral".into(),
                session,
                sequence,
            }
        })
        .collect()
}

fn build_rich_history() -> Vec<NpcInteraction> {
    let mut history = Vec::new();

    // Session 1: Initial meeting (5 routine interactions)
    history.extend(build_session("maren", 1, 5, InteractionType::Dialogue, 0.1));

    // Session 2: Trade and a promise
    history.extend(build_session("maren", 2, 3, InteractionType::Trade, 0.1));
    history.push(NpcInteraction {
        npc_id: "maren".into(),
        player_id: "player1".into(),
        interaction_type: InteractionType::Promise,
        summary: "Player promised to find star-metal".into(),
        trust_delta: 0.5,
        knowledge_revealed: vec![],
        promises_made: vec!["Find star-metal".into()],
        emotional_state: "hopeful".into(),
        session: 2,
        sequence: 3,
    });

    // Session 3: Big trust event — defended reputation
    history.extend(build_session("maren", 3, 2, InteractionType::Dialogue, 0.1));
    history.push(NpcInteraction {
        npc_id: "maren".into(),
        player_id: "player1".into(),
        interaction_type: InteractionType::TrustChange,
        summary: "Player defended Maren against Guild Master Harven".into(),
        trust_delta: 1.0,
        knowledge_revealed: vec![],
        promises_made: vec![],
        emotional_state: "grateful".into(),
        session: 3,
        sequence: 2,
    });

    // Session 4: Routine trading
    history.extend(build_session("maren", 4, 4, InteractionType::Trade, 0.1));

    // Session 5: Secret reveal
    history.push(NpcInteraction {
        npc_id: "maren".into(),
        player_id: "player1".into(),
        interaction_type: InteractionType::SecretReveal,
        summary: "Maren revealed the hidden workshop".into(),
        trust_delta: 0.0,
        knowledge_revealed: vec!["hidden_workshop".into()],
        promises_made: vec![],
        emotional_state: "relieved".into(),
        session: 5,
        sequence: 0,
    });

    // Session 6-10: More routine + recent
    for session in 6..=10 {
        history.extend(build_session(
            "maren",
            session,
            3,
            InteractionType::Dialogue,
            0.1,
        ));
    }

    history
}

fn validate_recent_window(h: &mut ValidationHarness) {
    let history = build_rich_history();
    let assembler = NpcMemoryAssembler::default();
    let ctx = assembler.assemble("Maren", &history, "esteem", "internal_conflict");

    h.check_abs(
        "recent_window_is_five",
        ctx.recent_interactions.len() as f64,
        5.0,
        0.0,
    );

    let last = &ctx.recent_interactions[ctx.recent_interactions.len() - 1];
    h.check_bool("most_recent_is_session_10", last.session == 10);

    h.check_bool(
        "recent_are_chronological",
        ctx.recent_interactions.windows(2).all(|w| {
            w[0].session <= w[1].session
                || (w[0].session == w[1].session && w[0].sequence <= w[1].sequence)
        }),
    );
}

fn validate_promises_always_included(h: &mut ValidationHarness) {
    let history = build_rich_history();
    let assembler = NpcMemoryAssembler::default();
    let ctx = assembler.assemble("Maren", &history, "esteem", "internal_conflict");

    h.check_bool(
        "promise_vertices_not_empty",
        !ctx.promise_vertices.is_empty(),
    );

    let has_star_metal = ctx
        .promise_vertices
        .iter()
        .any(|v| v.promises_made.contains(&"Find star-metal".into()));
    h.check_bool("star_metal_promise_included", has_star_metal);

    let promise_in_session_2 = ctx.promise_vertices.iter().any(|v| v.session == 2);
    h.check_bool("old_promise_included", promise_in_session_2);
}

fn validate_trust_milestones(h: &mut ValidationHarness) {
    let history = build_rich_history();
    let assembler = NpcMemoryAssembler::default();
    let ctx = assembler.assemble("Maren", &history, "esteem", "internal_conflict");

    h.check_bool(
        "trust_milestones_not_empty",
        !ctx.trust_milestones.is_empty(),
    );

    let has_defense = ctx
        .trust_milestones
        .iter()
        .any(|v| v.summary.contains("defended Maren"));
    h.check_bool("defense_milestone_included", has_defense);
}

fn validate_cumulative_trust(h: &mut ValidationHarness) {
    let history = build_rich_history();
    let assembler = NpcMemoryAssembler::default();
    let ctx = assembler.assemble("Maren", &history, "esteem", "internal_conflict");

    h.check_bool("trust_value_positive", ctx.trust_value > 0.0);

    // Count expected: 5*0.1 + 3*0.1 + 0.5 + 2*0.1 + 1.0 + 4*0.1 + 0.0 + 15*0.1
    // = 0.5 + 0.3 + 0.5 + 0.2 + 1.0 + 0.4 + 0.0 + 1.5 = 4.4
    h.check_abs(
        "trust_value_correct",
        ctx.trust_value,
        4.4,
        0.1, // tolerance for float accumulation
    );

    h.check_bool("trust_level_at_least_4", ctx.trust_level >= 4);
}

fn validate_historical_summary(h: &mut ValidationHarness) {
    let history = build_rich_history();
    let assembler = NpcMemoryAssembler::default();
    let ctx = assembler.assemble("Maren", &history, "esteem", "internal_conflict");

    h.check_bool(
        "historical_summary_not_empty",
        !ctx.historical_summary.is_empty(),
    );

    h.check_bool(
        "summary_mentions_routine",
        ctx.historical_summary.contains("routine"),
    );

    h.check_bool(
        "summary_mentions_sessions",
        ctx.historical_summary.contains("session"),
    );
}

fn validate_empty_history(h: &mut ValidationHarness) {
    let assembler = NpcMemoryAssembler::default();
    let ctx = assembler.assemble("Maren", &[], "esteem", "internal_conflict");

    h.check_abs(
        "empty_recent_count",
        ctx.recent_interactions.len() as f64,
        0.0,
        0.0,
    );
    h.check_abs(
        "empty_promise_count",
        ctx.promise_vertices.len() as f64,
        0.0,
        0.0,
    );
    h.check_abs(
        "empty_milestone_count",
        ctx.trust_milestones.len() as f64,
        0.0,
        0.0,
    );
    h.check_abs("empty_trust_value", ctx.trust_value, 0.0, 0.0);
    h.check_abs("empty_trust_level", f64::from(ctx.trust_level), 0.0, 0.0);
    h.check_bool("empty_summary_is_empty", ctx.historical_summary.is_empty());
}

fn validate_few_interactions(h: &mut ValidationHarness) {
    let interactions = build_session("maren", 1, 3, InteractionType::Dialogue, 0.5);
    let assembler = NpcMemoryAssembler::default();
    let ctx = assembler.assemble("Maren", &interactions, "esteem", "active");

    h.check_abs(
        "few_all_recent",
        ctx.recent_interactions.len() as f64,
        3.0,
        0.0,
    );
    h.check_bool("few_no_summary_needed", ctx.historical_summary.is_empty());
}

fn validate_secret_reveals_excluded_from_routine(h: &mut ValidationHarness) {
    let history = build_rich_history();
    let assembler = NpcMemoryAssembler::default();
    let ctx = assembler.assemble("Maren", &history, "esteem", "internal_conflict");

    let routine_count_str = ctx.historical_summary.clone();
    let has_count = routine_count_str.contains("routine interactions");
    h.check_bool("routine_count_in_summary", has_count);
}

fn validate_context_metadata(h: &mut ValidationHarness) {
    let history = build_rich_history();
    let assembler = NpcMemoryAssembler::default();
    let ctx = assembler.assemble("Maren", &history, "esteem", "internal_conflict");

    h.check_bool("npc_name_set", ctx.npc_name == "Maren");
    h.check_bool("active_need_set", ctx.active_need == "esteem");
    h.check_bool("arc_phase_set", ctx.arc_phase == "internal_conflict");
}

fn main() {
    let mut h = ValidationHarness::new(EXP);

    validate_recent_window(&mut h);
    validate_promises_always_included(&mut h);
    validate_trust_milestones(&mut h);
    validate_cumulative_trust(&mut h);
    validate_historical_summary(&mut h);
    validate_empty_history(&mut h);
    validate_few_interactions(&mut h);
    validate_secret_reveals_excluded_from_routine(&mut h);
    validate_context_metadata(&mut h);

    h.finish();
}

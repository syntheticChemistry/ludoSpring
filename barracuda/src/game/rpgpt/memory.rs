// SPDX-License-Identifier: AGPL-3.0-or-later
//! NPC memory as DAG — structural memory, not context window.
//!
//! Every player interaction with an NPC creates a vertex. When voicing an NPC,
//! the context assembly pipeline walks the NPC's memory subgraph, extracts
//! relevant vertices, summarizes older ones, and presents recent ones in full.

/// Type of NPC interaction vertex.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InteractionType {
    /// General conversation.
    Dialogue,
    /// Buying or selling goods.
    Trade,
    /// Advancing a quest or task.
    QuestProgress,
    /// An event that changes trust.
    TrustChange,
    /// A promise made or broken.
    Promise,
    /// A secret revealed.
    SecretReveal,
    /// A combat interaction.
    Combat,
}

/// A single NPC interaction vertex (stored in rhizoCrypt).
#[derive(Debug, Clone)]
pub struct NpcInteraction {
    /// NPC this interaction is with.
    pub npc_id: String,
    /// Player identifier.
    pub player_id: String,
    /// Type of interaction.
    pub interaction_type: InteractionType,
    /// Summary of what happened.
    pub summary: String,
    /// Trust change from this interaction.
    pub trust_delta: f64,
    /// Knowledge revealed during this interaction.
    pub knowledge_revealed: Vec<String>,
    /// Promises made during this interaction.
    pub promises_made: Vec<String>,
    /// NPC's emotional state after this interaction.
    pub emotional_state: String,
    /// Session number (for ordering).
    pub session: u32,
    /// Sequence number within session.
    pub sequence: u32,
}

/// Assembled context for AI narration of an NPC.
///
/// Built from walking the NPC's memory subgraph — not by stuffing
/// the context window with everything.
#[derive(Debug, Clone)]
pub struct NpcContext {
    /// NPC's name (from personality cert).
    pub npc_name: String,
    /// Current trust level (computed from cumulative deltas).
    pub trust_level: u8,
    /// Current trust value (raw).
    pub trust_value: f64,
    /// Recent interactions (included verbatim).
    pub recent_interactions: Vec<NpcInteraction>,
    /// Promise-related vertices (always included regardless of age).
    pub promise_vertices: Vec<NpcInteraction>,
    /// Trust milestone vertices (always included).
    pub trust_milestones: Vec<NpcInteraction>,
    /// Summarized older interactions.
    pub historical_summary: String,
    /// Active motivation and arc phase.
    pub active_need: String,
    /// Current arc phase ID.
    pub arc_phase: String,
}

/// Assembler that builds NPC context from a memory subgraph.
#[derive(Debug)]
pub struct NpcMemoryAssembler {
    /// How many recent interactions to include verbatim.
    pub recent_window: usize,
    /// Maximum token budget for the assembled context.
    pub token_budget: usize,
    /// Minimum trust delta magnitude to count as a milestone.
    pub milestone_threshold: f64,
}

impl Default for NpcMemoryAssembler {
    fn default() -> Self {
        Self {
            recent_window: 5,
            token_budget: 2048,
            milestone_threshold: 0.5,
        }
    }
}

impl NpcMemoryAssembler {
    /// Assemble NPC context from a list of interactions.
    ///
    /// The interactions should be pre-sorted by (session, sequence).
    #[must_use]
    pub fn assemble(
        &self,
        npc_name: &str,
        interactions: &[NpcInteraction],
        active_need: &str,
        arc_phase: &str,
    ) -> NpcContext {
        let total = interactions.len();
        let recent_start = total.saturating_sub(self.recent_window);
        let recent = interactions[recent_start..].to_vec();

        let older = &interactions[..recent_start];

        let promises: Vec<NpcInteraction> = older
            .iter()
            .filter(|i| {
                i.interaction_type == InteractionType::Promise || !i.promises_made.is_empty()
            })
            .cloned()
            .collect();

        let milestones: Vec<NpcInteraction> = older
            .iter()
            .filter(|i| i.trust_delta.abs() >= self.milestone_threshold)
            .cloned()
            .collect();

        let cumulative_trust: f64 = interactions.iter().map(|i| i.trust_delta).sum();
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "trust level is a small non-negative integer"
        )]
        let trust_level = cumulative_trust.max(0.0).floor() as u8;

        let routine_old: Vec<&NpcInteraction> = older
            .iter()
            .filter(|i| {
                i.trust_delta.abs() < self.milestone_threshold
                    && i.promises_made.is_empty()
                    && i.interaction_type != InteractionType::Promise
                    && i.interaction_type != InteractionType::SecretReveal
            })
            .collect();

        let summary = if routine_old.is_empty() {
            String::new()
        } else {
            format!(
                "{} routine interactions over {} sessions.",
                routine_old.len(),
                routine_old
                    .iter()
                    .map(|i| i.session)
                    .collect::<std::collections::HashSet<_>>()
                    .len()
            )
        };

        NpcContext {
            npc_name: npc_name.into(),
            trust_level,
            trust_value: cumulative_trust,
            recent_interactions: recent,
            promise_vertices: promises,
            trust_milestones: milestones,
            historical_summary: summary,
            active_need: active_need.into(),
            arc_phase: arc_phase.into(),
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn build_interactions(count: usize) -> Vec<NpcInteraction> {
        (0..count)
            .map(|i| {
                #[expect(
                    clippy::cast_possible_truncation,
                    reason = "test value fits in u32"
                )]
                let session = (i / 5) as u32;
                #[expect(
                    clippy::cast_possible_truncation,
                    reason = "test value fits in u32"
                )]
                let sequence = (i % 5) as u32;
                NpcInteraction {
                    npc_id: "maren".into(),
                    player_id: "player1".into(),
                    interaction_type: InteractionType::Dialogue,
                    summary: format!("Interaction {i}"),
                    trust_delta: 0.1,
                    knowledge_revealed: vec![],
                    promises_made: vec![],
                    emotional_state: "neutral".into(),
                    session,
                    sequence,
                }
            })
            .collect()
    }

    #[test]
    fn recent_window_selects_last_n() {
        let interactions = build_interactions(20);
        let assembler = NpcMemoryAssembler::default();
        let ctx = assembler.assemble("Maren", &interactions, "esteem", "internal_conflict");
        assert_eq!(ctx.recent_interactions.len(), 5);
        assert_eq!(ctx.recent_interactions[0].summary, "Interaction 15");
    }

    #[test]
    fn promises_always_included() {
        let mut interactions = build_interactions(20);
        interactions[2].interaction_type = InteractionType::Promise;
        interactions[2].promises_made = vec!["Find star-metal".into()];
        let assembler = NpcMemoryAssembler::default();
        let ctx = assembler.assemble("Maren", &interactions, "esteem", "internal_conflict");
        assert_eq!(ctx.promise_vertices.len(), 1);
        assert_eq!(ctx.promise_vertices[0].summary, "Interaction 2");
    }

    #[test]
    fn trust_milestones_included() {
        let mut interactions = build_interactions(20);
        interactions[3].trust_delta = 1.0; // milestone
        let assembler = NpcMemoryAssembler::default();
        let ctx = assembler.assemble("Maren", &interactions, "esteem", "internal_conflict");
        assert_eq!(ctx.trust_milestones.len(), 1);
    }

    #[test]
    fn cumulative_trust_calculated() {
        let mut interactions = build_interactions(10);
        for i in &mut interactions {
            i.trust_delta = 0.5; // 10 × 0.5 = 5.0 (exact in f64)
        }
        let assembler = NpcMemoryAssembler::default();
        let ctx = assembler.assemble("Maren", &interactions, "esteem", "internal_conflict");
        assert!((ctx.trust_value - 5.0).abs() < f64::EPSILON);
        assert_eq!(ctx.trust_level, 5);
    }

    #[test]
    fn empty_interactions_produce_empty_context() {
        let assembler = NpcMemoryAssembler::default();
        let ctx = assembler.assemble("Maren", &[], "esteem", "internal_conflict");
        assert!(ctx.recent_interactions.is_empty());
        assert!(ctx.promise_vertices.is_empty());
        assert!(ctx.trust_milestones.is_empty());
        assert_eq!(ctx.trust_level, 0);
    }

    #[test]
    fn historical_summary_counts_routine() {
        let interactions = build_interactions(20);
        let assembler = NpcMemoryAssembler::default();
        let ctx = assembler.assemble("Maren", &interactions, "esteem", "internal_conflict");
        assert!(ctx.historical_summary.contains("15 routine interactions"));
    }

    #[test]
    fn few_interactions_all_recent() {
        let interactions = build_interactions(3);
        let assembler = NpcMemoryAssembler::default();
        let ctx = assembler.assemble("Maren", &interactions, "esteem", "internal_conflict");
        assert_eq!(ctx.recent_interactions.len(), 3);
        assert!(ctx.historical_summary.is_empty());
    }

    #[test]
    fn secret_reveals_excluded_from_routine_summary() {
        let mut interactions = build_interactions(20);
        interactions[5].interaction_type = InteractionType::SecretReveal;
        let assembler = NpcMemoryAssembler::default();
        let ctx = assembler.assemble("Maren", &interactions, "esteem", "internal_conflict");
        assert!(ctx.historical_summary.contains("14 routine"));
    }
}

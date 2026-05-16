// SPDX-License-Identifier: AGPL-3.0-or-later
//! NPC personality dynamics — bridging Bartle types into RPGPT behavior.
//!
//! Maps Bartle Player Type theory to NPC behavioral tendencies.
//! Each NPC has an implicit "player type" that models how they engage
//! with the world — an Achiever-type NPC pursues goals relentlessly,
//! while a Socializer-type builds relationships.
//!
//! This enables:
//! - Predicting NPC behavior in unscripted situations
//! - Modeling NPC-NPC interactions via interest graph dynamics
//! - Content affinity for procedural quest generation
//! - Population equilibrium modeling for world simulation

use crate::metrics::player_types::{
    BartleProfile, MechanicCategory, PlayerType, interaction_valence,
};

use super::npc::NpcPersonality;

/// Derive a Bartle profile from an NPC's Maslow hierarchy motivations.
///
/// The mapping follows psychological correspondences:
/// - Survival/Safety → Achiever (resource acquisition)
/// - Belonging → Socializer (social bonds)
/// - Esteem → Killer (status through dominance) or Achiever (status through merit)
/// - Self-actualization → Explorer (understanding, discovery)
#[must_use]
pub fn derive_bartle_profile(npc: &NpcPersonality) -> BartleProfile {
    let needs = &npc.motivations;

    let achiever_signal = needs.esteem.urgency.mul_add(
        0.2,
        needs
            .survival
            .urgency
            .mul_add(0.4, needs.safety.urgency * 0.3),
    );

    let explorer_signal = needs
        .self_actualization
        .urgency
        .mul_add(0.6, needs.safety.urgency * 0.1);

    let socializer_signal = needs
        .belonging
        .urgency
        .mul_add(0.7, needs.esteem.urgency * 0.1);

    let killer_signal = needs
        .esteem
        .urgency
        .mul_add(0.4, needs.survival.urgency * 0.2);

    BartleProfile::new(
        achiever_signal,
        explorer_signal,
        socializer_signal,
        killer_signal,
    )
}

/// Predict how two NPCs will interact based on their derived Bartle types.
///
/// Returns a valence (-1.0 to +1.0) predicting cooperation vs conflict.
#[must_use]
pub fn predict_npc_interaction(npc_a: &NpcPersonality, npc_b: &NpcPersonality) -> f64 {
    let profile_a = derive_bartle_profile(npc_a);
    let profile_b = derive_bartle_profile(npc_b);

    let type_a = profile_a.dominant();
    let type_b = profile_b.dominant();

    let base = interaction_valence(type_a, type_b);

    let existing_bond = npc_a
        .relationships
        .iter()
        .find(|r| r.entity == npc_b.id)
        .map_or(0.0, |r| r.strength * 0.3);

    (base + existing_bond).clamp(-1.0, 1.0)
}

/// Recommend quest content type for an NPC based on their Bartle profile.
///
/// Returns mechanics sorted by affinity (highest first) — useful for
/// procedural quest generation that matches NPC personality.
#[must_use]
pub fn recommended_mechanics(npc: &NpcPersonality) -> Vec<(MechanicCategory, f64)> {
    let profile = derive_bartle_profile(npc);

    let all_mechanics = [
        MechanicCategory::Progression,
        MechanicCategory::Discovery,
        MechanicCategory::Cooperation,
        MechanicCategory::Competition,
        MechanicCategory::Narrative,
        MechanicCategory::Creation,
        MechanicCategory::Collection,
        MechanicCategory::Destruction,
    ];

    let mut ranked: Vec<(MechanicCategory, f64)> = all_mechanics
        .iter()
        .map(|m| (*m, profile.mechanic_affinity(m)))
        .collect();

    ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    ranked
}

/// Model how an NPC's personality shifts over time based on relationships.
///
/// NPCs in killer-heavy environments become more defensive (achiever),
/// while NPCs surrounded by socializers become more social. This mirrors
/// Bartle's population dynamics at the individual level.
#[must_use]
pub fn personality_drift(
    npc: &NpcPersonality,
    neighbors: &[(&NpcPersonality, f64)],
) -> BartleProfile {
    let base = derive_bartle_profile(npc);

    if neighbors.is_empty() {
        return base;
    }

    let mut pressure_a = 0.0_f64;
    let mut pressure_e = 0.0_f64;
    let mut pressure_s = 0.0_f64;
    let mut pressure_k = 0.0_f64;

    for (neighbor, influence) in neighbors {
        let neighbor_profile = derive_bartle_profile(neighbor);
        let dominant = neighbor_profile.dominant();

        let drift_rate = influence * 0.1;
        match dominant {
            PlayerType::Achiever => pressure_a += drift_rate,
            PlayerType::Explorer => pressure_e += drift_rate,
            PlayerType::Socializer => pressure_s += drift_rate,
            PlayerType::Killer => {
                pressure_k -= drift_rate * 0.5;
                pressure_a += drift_rate * 0.3;
            }
        }
    }

    BartleProfile::new(
        (base.achiever + pressure_a).max(0.0),
        (base.explorer + pressure_e).max(0.0),
        (base.socializer + pressure_s).max(0.0),
        (base.killer + pressure_k).max(0.0),
    )
}

/// Generate a behavioral tendency summary from an NPC's Bartle profile.
///
/// Returns a structured description useful for dialogue systems and AI prompts.
#[must_use]
pub fn behavioral_summary(npc: &NpcPersonality) -> BehavioralTendencies {
    let profile = derive_bartle_profile(npc);
    let dominant = profile.dominant();
    let purity = profile.purity();
    let (action_axis, focus_axis) = profile.to_axes();

    BehavioralTendencies {
        dominant_type: dominant,
        purity,
        action_tendency: if action_axis > 0.0 {
            ActionTendency::Proactive
        } else {
            ActionTendency::Reactive
        },
        focus_tendency: if focus_axis > 0.0 {
            FocusTendency::SocialOriented
        } else {
            FocusTendency::WorldOriented
        },
        top_mechanics: recommended_mechanics(npc).into_iter().take(3).collect(),
    }
}

/// Structured behavioral tendencies for an NPC.
#[derive(Debug, Clone)]
pub struct BehavioralTendencies {
    /// The NPC's dominant player type.
    pub dominant_type: PlayerType,
    /// How strongly the NPC leans toward one type (0.25 = balanced, 1.0 = pure).
    pub purity: f64,
    /// Whether the NPC tends to act or react.
    pub action_tendency: ActionTendency,
    /// Whether the NPC focuses on people or environment.
    pub focus_tendency: FocusTendency,
    /// Top 3 mechanic affinities.
    pub top_mechanics: Vec<(MechanicCategory, f64)>,
}

/// Whether an NPC tends to initiate or respond.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionTendency {
    /// Initiates actions, seeks goals, pushes events forward.
    Proactive,
    /// Responds to events, adapts, flows with circumstances.
    Reactive,
}

/// Whether an NPC focuses on social dynamics or world systems.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusTendency {
    /// Primarily concerned with people, relationships, status.
    SocialOriented,
    /// Primarily concerned with environment, resources, knowledge.
    WorldOriented,
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]
mod tests {
    use super::*;
    use crate::game::rpgpt::knowledge::KnowledgeBounds;
    use crate::game::rpgpt::npc::{MaslowNeeds, NeedState, NpcVoice};
    use crate::game::rpgpt::trust::TrustModel;

    fn make_npc(
        id: &str,
        survival: f64,
        belonging: f64,
        esteem: f64,
        actualization: f64,
    ) -> NpcPersonality {
        NpcPersonality {
            id: id.into(),
            name: id.into(),
            role: "test".into(),
            appearance: String::new(),
            mannerisms: vec![],
            motivations: MaslowNeeds {
                survival: NeedState::new(survival, "test"),
                safety: NeedState::new(0.3, "test"),
                belonging: NeedState::new(belonging, "test"),
                esteem: NeedState::new(esteem, "test"),
                self_actualization: NeedState::new(actualization, "test"),
            },
            motivation_conflicts: vec![],
            knowledge: KnowledgeBounds {
                knows: vec![],
                suspects: vec![],
                lies_about: vec![],
                does_not_know: vec![],
            },
            voice: NpcVoice {
                speech_patterns: vec![],
                vocabulary_level: "average".into(),
                emotional_baseline: "neutral".into(),
                catchphrases: vec![],
            },
            secrets: vec![],
            relationships: vec![],
            arc: vec![],
            trust: TrustModel::new(5),
        }
    }

    #[test]
    fn survivalist_npc_is_achiever() {
        let npc = make_npc("guard", 0.9, 0.1, 0.2, 0.1);
        let profile = derive_bartle_profile(&npc);
        assert_eq!(profile.dominant(), PlayerType::Achiever);
    }

    #[test]
    fn social_npc_is_socializer() {
        let npc = make_npc("bartender", 0.1, 0.9, 0.1, 0.1);
        let profile = derive_bartle_profile(&npc);
        assert_eq!(profile.dominant(), PlayerType::Socializer);
    }

    #[test]
    fn curious_npc_is_explorer() {
        let npc = make_npc("scholar", 0.1, 0.1, 0.1, 0.9);
        let profile = derive_bartle_profile(&npc);
        assert_eq!(profile.dominant(), PlayerType::Explorer);
    }

    #[test]
    fn competitive_npc_is_killer() {
        let npc = make_npc("warlord", 0.3, 0.0, 0.9, 0.0);
        let profile = derive_bartle_profile(&npc);
        assert_eq!(profile.dominant(), PlayerType::Killer);
    }

    #[test]
    fn interaction_prediction_bounded() {
        let a = make_npc("a", 0.9, 0.1, 0.1, 0.1);
        let b = make_npc("b", 0.1, 0.9, 0.1, 0.1);
        let v = predict_npc_interaction(&a, &b);
        assert!((-1.0..=1.0).contains(&v));
    }

    #[test]
    fn recommended_mechanics_sorted_descending() {
        let npc = make_npc("explorer", 0.1, 0.1, 0.1, 0.9);
        let recs = recommended_mechanics(&npc);
        assert_eq!(recs.len(), 8);
        for window in recs.windows(2) {
            assert!(window[0].1 >= window[1].1);
        }
    }

    #[test]
    fn personality_drift_with_no_neighbors() {
        let npc = make_npc("loner", 0.5, 0.5, 0.5, 0.5);
        let base = derive_bartle_profile(&npc);
        let drifted = personality_drift(&npc, &[]);
        assert!((base.achiever - drifted.achiever).abs() < 1e-10);
    }

    #[test]
    fn personality_drift_toward_socializer_neighbors() {
        let npc = make_npc("neutral", 0.5, 0.3, 0.3, 0.3);
        let social_neighbor = make_npc("social", 0.1, 0.9, 0.1, 0.1);
        let drifted = personality_drift(&npc, &[(&social_neighbor, 1.0)]);
        let base = derive_bartle_profile(&npc);
        assert!(drifted.socializer > base.socializer);
    }

    #[test]
    fn behavioral_summary_produces_valid_output() {
        let npc = make_npc("hero", 0.5, 0.3, 0.7, 0.4);
        let summary = behavioral_summary(&npc);
        assert!(summary.purity >= 0.25);
        assert_eq!(summary.top_mechanics.len(), 3);
    }

    #[test]
    fn killer_neighbor_increases_defensiveness() {
        let npc = make_npc("villager", 0.3, 0.5, 0.2, 0.3);
        let killer = make_npc("bandit", 0.3, 0.0, 0.9, 0.0);
        let drifted = personality_drift(&npc, &[(&killer, 1.0)]);
        let base = derive_bartle_profile(&npc);
        assert!(drifted.achiever >= base.achiever);
    }
}

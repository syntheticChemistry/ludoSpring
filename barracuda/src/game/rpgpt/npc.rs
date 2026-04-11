// SPDX-License-Identifier: AGPL-3.0-or-later
//! NPC personality certificates — the core of non-chatbot NPCs.
//!
//! Every NPC is defined by a loamSpine certificate containing identity,
//! Maslow-hierarchy motivations, knowledge bounds, voice patterns,
//! secrets, relationships, and character arc.

use super::knowledge::KnowledgeBounds;
use super::trust::TrustModel;

/// Maslow's hierarchy of needs — numeric state driving NPC behavior.
///
/// Higher urgency = drives more behavior. Values are 0.0 (satisfied) to
/// 1.0 (desperate). Public domain — Maslow 1943.
#[derive(Debug, Clone)]
pub struct MaslowNeeds {
    /// Physical survival (food, shelter, income).
    pub survival: NeedState,
    /// Safety and security (protection, stability).
    pub safety: NeedState,
    /// Belonging and love (community, relationships).
    pub belonging: NeedState,
    /// Esteem and recognition (respect, achievement).
    pub esteem: NeedState,
    /// Self-actualization (fulfilling potential).
    pub self_actualization: NeedState,
}

impl MaslowNeeds {
    /// Returns the active need — the highest-urgency unsatisfied need.
    #[must_use]
    pub fn active_need(&self) -> (&'static str, f64) {
        let needs = [
            ("survival", self.survival.urgency),
            ("safety", self.safety.urgency),
            ("belonging", self.belonging.urgency),
            ("esteem", self.esteem.urgency),
            ("self_actualization", self.self_actualization.urgency),
        ];
        needs
            .into_iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(("survival", 0.0))
    }

    /// All needs as (name, urgency) pairs for iteration.
    #[must_use]
    pub const fn all_needs(&self) -> [(&'static str, f64); 5] {
        [
            ("survival", self.survival.urgency),
            ("safety", self.safety.urgency),
            ("belonging", self.belonging.urgency),
            ("esteem", self.esteem.urgency),
            ("self_actualization", self.self_actualization.urgency),
        ]
    }
}

/// State of a single need in the Maslow hierarchy.
#[derive(Debug, Clone)]
pub struct NeedState {
    /// Urgency level (0.0 = fully satisfied, 1.0 = desperate).
    pub urgency: f64,
    /// Description of the current state.
    pub current_state: String,
    /// What threatens this need (if anything).
    pub threat: Option<String>,
    /// What would satisfy this need.
    pub satisfier: Option<String>,
}

impl NeedState {
    /// Create a need with the given urgency and description.
    #[must_use]
    pub fn new(urgency: f64, state: impl Into<String>) -> Self {
        Self {
            urgency: urgency.clamp(0.0, 1.0),
            current_state: state.into(),
            threat: None,
            satisfier: None,
        }
    }
}

/// A conflict between two needs.
#[derive(Debug, Clone)]
pub struct MotivationConflict {
    /// First conflicting need.
    pub need_a: String,
    /// Second conflicting need.
    pub need_b: String,
    /// Description of the conflict.
    pub description: String,
}

/// A relationship in the NPC's social graph.
#[derive(Debug, Clone)]
pub struct Relationship {
    /// Name or ID of the other entity.
    pub entity: String,
    /// Type of relationship.
    pub relationship_type: RelationshipType,
    /// Strength (-1.0 = hostile, 0.0 = neutral, 1.0 = devoted).
    pub strength: f64,
    /// Why this relationship exists.
    pub reason: String,
    /// Direction the relationship is heading.
    pub trajectory: String,
}

/// Type of NPC relationship.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationshipType {
    /// Formal organizational relationship.
    Institutional,
    /// Teacher-student or mentor-mentee.
    Mentorship,
    /// Opposed interests or active conflict.
    Adversarial,
    /// Deep respect or admiration.
    Reverence,
    /// Romantic or intimate bond.
    Romantic,
    /// Casual friendship.
    Friendship,
    /// Family bond.
    Familial,
}

/// A secret the NPC holds.
#[derive(Debug, Clone)]
pub struct NpcSecret {
    /// Unique identifier for the secret.
    pub id: String,
    /// Description of the secret.
    pub description: String,
    /// Conditions under which the secret may be revealed.
    pub reveal_conditions: Vec<String>,
}

/// Phase of an NPC's character arc.
#[derive(Debug, Clone)]
pub struct ArcPhase {
    /// Phase identifier.
    pub id: String,
    /// Description of this phase.
    pub description: String,
    /// Current status.
    pub status: ArcPhaseStatus,
    /// Events that trigger transition to the next phase.
    pub triggers: Vec<String>,
}

/// Status of a character arc phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArcPhaseStatus {
    /// Phase has been completed.
    Completed,
    /// Phase is currently active.
    Active,
    /// Phase has not yet been reached.
    Pending,
}

/// Voice characteristics for an NPC.
#[derive(Debug, Clone)]
pub struct NpcVoice {
    /// Speech patterns (list of descriptive strings).
    pub speech_patterns: Vec<String>,
    /// Vocabulary level description.
    pub vocabulary_level: String,
    /// Baseline emotional tone.
    pub emotional_baseline: String,
    /// Catchphrases the NPC uses.
    pub catchphrases: Vec<String>,
}

/// Complete NPC personality certificate.
///
/// This is a loamSpine certificate type — immutable reference identity with
/// evolving state tracked in rhizoCrypt.
#[derive(Debug, Clone)]
pub struct NpcPersonality {
    /// NPC unique identifier.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Role in the world.
    pub role: String,
    /// Physical appearance description.
    pub appearance: String,
    /// Observable mannerisms.
    pub mannerisms: Vec<String>,

    /// Maslow hierarchy motivations.
    pub motivations: MaslowNeeds,
    /// Conflicts between motivations.
    pub motivation_conflicts: Vec<MotivationConflict>,

    /// Knowledge bounds (knows / suspects / lies about / does not know).
    pub knowledge: KnowledgeBounds,

    /// Voice characteristics.
    pub voice: NpcVoice,

    /// Secrets held by this NPC.
    pub secrets: Vec<NpcSecret>,

    /// Relationships with other entities.
    pub relationships: Vec<Relationship>,

    /// Character arc phases.
    pub arc: Vec<ArcPhase>,

    /// Trust model.
    pub trust: TrustModel,
}

impl NpcPersonality {
    /// The NPC's active (highest-urgency) need.
    #[must_use]
    pub fn active_need(&self) -> (&'static str, f64) {
        self.motivations.active_need()
    }

    /// The NPC's current character arc phase.
    #[must_use]
    pub fn current_arc_phase(&self) -> Option<&ArcPhase> {
        self.arc.iter().find(|p| p.status == ArcPhaseStatus::Active)
    }

    /// Whether a specific secret can be revealed at the current trust level.
    #[must_use]
    pub fn can_reveal_secret(&self, secret_id: &str, trust_level: u8) -> bool {
        self.secrets.iter().any(|s| {
            s.id == secret_id
                && s.reveal_conditions.iter().any(|c| {
                    c.contains("trust_level")
                        && c.chars()
                            .filter(char::is_ascii_digit)
                            .collect::<String>()
                            .parse::<u8>()
                            .is_ok_and(|required| trust_level >= required)
                })
        })
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::super::knowledge::{KnowledgeBounds, LieTopic};
    use super::*;

    fn maren() -> NpcPersonality {
        NpcPersonality {
            id: "maren_blacksmith".into(),
            name: "Maren the Blacksmith".into(),
            role: "master smith, guild member, secret innovator".into(),
            appearance: "Broad shoulders, burn scars on forearms".into(),
            mannerisms: vec!["Taps hammer against thigh".into()],
            motivations: MaslowNeeds {
                survival: NeedState::new(0.2, "Adequate"),
                safety: NeedState {
                    urgency: 0.7,
                    current_state: "Guild provides protection but demands conformity".into(),
                    threat: Some("Guild master suspects experiments".into()),
                    satisfier: Some("Remaining in guild good standing".into()),
                },
                belonging: NeedState::new(0.5, "Respected but lonely"),
                esteem: NeedState {
                    urgency: 0.8,
                    current_state: "Knows she could forge better".into(),
                    threat: Some("Recognition requires revealing forbidden work".into()),
                    satisfier: Some("Acknowledged as a great smith".into()),
                },
                self_actualization: NeedState::new(0.4, "Dreams of a masterwork"),
            },
            motivation_conflicts: vec![MotivationConflict {
                need_a: "esteem".into(),
                need_b: "safety".into(),
                description: "Pursuing recognition requires revealing forbidden work".into(),
            }],
            knowledge: KnowledgeBounds {
                knows: vec![
                    "The king is ill".into(),
                    "The northern pass is blocked by snow".into(),
                ],
                suspects: vec![],
                lies_about: vec![LieTopic {
                    topic: "experiments".into(),
                    surface_claim: "I only forge what the guild approves".into(),
                    truth: "Hidden workshop beneath the forge".into(),
                    reason: "Guild expulsion".into(),
                    tell: "Hand moves to cover burn scars".into(),
                    detection_dc: 15,
                    detection_skills: vec!["Perception".into()],
                }],
                does_not_know: vec!["The dragon's weakness".into()],
            },
            voice: NpcVoice {
                speech_patterns: vec!["Uses forge metaphors".into()],
                vocabulary_level: "working-class but literate".into(),
                emotional_baseline: "warm but guarded".into(),
                catchphrases: vec!["Good steel doesn't come from a cold forge.".into()],
            },
            secrets: vec![NpcSecret {
                id: "hidden_workshop".into(),
                description: "Hidden workshop beneath the forge".into(),
                reveal_conditions: vec!["Player earns trust_level >= 3".into()],
            }],
            relationships: vec![
                Relationship {
                    entity: "Blacksmith Guild".into(),
                    relationship_type: RelationshipType::Institutional,
                    strength: 0.6,
                    reason: "Provides livelihood, constrains growth".into(),
                    trajectory: "eroding".into(),
                },
                Relationship {
                    entity: "Guild Master Harven".into(),
                    relationship_type: RelationshipType::Adversarial,
                    strength: -0.4,
                    reason: "Suspects experiments".into(),
                    trajectory: "worsening".into(),
                },
            ],
            arc: vec![
                ArcPhase {
                    id: "conformity".into(),
                    description: "Following guild rules".into(),
                    status: ArcPhaseStatus::Completed,
                    triggers: vec![],
                },
                ArcPhase {
                    id: "internal_conflict".into(),
                    description: "Experimenting in secret".into(),
                    status: ArcPhaseStatus::Active,
                    triggers: vec!["Guild inspection".into()],
                },
                ArcPhase {
                    id: "revelation".into(),
                    description: "Forced to choose".into(),
                    status: ArcPhaseStatus::Pending,
                    triggers: vec![],
                },
            ],
            trust: {
                let mut tm = TrustModel::new(5);
                tm.set_level(0, 0.0, "Polite but professional");
                tm.set_level(3, 3.0, "Reveals hidden workshop");
                tm
            },
        }
    }

    #[test]
    fn active_need_is_esteem() {
        let npc = maren();
        let (need, urgency) = npc.active_need();
        assert_eq!(need, "esteem");
        assert!((urgency - 0.8).abs() < f64::EPSILON);
    }

    #[test]
    fn current_arc_phase_is_internal_conflict() {
        let npc = maren();
        let phase = npc.current_arc_phase().expect("should have active phase");
        assert_eq!(phase.id, "internal_conflict");
    }

    #[test]
    fn secret_gated_by_trust() {
        let npc = maren();
        assert!(!npc.can_reveal_secret("hidden_workshop", 2));
        assert!(npc.can_reveal_secret("hidden_workshop", 3));
        assert!(npc.can_reveal_secret("hidden_workshop", 5));
    }

    #[test]
    fn two_relationships() {
        let npc = maren();
        assert_eq!(npc.relationships.len(), 2);
        assert!(npc.relationships[0].strength > 0.0);
        assert!(npc.relationships[1].strength < 0.0);
    }

    #[test]
    fn motivation_conflict_exists() {
        let npc = maren();
        assert_eq!(npc.motivation_conflicts.len(), 1);
        assert_eq!(npc.motivation_conflicts[0].need_a, "esteem");
    }

    #[test]
    fn all_five_needs_present() {
        let npc = maren();
        let needs = npc.motivations.all_needs();
        assert_eq!(needs.len(), 5);
        let names: Vec<&str> = needs.iter().map(|(n, _)| *n).collect();
        assert!(names.contains(&"survival"));
        assert!(names.contains(&"self_actualization"));
    }

    #[test]
    fn need_urgency_clamped() {
        let need = NeedState::new(1.5, "Over maximum");
        assert!((need.urgency - 1.0).abs() < f64::EPSILON);
        let need = NeedState::new(-0.5, "Under minimum");
        assert!((need.urgency - 0.0).abs() < f64::EPSILON);
    }
}

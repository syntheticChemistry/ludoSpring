// SPDX-License-Identifier: AGPL-3.0-or-later
//! Knowledge bounds — the system that makes NPCs not chatbots.
//!
//! NPCs know things, suspect things, lie about things, and genuinely don't know
//! things. These bounds are hard constraints on AI narration output.

/// A topic the NPC suspects but is not certain about.
#[derive(Debug, Clone)]
pub struct Suspicion {
    /// What the NPC suspects.
    pub topic: String,
    /// Their belief about the topic.
    pub belief: String,
    /// Confidence level (0.0 = wild guess, 1.0 = near-certain).
    pub confidence: f64,
}

/// A topic the NPC actively lies about.
#[derive(Debug, Clone)]
pub struct LieTopic {
    /// The topic of the lie.
    pub topic: String,
    /// What the NPC claims (the surface lie).
    pub surface_claim: String,
    /// The actual truth (hidden from the player).
    pub truth: String,
    /// Why the NPC lies about this.
    pub reason: String,
    /// Observable behavioral cue when lying (for passive checks).
    pub tell: String,
    /// Difficulty class to detect the lie.
    pub detection_dc: u8,
    /// Which skills can detect this lie.
    pub detection_skills: Vec<String>,
}

/// Complete knowledge bounds for an NPC.
///
/// These are hard constraints on AI output. The narration engine must:
/// - Freely share information in `knows`
/// - Hedge and express uncertainty for `suspects`
/// - Actively misdirect for `lies_about` topics
/// - Genuinely produce no information for `does_not_know`
#[derive(Debug, Clone, Default)]
pub struct KnowledgeBounds {
    /// Facts the NPC knows and will share freely.
    pub knows: Vec<String>,
    /// Topics the NPC suspects but is uncertain about.
    pub suspects: Vec<Suspicion>,
    /// Topics the NPC actively lies about (with tells).
    pub lies_about: Vec<LieTopic>,
    /// Topics the NPC genuinely has no information about.
    pub does_not_know: Vec<String>,
}

/// Result of querying an NPC's knowledge about a topic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KnowledgeQueryResult {
    /// NPC knows this — share freely.
    Known,
    /// NPC suspects this — hedge with confidence.
    Suspected,
    /// NPC lies about this — use surface claim, include tell.
    LiedAbout,
    /// NPC genuinely does not know — produce no information.
    Unknown,
    /// Topic doesn't match any bound — NPC has no special knowledge.
    Unbound,
}

impl KnowledgeBounds {
    /// Query what the NPC knows about a topic.
    ///
    /// Performs case-insensitive substring matching against all categories.
    /// Returns the most restrictive match (lies_about > does_not_know > suspects > knows).
    #[must_use]
    pub fn query(&self, topic: &str) -> KnowledgeQueryResult {
        let topic_lower = topic.to_lowercase();

        for lie in &self.lies_about {
            if lie.topic.to_lowercase().contains(&topic_lower)
                || topic_lower.contains(&lie.topic.to_lowercase())
            {
                return KnowledgeQueryResult::LiedAbout;
            }
        }

        for unknown in &self.does_not_know {
            if unknown.to_lowercase().contains(&topic_lower)
                || topic_lower.contains(&unknown.to_lowercase())
            {
                return KnowledgeQueryResult::Unknown;
            }
        }

        for suspicion in &self.suspects {
            if suspicion.topic.to_lowercase().contains(&topic_lower)
                || topic_lower.contains(&suspicion.topic.to_lowercase())
            {
                return KnowledgeQueryResult::Suspected;
            }
        }

        for known in &self.knows {
            if known.to_lowercase().contains(&topic_lower)
                || topic_lower.contains(&known.to_lowercase())
            {
                return KnowledgeQueryResult::Known;
            }
        }

        KnowledgeQueryResult::Unbound
    }

    /// Get the lie details for a topic, if the NPC lies about it.
    #[must_use]
    pub fn get_lie(&self, topic: &str) -> Option<&LieTopic> {
        let topic_lower = topic.to_lowercase();
        self.lies_about.iter().find(|lie| {
            lie.topic.to_lowercase().contains(&topic_lower)
                || topic_lower.contains(&lie.topic.to_lowercase())
        })
    }

    /// Get the suspicion details for a topic, if the NPC suspects it.
    #[must_use]
    pub fn get_suspicion(&self, topic: &str) -> Option<&Suspicion> {
        let topic_lower = topic.to_lowercase();
        self.suspects.iter().find(|s| {
            s.topic.to_lowercase().contains(&topic_lower)
                || topic_lower.contains(&s.topic.to_lowercase())
        })
    }

    /// Total number of knowledge entries across all categories.
    #[must_use]
    pub const fn total_entries(&self) -> usize {
        self.knows.len() + self.suspects.len() + self.lies_about.len() + self.does_not_know.len()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    fn maren_knowledge() -> KnowledgeBounds {
        KnowledgeBounds {
            knows: vec![
                "The king is ill".into(),
                "The northern pass is blocked by snow".into(),
                "There is a healer in the forest".into(),
            ],
            suspects: vec![Suspicion {
                topic: "The advisor".into(),
                belief: "May be poisoning the king".into(),
                confidence: 0.6,
            }],
            lies_about: vec![LieTopic {
                topic: "experiments".into(),
                surface_claim: "I only forge what the guild approves".into(),
                truth: "She has a hidden workshop beneath the forge".into(),
                reason: "Guild expulsion and imprisonment".into(),
                tell: "Right hand moves to cover burn scars".into(),
                detection_dc: 15,
                detection_skills: vec!["Perception".into(), "Empathy".into()],
            }],
            does_not_know: vec![
                "The dragon's weakness".into(),
                "What happened to the missing prince".into(),
            ],
        }
    }

    #[test]
    fn query_known_topic_returns_known() {
        let kb = maren_knowledge();
        assert_eq!(kb.query("king"), KnowledgeQueryResult::Known);
        assert_eq!(kb.query("northern pass"), KnowledgeQueryResult::Known);
        assert_eq!(kb.query("healer"), KnowledgeQueryResult::Known);
    }

    #[test]
    fn query_suspected_topic_returns_suspected() {
        let kb = maren_knowledge();
        assert_eq!(kb.query("advisor"), KnowledgeQueryResult::Suspected);
    }

    #[test]
    fn query_lie_topic_returns_lied_about() {
        let kb = maren_knowledge();
        assert_eq!(kb.query("experiments"), KnowledgeQueryResult::LiedAbout);
    }

    #[test]
    fn query_unknown_topic_returns_unknown() {
        let kb = maren_knowledge();
        assert_eq!(kb.query("dragon"), KnowledgeQueryResult::Unknown);
        assert_eq!(kb.query("missing prince"), KnowledgeQueryResult::Unknown);
    }

    #[test]
    fn query_unrelated_topic_returns_unbound() {
        let kb = maren_knowledge();
        assert_eq!(kb.query("weather tomorrow"), KnowledgeQueryResult::Unbound);
    }

    #[test]
    fn lies_take_priority_over_other_categories() {
        let kb = maren_knowledge();
        assert_eq!(kb.query("experiments"), KnowledgeQueryResult::LiedAbout);
    }

    #[test]
    fn get_lie_returns_details() {
        let kb = maren_knowledge();
        let lie = kb.get_lie("experiments").expect("should find lie");
        assert_eq!(lie.detection_dc, 15);
        assert!(lie.tell.contains("burn scars"));
        assert_eq!(lie.detection_skills.len(), 2);
    }

    #[test]
    fn get_suspicion_returns_details() {
        let kb = maren_knowledge();
        let s = kb.get_suspicion("advisor").expect("should find suspicion");
        assert!((s.confidence - 0.6).abs() < f64::EPSILON);
    }

    #[test]
    fn case_insensitive_matching() {
        let kb = maren_knowledge();
        assert_eq!(kb.query("KING"), KnowledgeQueryResult::Known);
        assert_eq!(kb.query("Experiments"), KnowledgeQueryResult::LiedAbout);
        assert_eq!(kb.query("DRAGON"), KnowledgeQueryResult::Unknown);
    }

    #[test]
    fn total_entries_count() {
        let kb = maren_knowledge();
        assert_eq!(kb.total_entries(), 7); // 3 + 1 + 1 + 2
    }

    #[test]
    fn empty_knowledge_bounds_returns_unbound() {
        let kb = KnowledgeBounds::default();
        assert_eq!(kb.query("anything"), KnowledgeQueryResult::Unbound);
        assert_eq!(kb.total_entries(), 0);
    }
}

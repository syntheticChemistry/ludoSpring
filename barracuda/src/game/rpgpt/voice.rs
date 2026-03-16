// SPDX-License-Identifier: AGPL-3.0-or-later
//! Internal voices — the Disco Elysium model.
//!
//! Skills are not just numbers — they are **perspectives**. Each skill is a
//! constrained AI call with its own personality. Passive checks trigger voices
//! without the player asking.

use super::plane::PassiveCheckPriority;

/// The ten internal voices.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ipc", derive(serde::Serialize, serde::Deserialize))]
pub enum VoiceId {
    /// Cold, analytical. Notices contradictions and patterns.
    Logic,
    /// Warm, intuitive. Reads emotional states and subtext.
    Empathy,
    /// Silver-tongued, political. Spots leverage and argument weaknesses.
    Rhetoric,
    /// Hyper-aware, detail-obsessed. Notices physical details and tells.
    Perception,
    /// Stoic, physical. Tracks pain, fatigue, danger.
    Endurance,
    /// Commanding, bold. Reads power dynamics.
    Authority,
    /// Cool, controlled. Warns when facade is slipping.
    Composure,
    /// Hedonistic, risk-seeking. Notices temptations and shortcuts.
    Electrochemistry,
    /// Pedantic, academic. Cross-references lore and history.
    Encyclopedia,
    /// Dreamy, mystical. Senses hidden connections and hunches.
    InlandEmpire,
}

impl VoiceId {
    /// Human-readable name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Logic => "Logic",
            Self::Empathy => "Empathy",
            Self::Rhetoric => "Rhetoric",
            Self::Perception => "Perception",
            Self::Endurance => "Endurance",
            Self::Authority => "Authority",
            Self::Composure => "Composure",
            Self::Electrochemistry => "Electrochemistry",
            Self::Encyclopedia => "Encyclopedia",
            Self::InlandEmpire => "Inland Empire",
        }
    }

    /// All ten voices.
    pub const ALL: [Self; 10] = [
        Self::Logic,
        Self::Empathy,
        Self::Rhetoric,
        Self::Perception,
        Self::Endurance,
        Self::Authority,
        Self::Composure,
        Self::Electrochemistry,
        Self::Encyclopedia,
        Self::InlandEmpire,
    ];

    /// Recommended LLM temperature for this voice.
    #[must_use]
    pub const fn recommended_temperature(self) -> f32 {
        match self {
            Self::Logic | Self::Perception | Self::Endurance | Self::Encyclopedia => 0.3,
            Self::Rhetoric | Self::Authority => 0.4,
            Self::Empathy => 0.5,
            Self::Composure => 0.2,
            Self::Electrochemistry => 0.7,
            Self::InlandEmpire => 0.8,
        }
    }

    /// Maximum token output for this voice.
    #[must_use]
    pub const fn max_tokens(self) -> u16 {
        match self {
            Self::Logic | Self::Encyclopedia | Self::InlandEmpire | Self::Empathy => 80,
            Self::Rhetoric | Self::Perception => 60,
            Self::Electrochemistry => 50,
            Self::Endurance | Self::Authority | Self::Composure => 40,
        }
    }

    /// Natural opposing voice (for contradiction dynamics).
    #[must_use]
    pub const fn opposite(self) -> Option<Self> {
        match self {
            Self::Logic => Some(Self::InlandEmpire),
            Self::InlandEmpire => Some(Self::Logic),
            Self::Empathy => Some(Self::Authority),
            Self::Authority | Self::Rhetoric => Some(Self::Empathy),
            Self::Electrochemistry => Some(Self::Composure),
            Self::Composure => Some(Self::Electrochemistry),
            _ => None,
        }
    }
}

/// A passive trigger that can fire a voice.
#[derive(Debug, Clone)]
pub struct VoiceTrigger {
    /// Which voice this trigger belongs to.
    pub voice: VoiceId,
    /// Condition description (matched against scene state).
    pub condition: String,
    /// Difficulty class for the passive check.
    pub dc: u8,
    /// Display priority if the check succeeds.
    pub priority: PassiveCheckPriority,
}

/// Result of evaluating a passive voice check.
#[derive(Debug, Clone)]
pub struct VoiceCheckResult {
    /// Which voice attempted the check.
    pub voice: VoiceId,
    /// The roll (1d20 + skill modifier).
    pub roll: i32,
    /// The DC that was checked against.
    pub dc: u8,
    /// Whether the check succeeded.
    pub success: bool,
    /// Priority for display ordering.
    pub priority: PassiveCheckPriority,
}

impl VoiceCheckResult {
    /// Evaluate a passive voice check.
    #[must_use]
    pub fn evaluate(
        voice: VoiceId,
        skill_modifier: i32,
        die_roll: i32,
        dc: u8,
        priority: PassiveCheckPriority,
    ) -> Self {
        let total = die_roll + skill_modifier;
        Self {
            voice,
            roll: total,
            dc,
            success: total >= i32::from(dc),
            priority,
        }
    }
}

/// A voice output that was triggered by a passive check.
#[derive(Debug, Clone)]
pub struct VoiceOutput {
    /// Which voice is speaking.
    pub voice: VoiceId,
    /// The text the voice says (generated by AI or template).
    pub text: String,
    /// Display priority.
    pub priority: PassiveCheckPriority,
    /// The roll that triggered this voice.
    pub roll: i32,
}

/// Select which voice outputs to present to the player.
///
/// Rules:
/// - Maximum 3 voices per action
/// - Priority ordering: critical > high > medium > low
/// - Ties broken by roll value (higher wins)
#[must_use]
pub fn select_voice_outputs(mut outputs: Vec<VoiceOutput>, max_voices: usize) -> Vec<VoiceOutput> {
    outputs.sort_by(|a, b| {
        b.priority
            .cmp(&a.priority)
            .then_with(|| b.roll.cmp(&a.roll))
    });
    outputs.truncate(max_voices);
    outputs
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ten_voices_all_distinct() {
        let all = VoiceId::ALL;
        assert_eq!(all.len(), 10);
        for (i, a) in all.iter().enumerate() {
            for (j, b) in all.iter().enumerate() {
                if i != j {
                    assert_ne!(a, b);
                }
            }
        }
    }

    #[test]
    fn all_voices_have_names() {
        for v in VoiceId::ALL {
            assert!(!v.name().is_empty());
        }
    }

    #[test]
    fn logic_opposes_inland_empire() {
        assert_eq!(VoiceId::Logic.opposite(), Some(VoiceId::InlandEmpire));
        assert_eq!(VoiceId::InlandEmpire.opposite(), Some(VoiceId::Logic));
    }

    #[test]
    fn empathy_opposes_authority() {
        assert_eq!(VoiceId::Empathy.opposite(), Some(VoiceId::Authority));
        assert_eq!(VoiceId::Authority.opposite(), Some(VoiceId::Empathy));
    }

    #[test]
    fn electrochemistry_opposes_composure() {
        assert_eq!(
            VoiceId::Electrochemistry.opposite(),
            Some(VoiceId::Composure)
        );
        assert_eq!(
            VoiceId::Composure.opposite(),
            Some(VoiceId::Electrochemistry)
        );
    }

    #[test]
    fn voice_check_succeeds_when_roll_meets_dc() {
        let result =
            VoiceCheckResult::evaluate(VoiceId::Logic, 5, 10, 15, PassiveCheckPriority::High);
        assert!(result.success); // 10 + 5 = 15 >= 15
    }

    #[test]
    fn voice_check_fails_when_roll_below_dc() {
        let result =
            VoiceCheckResult::evaluate(VoiceId::Logic, 5, 9, 15, PassiveCheckPriority::High);
        assert!(!result.success); // 9 + 5 = 14 < 15
    }

    #[test]
    fn select_outputs_respects_max_three() {
        let outputs = vec![
            VoiceOutput {
                voice: VoiceId::Logic,
                text: "Contradiction.".into(),
                priority: PassiveCheckPriority::High,
                roll: 18,
            },
            VoiceOutput {
                voice: VoiceId::Empathy,
                text: "Fear.".into(),
                priority: PassiveCheckPriority::High,
                roll: 15,
            },
            VoiceOutput {
                voice: VoiceId::Perception,
                text: "Detail.".into(),
                priority: PassiveCheckPriority::Critical,
                roll: 12,
            },
            VoiceOutput {
                voice: VoiceId::Encyclopedia,
                text: "History.".into(),
                priority: PassiveCheckPriority::Medium,
                roll: 20,
            },
            VoiceOutput {
                voice: VoiceId::Electrochemistry,
                text: "Temptation.".into(),
                priority: PassiveCheckPriority::Low,
                roll: 19,
            },
        ];
        let selected = select_voice_outputs(outputs, 3);
        assert_eq!(selected.len(), 3);
        assert_eq!(selected[0].voice, VoiceId::Perception); // critical
        assert_eq!(selected[1].voice, VoiceId::Logic); // high, roll 18
        assert_eq!(selected[2].voice, VoiceId::Empathy); // high, roll 15
    }

    #[test]
    fn select_outputs_with_zero_returns_empty() {
        let outputs = vec![VoiceOutput {
            voice: VoiceId::Logic,
            text: "Test.".into(),
            priority: PassiveCheckPriority::High,
            roll: 15,
        }];
        let selected = select_voice_outputs(outputs, 0);
        assert!(selected.is_empty());
    }

    #[test]
    fn temperature_ranges_valid() {
        for v in VoiceId::ALL {
            let t = v.recommended_temperature();
            assert!((0.0..=1.0).contains(&t), "{:?} has temperature {t}", v);
        }
    }

    #[test]
    fn max_tokens_positive() {
        for v in VoiceId::ALL {
            assert!(v.max_tokens() > 0, "{:?} has zero max_tokens", v);
        }
    }
}

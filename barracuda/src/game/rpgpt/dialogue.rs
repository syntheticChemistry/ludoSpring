// SPDX-License-Identifier: AGPL-3.0-or-later
//! Dialogue plane skill check resolver — D6 pool system.
//!
//! The Dialogue plane uses a D6 dice pool where each die showing 4+
//! counts as a success. Pool size = skill level + modifiers.
//! Resolution maps success count to five degrees.

use super::super::ruleset::DegreeOfSuccess;

/// Outcome of a single D6 pool roll.
#[derive(Debug, Clone)]
pub struct D6PoolResult {
    /// Individual die values.
    pub dice: Vec<u8>,
    /// Number of successes (dice showing >= threshold).
    pub successes: u8,
    /// Pool size used.
    pub pool_size: u8,
    /// Success threshold (default 4).
    pub threshold: u8,
}

impl D6PoolResult {
    /// Roll a D6 pool deterministically from a seed sequence.
    ///
    /// `die_values` should be pre-rolled values (1-6).
    #[must_use]
    pub fn from_dice(die_values: &[u8], threshold: u8) -> Self {
        let successes = die_values.iter().filter(|&&d| d >= threshold).count();
        #[expect(
            clippy::cast_possible_truncation,
            reason = "pool sizes are small (< 256)"
        )]
        Self {
            dice: die_values.to_vec(),
            successes: successes as u8,
            pool_size: die_values.len() as u8,
            threshold,
        }
    }
}

/// Map D6 pool success count to degree of success.
///
/// - 0 successes with pool >= 2: Critical Failure
/// - 0 successes with pool <= 1: Failure
/// - 1 success: Partial Success
/// - 2-3 successes: Success
/// - 4+ successes: Critical Success
#[must_use]
pub const fn resolve_d6_pool(successes: u8, pool_size: u8) -> DegreeOfSuccess {
    match successes {
        0 if pool_size >= 2 => DegreeOfSuccess::CriticalFailure,
        0 => DegreeOfSuccess::Failure,
        1 => DegreeOfSuccess::PartialSuccess,
        2 | 3 => DegreeOfSuccess::Success,
        _ => DegreeOfSuccess::CriticalSuccess,
    }
}

/// Environment and relationship modifiers that affect pool size.
#[derive(Debug, Clone, Default)]
pub struct DialogueModifiers {
    /// NPC trust-based modifier (positive at high trust).
    pub trust_bonus: i8,
    /// Environmental modifier (noisy tavern = penalty, private room = bonus).
    pub environment: i8,
    /// Emotional state modifier (calm = 0, agitated = penalty).
    pub emotional: i8,
    /// Knowledge advantage (player knows NPC's secret = bonus).
    pub knowledge: i8,
}

impl DialogueModifiers {
    /// Total modifier (can be negative).
    #[must_use]
    pub const fn total(&self) -> i8 {
        self.trust_bonus
            .saturating_add(self.environment)
            .saturating_add(self.emotional)
            .saturating_add(self.knowledge)
    }
}

/// Calculate effective pool size from skill level and modifiers.
///
/// Minimum pool size is 1 (you always roll at least one die).
#[must_use]
pub fn effective_pool_size(skill_level: u8, modifiers: &DialogueModifiers) -> u8 {
    let base = i16::from(skill_level) + i16::from(modifiers.total());
    #[expect(clippy::cast_sign_loss, reason = "clamped to [1, 255]")]
    {
        base.clamp(1, 255) as u8
    }
}

/// A Dialogue plane skill check with full context.
#[derive(Debug, Clone)]
pub struct DialogueCheck {
    /// What skill is being checked.
    pub skill: String,
    /// Base skill level.
    pub skill_level: u8,
    /// Applied modifiers.
    pub modifiers: DialogueModifiers,
    /// Effective pool size.
    pub pool_size: u8,
    /// The D6 pool result.
    pub result: D6PoolResult,
    /// Resolved degree of success.
    pub degree: DegreeOfSuccess,
}

impl DialogueCheck {
    /// Resolve a dialogue skill check from pre-rolled dice.
    #[must_use]
    pub fn resolve(
        skill: impl Into<String>,
        skill_level: u8,
        modifiers: DialogueModifiers,
        die_values: &[u8],
    ) -> Self {
        let pool_size = effective_pool_size(skill_level, &modifiers);
        let effective_dice = if die_values.len() >= usize::from(pool_size) {
            &die_values[..usize::from(pool_size)]
        } else {
            die_values
        };
        let result =
            D6PoolResult::from_dice(effective_dice, crate::tolerances::D6_SUCCESS_THRESHOLD);
        let degree = resolve_d6_pool(result.successes, result.pool_size);
        Self {
            skill: skill.into(),
            skill_level,
            modifiers,
            pool_size,
            result,
            degree,
        }
    }
}

/// Track conversation flow metrics for a dialogue session.
#[derive(Debug, Clone)]
pub struct DialogueFlowTracker {
    /// Exchanges in the current conversation.
    exchanges: Vec<DialogueExchange>,
    /// Running challenge estimate (0.0-1.0).
    challenge: f64,
    /// Running player skill estimate (0.0-1.0).
    skill: f64,
}

/// A single dialogue exchange (player action + outcome).
#[derive(Debug, Clone)]
pub struct DialogueExchange {
    /// What skill was checked (or "freeform" if no check).
    pub skill: String,
    /// Degree of success (or None for freeform).
    pub degree: Option<DegreeOfSuccess>,
    /// Number of options presented.
    pub options_count: u8,
    /// Whether this exchange advanced the conversation.
    pub advanced: bool,
}

impl Default for DialogueFlowTracker {
    fn default() -> Self {
        Self {
            exchanges: Vec::new(),
            challenge: 0.5,
            skill: 0.5,
        }
    }
}

impl DialogueFlowTracker {
    /// Record a dialogue exchange.
    pub fn record(&mut self, exchange: DialogueExchange) {
        if let Some(degree) = exchange.degree {
            let outcome = match degree {
                DegreeOfSuccess::CriticalFailure => 0.0,
                DegreeOfSuccess::Failure => 0.25,
                DegreeOfSuccess::PartialSuccess => 0.5,
                DegreeOfSuccess::Success => 0.75,
                DegreeOfSuccess::CriticalSuccess => 1.0,
            };
            let alpha = crate::tolerances::DIALOGUE_EMA_ALPHA;
            self.skill = self.skill.mul_add(1.0 - alpha, outcome * alpha);
        }
        self.exchanges.push(exchange);
    }

    /// Update the challenge level (e.g., from NPC disposition changes).
    pub const fn set_challenge(&mut self, challenge: f64) {
        self.challenge = challenge.clamp(0.0, 1.0);
    }

    /// Current challenge estimate.
    #[must_use]
    pub const fn challenge(&self) -> f64 {
        self.challenge
    }

    /// Current skill estimate.
    #[must_use]
    pub const fn skill(&self) -> f64 {
        self.skill
    }

    /// Number of exchanges recorded.
    #[must_use]
    pub const fn exchange_count(&self) -> usize {
        self.exchanges.len()
    }

    /// Average options count across exchanges.
    #[must_use]
    pub fn avg_options(&self) -> f64 {
        if self.exchanges.is_empty() {
            return 0.0;
        }
        let total: u32 = self
            .exchanges
            .iter()
            .map(|e| u32::from(e.options_count))
            .sum();
        #[expect(clippy::cast_precision_loss, reason = "exchange counts fit in f64")]
        {
            f64::from(total) / self.exchanges.len() as f64
        }
    }

    /// Success rate across exchanges with checks.
    #[must_use]
    pub fn success_rate(&self) -> f64 {
        let checks: Vec<&DialogueExchange> = self
            .exchanges
            .iter()
            .filter(|e| e.degree.is_some())
            .collect();
        if checks.is_empty() {
            return 0.0;
        }
        let successes = checks
            .iter()
            .filter(|e| {
                matches!(
                    e.degree,
                    Some(
                        DegreeOfSuccess::Success
                            | DegreeOfSuccess::CriticalSuccess
                            | DegreeOfSuccess::PartialSuccess
                    )
                )
            })
            .count();
        #[expect(clippy::cast_precision_loss, reason = "counts fit in f64")]
        {
            successes as f64 / checks.len() as f64
        }
    }

    /// Number of stalled exchanges (not advancing).
    #[must_use]
    pub fn stall_count(&self) -> usize {
        self.exchanges.iter().filter(|e| !e.advanced).count()
    }

    /// Read-only access to exchanges.
    #[must_use]
    pub fn exchanges(&self) -> &[DialogueExchange] {
        &self.exchanges
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn d6_pool_counts_successes() {
        let result = D6PoolResult::from_dice(&[1, 3, 4, 6, 2], 4);
        assert_eq!(result.successes, 2); // 4 and 6
        assert_eq!(result.pool_size, 5);
    }

    #[test]
    fn resolve_zero_successes_large_pool_critical_failure() {
        assert_eq!(resolve_d6_pool(0, 3), DegreeOfSuccess::CriticalFailure);
    }

    #[test]
    fn resolve_zero_successes_small_pool_failure() {
        assert_eq!(resolve_d6_pool(0, 1), DegreeOfSuccess::Failure);
    }

    #[test]
    fn resolve_one_success_partial() {
        assert_eq!(resolve_d6_pool(1, 3), DegreeOfSuccess::PartialSuccess);
    }

    #[test]
    fn resolve_two_successes() {
        assert_eq!(resolve_d6_pool(2, 5), DegreeOfSuccess::Success);
    }

    #[test]
    fn resolve_three_successes() {
        assert_eq!(resolve_d6_pool(3, 6), DegreeOfSuccess::Success);
    }

    #[test]
    fn resolve_four_plus_critical_success() {
        assert_eq!(resolve_d6_pool(4, 6), DegreeOfSuccess::CriticalSuccess);
        assert_eq!(resolve_d6_pool(6, 8), DegreeOfSuccess::CriticalSuccess);
    }

    #[test]
    fn modifiers_stack() {
        let mods = DialogueModifiers {
            trust_bonus: 2,
            environment: -1,
            emotional: 0,
            knowledge: 1,
        };
        assert_eq!(mods.total(), 2);
    }

    #[test]
    fn effective_pool_minimum_one() {
        let mods = DialogueModifiers {
            trust_bonus: -10,
            environment: -10,
            emotional: -10,
            knowledge: -10,
        };
        assert_eq!(effective_pool_size(1, &mods), 1);
    }

    #[test]
    fn dialogue_check_resolve() {
        let check = DialogueCheck::resolve(
            "Persuasion",
            3,
            DialogueModifiers::default(),
            &[1, 4, 6], // pool 3, successes = 2 (4,6)
        );
        assert_eq!(check.pool_size, 3);
        assert_eq!(check.result.successes, 2);
        assert_eq!(check.degree, DegreeOfSuccess::Success);
    }

    #[test]
    fn dialogue_check_with_modifiers() {
        let mods = DialogueModifiers {
            trust_bonus: 2,
            ..Default::default()
        };
        let check = DialogueCheck::resolve(
            "Deception",
            3,
            mods,
            &[1, 2, 3, 4, 5], // pool 5 (3+2), successes = 2 (4,5)
        );
        assert_eq!(check.pool_size, 5);
        assert_eq!(check.result.successes, 2);
    }

    #[test]
    fn flow_tracker_records_exchanges() {
        let mut tracker = DialogueFlowTracker::default();
        tracker.record(DialogueExchange {
            skill: "Persuasion".into(),
            degree: Some(DegreeOfSuccess::Success),
            options_count: 4,
            advanced: true,
        });
        assert_eq!(tracker.exchange_count(), 1);
        assert!(tracker.skill() > 0.5); // success shifts skill up
    }

    #[test]
    fn flow_tracker_success_rate() {
        let mut tracker = DialogueFlowTracker::default();
        for degree in [
            DegreeOfSuccess::Success,
            DegreeOfSuccess::Failure,
            DegreeOfSuccess::PartialSuccess,
        ] {
            tracker.record(DialogueExchange {
                skill: "Test".into(),
                degree: Some(degree),
                options_count: 3,
                advanced: true,
            });
        }
        let rate = tracker.success_rate();
        assert!((rate - 2.0 / 3.0).abs() < 0.01); // S + PS = 2/3
    }

    #[test]
    fn flow_tracker_stall_count() {
        let mut tracker = DialogueFlowTracker::default();
        tracker.record(DialogueExchange {
            skill: "Test".into(),
            degree: None,
            options_count: 3,
            advanced: true,
        });
        tracker.record(DialogueExchange {
            skill: "Test".into(),
            degree: None,
            options_count: 3,
            advanced: false,
        });
        assert_eq!(tracker.stall_count(), 1);
    }

    #[test]
    fn flow_tracker_avg_options() {
        let mut tracker = DialogueFlowTracker::default();
        tracker.record(DialogueExchange {
            skill: "Test".into(),
            degree: None,
            options_count: 4,
            advanced: true,
        });
        tracker.record(DialogueExchange {
            skill: "Test".into(),
            degree: None,
            options_count: 8,
            advanced: true,
        });
        assert!((tracker.avg_options() - 6.0).abs() < f64::EPSILON);
    }
}

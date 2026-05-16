// SPDX-License-Identifier: AGPL-3.0-or-later
//! Gamification metrics — Deterding et al. (2011) framework.
//!
//! Deterding, Dixon, Khaled & Nacke (2011) define gamification as "the use of
//! game design elements in non-game contexts." This module implements the
//! measurement framework for evaluating gamification effectiveness.
//!
//! # Key Concepts
//!
//! - **Game design elements**: points, badges, leaderboards, progress bars,
//!   challenges, narrative framing, social comparison, feedback loops
//! - **Motivation taxonomy**: intrinsic (autonomy, mastery, purpose) vs
//!   extrinsic (rewards, status, competition)
//! - **Engagement decay**: gamification effects diminish over time without
//!   meaningful progression (the "chocolate-covered broccoli" problem)
//!
//! # Application
//!
//! Used for Games@Home (Paper 19) — measuring whether game mechanics
//! effectively drive distributed computation participation. Also applies
//! to any primal that uses game-like feedback to encourage behavior.
//!
//! # References
//!
//! Deterding, S., Dixon, D., Khaled, R., & Nacke, L. (2011).
//! "From Game Design Elements to Gamefulness: Defining 'Gamification'."
//! Proceedings of MindTrek 2011.
//!
//! Ryan, R. & Deci, E. (2000). "Self-Determination Theory and the
//! Facilitation of Intrinsic Motivation." American Psychologist.

/// A gamification design element with measured effectiveness.
#[derive(Debug, Clone)]
pub struct GameElement {
    /// Element name (e.g., "daily_streak", "leaderboard").
    pub name: String,
    /// Primary motivation type this element targets.
    pub motivation: MotivationType,
    /// Measured engagement uplift (0.0 = no effect, 1.0 = doubles engagement).
    pub engagement_uplift: f64,
    /// Decay rate per time unit (how quickly the element loses effectiveness).
    pub decay_rate: f64,
}

/// Motivation taxonomy following Self-Determination Theory (Ryan & Deci 2000).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MotivationType {
    /// Autonomy: sense of choice and control.
    Autonomy,
    /// Mastery/Competence: growth, skill improvement, learning.
    Mastery,
    /// Purpose/Relatedness: connection, meaning, social belonging.
    Purpose,
    /// External: rewards, points, status (diminishing returns).
    Extrinsic,
}

/// Gamification profile measuring the balance of design elements.
#[derive(Debug, Clone)]
pub struct GamificationProfile {
    /// Design elements in use.
    pub elements: Vec<GameElement>,
    /// Time units since implementation (for decay calculation).
    pub time_elapsed: f64,
}

impl GamificationProfile {
    /// Create a new profile with given elements.
    #[must_use]
    pub const fn new(elements: Vec<GameElement>) -> Self {
        Self {
            elements,
            time_elapsed: 0.0,
        }
    }

    /// Advance time and compute current effective engagement.
    #[must_use]
    #[allow(
        clippy::cast_precision_loss,
        reason = "element count won't exceed 2^52"
    )]
    pub fn effective_engagement(&self) -> f64 {
        if self.elements.is_empty() {
            return 0.0;
        }

        let total: f64 = self
            .elements
            .iter()
            .map(|e| {
                let decay_factor = (-e.decay_rate * self.time_elapsed).exp();
                e.engagement_uplift * decay_factor
            })
            .sum();

        total / self.elements.len() as f64
    }

    /// Compute the intrinsic/extrinsic motivation balance.
    ///
    /// Returns a ratio where >1.0 means intrinsic-dominant (sustainable)
    /// and <1.0 means extrinsic-dominant (decay-prone).
    #[must_use]
    pub fn motivation_balance(&self) -> f64 {
        let (intrinsic, extrinsic) =
            self.elements
                .iter()
                .fold((0.0_f64, 0.0_f64), |(i, e), elem| match elem.motivation {
                    MotivationType::Extrinsic => (i, e + elem.engagement_uplift),
                    _ => (i + elem.engagement_uplift, e),
                });

        if extrinsic <= 0.0 {
            return f64::INFINITY;
        }
        intrinsic / extrinsic
    }

    /// Predict engagement at a future time point.
    #[must_use]
    #[allow(
        clippy::cast_precision_loss,
        reason = "element count won't exceed 2^52"
    )]
    pub fn predict_engagement_at(&self, future_time: f64) -> f64 {
        if self.elements.is_empty() {
            return 0.0;
        }

        let total: f64 = self
            .elements
            .iter()
            .map(|e| {
                let decay_factor = (-e.decay_rate * future_time).exp();
                e.engagement_uplift * decay_factor
            })
            .sum();

        total / self.elements.len() as f64
    }

    /// Compute the half-life of the gamification system's effectiveness.
    ///
    /// Returns the time at which engagement drops to 50% of initial.
    #[must_use]
    #[allow(
        clippy::cast_precision_loss,
        reason = "element count won't exceed 2^52"
    )]
    pub fn half_life(&self) -> f64 {
        if self.elements.is_empty() {
            return 0.0;
        }

        let avg_decay: f64 =
            self.elements.iter().map(|e| e.decay_rate).sum::<f64>() / self.elements.len() as f64;

        if avg_decay <= 0.0 {
            return f64::INFINITY;
        }

        std::f64::consts::LN_2 / avg_decay
    }
}

/// Evaluate whether a gamification design avoids the "overjustification effect."
///
/// The overjustification effect (Lepper et al. 1973) occurs when external
/// rewards undermine pre-existing intrinsic motivation. Returns a risk score
/// (0.0 = safe, 1.0 = high risk of undermining intrinsic motivation).
#[must_use]
#[allow(
    clippy::cast_precision_loss,
    reason = "element count won't exceed 2^52"
)]
pub fn overjustification_risk(profile: &GamificationProfile) -> f64 {
    if profile.elements.is_empty() {
        return 0.0;
    }

    let extrinsic_count = profile
        .elements
        .iter()
        .filter(|e| e.motivation == MotivationType::Extrinsic)
        .count();

    let ratio = extrinsic_count as f64 / profile.elements.len() as f64;
    let high_reward_externals = profile
        .elements
        .iter()
        .filter(|e| e.motivation == MotivationType::Extrinsic && e.engagement_uplift > 0.5)
        .count();

    let base_risk = ratio * 0.6;
    let intensity_risk = (high_reward_externals as f64 * 0.2).min(0.4);
    (base_risk + intensity_risk).min(1.0)
}

/// Games@Home computation credit model.
///
/// Models how game mechanics incentivize distributed computation participation
/// (Paper 19). Players earn credits by contributing CPU/GPU time, which
/// unlocks game content. The balance between computation reward and game
/// progression determines sustainability.
#[derive(Debug, Clone)]
pub struct ComputationCredit {
    /// Credits earned per computation unit completed.
    pub credits_per_unit: f64,
    /// Credits required for the next game unlock.
    pub unlock_threshold: f64,
    /// Player's current credit balance.
    pub balance: f64,
    /// Diminishing returns factor (each subsequent unlock costs more).
    pub escalation_factor: f64,
    /// Number of unlocks already earned.
    pub unlocks_earned: u32,
}

impl ComputationCredit {
    /// Create a new credit system with given parameters.
    #[must_use]
    pub const fn new(credits_per_unit: f64, unlock_threshold: f64, escalation_factor: f64) -> Self {
        Self {
            credits_per_unit,
            unlock_threshold,
            escalation_factor,
            balance: 0.0,
            unlocks_earned: 0,
        }
    }

    /// Compute the current unlock cost (escalates with each unlock).
    #[must_use]
    #[allow(
        clippy::cast_possible_wrap,
        reason = "unlocks_earned capped well below i32::MAX"
    )]
    pub fn current_threshold(&self) -> f64 {
        self.unlock_threshold * self.escalation_factor.powi(self.unlocks_earned as i32)
    }

    /// Record N computation units completed.
    pub fn contribute(&mut self, units: f64) {
        self.balance += units * self.credits_per_unit;
    }

    /// Check if an unlock is available and consume credits if so.
    pub fn try_unlock(&mut self) -> bool {
        let threshold = self.current_threshold();
        if self.balance >= threshold {
            self.balance -= threshold;
            self.unlocks_earned += 1;
            true
        } else {
            false
        }
    }

    /// Progress toward next unlock (0.0–1.0).
    #[must_use]
    pub fn progress(&self) -> f64 {
        let threshold = self.current_threshold();
        if threshold <= 0.0 {
            return 1.0;
        }
        (self.balance / threshold).min(1.0)
    }

    /// Compute units required to reach next unlock.
    #[must_use]
    pub fn units_to_unlock(&self) -> f64 {
        let remaining = self.current_threshold() - self.balance;
        if remaining <= 0.0 || self.credits_per_unit <= 0.0 {
            return 0.0;
        }
        remaining / self.credits_per_unit
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_elements() -> Vec<GameElement> {
        vec![
            GameElement {
                name: "daily_streak".into(),
                motivation: MotivationType::Mastery,
                engagement_uplift: 0.3,
                decay_rate: 0.01,
            },
            GameElement {
                name: "leaderboard".into(),
                motivation: MotivationType::Extrinsic,
                engagement_uplift: 0.5,
                decay_rate: 0.05,
            },
            GameElement {
                name: "narrative_progress".into(),
                motivation: MotivationType::Purpose,
                engagement_uplift: 0.4,
                decay_rate: 0.005,
            },
            GameElement {
                name: "skill_tree".into(),
                motivation: MotivationType::Mastery,
                engagement_uplift: 0.6,
                decay_rate: 0.02,
            },
        ]
    }

    #[test]
    fn effective_engagement_decreases_over_time() {
        let mut profile = GamificationProfile::new(sample_elements());
        let initial = profile.effective_engagement();
        profile.time_elapsed = 10.0;
        let later = profile.effective_engagement();
        assert!(later < initial);
    }

    #[test]
    fn effective_engagement_positive_at_start() {
        let profile = GamificationProfile::new(sample_elements());
        assert!(profile.effective_engagement() > 0.0);
    }

    #[test]
    fn motivation_balance_intrinsic_dominant() {
        let elements = vec![
            GameElement {
                name: "mastery_1".into(),
                motivation: MotivationType::Mastery,
                engagement_uplift: 0.5,
                decay_rate: 0.01,
            },
            GameElement {
                name: "mastery_2".into(),
                motivation: MotivationType::Autonomy,
                engagement_uplift: 0.5,
                decay_rate: 0.01,
            },
            GameElement {
                name: "points".into(),
                motivation: MotivationType::Extrinsic,
                engagement_uplift: 0.2,
                decay_rate: 0.05,
            },
        ];
        let profile = GamificationProfile::new(elements);
        assert!(profile.motivation_balance() > 1.0);
    }

    #[test]
    fn half_life_positive() {
        let profile = GamificationProfile::new(sample_elements());
        let hl = profile.half_life();
        assert!(hl > 0.0);
        assert!(hl.is_finite());
    }

    #[test]
    fn overjustification_risk_all_extrinsic() {
        let elements = vec![
            GameElement {
                name: "points".into(),
                motivation: MotivationType::Extrinsic,
                engagement_uplift: 0.8,
                decay_rate: 0.05,
            },
            GameElement {
                name: "badges".into(),
                motivation: MotivationType::Extrinsic,
                engagement_uplift: 0.7,
                decay_rate: 0.04,
            },
        ];
        let profile = GamificationProfile::new(elements);
        let risk = overjustification_risk(&profile);
        assert!(risk > 0.5, "all-extrinsic should be high risk: {risk}");
    }

    #[test]
    fn overjustification_risk_all_intrinsic() {
        let elements = vec![
            GameElement {
                name: "mastery".into(),
                motivation: MotivationType::Mastery,
                engagement_uplift: 0.5,
                decay_rate: 0.01,
            },
            GameElement {
                name: "purpose".into(),
                motivation: MotivationType::Purpose,
                engagement_uplift: 0.6,
                decay_rate: 0.01,
            },
        ];
        let profile = GamificationProfile::new(elements);
        let risk = overjustification_risk(&profile);
        assert!(risk < 0.1, "all-intrinsic should be low risk: {risk}");
    }

    #[test]
    fn computation_credit_basic_flow() {
        let mut credit = ComputationCredit::new(10.0, 100.0, 1.5);
        assert!(credit.progress().abs() < 1e-10);

        credit.contribute(5.0);
        assert!((credit.balance - 50.0).abs() < 1e-10);
        assert!((credit.progress() - 0.5).abs() < 1e-10);
        assert!(!credit.try_unlock());

        credit.contribute(5.0);
        assert!(credit.try_unlock());
        assert_eq!(credit.unlocks_earned, 1);
    }

    #[test]
    fn computation_credit_escalation() {
        let mut credit = ComputationCredit::new(10.0, 100.0, 2.0);
        credit.contribute(10.0);
        assert!(credit.try_unlock());

        let next_threshold = credit.current_threshold();
        assert!((next_threshold - 200.0).abs() < 1e-10);
    }

    #[test]
    fn units_to_unlock_accuracy() {
        let credit = ComputationCredit::new(10.0, 100.0, 1.0);
        let units = credit.units_to_unlock();
        assert!((units - 10.0).abs() < 1e-10);
    }

    #[test]
    fn predict_engagement_at_zero_equals_current() {
        let profile = GamificationProfile::new(sample_elements());
        let current = profile.effective_engagement();
        let predicted = profile.predict_engagement_at(0.0);
        assert!((current - predicted).abs() < 1e-10);
    }
}

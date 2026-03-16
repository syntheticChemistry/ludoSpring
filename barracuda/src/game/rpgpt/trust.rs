// SPDX-License-Identifier: AGPL-3.0-or-later
//! Trust model — earned through tracked interactions, gates information access.
//!
//! Trust accumulates from defined actions. Each action's delta is specified in the
//! NPC personality certificate. Betrayal has larger magnitude than helpfulness
//! (asymmetric by design).

/// A trust-changing action with its effect magnitude.
#[derive(Debug, Clone)]
pub struct TrustAction {
    /// Description of the action.
    pub action: String,
    /// Trust change magnitude (positive = trust gained, negative = trust lost).
    pub delta: f64,
}

/// The trust model for an NPC.
#[derive(Debug, Clone)]
pub struct TrustModel {
    /// Current accumulated trust value.
    current: f64,
    /// Maximum trust level.
    pub max_level: u8,
    /// Trust thresholds for each level (index = level, value = minimum trust).
    level_thresholds: Vec<f64>,
    /// Effects unlocked at each level.
    pub level_effects: Vec<String>,
    /// Actions that increase trust.
    pub positive_actions: Vec<TrustAction>,
    /// Actions that decrease trust.
    pub negative_actions: Vec<TrustAction>,
    /// History of trust changes.
    history: Vec<TrustDelta>,
}

/// A recorded trust change.
#[derive(Debug, Clone)]
pub struct TrustDelta {
    /// What caused the change.
    pub action: String,
    /// The magnitude of the change.
    pub delta: f64,
    /// Trust value after the change.
    pub trust_after: f64,
}

impl TrustModel {
    /// Create a new trust model with the given number of levels.
    ///
    /// Levels are evenly spaced from 0 to `max_level`, with thresholds at
    /// each integer boundary.
    #[must_use]
    pub fn new(max_level: u8) -> Self {
        let thresholds: Vec<f64> = (0..=max_level).map(f64::from).collect();
        let effects = vec![String::new(); usize::from(max_level) + 1];
        Self {
            current: 0.0,
            max_level,
            level_thresholds: thresholds,
            level_effects: effects,
            positive_actions: Vec::new(),
            negative_actions: Vec::new(),
            history: Vec::new(),
        }
    }

    /// Set the threshold and effect description for a specific trust level.
    pub fn set_level(&mut self, level: u8, threshold: f64, effect: impl Into<String>) {
        let idx = usize::from(level);
        if idx < self.level_thresholds.len() {
            self.level_thresholds[idx] = threshold;
            self.level_effects[idx] = effect.into();
        }
    }

    /// Current trust value (raw).
    #[must_use]
    pub const fn current_trust(&self) -> f64 {
        self.current
    }

    /// Current trust level (discrete).
    #[must_use]
    pub fn current_level(&self) -> u8 {
        let mut level = 0u8;
        for (i, &threshold) in self.level_thresholds.iter().enumerate() {
            if self.current >= threshold {
                #[expect(
                    clippy::cast_possible_truncation,
                    reason = "trust levels are small (< 256)"
                )]
                {
                    level = i as u8;
                }
            }
        }
        level
    }

    /// Apply a trust change.
    pub fn apply_delta(&mut self, action: impl Into<String>, delta: f64) {
        let action_str = action.into();
        self.current += delta;
        self.current = self.current.max(f64::from(-(i32::from(self.max_level))));
        self.current = self.current.min(f64::from(self.max_level));
        self.history.push(TrustDelta {
            action: action_str,
            delta,
            trust_after: self.current,
        });
    }

    /// Get the effect description for the current trust level.
    #[must_use]
    pub fn current_effect(&self) -> &str {
        let level = self.current_level();
        self.level_effects
            .get(usize::from(level))
            .map_or("", String::as_str)
    }

    /// Whether the NPC has reached at least the given trust level.
    #[must_use]
    pub fn has_reached_level(&self, level: u8) -> bool {
        self.current_level() >= level
    }

    /// Number of trust changes recorded.
    #[must_use]
    pub const fn history_len(&self) -> usize {
        self.history.len()
    }

    /// Read-only access to trust history.
    #[must_use]
    pub fn history(&self) -> &[TrustDelta] {
        &self.history
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn maren_trust() -> TrustModel {
        let mut tm = TrustModel::new(5);
        tm.set_level(0, 0.0, "Polite but professional.");
        tm.set_level(1, 1.0, "Warmer. Shares opinions.");
        tm.set_level(2, 2.0, "Confides frustrations.");
        tm.set_level(3, 3.0, "Reveals hidden workshop.");
        tm.set_level(4, 4.0, "Shares the master's journal.");
        tm.set_level(5, 5.0, "Full partnership.");
        tm.positive_actions.push(TrustAction {
            action: "Bring rare materials".into(),
            delta: 0.5,
        });
        tm.positive_actions.push(TrustAction {
            action: "Defend reputation".into(),
            delta: 1.0,
        });
        tm.negative_actions.push(TrustAction {
            action: "Threaten to reveal secrets".into(),
            delta: -2.0,
        });
        tm.negative_actions.push(TrustAction {
            action: "Betray confidence to guild".into(),
            delta: -5.0,
        });
        tm
    }

    #[test]
    fn initial_trust_is_zero() {
        let tm = maren_trust();
        assert!((tm.current_trust() - 0.0).abs() < f64::EPSILON);
        assert_eq!(tm.current_level(), 0);
    }

    #[test]
    fn positive_action_increases_trust() {
        let mut tm = maren_trust();
        tm.apply_delta("Brought rare materials", 0.5);
        assert!((tm.current_trust() - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn trust_level_advances_at_threshold() {
        let mut tm = maren_trust();
        tm.apply_delta("Defend reputation", 1.0);
        assert_eq!(tm.current_level(), 1);
        tm.apply_delta("Help with experiments", 1.0);
        assert_eq!(tm.current_level(), 2);
    }

    #[test]
    fn negative_action_decreases_trust() {
        let mut tm = maren_trust();
        tm.apply_delta("Defend reputation", 1.0);
        assert_eq!(tm.current_level(), 1);
        tm.apply_delta("Threaten to reveal secrets", -2.0);
        assert_eq!(tm.current_level(), 0);
        assert!(tm.current_trust() < 0.0);
    }

    #[test]
    fn betrayal_larger_than_help() {
        let tm = maren_trust();
        let max_positive = tm
            .positive_actions
            .iter()
            .map(|a| a.delta)
            .fold(0.0_f64, f64::max);
        let max_negative = tm
            .negative_actions
            .iter()
            .map(|a| a.delta.abs())
            .fold(0.0_f64, f64::max);
        assert!(
            max_negative > max_positive,
            "betrayal ({max_negative}) should be larger than help ({max_positive})"
        );
    }

    #[test]
    fn trust_clamped_to_range() {
        let mut tm = maren_trust();
        tm.apply_delta("Massive betrayal", -100.0);
        assert!(tm.current_trust() >= -5.0);
        tm.apply_delta("Miraculous help", 200.0);
        assert!(tm.current_trust() <= 5.0);
    }

    #[test]
    fn history_tracks_all_changes() {
        let mut tm = maren_trust();
        tm.apply_delta("Action 1", 0.5);
        tm.apply_delta("Action 2", 1.0);
        tm.apply_delta("Action 3", -0.5);
        assert_eq!(tm.history_len(), 3);
    }

    #[test]
    fn has_reached_level_check() {
        let mut tm = maren_trust();
        assert!(!tm.has_reached_level(3));
        tm.apply_delta("Big help", 3.5);
        assert!(tm.has_reached_level(3));
    }

    #[test]
    fn current_effect_matches_level() {
        let mut tm = maren_trust();
        assert_eq!(tm.current_effect(), "Polite but professional.");
        tm.apply_delta("Help", 3.0);
        assert_eq!(tm.current_effect(), "Reveals hidden workshop.");
    }
}

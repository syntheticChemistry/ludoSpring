// SPDX-License-Identifier: AGPL-3.0-or-later
//! GOMS model — Goals, Operators, Methods, Selection rules.
//!
//! Predicts expert task completion time by decomposing tasks into
//! primitive operators with empirically derived durations.
//!
//! # References
//!
//! - Card, S.K., Moran, T.P., & Newell, A. (1983). "The Psychology of
//!   Human-Computer Interaction." Lawrence Erlbaum Associates.
//! - Card, Moran, & Newell (1980). "The Keystroke-Level Model for User
//!   Performance Time with Interactive Systems." CACM 23(7).

/// A primitive operator in the Keystroke-Level Model (KLM).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Operator {
    /// Keystroke or button press. ~0.2s (average typist).
    Keystroke,
    /// Point to a target (Fitts's law applies). ~1.1s average.
    Point,
    /// Home hands between devices (keyboard→mouse). ~0.4s.
    Home,
    /// Mental preparation before an action. ~1.35s.
    Mental,
    /// System response wait. Duration is parameter.
    Response(f64),
}

/// Empirical operator times from Card, Moran & Newell (1983), Table 2.
///
/// These are *average expert* times. Novice times are 2-3x longer.
pub mod times {
    /// Keystroke: best typist ~0.08s, average ~0.20s, worst ~0.50s.
    pub const KEYSTROKE_BEST: f64 = 0.08;
    /// Average typist keystroke time (seconds).
    pub const KEYSTROKE_AVG: f64 = 0.20;
    /// Worst-case typist keystroke time (seconds).
    pub const KEYSTROKE_WORST: f64 = 0.50;

    /// Point to target (average Fitts's law acquisition, seconds).
    pub const POINT: f64 = 1.10;

    /// Home hands between devices (seconds).
    pub const HOME: f64 = 0.40;

    /// Mental preparation before action (seconds).
    pub const MENTAL: f64 = 1.35;
}

/// Compute total task time for a sequence of KLM operators.
///
/// Uses average operator times from Card et al. (1983).
#[must_use]
pub fn task_time(operators: &[Operator]) -> f64 {
    operators
        .iter()
        .map(|op| match op {
            Operator::Keystroke => times::KEYSTROKE_AVG,
            Operator::Point => times::POINT,
            Operator::Home => times::HOME,
            Operator::Mental => times::MENTAL,
            Operator::Response(t) => *t,
        })
        .sum()
}

/// Compute task time with custom keystroke speed.
#[must_use]
pub fn task_time_with_keystroke(operators: &[Operator], keystroke_s: f64) -> f64 {
    operators
        .iter()
        .map(|op| match op {
            Operator::Keystroke => keystroke_s,
            Operator::Point => times::POINT,
            Operator::Home => times::HOME,
            Operator::Mental => times::MENTAL,
            Operator::Response(t) => *t,
        })
        .sum()
}

/// Count operators by type.
#[must_use]
pub fn operator_counts(operators: &[Operator]) -> OperatorCounts {
    let mut counts = OperatorCounts::default();
    for op in operators {
        match op {
            Operator::Keystroke => counts.keystrokes += 1,
            Operator::Point => counts.points += 1,
            Operator::Home => counts.homes += 1,
            Operator::Mental => counts.mentals += 1,
            Operator::Response(_) => counts.responses += 1,
        }
    }
    counts
}

/// Breakdown of operator types in a task.
#[derive(Debug, Clone, Default)]
pub struct OperatorCounts {
    /// Number of keystrokes.
    pub keystrokes: usize,
    /// Number of pointing actions.
    pub points: usize,
    /// Number of homing actions.
    pub homes: usize,
    /// Number of mental preparations.
    pub mentals: usize,
    /// Number of system response waits.
    pub responses: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_task_is_zero() {
        assert!((task_time(&[]) - 0.0).abs() < 1e-10);
    }

    #[test]
    fn single_keystroke() {
        assert!((task_time(&[Operator::Keystroke]) - times::KEYSTROKE_AVG).abs() < 1e-10);
    }

    #[test]
    fn point_click_sequence() {
        let ops = [Operator::Mental, Operator::Point, Operator::Keystroke];
        let expected = times::MENTAL + times::POINT + times::KEYSTROKE_AVG;
        assert!((task_time(&ops) - expected).abs() < 1e-10);
    }

    #[test]
    fn response_uses_given_duration() {
        let ops = [Operator::Response(2.5)];
        assert!((task_time(&ops) - 2.5).abs() < 1e-10);
    }

    #[test]
    fn custom_keystroke_speed() {
        let ops = [Operator::Keystroke, Operator::Keystroke];
        let fast = task_time_with_keystroke(&ops, times::KEYSTROKE_BEST);
        let slow = task_time_with_keystroke(&ops, times::KEYSTROKE_WORST);
        assert!(fast < slow);
    }

    #[test]
    fn operator_counts_correct() {
        let ops = [
            Operator::Mental,
            Operator::Point,
            Operator::Keystroke,
            Operator::Keystroke,
            Operator::Home,
        ];
        let c = operator_counts(&ops);
        assert_eq!(c.mentals, 1);
        assert_eq!(c.points, 1);
        assert_eq!(c.keystrokes, 2);
        assert_eq!(c.homes, 1);
        assert_eq!(c.responses, 0);
    }
}

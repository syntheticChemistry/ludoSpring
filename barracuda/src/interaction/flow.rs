// SPDX-License-Identifier: AGPL-3.0-or-later
//! Flow state model — the science of optimal experience.
//!
//! Csikszentmihalyi's flow model predicts engagement based on the balance
//! between challenge (task difficulty) and skill (player ability). Games
//! that maintain the flow channel keep players engaged; deviation causes
//! either boredom (too easy) or anxiety (too hard).
//!
//! # References
//! - Csikszentmihalyi, M. (1990). "Flow: The Psychology of Optimal Experience"
//! - Chen, J. (2007). "Flow in Games." M.S. Thesis, USC.

/// The player's current experience state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowState {
    /// Challenge far below skill. Player disengages.
    Boredom,
    /// Challenge slightly below skill. Comfortable but not gripping.
    Relaxation,
    /// Challenge matches skill. Optimal engagement.
    Flow,
    /// Challenge slightly above skill. Stimulating but stressful.
    Arousal,
    /// Challenge far above skill. Player panics or quits.
    Anxiety,
}

/// Evaluate flow state given normalized challenge and skill (both 0.0–1.0).
///
/// The flow channel is a band around the `challenge == skill` diagonal.
/// `channel_width` controls how wide the band is (default ~0.15).
#[must_use]
pub fn evaluate_flow(challenge: f64, skill: f64, channel_width: f64) -> FlowState {
    let diff = challenge - skill;
    if diff.abs() <= channel_width {
        FlowState::Flow
    } else if diff > channel_width * 2.0 {
        FlowState::Anxiety
    } else if diff > channel_width {
        FlowState::Arousal
    } else if diff < -channel_width * 2.0 {
        FlowState::Boredom
    } else {
        FlowState::Relaxation
    }
}

/// A difficulty curve that maps progress (0.0–1.0) to challenge level.
#[derive(Debug, Clone)]
pub struct DifficultyCurve {
    /// Control points: (progress, challenge) pairs, sorted by progress.
    pub points: Vec<(f64, f64)>,
}

impl DifficultyCurve {
    /// Linear ramp from `start` to `end` challenge.
    #[must_use]
    pub fn linear(start: f64, end: f64) -> Self {
        Self {
            points: vec![(0.0, start), (1.0, end)],
        }
    }

    /// S-curve (sigmoid) difficulty ramp.
    #[must_use]
    pub fn sigmoid(floor: f64, ceiling: f64, steepness: f64) -> Self {
        let n = 20;
        let points = (0..=n)
            .map(|i| {
                let t = i as f64 / n as f64;
                let x = (t - 0.5) * steepness;
                let sigmoid = 1.0 / (1.0 + (-x).exp());
                (t, floor + (ceiling - floor) * sigmoid)
            })
            .collect();
        Self { points }
    }

    /// Sample the difficulty at a given progress value (0.0–1.0).
    #[must_use]
    pub fn sample(&self, progress: f64) -> f64 {
        if self.points.is_empty() {
            return 0.5;
        }
        if progress <= self.points[0].0 {
            return self.points[0].1;
        }
        if progress >= self.points[self.points.len() - 1].0 {
            return self.points[self.points.len() - 1].1;
        }
        for window in self.points.windows(2) {
            let (p0, c0) = window[0];
            let (p1, c1) = window[1];
            if progress >= p0 && progress <= p1 {
                let t = (progress - p0) / (p1 - p0);
                return c0 + t * (c1 - c0);
            }
        }
        0.5
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equal_challenge_skill_is_flow() {
        assert_eq!(evaluate_flow(0.5, 0.5, 0.15), FlowState::Flow);
    }

    #[test]
    fn high_challenge_low_skill_is_anxiety() {
        assert_eq!(evaluate_flow(0.9, 0.1, 0.1), FlowState::Anxiety);
    }

    #[test]
    fn low_challenge_high_skill_is_boredom() {
        assert_eq!(evaluate_flow(0.1, 0.9, 0.1), FlowState::Boredom);
    }

    #[test]
    fn linear_curve_endpoints() {
        let curve = DifficultyCurve::linear(0.1, 0.9);
        assert!((curve.sample(0.0) - 0.1).abs() < 1e-10);
        assert!((curve.sample(1.0) - 0.9).abs() < 1e-10);
        assert!((curve.sample(0.5) - 0.5).abs() < 1e-10);
    }

    #[test]
    fn sigmoid_curve_is_monotonic() {
        let curve = DifficultyCurve::sigmoid(0.1, 0.9, 10.0);
        let mut prev = 0.0;
        for i in 0..=100 {
            let t = i as f64 / 100.0;
            let val = curve.sample(t);
            assert!(val >= prev - 1e-10);
            prev = val;
        }
    }
}

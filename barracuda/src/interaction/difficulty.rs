// SPDX-License-Identifier: AGPL-3.0-or-later
//! Dynamic difficulty adjustment (DDA) — keeping challenge matched to skill.
//!
//! DDA systems observe player performance and adjust challenge in real-time
//! to maintain the flow channel. This module provides algorithms for
//! estimating player skill and adjusting difficulty.
//!
//! # References
//! - Hunicke, R. (2005). "The case for dynamic difficulty adjustment in games."
//!   ACM SIGCHI '05.
//! - Andrade, G. et al. (2006). "Dynamic Game Balancing: An Evaluation of
//!   User Satisfaction." AIIDE '06.

use std::collections::VecDeque;

/// Observed player performance metrics for DDA.
///
/// Uses `VecDeque` for O(1) front removal when the window is full.
#[derive(Debug, Clone, Default)]
pub struct PerformanceWindow {
    /// Recent success rates (0.0 = fail, 1.0 = success).
    pub outcomes: VecDeque<f64>,
    /// Maximum window size.
    pub max_size: usize,
}

impl PerformanceWindow {
    /// Create a new window with the given capacity.
    #[must_use]
    pub fn new(max_size: usize) -> Self {
        Self {
            outcomes: VecDeque::with_capacity(max_size),
            max_size,
        }
    }

    /// Record an outcome (0.0–1.0).
    pub fn record(&mut self, outcome: f64) {
        if self.outcomes.len() >= self.max_size {
            self.outcomes.pop_front();
        }
        self.outcomes.push_back(outcome.clamp(0.0, 1.0));
    }

    /// Estimated skill level (moving average of recent outcomes).
    #[must_use]
    #[expect(
        clippy::cast_precision_loss,
        reason = "window sizes are small (≤100); len fits in f64"
    )]
    pub fn estimated_skill(&self) -> f64 {
        if self.outcomes.is_empty() {
            return 0.5;
        }
        self.outcomes.iter().sum::<f64>() / self.outcomes.len() as f64
    }

    /// Trend: positive = improving, negative = declining.
    #[must_use]
    #[expect(
        clippy::cast_precision_loss,
        reason = "window sizes are small (≤100); len fits in f64"
    )]
    pub fn trend(&self) -> f64 {
        if self.outcomes.len() < 4 {
            return 0.0;
        }
        let mid = self.outcomes.len() / 2;
        let (first, second) = self.outcomes.as_slices();
        let all: Vec<f64> = first.iter().chain(second.iter()).copied().collect();
        let first_half: f64 = all[..mid].iter().sum::<f64>() / mid as f64;
        let second_half: f64 = all[mid..].iter().sum::<f64>() / (all.len() - mid) as f64;
        second_half - first_half
    }
}

/// Suggest a difficulty adjustment based on the performance window.
///
/// Returns a value in `[-1.0, 1.0]`:
/// - Negative = reduce difficulty
/// - Zero = no change
/// - Positive = increase difficulty
///
/// `target_success_rate` is the desired sweet spot (typically 0.6–0.75).
#[must_use]
pub fn suggest_adjustment(window: &PerformanceWindow, target_success_rate: f64) -> f64 {
    let skill = window.estimated_skill();
    let trend = window.trend();

    let deviation = skill - target_success_rate;
    let adjustment = deviation.mul_add(2.0, trend);

    adjustment.clamp(-1.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_window_estimates_neutral() {
        let w = PerformanceWindow::new(10);
        assert!((w.estimated_skill() - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn perfect_player_gets_harder() {
        let mut w = PerformanceWindow::new(10);
        for _ in 0..10 {
            w.record(1.0);
        }
        let adj = suggest_adjustment(&w, 0.7);
        assert!(adj > 0.0);
    }

    #[test]
    fn struggling_player_gets_easier() {
        let mut w = PerformanceWindow::new(10);
        for _ in 0..10 {
            w.record(0.1);
        }
        let adj = suggest_adjustment(&w, 0.7);
        assert!(adj < 0.0);
    }

    #[test]
    fn window_respects_max_size() {
        let mut w = PerformanceWindow::new(5);
        for i in 0..20 {
            w.record(f64::from(i) / 20.0);
        }
        assert_eq!(w.outcomes.len(), 5);
    }
}

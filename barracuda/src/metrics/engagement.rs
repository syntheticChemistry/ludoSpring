// SPDX-License-Identifier: AGPL-3.0-or-later
//! Engagement metrics — quantifying fun.
//!
//! "Fun" is measurable. Session length, voluntary return rate, action density,
//! exploration breadth, and challenge-seeking behavior are all observable
//! signals. This module provides the measurement framework.
//!
//! # References
//! - Lazzaro, N. (2004). "Why We Play Games: Four Keys to More Emotion
//!   Without Story." GDC '04.
//! - Yannakakis, G.N. & Togelius, J. (2018). "Artificial Intelligence and
//!   Games." Springer.

/// A snapshot of player behavior over a time window.
#[derive(Debug, Clone, Default)]
pub struct EngagementSnapshot {
    /// Session duration in seconds.
    pub session_duration_s: f64,
    /// Number of meaningful actions taken.
    pub action_count: u64,
    /// Number of distinct areas/states explored.
    pub exploration_breadth: u32,
    /// Number of voluntary difficulty increases (player chose harder path).
    pub challenge_seeking: u32,
    /// Number of times the player repeated a failed attempt.
    pub retry_count: u32,
    /// Number of voluntary pauses (player stopped to think, not frustrated).
    pub deliberate_pauses: u32,
}

/// Derived engagement metrics.
#[derive(Debug, Clone)]
pub struct EngagementMetrics {
    /// Actions per minute (higher = more engaged, up to a genre-specific ceiling).
    pub actions_per_minute: f64,
    /// Exploration rate (new areas per minute).
    pub exploration_rate: f64,
    /// Challenge appetite (challenge-seeking / total actions).
    pub challenge_appetite: f64,
    /// Persistence (retry rate — player keeps trying after failure).
    pub persistence: f64,
    /// Deliberation rate (pauses per action — indicates strategic thinking).
    pub deliberation: f64,
    /// Composite engagement score (0.0–1.0).
    pub composite: f64,
}

/// Compute engagement metrics from a behavior snapshot.
#[must_use]
pub fn compute_engagement(snap: &EngagementSnapshot) -> EngagementMetrics {
    let minutes = snap.session_duration_s / 60.0;
    let minutes = if minutes < 0.01 { 0.01 } else { minutes };

    let apm = snap.action_count as f64 / minutes;
    let exploration_rate = f64::from(snap.exploration_breadth) / minutes;
    let challenge_appetite = if snap.action_count > 0 {
        f64::from(snap.challenge_seeking) / snap.action_count as f64
    } else {
        0.0
    };
    let persistence = if snap.action_count > 0 {
        f64::from(snap.retry_count) / snap.action_count as f64
    } else {
        0.0
    };
    let deliberation = if snap.action_count > 0 {
        f64::from(snap.deliberate_pauses) / snap.action_count as f64
    } else {
        0.0
    };

    // Composite: weighted combination, normalized to 0–1.
    // Weights reflect research on engagement indicators.
    let raw = (apm / 60.0).min(1.0) * 0.2
        + (exploration_rate / 5.0).min(1.0) * 0.2
        + challenge_appetite.min(1.0) * 0.2
        + persistence.min(1.0) * 0.2
        + deliberation.min(1.0) * 0.2;

    EngagementMetrics {
        actions_per_minute: apm,
        exploration_rate,
        challenge_appetite,
        persistence,
        deliberation,
        composite: raw.clamp(0.0, 1.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn active_player_scores_high() {
        let snap = EngagementSnapshot {
            session_duration_s: 300.0,
            action_count: 200,
            exploration_breadth: 15,
            challenge_seeking: 10,
            retry_count: 20,
            deliberate_pauses: 15,
        };
        let metrics = compute_engagement(&snap);
        assert!(metrics.composite > 0.2);
        assert!(metrics.actions_per_minute > 30.0);
    }

    #[test]
    fn idle_player_scores_low() {
        let snap = EngagementSnapshot {
            session_duration_s: 300.0,
            action_count: 2,
            exploration_breadth: 1,
            challenge_seeking: 0,
            retry_count: 0,
            deliberate_pauses: 0,
        };
        let metrics = compute_engagement(&snap);
        assert!(metrics.composite < 0.1);
    }

    #[test]
    fn zero_duration_doesnt_panic() {
        let snap = EngagementSnapshot::default();
        let metrics = compute_engagement(&snap);
        assert!(metrics.composite.is_finite());
    }
}

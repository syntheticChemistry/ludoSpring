// SPDX-License-Identifier: AGPL-3.0-or-later
//! Four Keys to Fun — Lazzaro's emotion taxonomy for games.
//!
//! Nicole Lazzaro's (2004) framework classifies player emotions into four
//! categories, each producing distinct engagement patterns. This module
//! classifies game scenarios and predicts which fun type dominates, enabling
//! balanced experience design.
//!
//! # The Four Keys
//!
//! | Key | Emotion | Example | Signal |
//! |-----|---------|---------|--------|
//! | Hard Fun | Fiero (triumph) | Boss fight, puzzle solve | High challenge-seeking, high retry |
//! | Easy Fun | Curiosity | Exploration, sandbox | High exploration, low challenge |
//! | People Fun | Amusement | Co-op, social play | High social actions, communication |
//! | Serious Fun | Relaxation/excitement | Rhythm, collection | Steady pacing, completion focus |
//!
//! # References
//!
//! - Lazzaro, N. (2004). "Why We Play Games: Four Keys to More Emotion
//!   Without Story." GDC '04.
//! - Lazzaro, N. (2008). "The Four Fun Keys." In "Game Usability."

/// The four types of fun (Lazzaro 2004).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FunKey {
    /// Hard Fun: challenge, mastery, triumph (fiero).
    Hard,
    /// Easy Fun: curiosity, wonder, exploration.
    Easy,
    /// People Fun: social interaction, amusement, cooperation.
    People,
    /// Serious Fun: relaxation, excitement, collection, rhythm.
    Serious,
}

impl FunKey {
    /// String representation.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Hard => "hard_fun",
            Self::Easy => "easy_fun",
            Self::People => "people_fun",
            Self::Serious => "serious_fun",
        }
    }
}

impl std::fmt::Display for FunKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Behavioral signals used to classify fun type.
#[derive(Debug, Clone, Default)]
pub struct FunSignals {
    /// Challenge-seeking intensity (0.0–1.0).
    pub challenge: f64,
    /// Exploration/curiosity intensity (0.0–1.0).
    pub exploration: f64,
    /// Social interaction intensity (0.0–1.0).
    pub social: f64,
    /// Completion/collection intensity (0.0–1.0).
    pub completion: f64,
    /// Retry/persistence rate (0.0–1.0).
    pub retry_rate: f64,
}

/// Classification result with confidence scores.
#[derive(Debug, Clone)]
pub struct FunClassification {
    /// Dominant fun type.
    pub dominant: FunKey,
    /// Confidence score for each fun type (0.0–1.0).
    pub scores: FunScores,
}

/// Per-key confidence scores.
#[derive(Debug, Clone)]
pub struct FunScores {
    /// Hard fun confidence.
    pub hard: f64,
    /// Easy fun confidence.
    pub easy: f64,
    /// People fun confidence.
    pub people: f64,
    /// Serious fun confidence.
    pub serious: f64,
}

/// Classify a scenario's dominant fun type from behavioral signals.
///
/// Scoring follows Lazzaro's signal-to-emotion mapping:
/// - Hard Fun = challenge × 0.6 + `retry_rate` × 0.4
/// - Easy Fun = exploration × 0.8 + (1 − challenge) × 0.2
/// - People Fun = social × 1.0
/// - Serious Fun = completion × 0.7 + (1 − social) × 0.15 + (1 − challenge) × 0.15
#[must_use]
pub fn classify_fun(signals: &FunSignals) -> FunClassification {
    let hard = signals.challenge.mul_add(0.6, signals.retry_rate * 0.4);
    let easy = signals
        .exploration
        .mul_add(0.8, (1.0 - signals.challenge) * 0.2);
    let people = signals.social;
    let serious = signals.completion.mul_add(
        0.7,
        (1.0 - signals.social).mul_add(0.15, (1.0 - signals.challenge) * 0.15),
    );

    let scores = FunScores {
        hard,
        easy,
        people,
        serious,
    };

    let dominant = if hard >= easy && hard >= people && hard >= serious {
        FunKey::Hard
    } else if easy >= people && easy >= serious {
        FunKey::Easy
    } else if people >= serious {
        FunKey::People
    } else {
        FunKey::Serious
    };

    FunClassification { dominant, scores }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn high_challenge_is_hard_fun() {
        let c = classify_fun(&FunSignals {
            challenge: 0.9,
            retry_rate: 0.8,
            exploration: 0.1,
            social: 0.0,
            completion: 0.1,
        });
        assert_eq!(c.dominant, FunKey::Hard);
    }

    #[test]
    fn high_exploration_is_easy_fun() {
        let c = classify_fun(&FunSignals {
            challenge: 0.1,
            exploration: 0.9,
            social: 0.0,
            completion: 0.1,
            retry_rate: 0.0,
        });
        assert_eq!(c.dominant, FunKey::Easy);
    }

    #[test]
    fn high_social_is_people_fun() {
        let c = classify_fun(&FunSignals {
            social: 0.9,
            challenge: 0.1,
            exploration: 0.1,
            completion: 0.1,
            retry_rate: 0.0,
        });
        assert_eq!(c.dominant, FunKey::People);
    }

    #[test]
    fn high_completion_is_serious_fun() {
        let c = classify_fun(&FunSignals {
            completion: 0.9,
            challenge: 0.0,
            exploration: 0.1,
            social: 0.0,
            retry_rate: 0.0,
        });
        assert_eq!(c.dominant, FunKey::Serious);
    }

    #[test]
    fn scores_are_bounded() {
        let c = classify_fun(&FunSignals {
            challenge: 1.0,
            exploration: 1.0,
            social: 1.0,
            completion: 1.0,
            retry_rate: 1.0,
        });
        assert!(c.scores.hard <= 1.0);
        assert!(c.scores.easy <= 1.0);
        assert!(c.scores.people <= 1.0);
        assert!(c.scores.serious <= 1.0);
    }
}

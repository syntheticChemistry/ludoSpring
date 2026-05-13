// SPDX-License-Identifier: AGPL-3.0-or-later
//! BM-004: Matchmaking — skill-based lobby formation.
//!
//! Matchmaking is the gatekeeper of multiplayer: pair players whose skill
//! levels produce close, engaging matches. The standard model is Elo
//! (Arpad Elo, 1960 — originally for chess), extended by Glicko
//! (Glickman, 1995) and TrueSkill (Herbrich et al., 2006).
//!
//! This module implements:
//! - Elo rating update (classic K-factor model)
//! - Skill-based lobby formation (greedy nearest-neighbor)
//! - Match quality prediction (expected score differential)
//! - Lobby balance scoring (skill variance within a lobby)
//!
//! # BM-004 Benchmark Target
//!
//! Matchmaking throughput: form N lobbies from M players per second.
//! The benchmark validates that lobby formation scales sub-quadratically
//! (greedy sort is O(M log M), not O(M²) brute force).
//!
//! # References
//!
//! - Elo, A. (1978). "The Rating of Chessplayers, Past and Present."
//! - Glickman, M. (1995). "The Glicko System." Boston University.
//! - Herbrich, R., Minka, T., & Graepel, T. (2006). "TrueSkill."
//!   Microsoft Research.

/// A player with a skill rating.
#[derive(Debug, Clone)]
pub struct RatedPlayer {
    /// Unique identifier.
    pub id: u64,
    /// Current Elo rating.
    pub rating: f64,
    /// Number of games played (used for K-factor adjustment).
    pub games_played: u32,
}

/// Elo system configuration.
#[derive(Debug, Clone)]
pub struct EloConfig {
    /// K-factor for new players (< 30 games).
    pub k_new: f64,
    /// K-factor for established players.
    pub k_established: f64,
    /// Games threshold for switching from new to established.
    pub new_threshold: u32,
    /// Starting rating for new players.
    pub default_rating: f64,
}

impl Default for EloConfig {
    fn default() -> Self {
        Self {
            k_new: 40.0,
            k_established: 20.0,
            new_threshold: 30,
            default_rating: 1200.0,
        }
    }
}

impl EloConfig {
    /// K-factor for a player based on their experience.
    #[must_use]
    pub const fn k_factor(&self, games_played: u32) -> f64 {
        if games_played < self.new_threshold {
            self.k_new
        } else {
            self.k_established
        }
    }
}

/// Expected score for player A against player B (Elo formula).
///
/// Returns a value in [0, 1] representing the probability that A wins.
/// 0.5 means equally matched.
#[must_use]
pub fn expected_score(rating_a: f64, rating_b: f64) -> f64 {
    1.0 / (1.0 + 10.0_f64.powf((rating_b - rating_a) / 400.0))
}

/// Update ratings after a match. Returns (new_rating_a, new_rating_b).
///
/// `score_a` is 1.0 for a win, 0.5 for a draw, 0.0 for a loss.
#[must_use]
pub fn elo_update(
    config: &EloConfig,
    a: &RatedPlayer,
    b: &RatedPlayer,
    score_a: f64,
) -> (f64, f64) {
    let expected_a = expected_score(a.rating, b.rating);
    let k_a = config.k_factor(a.games_played);
    let k_b = config.k_factor(b.games_played);

    let new_a = k_a.mul_add(score_a - expected_a, a.rating);
    let new_b = k_b.mul_add((1.0 - score_a) - (1.0 - expected_a), b.rating);

    (new_a, new_b)
}

/// A formed lobby of players.
#[derive(Debug, Clone)]
pub struct Lobby {
    /// Players in this lobby.
    pub players: Vec<RatedPlayer>,
}

impl Lobby {
    /// Mean rating of players in the lobby.
    #[must_use]
    pub fn mean_rating(&self) -> f64 {
        if self.players.is_empty() {
            return 0.0;
        }
        let sum: f64 = self.players.iter().map(|p| p.rating).sum();
        #[allow(
            clippy::cast_precision_loss,
            reason = "lobby sizes are small (< 100), usize fits in f64 exactly"
        )]
        let count = self.players.len() as f64;
        sum / count
    }

    /// Skill variance within the lobby (lower = more balanced).
    #[must_use]
    pub fn skill_variance(&self) -> f64 {
        if self.players.len() < 2 {
            return 0.0;
        }
        let mean = self.mean_rating();
        let sum_sq: f64 = self.players.iter().map(|p| (p.rating - mean).powi(2)).sum();
        #[allow(
            clippy::cast_precision_loss,
            reason = "lobby sizes are small (< 100), usize fits in f64 exactly"
        )]
        let denom = (self.players.len() - 1) as f64;
        sum_sq / denom
    }

    /// Match quality score [0, 1]. Higher = more balanced lobby.
    ///
    /// Based on the ratio of actual variance to maximum possible variance
    /// for the rating range. 1.0 means all players are identically rated.
    #[must_use]
    pub fn balance_score(&self) -> f64 {
        if self.players.len() < 2 {
            return 1.0;
        }
        let variance = self.skill_variance();
        let max_variance = 200.0_f64.powi(2);
        (1.0 - (variance / max_variance).min(1.0)).max(0.0)
    }
}

/// Form lobbies from a pool of players using greedy nearest-neighbor.
///
/// Sorts players by rating and groups adjacent players into lobbies of
/// the requested size. O(N log N) for the sort, O(N) for grouping.
#[must_use]
pub fn form_lobbies(mut players: Vec<RatedPlayer>, lobby_size: usize) -> Vec<Lobby> {
    if lobby_size == 0 || players.is_empty() {
        return Vec::new();
    }

    players.sort_by(|a, b| {
        a.rating
            .partial_cmp(&b.rating)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    players
        .chunks(lobby_size)
        .filter(|chunk| chunk.len() == lobby_size)
        .map(|chunk| Lobby {
            players: chunk.to_vec(),
        })
        .collect()
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::cast_precision_loss,
    reason = "test helpers use small index casts and unwrap for clarity"
)]
mod tests {
    use super::*;

    fn test_players(n: usize) -> Vec<RatedPlayer> {
        (0..n)
            .map(|i| RatedPlayer {
                id: i as u64,
                rating: (i as f64).mul_add(50.0, 800.0),
                games_played: 10,
            })
            .collect()
    }

    #[test]
    fn expected_score_equal_ratings() {
        let e = expected_score(1200.0, 1200.0);
        assert!((e - 0.5).abs() < 0.001);
    }

    #[test]
    fn expected_score_higher_wins_more() {
        let e = expected_score(1400.0, 1200.0);
        assert!(e > 0.5);
        assert!(e < 1.0);
    }

    #[test]
    fn expected_score_symmetric() {
        let e_a = expected_score(1400.0, 1200.0);
        let e_b = expected_score(1200.0, 1400.0);
        assert!((e_a + e_b - 1.0).abs() < 0.001);
    }

    #[test]
    fn elo_update_winner_gains() {
        let config = EloConfig::default();
        let a = RatedPlayer {
            id: 1,
            rating: 1200.0,
            games_played: 50,
        };
        let b = RatedPlayer {
            id: 2,
            rating: 1200.0,
            games_played: 50,
        };
        let (new_a, new_b) = elo_update(&config, &a, &b, 1.0);
        assert!(new_a > 1200.0);
        assert!(new_b < 1200.0);
    }

    #[test]
    fn elo_update_draw_no_change_for_equals() {
        let config = EloConfig::default();
        let a = RatedPlayer {
            id: 1,
            rating: 1200.0,
            games_played: 50,
        };
        let b = RatedPlayer {
            id: 2,
            rating: 1200.0,
            games_played: 50,
        };
        let (new_a, new_b) = elo_update(&config, &a, &b, 0.5);
        assert!((new_a - 1200.0).abs() < 0.01);
        assert!((new_b - 1200.0).abs() < 0.01);
    }

    #[test]
    fn elo_update_conserves_rating_sum() {
        let config = EloConfig::default();
        let a = RatedPlayer {
            id: 1,
            rating: 1300.0,
            games_played: 50,
        };
        let b = RatedPlayer {
            id: 2,
            rating: 1100.0,
            games_played: 50,
        };
        let (new_a, new_b) = elo_update(&config, &a, &b, 1.0);
        assert!(
            ((new_a + new_b) - (a.rating + b.rating)).abs() < 0.01,
            "Elo should be zero-sum for equal K"
        );
    }

    #[test]
    fn form_lobbies_correct_count() {
        let players = test_players(20);
        let lobbies = form_lobbies(players, 4);
        assert_eq!(lobbies.len(), 5);
    }

    #[test]
    fn form_lobbies_drops_remainder() {
        let players = test_players(22);
        let lobbies = form_lobbies(players, 4);
        assert_eq!(lobbies.len(), 5);
    }

    #[test]
    fn form_lobbies_empty_input() {
        let lobbies = form_lobbies(Vec::new(), 4);
        assert!(lobbies.is_empty());
    }

    #[test]
    fn lobby_balance_identical_ratings() {
        let lobby = Lobby {
            players: vec![
                RatedPlayer {
                    id: 1,
                    rating: 1200.0,
                    games_played: 10,
                },
                RatedPlayer {
                    id: 2,
                    rating: 1200.0,
                    games_played: 10,
                },
                RatedPlayer {
                    id: 3,
                    rating: 1200.0,
                    games_played: 10,
                },
                RatedPlayer {
                    id: 4,
                    rating: 1200.0,
                    games_played: 10,
                },
            ],
        };
        assert!((lobby.balance_score() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn lobby_sorted_by_rating_has_good_balance() {
        let players = test_players(20);
        let lobbies = form_lobbies(players, 4);
        for lobby in &lobbies {
            assert!(
                lobby.balance_score() > 0.5,
                "sorted lobbies should be balanced"
            );
        }
    }

    #[test]
    fn skill_variance_single_player() {
        let lobby = Lobby {
            players: vec![RatedPlayer {
                id: 1,
                rating: 1200.0,
                games_played: 10,
            }],
        };
        assert!(lobby.skill_variance().abs() < f64::EPSILON);
    }

    #[test]
    fn k_factor_new_vs_established() {
        let config = EloConfig::default();
        assert!(config.k_factor(5) > config.k_factor(50));
    }
}

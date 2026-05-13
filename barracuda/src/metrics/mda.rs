// SPDX-License-Identifier: AGPL-3.0-or-later
//! MDA Framework — Mechanics, Dynamics, Aesthetics decomposition.
//!
//! Hunicke, LeBlanc, and Zubek (2004) formalize game design into three
//! coupled layers. Designers work Mechanics → Dynamics → Aesthetics
//! (forward), players experience Aesthetics → Dynamics → Mechanics
//! (reverse). This duality means the same mechanic produces different
//! aesthetics depending on the dynamics it generates.
//!
//! This module provides:
//! - Enumerated aesthetic taxonomy (8 canonical types)
//! - Mechanic → dynamic → aesthetic classification pipeline
//! - Aesthetic balance scoring (evenness of coverage)
//! - Designer vs player perspective analysis
//!
//! # References
//!
//! - Hunicke, R., LeBlanc, M., & Zubek, R. (2004). "MDA: A Formal
//!   Approach to Game Design and Game Research." AAAI Workshop on
//!   Challenges in Game AI.
//! - LeBlanc, M. (2006). "Tools for Creating Dramatic Game Dynamics."
//!   In "The Game Design Reader," MIT Press.

/// The eight canonical aesthetic types (Hunicke et al. 2004, Table 1).
///
/// Games rarely target a single aesthetic — most blend 2–4, with one or
/// two dominant. The taxonomy gives designers a shared vocabulary for
/// what a game is *trying to make the player feel*.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Aesthetic {
    /// Sensation: game as sense-pleasure (visuals, audio, haptics).
    Sensation,
    /// Fantasy: game as make-believe (role-playing, world-building).
    Fantasy,
    /// Narrative: game as drama (unfolding story, authored or emergent).
    Narrative,
    /// Challenge: game as obstacle course (mastery, competition).
    Challenge,
    /// Fellowship: game as social framework (cooperation, community).
    Fellowship,
    /// Discovery: game as uncharted territory (exploration, secrets).
    Discovery,
    /// Expression: game as self-discovery (creativity, customization).
    Expression,
    /// Submission: game as pastime (relaxation, repetition, comfort).
    Submission,
}

impl Aesthetic {
    /// All eight aesthetics in canonical order (Hunicke et al. Table 1).
    pub const ALL: [Self; 8] = [
        Self::Sensation,
        Self::Fantasy,
        Self::Narrative,
        Self::Challenge,
        Self::Fellowship,
        Self::Discovery,
        Self::Expression,
        Self::Submission,
    ];

    /// Short label for display and serialization.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Sensation => "sensation",
            Self::Fantasy => "fantasy",
            Self::Narrative => "narrative",
            Self::Challenge => "challenge",
            Self::Fellowship => "fellowship",
            Self::Discovery => "discovery",
            Self::Expression => "expression",
            Self::Submission => "submission",
        }
    }
}

/// A mechanic: a concrete rule or system within the game.
///
/// Mechanics are the atoms of game design — point systems, turn order,
/// resource management, physics. They produce dynamics when players
/// interact with them.
#[derive(Debug, Clone)]
pub struct Mechanic {
    /// Short identifier (e.g. "resource_management", "turn_timer").
    pub id: String,
    /// Which aesthetics this mechanic *can* generate, weighted [0, 1].
    /// The same mechanic may contribute to multiple aesthetics: a
    /// leaderboard feeds both Challenge and Fellowship.
    pub aesthetic_weights: Vec<(Aesthetic, f64)>,
}

/// A dynamic: emergent runtime behavior from mechanics + player input.
///
/// Dynamics are observable at runtime — player strategies, emergent
/// alliances, difficulty curves, resource hoarding. They are the
/// *behavior* layer between rules and feelings.
#[derive(Debug, Clone)]
pub struct Dynamic {
    /// Short identifier (e.g. "turtling", "rush_strategy", "exploration_loop").
    pub id: String,
    /// Which mechanics contribute to this dynamic.
    pub source_mechanics: Vec<String>,
    /// Which aesthetic this dynamic primarily produces.
    pub primary_aesthetic: Aesthetic,
    /// Observed intensity [0, 1] — how strongly this dynamic manifests.
    pub intensity: f64,
}

/// Full MDA decomposition of a game or game session.
#[derive(Debug, Clone)]
pub struct MdaProfile {
    /// Game rules and systems (designer-facing atoms of design).
    pub mechanics: Vec<Mechanic>,
    /// Emergent runtime behaviors (observable player patterns).
    pub dynamics: Vec<Dynamic>,
}

/// Aesthetic distribution — normalized weights across all 8 aesthetics.
#[derive(Debug, Clone)]
pub struct AestheticDistribution {
    weights: [f64; 8],
}

impl AestheticDistribution {
    /// Build from raw aesthetic contributions, normalizing to sum = 1.
    #[must_use]
    pub fn from_weights(raw: &[(Aesthetic, f64)]) -> Self {
        let mut weights = [0.0_f64; 8];
        for &(aesthetic, w) in raw {
            weights[aesthetic as usize] += w;
        }
        let total: f64 = weights.iter().sum();
        if total > 0.0 {
            for w in &mut weights {
                *w /= total;
            }
        }
        Self { weights }
    }

    /// Weight for a specific aesthetic [0, 1].
    #[must_use]
    pub const fn weight(&self, aesthetic: Aesthetic) -> f64 {
        self.weights[aesthetic as usize]
    }

    /// Shannon entropy of the distribution (bits).
    ///
    /// Maximum entropy (3.0 bits) means all 8 aesthetics equally present.
    /// Low entropy means the game concentrates on few aesthetics.
    /// Neither extreme is inherently good — a puzzle game *should* have
    /// low entropy (Challenge-dominant).
    #[must_use]
    pub fn entropy(&self) -> f64 {
        self.weights
            .iter()
            .filter(|&&w| w > 0.0)
            .map(|&w| -w * w.log2())
            .sum()
    }

    /// Maximum possible entropy for 8 categories.
    pub const MAX_ENTROPY: f64 = 3.0; // log2(8)

    /// Evenness: entropy / max_entropy. 1.0 = perfectly even, 0.0 = single aesthetic.
    #[must_use]
    pub fn evenness(&self) -> f64 {
        self.entropy() / Self::MAX_ENTROPY
    }

    /// Dominant aesthetic (highest weight). Ties broken by canonical order.
    #[must_use]
    pub fn dominant(&self) -> Aesthetic {
        let idx = self
            .weights
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map_or(0, |(i, _)| i);
        Aesthetic::ALL[idx]
    }

    /// Top-N aesthetics by weight.
    #[must_use]
    pub fn top_n(&self, n: usize) -> Vec<(Aesthetic, f64)> {
        let mut indexed: Vec<(usize, f64)> = self.weights.iter().copied().enumerate().collect();
        indexed.sort_by(|(_, a), (_, b)| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
        indexed
            .into_iter()
            .take(n)
            .filter(|&(_, w)| w > 0.0)
            .map(|(i, w)| (Aesthetic::ALL[i], w))
            .collect()
    }
}

impl MdaProfile {
    /// Compute the aesthetic distribution from the designer perspective
    /// (Mechanics → Dynamics → Aesthetics — forward pass).
    ///
    /// Aggregates mechanic aesthetic weights, then overlays dynamic
    /// intensities for the primary aesthetics they produce.
    #[must_use]
    pub fn designer_aesthetics(&self) -> AestheticDistribution {
        let mut raw: Vec<(Aesthetic, f64)> = Vec::new();

        for mechanic in &self.mechanics {
            for &(aesthetic, weight) in &mechanic.aesthetic_weights {
                raw.push((aesthetic, weight));
            }
        }

        for dynamic in &self.dynamics {
            raw.push((dynamic.primary_aesthetic, dynamic.intensity));
        }

        AestheticDistribution::from_weights(&raw)
    }

    /// Compute the aesthetic distribution from the player perspective
    /// (Aesthetics → Dynamics → Mechanics — reverse pass).
    ///
    /// Only dynamics that actually manifest (intensity > 0) contribute.
    /// This reflects what the player *actually experiences* vs what the
    /// designer *intended*.
    #[must_use]
    pub fn player_aesthetics(&self) -> AestheticDistribution {
        let active_dynamics: Vec<&Dynamic> =
            self.dynamics.iter().filter(|d| d.intensity > 0.0).collect();

        let raw: Vec<(Aesthetic, f64)> = active_dynamics
            .iter()
            .map(|d| (d.primary_aesthetic, d.intensity))
            .collect();

        AestheticDistribution::from_weights(&raw)
    }

    /// Alignment score between designer intent and player experience.
    ///
    /// Computed as 1 - Jensen-Shannon divergence between the two
    /// distributions. 1.0 = perfect alignment, 0.0 = complete mismatch.
    #[must_use]
    pub fn alignment(&self) -> f64 {
        let designer = self.designer_aesthetics();
        let player = self.player_aesthetics();

        let midpoint: Vec<f64> = designer
            .weights
            .iter()
            .zip(player.weights.iter())
            .map(|(d, p)| f64::midpoint(*d, *p))
            .collect();

        let div_designer = kl_divergence(&designer.weights, &midpoint);
        let div_player = kl_divergence(&player.weights, &midpoint);
        let jsd = f64::midpoint(div_designer, div_player);

        1.0 - jsd.sqrt()
    }
}

/// KL divergence D_KL(P || Q) in bits.
fn kl_divergence(p: &[f64], q: &[f64]) -> f64 {
    p.iter()
        .zip(q.iter())
        .filter(|(pi, qi)| **pi > 0.0 && **qi > 0.0)
        .map(|(pi, qi)| pi * (pi / qi).log2())
        .sum()
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::float_cmp,
    reason = "test assertions use unwrap and exact float comparison for clarity"
)]
mod tests {
    use super::*;

    fn tetris_profile() -> MdaProfile {
        MdaProfile {
            mechanics: vec![
                Mechanic {
                    id: "falling_pieces".into(),
                    aesthetic_weights: vec![
                        (Aesthetic::Challenge, 0.8),
                        (Aesthetic::Sensation, 0.2),
                    ],
                },
                Mechanic {
                    id: "line_clearing".into(),
                    aesthetic_weights: vec![
                        (Aesthetic::Challenge, 0.6),
                        (Aesthetic::Submission, 0.4),
                    ],
                },
                Mechanic {
                    id: "speed_increase".into(),
                    aesthetic_weights: vec![
                        (Aesthetic::Challenge, 0.9),
                        (Aesthetic::Sensation, 0.1),
                    ],
                },
            ],
            dynamics: vec![
                Dynamic {
                    id: "pattern_recognition".into(),
                    source_mechanics: vec!["falling_pieces".into(), "line_clearing".into()],
                    primary_aesthetic: Aesthetic::Challenge,
                    intensity: 0.9,
                },
                Dynamic {
                    id: "flow_state".into(),
                    source_mechanics: vec!["speed_increase".into()],
                    primary_aesthetic: Aesthetic::Submission,
                    intensity: 0.7,
                },
            ],
        }
    }

    fn sandbox_profile() -> MdaProfile {
        MdaProfile {
            mechanics: vec![
                Mechanic {
                    id: "open_world".into(),
                    aesthetic_weights: vec![
                        (Aesthetic::Discovery, 0.7),
                        (Aesthetic::Expression, 0.3),
                    ],
                },
                Mechanic {
                    id: "crafting".into(),
                    aesthetic_weights: vec![
                        (Aesthetic::Expression, 0.6),
                        (Aesthetic::Discovery, 0.2),
                        (Aesthetic::Submission, 0.2),
                    ],
                },
                Mechanic {
                    id: "building".into(),
                    aesthetic_weights: vec![
                        (Aesthetic::Expression, 0.8),
                        (Aesthetic::Fantasy, 0.2),
                    ],
                },
            ],
            dynamics: vec![
                Dynamic {
                    id: "exploration_loop".into(),
                    source_mechanics: vec!["open_world".into()],
                    primary_aesthetic: Aesthetic::Discovery,
                    intensity: 0.8,
                },
                Dynamic {
                    id: "creative_building".into(),
                    source_mechanics: vec!["crafting".into(), "building".into()],
                    primary_aesthetic: Aesthetic::Expression,
                    intensity: 0.9,
                },
            ],
        }
    }

    #[test]
    fn tetris_is_challenge_dominant() {
        let profile = tetris_profile();
        let dist = profile.designer_aesthetics();
        assert_eq!(dist.dominant(), Aesthetic::Challenge);
        assert!(dist.weight(Aesthetic::Challenge) > 0.4);
    }

    #[test]
    fn sandbox_is_expression_discovery() {
        let profile = sandbox_profile();
        let top = profile.designer_aesthetics().top_n(2);
        let labels: Vec<&str> = top.iter().map(|(a, _)| a.label()).collect();
        assert!(labels.contains(&"expression") || labels.contains(&"discovery"));
    }

    #[test]
    fn entropy_bounds() {
        let profile = tetris_profile();
        let e = profile.designer_aesthetics().entropy();
        assert!(e >= 0.0);
        assert!(e <= AestheticDistribution::MAX_ENTROPY + 0.001);
    }

    #[test]
    fn tetris_low_evenness() {
        let profile = tetris_profile();
        let evenness = profile.designer_aesthetics().evenness();
        assert!(
            evenness < 0.7,
            "Tetris should be concentrated, got {evenness}"
        );
    }

    #[test]
    fn sandbox_higher_evenness_than_tetris() {
        let tetris_e = tetris_profile().designer_aesthetics().evenness();
        let sandbox_e = sandbox_profile().designer_aesthetics().evenness();
        assert!(
            sandbox_e > tetris_e,
            "sandbox ({sandbox_e}) should be more even than tetris ({tetris_e})"
        );
    }

    #[test]
    fn player_perspective_uses_active_dynamics() {
        let profile = tetris_profile();
        let player = profile.player_aesthetics();
        assert!(player.weight(Aesthetic::Challenge) > 0.0);
        assert!(player.weight(Aesthetic::Submission) > 0.0);
    }

    #[test]
    fn alignment_is_bounded() {
        let profile = tetris_profile();
        let a = profile.alignment();
        assert!((0.0..=1.0).contains(&a), "alignment out of bounds: {a}");
    }

    #[test]
    fn high_alignment_when_dynamics_match_mechanics() {
        let profile = tetris_profile();
        let a = profile.alignment();
        assert!(a > 0.5, "tetris should have reasonable alignment: {a}");
    }

    #[test]
    fn all_aesthetics_have_labels() {
        for a in Aesthetic::ALL {
            assert!(!a.label().is_empty());
        }
    }

    #[test]
    fn empty_profile_has_zero_entropy() {
        let profile = MdaProfile {
            mechanics: vec![],
            dynamics: vec![],
        };
        let e = profile.designer_aesthetics().entropy();
        assert!(e.abs() < f64::EPSILON || e.is_nan());
    }

    #[test]
    fn uniform_distribution_max_entropy() {
        let raw: Vec<(Aesthetic, f64)> = Aesthetic::ALL.iter().map(|&a| (a, 1.0)).collect();
        let dist = AestheticDistribution::from_weights(&raw);
        let e = dist.entropy();
        assert!(
            (e - AestheticDistribution::MAX_ENTROPY).abs() < 0.01,
            "expected ~3.0 bits, got {e}"
        );
        assert!(
            (dist.evenness() - 1.0).abs() < 0.01,
            "expected ~1.0 evenness, got {}",
            dist.evenness()
        );
    }

    #[test]
    fn single_aesthetic_zero_entropy() {
        let raw = vec![(Aesthetic::Challenge, 1.0)];
        let dist = AestheticDistribution::from_weights(&raw);
        assert!(
            dist.entropy().abs() < f64::EPSILON,
            "single aesthetic should have zero entropy"
        );
    }

    #[test]
    fn top_n_respects_limit() {
        let profile = tetris_profile();
        let top = profile.designer_aesthetics().top_n(1);
        assert_eq!(top.len(), 1);
        assert_eq!(top[0].0, Aesthetic::Challenge);
    }
}

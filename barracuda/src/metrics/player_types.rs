// SPDX-License-Identifier: AGPL-3.0-or-later
//! Bartle Player Types (1996) — taxonomy of player motivation.
//!
//! Richard Bartle's model classifies players along two orthogonal axes:
//! - **Action axis**: Acting (unilateral) ↔ Interacting (reciprocal)
//! - **Focus axis**: Players (social) ↔ World (environment)
//!
//! This yields four archetypes:
//! - **Achiever** (Acting × World): goal completion, progression, mastery
//! - **Explorer** (Interacting × World): discovery, mapping, understanding
//! - **Socializer** (Interacting × Players): relationships, communication, community
//! - **Killer** (Acting × Players): competition, dominance, PvP
//!
//! # Extended Model (Bartle 2003)
//!
//! The 2003 extension adds an implicit/explicit dimension, yielding 8 types.
//! We implement both the classic 4-type and extended 8-type models.
//!
//! # Application
//!
//! Used for NPC personality modeling (Paper 18/RPGPT), engagement prediction,
//! and content recommendation. A player's type distribution predicts which
//! game mechanics will produce flow states (Csikszentmihalyi 1990).
//!
//! # Reference
//!
//! Bartle, R. (1996). "Hearts, Clubs, Diamonds, Spades: Players Who Suit MUDs."
//! Journal of MUD Research, 1(1).

/// The four classic Bartle player types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlayerType {
    /// Acting × World: seeks progression, points, status, completion.
    Achiever,
    /// Interacting × World: seeks discovery, mapping, hidden knowledge.
    Explorer,
    /// Interacting × Players: seeks relationships, conversation, cooperation.
    Socializer,
    /// Acting × Players: seeks competition, dominance, imposing on others.
    Killer,
}

/// Extended 8-type model (Bartle 2003) adding implicit/explicit dimension.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExtendedType {
    /// Explicit Achiever: planners — optimize paths to goals.
    Planner,
    /// Implicit Achiever: opportunists — seize advantages as they appear.
    Opportunist,
    /// Explicit Explorer: scientists — systematic hypothesis testing.
    Scientist,
    /// Implicit Explorer: hackers — push boundaries, exploit mechanics.
    Hacker,
    /// Explicit Socializer: networkers — build social structures deliberately.
    Networker,
    /// Implicit Socializer: friends — form deep bonds naturally.
    Friend,
    /// Explicit Killer: politicians — manipulate social dynamics for power.
    Politician,
    /// Implicit Killer: griefers — derive enjoyment from others' frustration.
    Griefer,
}

/// A player's motivation profile as a distribution across types.
///
/// Normalized so all weights sum to 1.0. Represents the *degree* to which
/// each archetype describes a player's motivations — most players are blends.
#[derive(Debug, Clone)]
pub struct BartleProfile {
    /// Weight for Achiever archetype (0.0–1.0).
    pub achiever: f64,
    /// Weight for Explorer archetype (0.0–1.0).
    pub explorer: f64,
    /// Weight for Socializer archetype (0.0–1.0).
    pub socializer: f64,
    /// Weight for Killer archetype (0.0–1.0).
    pub killer: f64,
}

impl BartleProfile {
    /// Create a profile from raw scores, auto-normalizing to sum = 1.0.
    #[must_use]
    pub fn new(achiever: f64, explorer: f64, socializer: f64, killer: f64) -> Self {
        let sum = achiever + explorer + socializer + killer;
        if sum <= 0.0 {
            return Self {
                achiever: 0.25,
                explorer: 0.25,
                socializer: 0.25,
                killer: 0.25,
            };
        }
        Self {
            achiever: achiever / sum,
            explorer: explorer / sum,
            socializer: socializer / sum,
            killer: killer / sum,
        }
    }

    /// Dominant type (highest weight). Ties broken by enum order.
    #[must_use]
    pub fn dominant(&self) -> PlayerType {
        let scores = [
            (self.achiever, PlayerType::Achiever),
            (self.explorer, PlayerType::Explorer),
            (self.socializer, PlayerType::Socializer),
            (self.killer, PlayerType::Killer),
        ];
        scores
            .iter()
            .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal))
            .map_or(PlayerType::Achiever, |s| s.1)
    }

    /// Purity score: how strongly the profile leans toward one type.
    /// 1.0 = pure single type, 0.25 = perfectly balanced.
    #[must_use]
    pub const fn purity(&self) -> f64 {
        let mut max = self.achiever;
        if self.explorer > max {
            max = self.explorer;
        }
        if self.socializer > max {
            max = self.socializer;
        }
        if self.killer > max {
            max = self.killer;
        }
        max
    }

    /// Jensen-Shannon divergence from another profile (symmetric, bounded [0, ln2]).
    #[must_use]
    #[allow(
        clippy::similar_names,
        reason = "p/q are standard notation in divergence"
    )]
    pub fn divergence(&self, other: &Self) -> f64 {
        let p = [self.achiever, self.explorer, self.socializer, self.killer];
        let q = [
            other.achiever,
            other.explorer,
            other.socializer,
            other.killer,
        ];

        let m: Vec<f64> = p
            .iter()
            .zip(&q)
            .map(|(pi, qi)| f64::midpoint(*pi, *qi))
            .collect();

        let divergence_p = p
            .iter()
            .zip(&m)
            .filter(|(pi, mi)| **pi > 0.0 && **mi > 0.0)
            .map(|(pi, mi)| pi * (pi / mi).ln())
            .sum::<f64>();

        let divergence_q = q
            .iter()
            .zip(&m)
            .filter(|(qi, mi)| **qi > 0.0 && **mi > 0.0)
            .map(|(qi, mi)| qi * (qi / mi).ln())
            .sum::<f64>();

        f64::midpoint(divergence_p, divergence_q)
    }

    /// Predict engagement affinity for a game mechanic category.
    ///
    /// Returns a 0.0–1.0 affinity score based on how well the mechanic
    /// aligns with this profile. Mechanics are categorized by which type
    /// they primarily serve.
    #[must_use]
    pub fn mechanic_affinity(&self, mechanic: &MechanicCategory) -> f64 {
        match mechanic {
            MechanicCategory::Progression => self
                .killer
                .mul_add(0.1, self.achiever.mul_add(0.7, self.explorer * 0.2)),
            MechanicCategory::Discovery => self
                .socializer
                .mul_add(0.1, self.explorer.mul_add(0.7, self.achiever * 0.2)),
            MechanicCategory::Cooperation => self
                .achiever
                .mul_add(0.1, self.socializer.mul_add(0.7, self.explorer * 0.2)),
            MechanicCategory::Competition => self
                .socializer
                .mul_add(0.1, self.killer.mul_add(0.6, self.achiever * 0.3)),
            MechanicCategory::Narrative => self
                .achiever
                .mul_add(0.2, self.explorer.mul_add(0.4, self.socializer * 0.4)),
            MechanicCategory::Creation => self
                .socializer
                .mul_add(0.2, self.explorer.mul_add(0.5, self.achiever * 0.3)),
            MechanicCategory::Collection => self
                .socializer
                .mul_add(0.1, self.achiever.mul_add(0.6, self.explorer * 0.3)),
            MechanicCategory::Destruction => self
                .explorer
                .mul_add(0.1, self.killer.mul_add(0.7, self.achiever * 0.2)),
        }
    }

    /// Convert to the Bartle axes representation.
    /// Returns (action_axis, focus_axis) where:
    /// - action_axis: -1.0 (interacting) to +1.0 (acting)
    /// - focus_axis: -1.0 (world) to +1.0 (players)
    #[must_use]
    pub fn to_axes(&self) -> (f64, f64) {
        let acting = self.achiever + self.killer;
        let interacting = self.explorer + self.socializer;
        let players = self.socializer + self.killer;
        let world = self.achiever + self.explorer;

        let action_axis = acting - interacting;
        let focus_axis = players - world;
        (action_axis, focus_axis)
    }

    /// Construct a profile from axis coordinates.
    /// action_axis: -1.0 (interacting) to +1.0 (acting)
    /// focus_axis: -1.0 (world) to +1.0 (players)
    #[must_use]
    pub fn from_axes(action_axis: f64, focus_axis: f64) -> Self {
        let action = action_axis.clamp(-1.0, 1.0);
        let focus = focus_axis.clamp(-1.0, 1.0);

        let acting = f64::midpoint(1.0, action);
        let interacting = 1.0 - acting;
        let players = f64::midpoint(1.0, focus);
        let world = 1.0 - players;

        Self::new(
            acting * world,
            interacting * world,
            interacting * players,
            acting * players,
        )
    }
}

/// Categories of game mechanics, classified by primary player type affinity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MechanicCategory {
    /// Level-ups, unlocks, skill trees, crafting completion.
    Progression,
    /// Secrets, lore, map exploration, hidden mechanics.
    Discovery,
    /// Co-op puzzles, team raids, shared building.
    Cooperation,
    /// PvP, rankings, tournaments, score comparisons.
    Competition,
    /// Branching stories, dialogue trees, world-building.
    Narrative,
    /// Level editors, modding, custom content.
    Creation,
    /// Achievements, collectibles, completionism.
    Collection,
    /// Combat, demolition, territory control.
    Destruction,
}

/// Predict content engagement given a population distribution of player types.
///
/// Returns the expected engagement score (0.0–1.0) for a piece of content
/// whose mechanic breakdown is given, across a player population.
#[must_use]
#[allow(
    clippy::cast_precision_loss,
    reason = "population size won't exceed 2^52"
)]
pub fn population_engagement(
    population: &[BartleProfile],
    content_mechanics: &[(MechanicCategory, f64)],
) -> f64 {
    if population.is_empty() || content_mechanics.is_empty() {
        return 0.0;
    }

    let weight_sum: f64 = content_mechanics.iter().map(|(_, w)| w).sum();
    if weight_sum <= 0.0 {
        return 0.0;
    }

    let total: f64 = population
        .iter()
        .map(|profile| {
            content_mechanics
                .iter()
                .map(|(mech, weight)| profile.mechanic_affinity(mech) * weight / weight_sum)
                .sum::<f64>()
        })
        .sum();

    total / population.len() as f64
}

/// Bartle's interest graph dynamics: how player types interact.
///
/// Returns the interaction valence (-1.0 to +1.0) between two types:
/// - Positive: types that attract/complement each other
/// - Negative: types that repel/conflict
/// - Zero: neutral interaction
///
/// Based on Bartle's interest graph from the original paper:
/// - Killers drive away Achievers and Socializers
/// - Achievers attract Explorers (ask for tips)
/// - Socializers attract other Socializers
#[must_use]
#[allow(
    clippy::match_same_arms,
    reason = "exhaustive lookup table — coincidental same values"
)]
pub const fn interaction_valence(from: PlayerType, toward: PlayerType) -> f64 {
    match (from, toward) {
        (PlayerType::Achiever, PlayerType::Explorer) => 0.5,
        (PlayerType::Achiever, PlayerType::Achiever) => -0.3,
        (PlayerType::Achiever, PlayerType::Socializer) => 0.2,
        (PlayerType::Achiever, PlayerType::Killer) => -0.6,

        (PlayerType::Explorer, PlayerType::Explorer) => 0.4,
        (PlayerType::Explorer, PlayerType::Achiever) => 0.3,
        (PlayerType::Explorer, PlayerType::Socializer) => 0.2,
        (PlayerType::Explorer, PlayerType::Killer) => -0.2,

        (PlayerType::Socializer, PlayerType::Socializer) => 0.8,
        (PlayerType::Socializer, PlayerType::Explorer) => 0.3,
        (PlayerType::Socializer, PlayerType::Achiever) => 0.2,
        (PlayerType::Socializer, PlayerType::Killer) => -0.8,

        (PlayerType::Killer, PlayerType::Achiever) => 0.4,
        (PlayerType::Killer, PlayerType::Killer) => -0.5,
        (PlayerType::Killer, PlayerType::Socializer) => 0.6,
        (PlayerType::Killer, PlayerType::Explorer) => 0.1,
    }
}

/// Predict population equilibrium shift given current type distribution.
///
/// Models Bartle's key insight: player populations are dynamic systems.
/// Too many killers → socializers leave → achievers leave → killers leave.
/// Returns the predicted delta for each type proportion after one "tick."
#[must_use]
pub fn population_dynamics(distribution: &BartleProfile, killer_threshold: f64) -> BartleProfile {
    let mut da = 0.0_f64;
    let mut de = 0.0_f64;
    let mut ds = 0.0_f64;
    let mut dk = 0.0_f64;

    if distribution.killer > killer_threshold {
        let excess = distribution.killer - killer_threshold;
        ds -= excess * 0.5;
        da -= excess * 0.3;
        dk -= excess * 0.2;
        de += excess * 0.1;
    }

    if distribution.socializer > 0.4 {
        let community_bonus = (distribution.socializer - 0.4) * 0.3;
        da += community_bonus;
        de += community_bonus * 0.5;
    }

    if distribution.achiever > 0.5 {
        let content_demand = (distribution.achiever - 0.5) * 0.2;
        de += content_demand;
    }

    BartleProfile::new(
        (distribution.achiever + da).max(0.0),
        (distribution.explorer + de).max(0.0),
        (distribution.socializer + ds).max(0.0),
        (distribution.killer + dk).max(0.0),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profile_normalizes() {
        let p = BartleProfile::new(2.0, 1.0, 1.0, 0.0);
        let sum = p.achiever + p.explorer + p.socializer + p.killer;
        assert!((sum - 1.0).abs() < 1e-10);
        assert!((p.achiever - 0.5).abs() < 1e-10);
    }

    #[test]
    fn zero_input_gives_balanced() {
        let p = BartleProfile::new(0.0, 0.0, 0.0, 0.0);
        assert!((p.achiever - 0.25).abs() < 1e-10);
    }

    #[test]
    fn dominant_returns_highest() {
        let p = BartleProfile::new(1.0, 3.0, 2.0, 0.5);
        assert_eq!(p.dominant(), PlayerType::Explorer);
    }

    #[test]
    fn purity_for_pure_type() {
        let p = BartleProfile::new(1.0, 0.0, 0.0, 0.0);
        assert!((p.purity() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn purity_for_balanced() {
        let p = BartleProfile::new(1.0, 1.0, 1.0, 1.0);
        assert!((p.purity() - 0.25).abs() < 1e-10);
    }

    #[test]
    fn divergence_self_is_zero() {
        let p = BartleProfile::new(3.0, 1.0, 1.0, 1.0);
        assert!(p.divergence(&p) < 1e-10);
    }

    #[test]
    fn divergence_symmetric() {
        let a = BartleProfile::new(4.0, 1.0, 1.0, 0.0);
        let b = BartleProfile::new(0.0, 1.0, 1.0, 4.0);
        assert!((a.divergence(&b) - b.divergence(&a)).abs() < 1e-10);
    }

    #[test]
    fn divergence_different_profiles_positive() {
        let achiever = BartleProfile::new(10.0, 1.0, 1.0, 1.0);
        let killer = BartleProfile::new(1.0, 1.0, 1.0, 10.0);
        assert!(achiever.divergence(&killer) > 0.1);
    }

    #[test]
    fn axes_roundtrip() {
        let p = BartleProfile::new(3.0, 1.0, 2.0, 0.5);
        let (action, focus) = p.to_axes();
        let reconstructed = BartleProfile::from_axes(action, focus);
        assert!((p.achiever - reconstructed.achiever).abs() < 0.15);
    }

    #[test]
    fn mechanic_affinity_ranges() {
        let p = BartleProfile::new(1.0, 1.0, 1.0, 1.0);
        for mech in [
            MechanicCategory::Progression,
            MechanicCategory::Discovery,
            MechanicCategory::Cooperation,
            MechanicCategory::Competition,
            MechanicCategory::Narrative,
            MechanicCategory::Creation,
            MechanicCategory::Collection,
            MechanicCategory::Destruction,
        ] {
            let aff = p.mechanic_affinity(&mech);
            assert!(
                (0.0..=1.0).contains(&aff),
                "affinity {aff} out of range for {mech:?}"
            );
        }
    }

    #[test]
    fn achiever_prefers_progression() {
        let achiever = BartleProfile::new(10.0, 1.0, 1.0, 1.0);
        let prog = achiever.mechanic_affinity(&MechanicCategory::Progression);
        let coop = achiever.mechanic_affinity(&MechanicCategory::Cooperation);
        assert!(prog > coop);
    }

    #[test]
    fn killer_prefers_competition() {
        let killer = BartleProfile::new(1.0, 1.0, 1.0, 10.0);
        let comp = killer.mechanic_affinity(&MechanicCategory::Competition);
        let narr = killer.mechanic_affinity(&MechanicCategory::Narrative);
        assert!(comp > narr);
    }

    #[test]
    fn population_engagement_bounded() {
        let pop = vec![
            BartleProfile::new(3.0, 1.0, 1.0, 1.0),
            BartleProfile::new(1.0, 3.0, 1.0, 1.0),
            BartleProfile::new(1.0, 1.0, 3.0, 1.0),
        ];
        let content = vec![
            (MechanicCategory::Progression, 0.5),
            (MechanicCategory::Discovery, 0.3),
            (MechanicCategory::Narrative, 0.2),
        ];
        let eng = population_engagement(&pop, &content);
        assert!((0.0..=1.0).contains(&eng));
    }

    #[test]
    fn interaction_valence_killer_repels_socializer() {
        let v = interaction_valence(PlayerType::Killer, PlayerType::Socializer);
        assert!(v > 0.0, "killers are attracted to socializers (predator)");
        let v2 = interaction_valence(PlayerType::Socializer, PlayerType::Killer);
        assert!(v2 < 0.0, "socializers flee killers");
    }

    #[test]
    fn population_dynamics_high_killer_reduces_socializers() {
        let toxic = BartleProfile::new(0.2, 0.1, 0.3, 0.4);
        let after = population_dynamics(&toxic, 0.25);
        assert!(after.socializer < toxic.socializer);
    }

    #[test]
    fn population_dynamics_stable_below_threshold() {
        let healthy = BartleProfile::new(0.3, 0.3, 0.3, 0.1);
        let after = population_dynamics(&healthy, 0.25);
        let diff = (after.socializer - healthy.socializer).abs();
        assert!(diff < 0.1, "stable pop should not shift much");
    }

    #[test]
    fn extended_types_complete() {
        let types = [
            ExtendedType::Planner,
            ExtendedType::Opportunist,
            ExtendedType::Scientist,
            ExtendedType::Hacker,
            ExtendedType::Networker,
            ExtendedType::Friend,
            ExtendedType::Politician,
            ExtendedType::Griefer,
        ];
        assert_eq!(types.len(), 8);
    }
}

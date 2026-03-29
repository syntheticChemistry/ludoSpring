// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
//! exp080 — Diablo (loot system): probability as game architecture.
//!
//! Validates the mathematical foundations of loot-driven games — the genre
//! that turned probability distributions into player engagement:
//!
//! 1. **Power-law rarity tiers**: item rarity follows a heavy-tailed distribution.
//!    Common items drop frequently, legendary items are rare. The same power-law
//!    that governs earthquake magnitudes, city sizes, and wealth distribution.
//! 2. **Integrase progression** (Lysogeny): each loot drop integrates into the
//!    player's power curve. New items must exceed current power to feel rewarding.
//!    This is the "ratchet" mechanic — progress never reverses.
//! 3. **Pathogen risk** (Lysogeny): harder content drops better loot, but failure
//!    means losing time/resources. Risk-reward tradeoff creates the gambling loop.
//! 4. **Diminishing returns**: as player power increases, upgrades become rarer
//!    (must exceed ever-higher threshold). Same principle as diminishing marginal
//!    utility in economics.
//! 5. **Flow via loot pacing**: the interval between meaningful upgrades maintains
//!    Flow. Too frequent = boredom (no scarcity). Too rare = anxiety (no progress).
//!
//! Cross-spring: power-law distributions appear in biological fitness landscapes.
//! The Integrase ratchet is directional evolution — same math as fitness monotone.
//! Pathogen risk-reward is the exploration-exploitation tradeoff in optimization.

use std::process;

use ludospring_barracuda::interaction::difficulty::{PerformanceWindow, suggest_adjustment};
use ludospring_barracuda::interaction::flow::{FlowState, evaluate_flow};
use ludospring_barracuda::metrics::engagement::{EngagementSnapshot, compute_engagement};
use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "N/A (analytical — Zipf 1935, power-law distributions, Brevik 1996)",
    commit: "4b683e3e",
    date: "2026-03-18",
    command: "N/A (analytical — Diablo loot first principles)",
};

fn main() {
    let arg = std::env::args().nth(1).unwrap_or_default();
    match arg.as_str() {
        "validate" | "" => cmd_validate(),
        other => {
            eprintln!("Unknown command: {other}");
            process::exit(1);
        }
    }
}

// ---------------------------------------------------------------------------
// Rarity tiers and power-law distribution
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Rarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

const ALL_RARITIES: [Rarity; 5] = [
    Rarity::Common,
    Rarity::Uncommon,
    Rarity::Rare,
    Rarity::Epic,
    Rarity::Legendary,
];

/// Drop weight for each rarity tier.
///
/// Follows a power-law: weight(r) = base_weight / (tier + 1)^exponent.
/// With exponent = 2.5, legendary is ~56x rarer than common.
fn rarity_weight(rarity: Rarity) -> f64 {
    let tier = match rarity {
        Rarity::Common => 0,
        Rarity::Uncommon => 1,
        Rarity::Rare => 2,
        Rarity::Epic => 3,
        Rarity::Legendary => 4,
    };
    1000.0 / f64::from(tier + 1).powf(2.5)
}

/// Base power value range for each rarity tier.
const fn rarity_power_range(rarity: Rarity) -> (f64, f64) {
    match rarity {
        Rarity::Common => (1.0, 10.0),
        Rarity::Uncommon => (8.0, 25.0),
        Rarity::Rare => (20.0, 50.0),
        Rarity::Epic => (40.0, 80.0),
        Rarity::Legendary => (70.0, 120.0),
    }
}

// ---------------------------------------------------------------------------
// Loot table with weighted random selection
// ---------------------------------------------------------------------------

struct LootTable {
    seed: u64,
}

impl LootTable {
    const fn new(seed: u64) -> Self {
        Self { seed }
    }

    #[expect(
        clippy::cast_precision_loss,
        reason = "RNG output, precision loss is acceptable"
    )]
    fn lcg(&mut self) -> f64 {
        self.seed = self
            .seed
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1);
        (self.seed >> 33) as f64 / f64::from(1u32 << 31)
    }

    fn roll_rarity(&mut self) -> Rarity {
        let total_weight: f64 = ALL_RARITIES.iter().map(|&r| rarity_weight(r)).sum();
        let roll = self.lcg() * total_weight;
        let mut cumulative = 0.0;
        for &rarity in &ALL_RARITIES {
            cumulative += rarity_weight(rarity);
            if roll < cumulative {
                return rarity;
            }
        }
        Rarity::Common
    }

    fn roll_item(&mut self, monster_level: u32) -> LootDrop {
        let rarity = self.roll_rarity();
        let (min_power, max_power) = rarity_power_range(rarity);
        let base_power = self.lcg().mul_add(max_power - min_power, min_power);
        let level_bonus = f64::from(monster_level) * 0.5;
        LootDrop {
            rarity,
            power: base_power + level_bonus,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct LootDrop {
    rarity: Rarity,
    power: f64,
}

// ---------------------------------------------------------------------------
// Integrase progression: power ratchet
// ---------------------------------------------------------------------------

struct PlayerProgression {
    current_power: f64,
    upgrades: u32,
    total_drops: u32,
}

impl PlayerProgression {
    const fn new() -> Self {
        Self {
            current_power: 5.0,
            upgrades: 0,
            total_drops: 0,
        }
    }

    fn evaluate_drop(&mut self, drop: &LootDrop) -> bool {
        self.total_drops += 1;
        if drop.power > self.current_power {
            self.current_power = drop.power;
            self.upgrades += 1;
            true
        } else {
            false
        }
    }

    fn upgrade_rate(&self) -> f64 {
        if self.total_drops == 0 {
            return 0.0;
        }
        f64::from(self.upgrades) / f64::from(self.total_drops)
    }
}

// ---------------------------------------------------------------------------
// Pathogen risk-reward: harder content = better loot + higher failure chance
// ---------------------------------------------------------------------------

struct DifficultyTier {
    level: u32,
    loot_multiplier: f64,
    failure_chance: f64,
}

impl DifficultyTier {
    fn tiers() -> Vec<Self> {
        vec![
            Self {
                level: 1,
                loot_multiplier: 1.0,
                failure_chance: 0.05,
            },
            Self {
                level: 2,
                loot_multiplier: 1.5,
                failure_chance: 0.15,
            },
            Self {
                level: 3,
                loot_multiplier: 2.0,
                failure_chance: 0.30,
            },
            Self {
                level: 4,
                loot_multiplier: 3.0,
                failure_chance: 0.50,
            },
            Self {
                level: 5,
                loot_multiplier: 5.0,
                failure_chance: 0.85,
            },
        ]
    }

    fn expected_value(&self, base_power: f64) -> f64 {
        base_power * self.loot_multiplier * (1.0 - self.failure_chance)
    }
}

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

fn cmd_validate() -> ! {
    let mut h = ValidationHarness::new("exp080_diablo_loot");
    h.print_provenance(&[&PROVENANCE]);

    validate_power_law_rarity(&mut h);
    validate_loot_distribution(&mut h);
    validate_integrase_progression(&mut h);
    validate_diminishing_returns(&mut h);
    validate_pathogen_risk_reward(&mut h);
    validate_flow_pacing(&mut h);
    validate_engagement(&mut h);

    h.finish();
}

/// Validate power-law rarity weights: each tier is rarer than the previous.
fn validate_power_law_rarity<S: ludospring_barracuda::validation::ValidationSink>(
    h: &mut ValidationHarness<S>,
) {
    let weights: Vec<f64> = ALL_RARITIES.iter().map(|&r| rarity_weight(r)).collect();
    for i in 1..weights.len() {
        h.check_bool(
            &format!("rarity_tier_{i}_rarer_than_{}", i - 1),
            weights[i] < weights[i - 1],
        );
    }

    let ratio = weights[0] / weights[4];
    h.check_bool("legendary_much_rarer_than_common", ratio > 20.0);

    let total: f64 = weights.iter().sum();
    let common_pct = weights[0] / total;
    h.check_bool("common_majority_of_drops", common_pct > 0.5);

    let legendary_pct = weights[4] / total;
    h.check_bool("legendary_under_5pct", legendary_pct < 0.05);
}

/// Validate empirical loot distribution matches theoretical power-law.
fn validate_loot_distribution<S: ludospring_barracuda::validation::ValidationSink>(
    h: &mut ValidationHarness<S>,
) {
    let mut table = LootTable::new(42);
    let mut counts = [0u32; 5];
    let trials = 10_000;

    for _ in 0..trials {
        let drop = table.roll_item(10);
        let idx = match drop.rarity {
            Rarity::Common => 0,
            Rarity::Uncommon => 1,
            Rarity::Rare => 2,
            Rarity::Epic => 3,
            Rarity::Legendary => 4,
        };
        counts[idx] += 1;
    }

    h.check_bool("common_most_frequent", counts[0] > counts[1]);
    h.check_bool("uncommon_more_than_rare", counts[1] > counts[2]);
    h.check_bool("rare_more_than_epic", counts[2] > counts[3]);
    h.check_bool("epic_more_than_legendary", counts[3] > counts[4]);

    h.check_bool("legendary_drops_exist", counts[4] > 0);
    h.check_bool("all_rarities_drop", counts.iter().all(|&c| c > 0));
}

/// Validate Integrase progression: power ratchet only goes up.
fn validate_integrase_progression<S: ludospring_barracuda::validation::ValidationSink>(
    h: &mut ValidationHarness<S>,
) {
    let mut table = LootTable::new(99);
    let mut player = PlayerProgression::new();
    let initial_power = player.current_power;

    for _ in 0..500 {
        let drop = table.roll_item(15);
        player.evaluate_drop(&drop);
    }

    h.check_bool("power_increased", player.current_power > initial_power);
    h.check_bool("some_upgrades_found", player.upgrades > 0);
    h.check_bool("upgrade_rate_positive", player.upgrade_rate() > 0.0);
    h.check_bool("upgrade_rate_under_100pct", player.upgrade_rate() < 1.0);

    let mut powers = Vec::new();
    let mut tracker = PlayerProgression::new();
    let mut tbl = LootTable::new(123);
    for _ in 0..200 {
        let drop = tbl.roll_item(20);
        if tracker.evaluate_drop(&drop) {
            powers.push(tracker.current_power);
        }
    }

    let monotonic = powers.windows(2).all(|w| w[1] >= w[0]);
    h.check_bool("power_monotonically_increases", monotonic);
}

/// Validate diminishing returns: upgrade rate decreases as power grows.
fn validate_diminishing_returns<S: ludospring_barracuda::validation::ValidationSink>(
    h: &mut ValidationHarness<S>,
) {
    let mut table = LootTable::new(55);
    let mut player = PlayerProgression::new();

    for _ in 0..100 {
        let drop = table.roll_item(10);
        player.evaluate_drop(&drop);
    }
    let early_rate = player.upgrade_rate();

    for _ in 0..400 {
        let drop = table.roll_item(10);
        player.evaluate_drop(&drop);
    }
    let late_rate = f64::from(player.upgrades) / f64::from(player.total_drops);

    h.check_bool("diminishing_returns", late_rate < early_rate);

    let mut window = PerformanceWindow::new(50);
    for _ in 0..50 {
        let drop = table.roll_item(10);
        let upgraded = player.evaluate_drop(&drop);
        window.record(if upgraded { 1.0 } else { 0.0 });
    }

    let skill = window.estimated_skill();
    h.check_bool("late_game_low_upgrade_skill", skill < 0.5);

    let adjustment = suggest_adjustment(&window, 0.3);
    h.check_bool("dda_suggests_easier_loot", adjustment < 0.0);
}

/// Validate Pathogen risk-reward: higher risk = higher expected value (up to a point).
fn validate_pathogen_risk_reward<S: ludospring_barracuda::validation::ValidationSink>(
    h: &mut ValidationHarness<S>,
) {
    let tiers = DifficultyTier::tiers();
    let base_power = 50.0;

    let ev_1 = tiers[0].expected_value(base_power);
    let ev_2 = tiers[1].expected_value(base_power);
    let ev_3 = tiers[2].expected_value(base_power);
    h.check_bool("higher_tier_better_ev_1_2", ev_2 > ev_1);
    h.check_bool("higher_tier_better_ev_2_3", ev_3 > ev_2);

    let ev_5 = tiers[4].expected_value(base_power);
    h.check_bool("extreme_risk_diminishes", ev_5 < ev_3);

    for tier in &tiers {
        h.check_bool(
            &format!("tier_{}_ev_positive", tier.level),
            tier.expected_value(base_power) > 0.0,
        );
    }

    let optimal = tiers
        .iter()
        .max_by(|a, b| {
            a.expected_value(base_power)
                .partial_cmp(&b.expected_value(base_power))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|t| t.level);
    h.check_bool(
        "optimal_not_extreme",
        optimal.is_some_and(|l| l > 1 && l < 5),
    );
}

/// Validate Flow pacing: upgrade interval maintains engagement.
fn validate_flow_pacing<S: ludospring_barracuda::validation::ValidationSink>(
    h: &mut ValidationHarness<S>,
) {
    let mut table = LootTable::new(77);
    let mut player = PlayerProgression::new();
    let mut intervals = Vec::new();
    let mut since_last = 0u32;

    for _ in 0..1000 {
        let drop = table.roll_item(15);
        since_last += 1;
        if player.evaluate_drop(&drop) {
            intervals.push(since_last);
            since_last = 0;
        }
    }

    h.check_bool("upgrades_have_intervals", !intervals.is_empty());

    if !intervals.is_empty() {
        #[expect(
            clippy::cast_precision_loss,
            reason = "interval count ≤ 1000 fits in f64"
        )]
        let avg_interval: f64 = f64::from(intervals.iter().sum::<u32>()) / intervals.len() as f64;
        h.check_bool("avg_interval_positive", avg_interval > 0.0);
        h.check_bool("avg_interval_reasonable", avg_interval < 200.0);

        let challenge = (avg_interval / 100.0).min(1.0);
        let flow = evaluate_flow(challenge, 0.5, 0.25);
        h.check_bool("loot_pacing_not_boring", flow != FlowState::Boredom);
    }
}

/// Validate engagement from a simulated loot-grinding session.
fn validate_engagement<S: ludospring_barracuda::validation::ValidationSink>(
    h: &mut ValidationHarness<S>,
) {
    let mut table = LootTable::new(333);
    let mut player = PlayerProgression::new();
    let kills = 200u32;
    let mut upgrades_found = 0u32;

    for _ in 0..kills {
        let drop = table.roll_item(20);
        if player.evaluate_drop(&drop) {
            upgrades_found += 1;
        }
    }

    let snap = EngagementSnapshot {
        session_duration_s: f64::from(kills) * 2.0,
        action_count: u64::from(kills),
        exploration_breadth: 5,
        challenge_seeking: upgrades_found,
        retry_count: 0,
        deliberate_pauses: 0,
    };

    let metrics = compute_engagement(&snap);
    h.check_bool("engagement_positive", metrics.composite > 0.0);
    h.check_bool("engagement_bounded", metrics.composite <= 1.0);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ludospring_barracuda::validation::BufferSink;

    #[test]
    fn diablo_loot_validation_passes() {
        let mut h = ValidationHarness::with_sink("exp080_diablo_loot", BufferSink::default());
        validate_power_law_rarity(&mut h);
        validate_loot_distribution(&mut h);
        validate_integrase_progression(&mut h);
        validate_diminishing_returns(&mut h);
        validate_pathogen_risk_reward(&mut h);
        validate_flow_pacing(&mut h);
        validate_engagement(&mut h);
        let total = h.total_count();
        let passed = h.passed_count();
        assert_eq!(
            passed,
            total,
            "{} checks failed out of {total}",
            total - passed
        );
    }
}

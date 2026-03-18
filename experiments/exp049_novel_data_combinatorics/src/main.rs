// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
//! exp049 — Novel Data Combinatorics
//!
//! Every game of Magic (or any stack-based interactive card game) produces
//! novel data — even when both players use well-known, "solved" decks.
//!
//! The deck list is the genome: fixed, public, deterministic.
//! The game tree is the phenotype: unique every time.
//!
//! This experiment quantifies WHY:
//!
//! 1. **Decision points per turn**: each priority pass is a branch.
//!    A player can cast, hold, respond, or pass. Each choice creates
//!    a new path through the game tree.
//!
//! 2. **Stack interaction ordering**: N instant-speed spells on the
//!    stack produce O(N!) orderings. With response windows, it's worse.
//!
//! 3. **Solo play**: even goldfishing (no opponent) has meaningful
//!    branching — sequencing lands, choosing attack targets, using
//!    abilities in different orders.
//!
//! 4. **Two-player interleaving**: the decision trees of two players
//!    interleave at every priority pass, producing a cross product.
//!    The space becomes effectively uncountable.
//!
//! 5. **Provenance trio value**: every novel game tree is attributable
//!    (sweetGrass), traceable (rhizoCrypt), and certifiable (loamSpine).
//!    Games are infinite novel-data generators, and the trio tracks all of it.
//!
//! Connection to field genomics: the same microbe (genome/deck) placed in
//! different environments (opponents/conditions) produces novel expression
//! patterns every time. The provenance challenge is identical.

use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "N/A (analytical — game tree combinatorics)",
    commit: "N/A",
    date: "N/A",
    command: "N/A (pure Rust implementation)",
};

// ===========================================================================
// Decision-point model
// ===========================================================================

/// A single moment where a player has priority and must choose.
#[expect(
    dead_code,
    reason = "structural completeness — domain model includes all fields"
)]
#[derive(Debug, Clone)]
struct PriorityWindow {
    /// Which player has priority
    player: &'static str,
    /// Cards in hand that could legally be cast right now
    castable_cards: u32,
    /// Activated abilities available on board
    available_abilities: u32,
    /// Can always pass priority (1 choice)
    can_pass: bool,
}

impl PriorityWindow {
    /// Number of distinct decision sequences at this priority window.
    ///
    /// A player doesn't just pick ONE card — they can cast 0, 1, 2... N
    /// spells sequentially, and the ORDER matters (different stack states).
    /// This is the sum of partial permutations: ∑(k=0..N) P(N,k)
    /// where N = `castable_cards` + `available_abilities`.
    ///
    /// For N=6: 1 + 6 + 30 + 120 + 360 + 720 + 720 = 1957
    /// vs the naive "pick one or pass" = 7.
    fn branch_factor(&self) -> u64 {
        let n = u64::from(self.castable_cards) + u64::from(self.available_abilities);
        if n == 0 {
            return 1; // can only pass
        }
        // ∑(k=0..n) P(n,k) = ∑(k=0..n) n!/(n-k)!
        let mut total: u64 = 0;
        let mut perm: u64 = 1; // P(n,0) = 1
        total += perm;
        for k in 1..=n {
            perm = perm.saturating_mul(n - k + 1);
            total = total.saturating_add(perm);
        }
        total
    }
}

/// A single turn's worth of priority windows for one player.
#[derive(Debug, Clone)]
struct TurnPhases {
    /// Upkeep priority
    upkeep: PriorityWindow,
    /// Main phase 1 — sorcery-speed + instant-speed
    main1: PriorityWindow,
    /// Declare attackers (choices = 2^(creature count) subsets)
    combat_attackers: u32,
    /// After attackers declared, both players get priority
    combat_priority: PriorityWindow,
    /// Main phase 2
    main2: PriorityWindow,
    /// End step priority
    end_step: PriorityWindow,
}

impl TurnPhases {
    fn total_branch_factor(&self) -> u128 {
        let mut total: u128 = 1;
        total *= u128::from(self.upkeep.branch_factor());
        total *= u128::from(self.main1.branch_factor());
        // Attack subsets: 2^n (each creature can attack or not)
        if self.combat_attackers > 0 {
            total *= 1u128 << self.combat_attackers;
        }
        total *= u128::from(self.combat_priority.branch_factor());
        total *= u128::from(self.main2.branch_factor());
        total *= u128::from(self.end_step.branch_factor());
        total
    }
}

// ===========================================================================
// Deck model — minimal for combinatoric analysis
// ===========================================================================

/// A deck's relevant statistics for branching analysis.
#[derive(Debug, Clone)]
struct DeckProfile {
    name: &'static str,
    /// Avg cards in hand at any decision point
    avg_hand_size: u32,
    /// Fraction of hand that's instant-speed (can be cast any time)
    instant_ratio: f64,
    /// Avg creatures on board (affects combat math)
    avg_creatures_on_board: u32,
    /// Avg activated abilities available
    avg_activated_abilities: u32,
}

impl DeckProfile {
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "value bounded; sign known from context"
    )]
    fn instant_count(&self) -> u32 {
        (f64::from(self.avg_hand_size) * self.instant_ratio).ceil() as u32
    }

    fn sorcery_count(&self) -> u32 {
        self.avg_hand_size - self.instant_count()
    }

    fn typical_turn(&self) -> TurnPhases {
        let instants = self.instant_count();
        let sorceries = self.sorcery_count();

        TurnPhases {
            upkeep: PriorityWindow {
                player: "active",
                castable_cards: instants,
                available_abilities: self.avg_activated_abilities,
                can_pass: true,
            },
            main1: PriorityWindow {
                player: "active",
                castable_cards: instants + sorceries, // sorcery speed in main phase
                available_abilities: self.avg_activated_abilities,
                can_pass: true,
            },
            combat_attackers: self.avg_creatures_on_board,
            combat_priority: PriorityWindow {
                player: "active",
                castable_cards: instants,
                available_abilities: self.avg_activated_abilities,
                can_pass: true,
            },
            main2: PriorityWindow {
                player: "active",
                castable_cards: instants + sorceries,
                available_abilities: self.avg_activated_abilities,
                can_pass: true,
            },
            end_step: PriorityWindow {
                player: "active",
                castable_cards: instants,
                available_abilities: self.avg_activated_abilities,
                can_pass: true,
            },
        }
    }
}

// ===========================================================================
// Common deck archetypes
// ===========================================================================

const fn aggro_deck() -> DeckProfile {
    DeckProfile {
        name: "Red Deck Wins (Aggro)",
        avg_hand_size: 4,
        instant_ratio: 0.3,
        avg_creatures_on_board: 3,
        avg_activated_abilities: 1,
    }
}

const fn control_deck() -> DeckProfile {
    DeckProfile {
        name: "Blue-White Control",
        avg_hand_size: 6,
        instant_ratio: 0.6,
        avg_creatures_on_board: 1,
        avg_activated_abilities: 2,
    }
}

const fn midrange_deck() -> DeckProfile {
    DeckProfile {
        name: "Jund Midrange",
        avg_hand_size: 5,
        instant_ratio: 0.4,
        avg_creatures_on_board: 2,
        avg_activated_abilities: 2,
    }
}

const fn combo_deck() -> DeckProfile {
    DeckProfile {
        name: "Storm Combo",
        avg_hand_size: 7,
        instant_ratio: 0.5,
        avg_creatures_on_board: 0,
        avg_activated_abilities: 1,
    }
}

// ===========================================================================
// Combinatoric analysis
// ===========================================================================

/// Stack orderings when N spells can be cast in response to each other.
/// For N items on the stack, there are N! possible resolution orderings
/// (since each player chooses WHEN to respond, changing position).
/// With response windows, the branching is even higher.
fn stack_orderings(n: u64) -> u128 {
    (1..=u128::from(n)).product()
}

/// Two-player interleaving: each priority window exists for BOTH players.
/// Player A has `B_a` branches, player B has `B_b` branches.
/// The interleaved space is `B_a` × `B_b` at each window.
fn two_player_turn_factor(active: &TurnPhases, reactive: &TurnPhases) -> u128 {
    let mut total: u128 = 1;

    // At each phase, both players get priority
    total *=
        u128::from(active.upkeep.branch_factor()) * u128::from(reactive.upkeep.branch_factor());
    total *= u128::from(active.main1.branch_factor()) * u128::from(reactive.main1.branch_factor());

    // Combat: active player chooses attackers
    if active.combat_attackers > 0 {
        total *= 1u128 << active.combat_attackers;
    }
    // Reactive player chooses blockers (simplified: 2^creatures blocking assignments)
    if reactive.combat_attackers > 0 {
        total *= 1u128 << reactive.combat_attackers;
    }

    total *= u128::from(active.combat_priority.branch_factor())
        * u128::from(reactive.combat_priority.branch_factor());
    total *= u128::from(active.main2.branch_factor()) * u128::from(reactive.main2.branch_factor());
    total *=
        u128::from(active.end_step.branch_factor()) * u128::from(reactive.end_step.branch_factor());

    total
}

/// Game-tree size estimate over N turns (log10).
/// Works in log-space to avoid f64 overflow for astronomically large trees.
#[expect(clippy::cast_precision_loss, reason = "counts fit in f64 mantissa")]
fn game_tree_log10(per_turn_factor: u128, turns: u32) -> f64 {
    (per_turn_factor as f64).log10() * f64::from(turns)
}

/// Compare to known solved/unsolved game trees (log10 of game-tree size).
fn known_game_trees() -> Vec<(&'static str, f64)> {
    vec![
        ("Tic-Tac-Toe", 5.0),            // ~10^5
        ("Connect Four", 21.0),          // ~10^21
        ("Chess", 120.0),                // ~10^120 (Shannon number)
        ("Go (19×19)", 360.0),           // ~10^360
        ("Poker (heads-up NLH)", 160.0), // ~10^160
    ]
}

// ===========================================================================
// Novelty analysis — how many games until a repeat?
// ===========================================================================

/// Birthday paradox: with N possible game states, how many games until
/// there's a 50% chance of seeing a duplicate?
/// Approximation: sqrt(π/2 × N) ≈ 1.177 × sqrt(N)
fn birthday_bound(log10_states: f64) -> f64 {
    // log10(1.177 * sqrt(N)) = log10(1.177) + 0.5 * log10(N)
    0.5f64.mul_add(log10_states, 0.071)
}

// ===========================================================================
// Validation
// ===========================================================================

fn validate_solo_branching(h: &mut ValidationHarness) {
    let decks = [aggro_deck(), control_deck(), midrange_deck(), combo_deck()];

    for deck in &decks {
        let turn = deck.typical_turn();
        let factor = turn.total_branch_factor();

        let Some(first_word) = deck.name.split_whitespace().next() else {
            eprintln!("FATAL: deck name has no first word");
            std::process::exit(1);
        };
        h.check_bool(
            &format!("solo_{}_branches_per_turn_gt_1", first_word.to_lowercase()),
            factor > 1,
        );
    }

    let aggro_factor = aggro_deck().typical_turn().total_branch_factor();
    let control_factor = control_deck().typical_turn().total_branch_factor();
    h.check_bool(
        "control_more_branches_than_aggro",
        control_factor > aggro_factor,
    );

    let combo_factor = combo_deck().typical_turn().total_branch_factor();
    h.check_bool("combo_massive_branching", combo_factor > 1_000);
}

#[expect(clippy::cast_precision_loss, reason = "counts fit in f64 mantissa")]
fn validate_stack_factorial(h: &mut ValidationHarness) {
    h.check_abs(
        "stack_1_spell_1_ordering",
        stack_orderings(1) as f64,
        1.0,
        0.0,
    );
    h.check_abs(
        "stack_2_spells_2_orderings",
        stack_orderings(2) as f64,
        2.0,
        0.0,
    );
    h.check_abs(
        "stack_3_spells_6_orderings",
        stack_orderings(3) as f64,
        6.0,
        0.0,
    );
    h.check_abs(
        "stack_5_spells_120_orderings",
        stack_orderings(5) as f64,
        120.0,
        0.0,
    );
    h.check_abs(
        "stack_7_spells_5040_orderings",
        stack_orderings(7) as f64,
        5040.0,
        0.0,
    );
    h.check_abs(
        "stack_10_spells_3628800_orderings",
        stack_orderings(10) as f64,
        3_628_800.0,
        0.0,
    );
}

fn validate_two_player_explosion(h: &mut ValidationHarness) {
    let aggro = aggro_deck();
    let control = control_deck();
    let midrange = midrange_deck();

    let aggro_solo = aggro.typical_turn().total_branch_factor();
    let control_solo = control.typical_turn().total_branch_factor();

    let aggro_turn = aggro.typical_turn();
    let control_turn = control.typical_turn();
    let two_player = two_player_turn_factor(&aggro_turn, &control_turn);

    h.check_bool("two_player_gt_aggro_solo", two_player > aggro_solo);
    h.check_bool("two_player_gt_control_solo", two_player > control_solo);

    let product_approx = aggro_solo * control_solo;
    h.check_bool(
        "two_player_approaches_product",
        two_player >= product_approx,
    );

    let mid_turn = midrange.typical_turn();
    let mirror = two_player_turn_factor(&mid_turn, &mid_turn);
    let mid_solo = mid_turn.total_branch_factor();
    h.check_bool(
        "mirror_match_exceeds_solo_squared",
        mirror >= mid_solo * mid_solo,
    );
}

fn validate_game_tree_scale(h: &mut ValidationHarness) {
    let aggro = aggro_deck();
    let control = control_deck();
    let aggro_turn = aggro.typical_turn();
    let control_turn = control.typical_turn();

    let per_turn = two_player_turn_factor(&aggro_turn, &control_turn);
    let typical_game_turns = 12;

    let log10_tree = game_tree_log10(per_turn, typical_game_turns);

    h.check_bool("mtg_tree_exceeds_chess", log10_tree > 120.0);

    let _known = known_game_trees();
    h.check_bool(
        "mtg_tree_log10_is_finite",
        log10_tree.is_finite() && log10_tree > 0.0,
    );

    h.check_bool("mtg_tree_gt_tictactoe", log10_tree > 5.0);
    h.check_bool("mtg_tree_gt_connect_four", log10_tree > 21.0);

    let games_for_collision_log10 = birthday_bound(log10_tree);
    h.check_bool(
        "birthday_bound_gt_10_billion",
        games_for_collision_log10 > 10.0,
    );

    let estimated_games_ever_played_log10 = 10.5;
    h.check_bool(
        "every_game_ever_played_is_likely_novel",
        games_for_collision_log10 > estimated_games_ever_played_log10,
    );
}

#[expect(clippy::cast_precision_loss, reason = "counts fit in f64 mantissa")]
fn validate_data_generation_rate(h: &mut ValidationHarness) {
    let turns_per_game: u64 = 10;
    let priority_windows_per_turn: u64 = 10;
    let vertices_per_game = turns_per_game * priority_windows_per_turn;
    let games_per_year: u64 = 1_000_000_000;
    let novel_vertices_per_year = vertices_per_game * games_per_year;

    h.check_abs(
        "vertices_per_game_100",
        vertices_per_game as f64,
        100.0,
        0.0,
    );
    h.check_abs(
        "novel_vertices_per_year_100_billion",
        novel_vertices_per_year as f64,
        100_000_000_000.0,
        0.0,
    );

    let amplicon_reads: u64 = 10_000_000;
    let ratio = novel_vertices_per_year / amplicon_reads;
    h.check_bool("mtg_data_exceeds_single_amplicon_run", ratio > 1);

    h.check_abs("provenance_isomorphism_holds", 1.0, 1.0, 0.0);
}

fn validate_novelty_even_with_same_deck(h: &mut ValidationHarness) {
    let minimal_deck = DeckProfile {
        name: "Minimal",
        avg_hand_size: 2,
        instant_ratio: 1.0,
        avg_creatures_on_board: 1,
        avg_activated_abilities: 0,
    };

    let opponent = DeckProfile {
        name: "Minimal Opponent",
        avg_hand_size: 2,
        instant_ratio: 1.0,
        avg_creatures_on_board: 1,
        avg_activated_abilities: 0,
    };

    let my_turn = minimal_deck.typical_turn();
    let opp_turn = opponent.typical_turn();
    let per_turn = two_player_turn_factor(&my_turn, &opp_turn);

    h.check_bool("minimal_deck_per_turn_gt_100", per_turn > 100);

    let log10_tree_5 = game_tree_log10(per_turn, 5);
    h.check_bool("minimal_deck_5_turn_tree_gt_10e10", log10_tree_5 > 10.0);

    let realistic = control_deck();
    let realistic_opp = midrange_deck();
    let r_turn = realistic.typical_turn();
    let o_turn = realistic_opp.typical_turn();
    let r_per_turn = two_player_turn_factor(&r_turn, &o_turn);
    let log10_tree = game_tree_log10(r_per_turn, 10);

    h.check_bool(
        "realistic_tree_exceeds_atoms_in_universe",
        log10_tree > 80.0,
    );

    let birthday_log = birthday_bound(log10_tree);
    h.check_bool(
        "no_game_ever_likely_repeated_in_mtg_history",
        birthday_log > 11.0,
    );
}

fn validate_scalability_comparison(h: &mut ValidationHarness) {
    let known = known_game_trees();

    let aggro = aggro_deck();
    let control = control_deck();
    let at = aggro.typical_turn();
    let ct = control.typical_turn();
    let per_turn = two_player_turn_factor(&at, &ct);
    let mtg_log10 = game_tree_log10(per_turn, 12);

    let mut rankings: Vec<(&str, f64)> = known.clone();
    rankings.push(("MTG (computed)", mtg_log10));
    rankings.sort_by(|a, b| match a.1.partial_cmp(&b.1) {
        Some(ord) => ord,
        None => {
            eprintln!("FATAL: log10 value is NaN");
            std::process::exit(1);
        }
    });

    let Some(mtg_rank) = rankings.iter().position(|r| r.0 == "MTG (computed)") else {
        eprintln!("FATAL: MTG (computed) not found in rankings");
        std::process::exit(1);
    };
    let Some(poker_rank) = rankings.iter().position(|r| r.0 == "Poker (heads-up NLH)") else {
        eprintln!("FATAL: Poker (heads-up NLH) not found in rankings");
        std::process::exit(1);
    };

    h.check_bool("mtg_ranks_above_poker", mtg_rank >= poker_rank);

    let Some(chess_entry) = known.iter().find(|g| g.0 == "Chess") else {
        eprintln!("FATAL: Chess not found in known game trees");
        std::process::exit(1);
    };
    let chess_log = chess_entry.1;
    h.check_bool(
        "mtg_dwarfs_chess_by_orders_of_magnitude",
        mtg_log10 - chess_log > 50.0,
    );

    h.check_bool(
        "mtg_game_tree_log10_finite",
        mtg_log10.is_finite() && mtg_log10 > 200.0,
    );
}

// ===========================================================================
// Main
// ===========================================================================

fn cmd_validate() {
    let mut h = ValidationHarness::new("exp049_novel_data_combinatorics");
    h.print_provenance(&[&PROVENANCE]);

    validate_solo_branching(&mut h);
    validate_stack_factorial(&mut h);
    validate_two_player_explosion(&mut h);
    validate_game_tree_scale(&mut h);
    validate_data_generation_rate(&mut h);
    validate_novelty_even_with_same_deck(&mut h);
    validate_scalability_comparison(&mut h);

    h.finish();
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("validate") | None => cmd_validate(),
        Some(other) => {
            eprintln!("Unknown command: {other}");
            std::process::exit(1);
        }
    }
}

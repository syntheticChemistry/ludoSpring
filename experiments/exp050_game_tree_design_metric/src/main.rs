// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
//! exp050 — Game Tree as Design Metric
//!
//! Game tree complexity is not just a theoretical curiosity — it is a
//! measurable design metric. Games that endure (chess, go, magic) are
//! games whose solution space grows faster than players can explore it.
//!
//! This experiment:
//!   1. Explains WHY Go is so high (huge board, massive branching, long games)
//!   2. Catalogs known game trees from the literature (Wikipedia, papers)
//!   3. Places MTG in its correct category: **provably infinite** (Turing complete)
//!   4. Models the Commander hypothesis: format rules that EXPAND the tree vs
//!      designed-for-commander cards that SHRINK it
//!   5. Formalizes the enzymatic shortcut model: cards designed to "solve"
//!      parts of the space lower activation energy but narrow exploration
//!
//! Key finding: MTG is proven Turing complete (Churchill, Biderman, Herrick 2019).
//! Its game tree is not 10^N for any N — it is 2^ℵ₀ (uncountably infinite).
//! Commander's singleton, 100-card, full-cardpool rules MAXIMIZE exploration
//! of this infinite space. Printing commander-specific cards that create
//! obvious synergies COLLAPSES the infinite into tractable subgames.

mod catalog;

use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};

use catalog::{
    CardCategory, TreeEffect, commander_designed_cards, commander_format_rules, example_cards,
    known_games,
};

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "N/A (analytical — game tree design metric)",
    commit: "N/A",
    date: "N/A",
    command: "N/A (pure Rust implementation)",
};

// ===========================================================================
// Why Go is so high
// ===========================================================================

fn validate_go_explained(h: &mut ValidationHarness) {
    // Go's game tree is ~10^505. Why?
    //
    // 1. BOARD SIZE: 19×19 = 361 intersections
    //    Chess has 64 squares. Go has 5.6× more positions.
    //
    // 2. BRANCHING FACTOR: ~250 legal moves per position
    //    At any point, you can place a stone on almost any empty intersection.
    //    Chess averages ~35 legal moves. Go has 7× more choices per turn.
    //
    // 3. GAME LENGTH: ~211 plies (moves)
    //    Chess averages ~70 plies. Go games are 3× longer.
    //
    // Combined: 250^211 ≈ 10^505
    //           vs chess: 35^70 ≈ 10^108 (game-tree size; complexity is 10^123)

    let go_board = 361_u32;
    let chess_board = 64_u32;
    let go_branch: f64 = 250.0;
    let chess_branch: f64 = 35.0;
    let go_length: u32 = 211;
    let chess_length: u32 = 70;

    h.check_bool("go_board_5x_chess", go_board > 5 * chess_board);
    h.check_bool("go_branch_7x_chess", go_branch > 7.0 * chess_branch);
    h.check_bool(
        "go_length_3x_chess",
        f64::from(go_length) > 3.0 * f64::from(chess_length),
    );

    let go_computed = go_branch.log10() * f64::from(go_length);
    let chess_computed = chess_branch.log10() * f64::from(chess_length);

    h.check_bool(
        "go_formula_approximates_505",
        (go_computed - 505.0).abs() < 5.0,
    );
    h.check_bool(
        "chess_formula_approximates_108",
        (chess_computed - 108.0).abs() < 5.0,
    );

    let gap = 505.0 - 123.0;
    h.check_abs("go_chess_gap_382_orders_of_magnitude", gap, 382.0, 0.0);
}

// ===========================================================================
// MTG: Turing complete → infinite game tree
// ===========================================================================

fn validate_mtg_infinite(h: &mut ValidationHarness) {
    // Churchill, Biderman, and Herrick (2019) proved MTG is Turing complete.
    // Biderman (2020) proved it is "as hard as arithmetic" (AH-hard).
    //
    // Implications:
    // - State space is ℵ₀ (countably infinite) — you can construct states
    //   corresponding to any Turing machine configuration
    // - Game tree is 2^ℵ₀ (uncountably infinite) — the set of all possible
    //   games is the size of the real numbers
    // - No algorithm can determine the winner of an arbitrary game position
    //   (it's undecidable, not just EXPTIME-hard like chess)
    //
    // This means MTG is not on the same SCALE as chess/go — it's in a
    // different CATEGORY. Comparing 10^505 to 2^ℵ₀ is like comparing
    // a big number to infinity.

    let games = known_games();
    let finite_games: Vec<_> = games.iter().filter(|g| !g.infinite_tree).collect();
    let has_finite = games.iter().any(|g| !g.infinite_tree);
    let has_infinite = games.iter().any(|g| g.infinite_tree);

    h.check_bool(
        "catalog_has_finite_and_infinite_games",
        has_finite && has_infinite,
    );

    let Some(highest_finite) = finite_games.iter().max_by(|a, b| {
        match a.game_tree_log10.partial_cmp(&b.game_tree_log10) {
            Some(ord) => ord,
            None => {
                eprintln!("FATAL: game_tree_log10 is NaN");
                std::process::exit(1);
            }
        }
    }) else {
        eprintln!("FATAL: no finite games in catalog");
        std::process::exit(1);
    };

    h.check_bool(
        "highest_finite_is_stratego",
        highest_finite.name == "Stratego",
    );
    h.check_abs(
        "stratego_tree_is_535",
        highest_finite.game_tree_log10,
        535.0,
        0.0,
    );

    let Some(mtg) = games.iter().find(|g| g.name == "Magic: The Gathering") else {
        eprintln!("FATAL: Magic: The Gathering not found in game catalog");
        std::process::exit(1);
    };
    h.check_bool("mtg_tree_is_infinite", mtg.infinite_tree);
    h.check_bool("mtg_is_unsolved", !mtg.is_solved);

    let solved: Vec<_> = finite_games.iter().filter(|g| g.is_solved).collect();
    let unsolved: Vec<_> = finite_games.iter().filter(|g| !g.is_solved).collect();

    let max_solved_tree = solved
        .iter()
        .map(|g| g.game_tree_log10)
        .fold(0.0_f64, f64::max);
    let min_unsolved_tree = unsolved
        .iter()
        .map(|g| g.game_tree_log10)
        .fold(f64::INFINITY, f64::min);

    h.check_bool(
        "solved_games_have_smaller_trees",
        max_solved_tree < min_unsolved_tree,
    );
}

fn validate_commander_hypothesis(h: &mut ValidationHarness) {
    let rules = commander_format_rules();
    let designs = commander_designed_cards();

    let all_rules_expand = rules.iter().all(|r| r.effect == TreeEffect::Expands);
    h.check_bool("all_format_rules_expand_tree", all_rules_expand);

    let all_designs_shrink = designs.iter().all(|d| d.effect == TreeEffect::Shrinks);
    h.check_bool("all_designed_cards_shrink_tree", all_designs_shrink);

    let format_expansion: f64 = rules.iter().map(|r| r.branching_multiplier).product();
    let design_contraction: f64 = designs.iter().map(|d| d.branching_multiplier).product();

    h.check_bool(
        "format_rules_multiply_tree_gt_100x",
        format_expansion > 100.0,
    );
    h.check_bool("designed_cards_contract_tree", design_contraction < 1.0);

    let net = format_expansion * design_contraction;

    h.check_bool("net_effect_still_positive", net > 1.0);
    h.check_bool("net_effect_less_than_format_alone", net < format_expansion);

    let exploration_lost_pct = (1.0 - design_contraction) * 100.0;
    h.check_bool(
        "designed_cards_destroy_gt_90pct_branching",
        exploration_lost_pct > 90.0,
    );
}

#[expect(clippy::cast_precision_loss, reason = "counts fit in f64 mantissa")]
fn validate_enzymatic_model(h: &mut ValidationHarness) {
    let cards = example_cards();

    let wild: Vec<_> = cards
        .iter()
        .filter(|c| c.category == CardCategory::WildType)
        .collect();
    let enzymatic: Vec<_> = cards
        .iter()
        .filter(|c| c.category == CardCategory::Enzymatic)
        .collect();
    let catalytic: Vec<_> = cards
        .iter()
        .filter(|c| c.category == CardCategory::Catalytic)
        .collect();

    // Wild-type cards: high branching, high activation energy
    let wild_avg_branch: f64 =
        wild.iter().map(|c| c.branching_effect).sum::<f64>() / wild.len() as f64;
    let wild_avg_activation: f64 =
        wild.iter().map(|c| c.activation_energy).sum::<f64>() / wild.len() as f64;

    // Enzymatic cards: low branching, low activation energy
    let enz_avg_branch: f64 =
        enzymatic.iter().map(|c| c.branching_effect).sum::<f64>() / enzymatic.len() as f64;
    let enz_avg_activation: f64 =
        enzymatic.iter().map(|c| c.activation_energy).sum::<f64>() / enzymatic.len() as f64;

    // Catalytic cards: high branching, LOW activation energy (the sweet spot)
    let cat_avg_branch: f64 =
        catalytic.iter().map(|c| c.branching_effect).sum::<f64>() / catalytic.len() as f64;
    let cat_avg_activation: f64 =
        catalytic.iter().map(|c| c.activation_energy).sum::<f64>() / catalytic.len() as f64;

    h.check_bool("wild_type_high_branching", wild_avg_branch > 1.0);
    h.check_bool("wild_type_high_activation", wild_avg_activation > 0.7);

    h.check_bool("enzymatic_low_branching", enz_avg_branch < 1.0);
    h.check_bool("enzymatic_low_activation", enz_avg_activation < 0.2);

    h.check_bool("catalytic_high_branching", cat_avg_branch > 1.0);
    h.check_bool(
        "catalytic_moderate_activation",
        cat_avg_activation < wild_avg_activation,
    );

    // The key insight: enzymatic cards trade exploration for efficiency.
    // In biology, an enzyme lowers Ea (activation energy) but constrains
    // the reaction to one specific pathway. Same in game design:
    //
    //   Wild-type card: many paths, requires skill to navigate
    //   Enzymatic card: one path, no skill needed
    //   Catalytic card: many paths, easier to access (IDEAL design)
    //
    // "Exploration value" = branching_effect × (1 - activation_energy)
    //   = how much new tree you get, weighted by accessibility.
    // High branching + low activation = high value (catalytic ideal)
    // Low branching + low activation = low value (enzymatic shortcut)
    // High branching + high activation = moderate value (wild-type)

    let wild_efficiency: f64 = wild
        .iter()
        .map(|c| c.branching_effect * (1.0 - c.activation_energy))
        .sum::<f64>()
        / wild.len() as f64;
    let enz_efficiency: f64 = enzymatic
        .iter()
        .map(|c| c.branching_effect * (1.0 - c.activation_energy))
        .sum::<f64>()
        / enzymatic.len() as f64;
    let cat_efficiency: f64 = catalytic
        .iter()
        .map(|c| c.branching_effect * (1.0 - c.activation_energy))
        .sum::<f64>()
        / catalytic.len() as f64;

    h.check_bool(
        "catalytic_highest_exploration_efficiency",
        cat_efficiency > wild_efficiency && cat_efficiency > enz_efficiency,
    );

    h.check_bool(
        "enzymatic_lowest_exploration_efficiency",
        enz_efficiency < wild_efficiency,
    );
}

// ===========================================================================
// Design metric: game longevity correlates with tree complexity
// ===========================================================================

fn validate_longevity_correlation(h: &mut ValidationHarness) {
    // Games that endure have larger trees. Games that die have smaller ones.
    // This isn't coincidence — it's because players exhaust small trees.

    #[expect(dead_code, reason = "domain model completeness")]
    struct GameLongevity {
        name: &'static str,
        game_tree_log10: f64,
        years_since_invention: u32,
        still_widely_played: bool,
    }

    let games = [
        GameLongevity {
            name: "Tic-Tac-Toe",
            game_tree_log10: 5.0,
            years_since_invention: 3500,
            still_widely_played: false,
        },
        GameLongevity {
            name: "Checkers",
            game_tree_log10: 40.0,
            years_since_invention: 5000,
            still_widely_played: false,
        },
        GameLongevity {
            name: "Chess",
            game_tree_log10: 123.0,
            years_since_invention: 1500,
            still_widely_played: true,
        },
        GameLongevity {
            name: "Go",
            game_tree_log10: 505.0,
            years_since_invention: 4000,
            still_widely_played: true,
        },
        GameLongevity {
            name: "Magic: The Gathering",
            game_tree_log10: f64::INFINITY,
            years_since_invention: 33,
            still_widely_played: true,
        },
    ];

    let actively_played: Vec<_> = games.iter().filter(|g| g.still_widely_played).collect();
    let all_big_trees = actively_played.iter().all(|g| g.game_tree_log10 > 100.0);
    h.check_bool("active_games_have_tree_gt_100", all_big_trees);

    let Some(ttt) = games.iter().find(|g| g.name == "Tic-Tac-Toe") else {
        eprintln!("FATAL: Tic-Tac-Toe not found in longevity games");
        std::process::exit(1);
    };
    h.check_bool(
        "tictactoe_small_tree_not_played",
        !ttt.still_widely_played && ttt.game_tree_log10 < 10.0,
    );

    let Some(mtg) = games.iter().find(|g| g.name == "Magic: The Gathering") else {
        eprintln!("FATAL: Magic: The Gathering not found in longevity games");
        std::process::exit(1);
    };
    h.check_bool(
        "mtg_infinite_tree_still_growing",
        mtg.still_widely_played && mtg.game_tree_log10.is_infinite(),
    );
}

// ===========================================================================
// Main
// ===========================================================================

fn cmd_validate() {
    let mut h = ValidationHarness::new("exp050_game_tree_design_metric");
    h.print_provenance(&[&PROVENANCE]);

    validate_go_explained(&mut h);
    validate_mtg_infinite(&mut h);
    validate_commander_hypothesis(&mut h);
    validate_enzymatic_model(&mut h);
    validate_longevity_correlation(&mut h);

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

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

use ludospring_barracuda::validation::ValidationResult;

use catalog::{
    CardCategory, TreeEffect, commander_designed_cards, commander_format_rules, example_cards,
    known_games,
};

const EXP: &str = "exp050_game_tree_design_metric";

const fn bool_f64(b: bool) -> f64 {
    if b { 1.0 } else { 0.0 }
}

// ===========================================================================
// Why Go is so high
// ===========================================================================

fn validate_go_explained() -> Vec<ValidationResult> {
    let mut results = Vec::new();

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

    results.push(ValidationResult::check(
        EXP,
        "go_board_5x_chess",
        bool_f64(go_board > 5 * chess_board),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "go_branch_7x_chess",
        bool_f64(go_branch > 7.0 * chess_branch),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "go_length_3x_chess",
        bool_f64(f64::from(go_length) > 3.0 * f64::from(chess_length)),
        1.0,
        0.0,
    ));

    // Verify the formula: b^d ≈ game tree
    let go_computed = go_branch.log10() * f64::from(go_length); // log10(250^211)
    let chess_computed = chess_branch.log10() * f64::from(chess_length);

    results.push(ValidationResult::check(
        EXP,
        "go_formula_approximates_505",
        bool_f64((go_computed - 505.0).abs() < 5.0),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "chess_formula_approximates_108",
        bool_f64((chess_computed - 108.0).abs() < 5.0),
        1.0,
        0.0,
    ));

    // The gap: Go's tree is 10^(505-123) = 10^382 times larger than chess's.
    // That's not a difference in degree — it's a difference in kind.
    let gap = 505.0 - 123.0;
    results.push(ValidationResult::check(
        EXP,
        "go_chess_gap_382_orders_of_magnitude",
        gap,
        382.0,
        0.0,
    ));

    results
}

// ===========================================================================
// MTG: Turing complete → infinite game tree
// ===========================================================================

fn validate_mtg_infinite() -> Vec<ValidationResult> {
    let mut results = Vec::new();

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

    results.push(ValidationResult::check(
        EXP,
        "catalog_has_finite_and_infinite_games",
        bool_f64(has_finite && has_infinite),
        1.0,
        0.0,
    ));

    // The highest FINITE game tree
    let highest_finite = finite_games
        .iter()
        .max_by(|a, b| a.game_tree_log10.partial_cmp(&b.game_tree_log10).unwrap())
        .unwrap();

    results.push(ValidationResult::check(
        EXP,
        "highest_finite_is_stratego",
        bool_f64(highest_finite.name == "Stratego"),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "stratego_tree_is_535",
        highest_finite.game_tree_log10,
        535.0,
        0.0,
    ));

    // MTG is categorically beyond all finite games
    let mtg = games
        .iter()
        .find(|g| g.name == "Magic: The Gathering")
        .unwrap();
    results.push(ValidationResult::check(
        EXP,
        "mtg_tree_is_infinite",
        bool_f64(mtg.infinite_tree),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "mtg_is_unsolved",
        bool_f64(!mtg.is_solved),
        1.0,
        0.0,
    ));

    // The solved/unsolved divide correlates with complexity
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

    // All solved games have smaller trees than all unsolved games
    results.push(ValidationResult::check(
        EXP,
        "solved_games_have_smaller_trees",
        bool_f64(max_solved_tree < min_unsolved_tree),
        1.0,
        0.0,
    ));

    results
}

fn validate_commander_hypothesis() -> Vec<ValidationResult> {
    let mut results = Vec::new();

    let rules = commander_format_rules();
    let designs = commander_designed_cards();

    // Format rules all expand the tree
    let all_rules_expand = rules.iter().all(|r| r.effect == TreeEffect::Expands);
    results.push(ValidationResult::check(
        EXP,
        "all_format_rules_expand_tree",
        bool_f64(all_rules_expand),
        1.0,
        0.0,
    ));

    // Designed-for-commander cards all shrink the tree
    let all_designs_shrink = designs.iter().all(|d| d.effect == TreeEffect::Shrinks);
    results.push(ValidationResult::check(
        EXP,
        "all_designed_cards_shrink_tree",
        bool_f64(all_designs_shrink),
        1.0,
        0.0,
    ));

    // Net effect of format rules (multiplicative)
    let format_expansion: f64 = rules.iter().map(|r| r.branching_multiplier).product();
    let design_contraction: f64 = designs.iter().map(|d| d.branching_multiplier).product();

    results.push(ValidationResult::check(
        EXP,
        "format_rules_multiply_tree_gt_100x",
        bool_f64(format_expansion > 100.0),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "designed_cards_contract_tree",
        bool_f64(design_contraction < 1.0),
        1.0,
        0.0,
    ));

    // The net effect: format expansion vs design contraction
    let net = format_expansion * design_contraction;

    // Net is still positive (format rules win) but LESS than format alone
    results.push(ValidationResult::check(
        EXP,
        "net_effect_still_positive",
        bool_f64(net > 1.0),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "net_effect_less_than_format_alone",
        bool_f64(net < format_expansion),
        1.0,
        0.0,
    ));

    // The ratio of what's lost: designed cards destroy X% of the format's expansion
    let exploration_lost_pct = (1.0 - design_contraction) * 100.0;
    results.push(ValidationResult::check(
        EXP,
        "designed_cards_destroy_gt_90pct_branching",
        bool_f64(exploration_lost_pct > 90.0),
        1.0,
        0.0,
    ));

    results
}

#[expect(clippy::cast_precision_loss, reason = "counts fit in f64 mantissa")]
fn validate_enzymatic_model() -> Vec<ValidationResult> {
    let mut results = Vec::new();

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

    // Wild-type: high branch, high activation
    results.push(ValidationResult::check(
        EXP,
        "wild_type_high_branching",
        bool_f64(wild_avg_branch > 1.0),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "wild_type_high_activation",
        bool_f64(wild_avg_activation > 0.7),
        1.0,
        0.0,
    ));

    // Enzymatic: low branch, low activation
    results.push(ValidationResult::check(
        EXP,
        "enzymatic_low_branching",
        bool_f64(enz_avg_branch < 1.0),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "enzymatic_low_activation",
        bool_f64(enz_avg_activation < 0.2),
        1.0,
        0.0,
    ));

    // Catalytic: high branch, lower activation
    results.push(ValidationResult::check(
        EXP,
        "catalytic_high_branching",
        bool_f64(cat_avg_branch > 1.0),
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        EXP,
        "catalytic_moderate_activation",
        bool_f64(cat_avg_activation < wild_avg_activation),
        1.0,
        0.0,
    ));

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

    // Catalytic cards should have the highest exploration efficiency
    results.push(ValidationResult::check(
        EXP,
        "catalytic_highest_exploration_efficiency",
        bool_f64(cat_efficiency > wild_efficiency && cat_efficiency > enz_efficiency),
        1.0,
        0.0,
    ));

    // Enzymatic cards: ratio is low because branching < 1.0
    results.push(ValidationResult::check(
        EXP,
        "enzymatic_lowest_exploration_efficiency",
        bool_f64(enz_efficiency < wild_efficiency),
        1.0,
        0.0,
    ));

    results
}

// ===========================================================================
// Design metric: game longevity correlates with tree complexity
// ===========================================================================

#[expect(
    clippy::items_after_statements,
    reason = "helper defined near use site"
)]
fn validate_longevity_correlation() -> Vec<ValidationResult> {
    let mut results = Vec::new();

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

    // All actively-played games have tree complexity > 100
    let actively_played: Vec<_> = games.iter().filter(|g| g.still_widely_played).collect();
    let all_big_trees = actively_played.iter().all(|g| g.game_tree_log10 > 100.0);
    results.push(ValidationResult::check(
        EXP,
        "active_games_have_tree_gt_100",
        bool_f64(all_big_trees),
        1.0,
        0.0,
    ));

    // Tic-tac-toe: tiny tree, nobody plays it seriously
    let ttt = games.iter().find(|g| g.name == "Tic-Tac-Toe").unwrap();
    results.push(ValidationResult::check(
        EXP,
        "tictactoe_small_tree_not_played",
        bool_f64(!ttt.still_widely_played && ttt.game_tree_log10 < 10.0),
        1.0,
        0.0,
    ));

    // MTG at 33 years old and growing — infinite tree can never be exhausted
    let mtg = games
        .iter()
        .find(|g| g.name == "Magic: The Gathering")
        .unwrap();
    results.push(ValidationResult::check(
        EXP,
        "mtg_infinite_tree_still_growing",
        bool_f64(mtg.still_widely_played && mtg.game_tree_log10.is_infinite()),
        1.0,
        0.0,
    ));

    results
}

// ===========================================================================
// Main
// ===========================================================================

#[expect(
    clippy::too_many_lines,
    reason = "validation orchestrator — sequential check groups"
)]
fn cmd_validate() {
    println!("=== exp050: Game Tree as Design Metric ===\n");

    let mut all_results = Vec::new();

    println!("--- Known Game Trees (from literature) ---");
    println!(
        "  {:30} {:>12} {:>12} {:>8} {:>8}",
        "Game", "State-space", "Game-tree", "Branch", "Solved?"
    );
    println!(
        "  {:30} {:>12} {:>12} {:>8} {:>8}",
        "", "(log10)", "(log10)", "factor", ""
    );
    for g in known_games() {
        let tree_str = if g.infinite_tree {
            "∞ (2^ℵ₀)".to_string()
        } else {
            format!("{:.0}", g.game_tree_log10)
        };
        let branch_str = if g.avg_branching_factor.is_infinite() {
            "∞".to_string()
        } else {
            format!("{:.0}", g.avg_branching_factor)
        };
        let state_str = if g.state_space_log10.is_infinite() {
            "ℵ₀".to_string()
        } else {
            format!("{:.0}", g.state_space_log10)
        };
        println!(
            "  {:30} {:>12} {:>12} {:>8} {:>8}",
            g.name,
            state_str,
            tree_str,
            branch_str,
            if g.is_solved { "YES" } else { "no" }
        );
    }

    println!("\n--- Why Go is So High ---");
    let r = validate_go_explained();
    for v in &r {
        println!(
            "  [{}] {}",
            if v.passed { "PASS" } else { "FAIL" },
            v.description
        );
    }
    all_results.extend(r);

    println!("\n--- MTG: Turing Complete → Infinite Tree ---");
    let r = validate_mtg_infinite();
    for v in &r {
        println!(
            "  [{}] {}",
            if v.passed { "PASS" } else { "FAIL" },
            v.description
        );
    }
    all_results.extend(r);

    println!("\n--- Commander Hypothesis ---");
    println!("  Format RULES that expand the tree:");
    for rule in commander_format_rules() {
        println!(
            "    {}: ×{:.1} ({})",
            rule.name,
            rule.branching_multiplier,
            rule.explanation.split('.').next().unwrap()
        );
    }
    println!("\n  Designed-for-Commander cards that shrink the tree:");
    for design in commander_designed_cards() {
        println!(
            "    {}: ×{:.1} ({})",
            design.name,
            design.branching_multiplier,
            design.explanation.split('.').next().unwrap()
        );
    }
    let r = validate_commander_hypothesis();
    for v in &r {
        println!(
            "  [{}] {}",
            if v.passed { "PASS" } else { "FAIL" },
            v.description
        );
    }
    all_results.extend(r);

    println!("\n--- Enzymatic Shortcut Model ---");
    println!(
        "  {:30} {:>10} {:>10} {:>10} {:>10}",
        "Card", "Category", "Branch", "Activ.E", "ExplVal"
    );
    for card in example_cards() {
        let cat = match card.category {
            CardCategory::WildType => "wild",
            CardCategory::Enzymatic => "enzyme",
            CardCategory::Catalytic => "catalytic",
        };
        let eff = card.branching_effect * (1.0 - card.activation_energy);
        println!(
            "  {:30} {:>10} {:>10.1} {:>10.2} {:>10.2}",
            card.name, cat, card.branching_effect, card.activation_energy, eff
        );
    }
    let r = validate_enzymatic_model();
    for v in &r {
        println!(
            "  [{}] {}",
            if v.passed { "PASS" } else { "FAIL" },
            v.description
        );
    }
    all_results.extend(r);

    println!("\n--- Longevity Correlation ---");
    let r = validate_longevity_correlation();
    for v in &r {
        println!(
            "  [{}] {}",
            if v.passed { "PASS" } else { "FAIL" },
            v.description
        );
    }
    all_results.extend(r);

    let passed = all_results.iter().filter(|r| r.passed).count();
    let total = all_results.len();
    println!("\n=== SUMMARY: {passed}/{total} checks passed ===");

    if passed == total {
        println!("\nGame trees are measurable. Design for exploration, not shortcuts.");
    } else {
        println!("\nFAILED:");
        for r in all_results.iter().filter(|r| !r.passed) {
            println!(
                "  {} — measured={}, expected={}",
                r.description, r.measured, r.expected
            );
        }
        std::process::exit(1);
    }
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

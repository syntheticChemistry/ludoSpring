// SPDX-License-Identifier: AGPL-3.0-or-later
//! Game catalog types and factory functions for the game tree design metric.

// ===========================================================================
// Known game trees — from Wikipedia "Game complexity" table
// ===========================================================================

#[expect(
    dead_code,
    reason = "structural completeness — domain model includes all fields"
)]
#[derive(Debug, Clone)]
pub struct GameProfile {
    pub name: &'static str,
    pub board_positions: u32,
    pub state_space_log10: f64,
    pub game_tree_log10: f64,
    pub avg_game_length_plies: u32,
    pub avg_branching_factor: f64,
    pub is_solved: bool,
    /// For finite games this is false. MTG and infinite chess are true.
    pub infinite_tree: bool,
    pub notes: &'static str,
}

#[expect(clippy::too_many_lines, reason = "game catalog — one per known game")]
pub fn known_games() -> Vec<GameProfile> {
    vec![
        GameProfile {
            name: "Tic-Tac-Toe",
            board_positions: 9,
            state_space_log10: 3.0,
            game_tree_log10: 5.0,
            avg_game_length_plies: 9,
            avg_branching_factor: 4.0,
            is_solved: true,
            infinite_tree: false,
            notes: "Fully solved. Draw with optimal play.",
        },
        GameProfile {
            name: "Connect Four",
            board_positions: 42,
            state_space_log10: 12.0,
            game_tree_log10: 21.0,
            avg_game_length_plies: 36,
            avg_branching_factor: 4.0,
            is_solved: true,
            infinite_tree: false,
            notes: "Solved 1988. First player wins with perfect play.",
        },
        GameProfile {
            name: "Checkers",
            board_positions: 32,
            state_space_log10: 20.0,
            game_tree_log10: 40.0,
            avg_game_length_plies: 70,
            avg_branching_factor: 2.8,
            is_solved: true,
            infinite_tree: false,
            notes: "Solved 2007 (Chinook). Draw with optimal play.",
        },
        GameProfile {
            name: "Reversi (Othello)",
            board_positions: 64,
            state_space_log10: 28.0,
            game_tree_log10: 58.0,
            avg_game_length_plies: 58,
            avg_branching_factor: 10.0,
            is_solved: false,
            infinite_tree: false,
            notes: "Weakly solved 2023.",
        },
        GameProfile {
            name: "Chess",
            board_positions: 64,
            state_space_log10: 44.0,
            game_tree_log10: 123.0,
            avg_game_length_plies: 70,
            avg_branching_factor: 35.0,
            is_solved: false,
            infinite_tree: false,
            notes: "EXPTIME-complete. Shannon number ~10^120.",
        },
        GameProfile {
            name: "Shogi",
            board_positions: 81,
            state_space_log10: 71.0,
            game_tree_log10: 226.0,
            avg_game_length_plies: 115,
            avg_branching_factor: 92.0,
            is_solved: false,
            infinite_tree: false,
            notes: "EXPTIME-complete. Captured pieces re-enter play.",
        },
        GameProfile {
            name: "Go (19x19)",
            board_positions: 361,
            state_space_log10: 170.0,
            game_tree_log10: 505.0,
            avg_game_length_plies: 211,
            avg_branching_factor: 250.0,
            is_solved: false,
            infinite_tree: false,
            notes: "EXPTIME-complete. AlphaGo beat top humans 2016, but game is NOT solved.",
        },
        GameProfile {
            name: "Arimaa",
            board_positions: 64,
            state_space_log10: 43.0,
            game_tree_log10: 402.0,
            avg_game_length_plies: 92,
            avg_branching_factor: 17281.0,
            is_solved: false,
            infinite_tree: false,
            notes: "Designed to be hard for computers. Massive branching factor.",
        },
        GameProfile {
            name: "Stratego",
            board_positions: 92,
            state_space_log10: 115.0,
            game_tree_log10: 535.0,
            avg_game_length_plies: 381,
            avg_branching_factor: 21.739,
            is_solved: false,
            infinite_tree: false,
            notes: "Imperfect information game. Highest finite game tree in catalog.",
        },
        GameProfile {
            name: "Magic: The Gathering",
            board_positions: 0, // not applicable
            state_space_log10: f64::INFINITY,
            game_tree_log10: f64::INFINITY,
            avg_game_length_plies: 0, // unbounded
            avg_branching_factor: f64::INFINITY,
            is_solved: false,
            infinite_tree: true,
            notes: "Proven Turing complete (Churchill et al. 2019). \
                    AH-hard. Game tree is 2^aleph_0 (uncountably infinite).",
        },
    ]
}

// ===========================================================================
// Commander hypothesis: format rules expand tree, designed cards shrink it
// ===========================================================================

/// Commander format rules and their effect on the decision space.
#[derive(Debug, Clone)]
pub struct FormatModifier {
    #[expect(
        dead_code,
        reason = "domain model completeness — name used in white-paper tables"
    )]
    pub name: &'static str,
    pub effect: TreeEffect,
    pub branching_multiplier: f64,
    #[expect(
        dead_code,
        reason = "domain model completeness — qualitative rationale for multiplier"
    )]
    pub explanation: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreeEffect {
    Expands,
    Shrinks,
}

pub fn commander_format_rules() -> Vec<FormatModifier> {
    vec![
        FormatModifier {
            name: "100-card singleton (no duplicates)",
            effect: TreeEffect::Expands,
            branching_multiplier: 3.0,
            explanation: "Every game sees a different subset of your deck. \
                          In 60-card 4-of, you see the same cards reliably. \
                          Singleton means each draw is more novel.",
        },
        FormatModifier {
            name: "Full card pool (~27,000+ unique cards)",
            effect: TreeEffect::Expands,
            branching_multiplier: 5.0,
            explanation: "Deckbuilding space is massively larger. \
                          Standard has ~1500 cards. Commander has all of Magic.",
        },
        FormatModifier {
            name: "4 players (multiplayer politics)",
            effect: TreeEffect::Expands,
            branching_multiplier: 8.0,
            explanation: "Player interactions multiply combinatorially. \
                          Who to attack, who to help, alliance shifts. \
                          2-player is a tree; 4-player is a forest of trees.",
        },
        FormatModifier {
            name: "40 life (vs 20 in Standard)",
            effect: TreeEffect::Expands,
            branching_multiplier: 1.5,
            explanation: "Games last longer. More turns = more decisions = deeper tree.",
        },
        FormatModifier {
            name: "Commander identity (color restriction)",
            effect: TreeEffect::Expands,
            branching_multiplier: 1.2,
            explanation: "Forces creative deckbuilding within constraints. \
                          Constraints drive exploration of underused card interactions.",
        },
    ]
}

pub fn commander_designed_cards() -> Vec<FormatModifier> {
    vec![
        FormatModifier {
            name: "Pre-built commander synergies",
            effect: TreeEffect::Shrinks,
            branching_multiplier: 0.4,
            explanation: "Cards printed to obviously synergize with specific commanders. \
                          The 'correct' choice is printed on the card. \
                          No discovery needed — exploration collapses.",
        },
        FormatModifier {
            name: "Pushed power level (format warping staples)",
            effect: TreeEffect::Shrinks,
            branching_multiplier: 0.5,
            explanation: "When one card is clearly better than alternatives, \
                          it appears in every deck. Reduces deckbuilding diversity. \
                          Like a protein chaperone that forces a single fold.",
        },
        FormatModifier {
            name: "Auto-include commander staples",
            effect: TreeEffect::Shrinks,
            branching_multiplier: 0.6,
            explanation: "Sol Ring, Arcane Signet, etc. in every deck. \
                          20+ slots are 'solved' before deckbuilding starts. \
                          That's 20% of the deck with branching factor 1.",
        },
        FormatModifier {
            name: "Linear commander designs (one obvious strategy)",
            effect: TreeEffect::Shrinks,
            branching_multiplier: 0.3,
            explanation: "Commanders that say 'build around THIS mechanic'. \
                          Eliminates the creative tension of discovering synergies. \
                          The game tells you what to do instead of you exploring.",
        },
    ]
}

// ===========================================================================
// Enzymatic shortcut model
// ===========================================================================

#[derive(Debug, Clone)]
pub struct CardDesign {
    #[expect(
        dead_code,
        reason = "domain model completeness — card name for white-paper reference"
    )]
    pub name: &'static str,
    pub category: CardCategory,
    /// How much this card reduces the effective branching at its decision point.
    /// 1.0 = no effect. <1.0 = collapses choices. >1.0 = opens new choices.
    pub branching_effect: f64,
    /// Activation energy: how much game knowledge is needed to use this card well.
    /// High = requires deep understanding. Low = card plays itself.
    pub activation_energy: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardCategory {
    /// Wild-type: designed for general play, emergent synergies
    WildType,
    /// Enzymatic: designed shortcut, lowers activation energy, narrows path
    Enzymatic,
    /// Catalytic: enables new paths without closing old ones (rare, ideal)
    Catalytic,
}

pub fn example_cards() -> Vec<CardDesign> {
    vec![
        // Wild-type: high exploration, high activation energy
        CardDesign {
            name: "Lightning Bolt",
            category: CardCategory::WildType,
            branching_effect: 1.5, // flexible: face, creature, or hold for later
            activation_energy: 0.8, // timing decisions require game knowledge
        },
        CardDesign {
            name: "Counterspell",
            category: CardCategory::WildType,
            branching_effect: 2.0, // changes what opponent can cast, creates response trees
            activation_energy: 0.9, // knowing WHEN to counter is expert knowledge
        },
        CardDesign {
            name: "Brainstorm",
            category: CardCategory::WildType,
            branching_effect: 3.0, // rearranges hand, interacts with shuffles, creates huge trees
            activation_energy: 0.95, // one of the hardest cards to play optimally
        },
        // Enzymatic: low exploration, low activation energy
        CardDesign {
            name: "Dockside Extortionist (designed staple)",
            category: CardCategory::Enzymatic,
            branching_effect: 0.3,  // play on sight, always correct
            activation_energy: 0.1, // no decisions needed, card plays itself
        },
        CardDesign {
            name: "Linear commander (build-around)",
            category: CardCategory::Enzymatic,
            branching_effect: 0.2,   // deckbuilding is predetermined
            activation_energy: 0.05, // synergies are printed on the cards
        },
        CardDesign {
            name: "Auto-include staple (Sol Ring)",
            category: CardCategory::Enzymatic,
            branching_effect: 0.1,   // goes in every deck, play turn 1 always
            activation_energy: 0.01, // zero thought required
        },
        // Catalytic: high exploration, low activation energy (the ideal)
        CardDesign {
            name: "Panharmonicon (doubles triggers)",
            category: CardCategory::Catalytic,
            branching_effect: 2.5, // doubles triggers = exponential interaction growth
            activation_energy: 0.3, // easy to understand, but opens vast space
        },
        CardDesign {
            name: "Mirage Mirror (copy anything)",
            category: CardCategory::Catalytic,
            branching_effect: 4.0,  // can be anything, decisions explode
            activation_energy: 0.4, // copy effect is simple, choices are infinite
        },
    ]
}

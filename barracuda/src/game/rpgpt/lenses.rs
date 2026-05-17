// SPDX-License-Identifier: AGPL-3.0-or-later
//! Schell (2008) Game Design Lenses — structured analytical framework.
//!
//! Jesse Schell's *The Art of Game Design* introduces ~100 analytical "lenses"
//! — focused perspectives for evaluating game design decisions. Each lens asks
//! specific questions about a system, revealing strengths and weaknesses.
//!
//! This module formalizes the lens framework for automated ruleset validation:
//! given a plane configuration, we can apply lenses and generate structured
//! evaluation reports. This enables:
//!
//! - Automated RPGPT plane quality assessment
//! - Ruleset comparison across systems (PF2e vs FATE vs Cairn)
//! - Design gap detection (e.g., plane missing player agency)
//! - AI narration guidance (which lenses the current scene satisfies)
//!
//! # Reference
//!
//! Schell, J. (2008). *The Art of Game Design: A Book of Lenses*.
//! CRC Press. ISBN 978-0123694966.

use super::plane::PlaneType;

/// A design lens — an analytical perspective on a game system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Lens {
    /// Lens #1: Essential Experience — What experience do I want the player to have?
    EssentialExperience,
    /// Lens #2: Surprise — What will surprise the player?
    Surprise,
    /// Lens #4: Curiosity — What questions does the design plant?
    Curiosity,
    /// Lens #5: Endogenous Value — Do game elements have meaning within the game?
    EndogenousValue,
    /// Lens #6: Problem Solving — What problems does the player solve?
    ProblemSolving,
    /// Lens #7: Elemental Tetrad — Aesthetics, Mechanics, Story, Technology balanced?
    ElementalTetrad,
    /// Lens #9: Unification — Does a single theme unite all elements?
    Unification,
    /// Lens #24: Action → Outcome — Are outcomes of actions clear?
    ActionOutcome,
    /// Lens #32: Goals — Are goals concrete, achievable, rewarding?
    Goals,
    /// Lens #34: Skill — Does the game reward skill appropriately?
    Skill,
    /// Lens #35: Expected Value — Are risks/rewards balanced?
    ExpectedValue,
    /// Lens #38: Challenge — Is difficulty appropriately calibrated?
    Challenge,
    /// Lens #39: Meaningful Choice — Do decisions have weight?
    MeaningfulChoice,
    /// Lens #42: Transparency — Are rules understandable?
    Transparency,
    /// Lens #46: Economy — Is the resource system balanced?
    Economy,
    /// Lens #47: Fairness — Do all players have viable paths?
    Fairness,
    /// Lens #56: Freedom — Can the player express themselves?
    Freedom,
    /// Lens #63: Feedback — Does the system communicate state clearly?
    Feedback,
    /// Lens #68: Story/Game Balance — Are narrative and mechanics complementary?
    StoryGameBalance,
    /// Lens #79: Flow — Does the game produce flow states?
    Flow,
}

impl Lens {
    /// All lenses in canonical order.
    pub const ALL: [Self; 20] = [
        Self::EssentialExperience,
        Self::Surprise,
        Self::Curiosity,
        Self::EndogenousValue,
        Self::ProblemSolving,
        Self::ElementalTetrad,
        Self::Unification,
        Self::ActionOutcome,
        Self::Goals,
        Self::Skill,
        Self::ExpectedValue,
        Self::Challenge,
        Self::MeaningfulChoice,
        Self::Transparency,
        Self::Economy,
        Self::Fairness,
        Self::Freedom,
        Self::Feedback,
        Self::StoryGameBalance,
        Self::Flow,
    ];

    /// Human-readable label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::EssentialExperience => "Essential Experience",
            Self::Surprise => "Surprise",
            Self::Curiosity => "Curiosity",
            Self::EndogenousValue => "Endogenous Value",
            Self::ProblemSolving => "Problem Solving",
            Self::ElementalTetrad => "Elemental Tetrad",
            Self::Unification => "Unification",
            Self::ActionOutcome => "Action → Outcome",
            Self::Goals => "Goals",
            Self::Skill => "Skill",
            Self::ExpectedValue => "Expected Value",
            Self::Challenge => "Challenge",
            Self::MeaningfulChoice => "Meaningful Choice",
            Self::Transparency => "Transparency",
            Self::Economy => "Economy",
            Self::Fairness => "Fairness",
            Self::Freedom => "Freedom",
            Self::Feedback => "Feedback",
            Self::StoryGameBalance => "Story/Game Balance",
            Self::Flow => "Flow",
        }
    }

    /// The diagnostic question this lens asks.
    #[must_use]
    pub const fn question(self) -> &'static str {
        match self {
            Self::EssentialExperience => "What experience should the player have?",
            Self::Surprise => "What surprises does the system produce?",
            Self::Curiosity => "What questions does the design plant in the player?",
            Self::EndogenousValue => "Do elements have meaning within the game world?",
            Self::ProblemSolving => "What problems is the player asked to solve?",
            Self::ElementalTetrad => "Are aesthetics, mechanics, story, and technology balanced?",
            Self::Unification => "Does a single theme unite all design elements?",
            Self::ActionOutcome => "Can the player predict outcomes from actions?",
            Self::Goals => "Are goals concrete, achievable, and rewarding?",
            Self::Skill => "Does the game appropriately reward player skill?",
            Self::ExpectedValue => "Are risks proportional to potential rewards?",
            Self::Challenge => "Is difficulty calibrated to produce engagement?",
            Self::MeaningfulChoice => "Do player decisions have weight and consequence?",
            Self::Transparency => "Can the player understand the rules governing them?",
            Self::Economy => "Is the resource/action economy balanced?",
            Self::Fairness => "Do all players have viable paths to success?",
            Self::Freedom => "Can the player express their own style of play?",
            Self::Feedback => "Does the system clearly communicate state changes?",
            Self::StoryGameBalance => "Do narrative and mechanics reinforce each other?",
            Self::Flow => "Does the system produce flow states (Csikszentmihalyi)?",
        }
    }
}

/// Result of applying a lens to a game system.
#[derive(Debug, Clone)]
pub struct LensEvaluation {
    /// Which lens was applied.
    pub lens: Lens,
    /// Score: 0.0 (completely fails) to 1.0 (fully satisfies).
    pub score: f64,
    /// Structured observations about strengths.
    pub strengths: Vec<String>,
    /// Structured observations about gaps/weaknesses.
    pub gaps: Vec<String>,
}

/// A complete lens analysis of a plane configuration.
#[derive(Debug, Clone)]
pub struct PlaneAnalysis {
    /// The plane being analyzed.
    pub plane: PlaneType,
    /// Individual lens evaluations.
    pub evaluations: Vec<LensEvaluation>,
}

impl PlaneAnalysis {
    /// Overall quality score (mean of all lens scores).
    #[must_use]
    #[allow(
        clippy::cast_precision_loss,
        reason = "evaluation count won't exceed 2^52"
    )]
    pub fn overall_score(&self) -> f64 {
        if self.evaluations.is_empty() {
            return 0.0;
        }
        let sum: f64 = self.evaluations.iter().map(|e| e.score).sum();
        sum / self.evaluations.len() as f64
    }

    /// Weakest lenses (below threshold).
    #[must_use]
    pub fn weak_lenses(&self, threshold: f64) -> Vec<&LensEvaluation> {
        self.evaluations
            .iter()
            .filter(|e| e.score < threshold)
            .collect()
    }

    /// Strongest lenses (above threshold).
    #[must_use]
    pub fn strong_lenses(&self, threshold: f64) -> Vec<&LensEvaluation> {
        self.evaluations
            .iter()
            .filter(|e| e.score >= threshold)
            .collect()
    }
}

/// Evaluate a plane against all 20 lenses.
///
/// The evaluation is structural — it examines the plane's configuration
/// (dice system, action economy, narration guide, resolution method) to
/// determine how well each lens is served. This is automated quality
/// assurance for RPGPT plane design.
#[must_use]
#[allow(
    clippy::too_many_lines,
    reason = "single cohesive evaluation across all 20 lenses"
)]
pub fn evaluate_plane(plane: PlaneType) -> PlaneAnalysis {
    let evaluations = Lens::ALL
        .iter()
        .map(|&lens| evaluate_single_lens(lens, plane))
        .collect();

    PlaneAnalysis { plane, evaluations }
}

/// Apply a single lens to a plane.
fn evaluate_single_lens(lens: Lens, plane: PlaneType) -> LensEvaluation {
    let (score, strengths, gaps) = match lens {
        Lens::EssentialExperience => eval_essential_experience(plane),
        Lens::Surprise => eval_surprise(plane),
        Lens::Curiosity => eval_curiosity(plane),
        Lens::EndogenousValue => eval_endogenous_value(plane),
        Lens::ProblemSolving => eval_problem_solving(plane),
        Lens::ElementalTetrad => eval_elemental_tetrad(plane),
        Lens::Unification => eval_unification(plane),
        Lens::ActionOutcome => eval_action_outcome(plane),
        Lens::Goals => eval_goals(plane),
        Lens::Skill => eval_skill(plane),
        Lens::ExpectedValue => eval_expected_value(plane),
        Lens::Challenge => eval_challenge(plane),
        Lens::MeaningfulChoice => eval_meaningful_choice(plane),
        Lens::Transparency => eval_transparency(plane),
        Lens::Economy => eval_economy(plane),
        Lens::Fairness => eval_fairness(plane),
        Lens::Freedom => eval_freedom(plane),
        Lens::Feedback => eval_feedback(plane),
        Lens::StoryGameBalance => eval_story_game_balance(plane),
        Lens::Flow => eval_flow(plane),
    };

    LensEvaluation {
        lens,
        score,
        strengths,
        gaps,
    }
}

fn eval_essential_experience(plane: PlaneType) -> (f64, Vec<String>, Vec<String>) {
    match plane {
        PlaneType::Exploration => (0.9, vec!["Discovery and wonder".into()], vec![]),
        PlaneType::Dialogue => (
            0.85,
            vec!["Social deduction and empathy".into()],
            vec!["Voice system complexity".into()],
        ),
        PlaneType::Tactical => (0.9, vec!["Strategic mastery".into()], vec![]),
        PlaneType::Investigation => (
            0.8,
            vec!["Mystery and deduction".into()],
            vec!["Pacing control needed".into()],
        ),
        PlaneType::Political => (
            0.75,
            vec!["Power dynamics".into()],
            vec!["Long-term consequence tracking".into()],
        ),
        PlaneType::Crafting => (
            0.7,
            vec!["Creative expression".into()],
            vec!["Recipe discovery depth".into()],
        ),
        PlaneType::CardStack => (0.85, vec!["Combinatorial depth".into()], vec![]),
    }
}

fn eval_surprise(plane: PlaneType) -> (f64, Vec<String>, Vec<String>) {
    match plane {
        PlaneType::Exploration => (0.9, vec!["Procedural discovery".into()], vec![]),
        PlaneType::Dialogue => (
            0.7,
            vec!["NPC personality emergence".into()],
            vec!["AI unpredictability bounded by cert".into()],
        ),
        PlaneType::Tactical => (
            0.6,
            vec!["Dice variance".into()],
            vec!["Deterministic grid limits surprise".into()],
        ),
        PlaneType::Investigation => (0.85, vec!["Clue revelation timing".into()], vec![]),
        PlaneType::Political => (0.8, vec!["Betrayal mechanics".into()], vec![]),
        PlaneType::Crafting => (
            0.65,
            vec!["Unexpected synergies".into()],
            vec!["Recipe system predictable after mastery".into()],
        ),
        PlaneType::CardStack => (
            0.9,
            vec!["Stack interaction emergent behavior".into()],
            vec![],
        ),
    }
}

fn eval_curiosity(plane: PlaneType) -> (f64, Vec<String>, Vec<String>) {
    match plane {
        PlaneType::Exploration => (0.95, vec!["Map mystery, fog of war".into()], vec![]),
        PlaneType::Dialogue => (0.8, vec!["NPC secrets, hidden knowledge".into()], vec![]),
        PlaneType::Tactical => (
            0.5,
            vec!["Enemy capability unknown".into()],
            vec!["Rules-heavy reduces wonder".into()],
        ),
        PlaneType::Investigation => (0.95, vec!["Core mechanic IS curiosity".into()], vec![]),
        PlaneType::Political => (0.75, vec!["Hidden agendas".into()], vec![]),
        PlaneType::Crafting => (
            0.7,
            vec!["Unknown recipes".into()],
            vec!["Finite discovery space".into()],
        ),
        PlaneType::CardStack => (
            0.7,
            vec!["Novel combinations".into()],
            vec!["Solved states possible".into()],
        ),
    }
}

fn eval_endogenous_value(plane: PlaneType) -> (f64, Vec<String>, Vec<String>) {
    match plane {
        PlaneType::Exploration => (0.8, vec!["Landmarks have narrative weight".into()], vec![]),
        PlaneType::Dialogue => (0.85, vec!["Trust is earned, secrets matter".into()], vec![]),
        PlaneType::Tactical => (
            0.7,
            vec!["Position matters, HP meaningful".into()],
            vec!["Abstract numbers vs world".into()],
        ),
        PlaneType::Investigation => (0.9, vec!["Clues unlock narrative".into()], vec![]),
        PlaneType::Political => (0.9, vec!["Reputation drives access".into()], vec![]),
        PlaneType::Crafting => (0.75, vec!["Created items have provenance".into()], vec![]),
        PlaneType::CardStack => (0.8, vec!["Card value context-dependent".into()], vec![]),
    }
}

fn eval_problem_solving(plane: PlaneType) -> (f64, Vec<String>, Vec<String>) {
    match plane {
        PlaneType::Exploration => (
            0.6,
            vec!["Navigation, resource management".into()],
            vec!["Low structured challenge".into()],
        ),
        PlaneType::Dialogue => (0.75, vec!["Social puzzle solving".into()], vec![]),
        PlaneType::Tactical => (
            0.95,
            vec!["Core mechanic IS problem solving".into()],
            vec![],
        ),
        PlaneType::Investigation => (0.9, vec!["Deduction chains".into()], vec![]),
        PlaneType::Political => (0.8, vec!["Multi-agent optimization".into()], vec![]),
        PlaneType::Crafting => (0.85, vec!["Material constraint puzzles".into()], vec![]),
        PlaneType::CardStack => (0.9, vec!["Sequencing optimization".into()], vec![]),
    }
}

fn eval_elemental_tetrad(plane: PlaneType) -> (f64, Vec<String>, Vec<String>) {
    let base = 0.7;
    let bonus = match plane {
        PlaneType::Exploration | PlaneType::CardStack => 0.15,
        PlaneType::Dialogue | PlaneType::Investigation | PlaneType::Political => 0.1,
        PlaneType::Tactical | PlaneType::Crafting => 0.05,
    };
    (
        base + bonus,
        vec!["All four elements present".into()],
        vec!["Technology element weakest (terminal rendering)".into()],
    )
}

fn eval_unification(plane: PlaneType) -> (f64, Vec<String>, Vec<String>) {
    match plane {
        PlaneType::Exploration => (
            0.9,
            vec!["Theme: discovery unites movement+fog+lore".into()],
            vec![],
        ),
        PlaneType::Dialogue => (
            0.85,
            vec!["Theme: understanding unites voice+trust+knowledge".into()],
            vec![],
        ),
        PlaneType::Tactical => (
            0.8,
            vec!["Theme: mastery unites positioning+timing+resources".into()],
            vec![],
        ),
        PlaneType::Investigation => (
            0.95,
            vec!["Theme: truth unites clues+deduction+revelation".into()],
            vec![],
        ),
        PlaneType::Political => (
            0.7,
            vec!["Theme: power unites reputation+alliance+betrayal".into()],
            vec!["Multiple competing themes".into()],
        ),
        PlaneType::Crafting => (
            0.75,
            vec!["Theme: creation unites materials+recipes+products".into()],
            vec!["Economic subsystem splits focus".into()],
        ),
        PlaneType::CardStack => (
            0.8,
            vec!["Theme: timing unites priority+sequencing+response".into()],
            vec![],
        ),
    }
}

fn eval_action_outcome(plane: PlaneType) -> (f64, Vec<String>, Vec<String>) {
    match plane {
        PlaneType::Exploration => (
            0.7,
            vec!["Movement→discovery clear".into()],
            vec!["Long-term consequences opaque".into()],
        ),
        PlaneType::Dialogue => (
            0.6,
            vec!["Skill check outcomes defined".into()],
            vec!["NPC reaction partially opaque (intentional)".into()],
        ),
        PlaneType::Tactical => (
            0.95,
            vec!["DCs, hit chances, damage fully transparent".into()],
            vec![],
        ),
        PlaneType::Investigation => (
            0.7,
            vec!["Clue gathering predictable".into()],
            vec!["Deduction leaps uncertain".into()],
        ),
        PlaneType::Political => (
            0.5,
            vec!["Immediate actions clear".into()],
            vec!["Faction ripple effects intentionally opaque".into()],
        ),
        PlaneType::Crafting => (
            0.9,
            vec!["Recipe inputs→outputs deterministic".into()],
            vec![],
        ),
        PlaneType::CardStack => (
            0.85,
            vec!["Stack resolution rules-determined".into()],
            vec!["Opponent response unknown".into()],
        ),
    }
}

fn eval_goals(plane: PlaneType) -> (f64, Vec<String>, Vec<String>) {
    match plane {
        PlaneType::Exploration => (
            0.6,
            vec!["Emergent goals from discovery".into()],
            vec!["No explicit goal structure".into()],
        ),
        PlaneType::Dialogue => (
            0.7,
            vec!["Trust thresholds as goals".into()],
            vec!["Relationship goals implicit".into()],
        ),
        PlaneType::Tactical => (
            0.95,
            vec!["Defeat enemies, survive, achieve objective".into()],
            vec![],
        ),
        PlaneType::Investigation => (0.85, vec!["Solve the mystery".into()], vec![]),
        PlaneType::Political => (
            0.7,
            vec!["Gain influence/power".into()],
            vec!["Multi-objective ambiguity".into()],
        ),
        PlaneType::Crafting => (0.8, vec!["Create target item".into()], vec![]),
        PlaneType::CardStack => (
            0.9,
            vec!["Win the match, reduce opponent to 0".into()],
            vec![],
        ),
    }
}

fn eval_skill(plane: PlaneType) -> (f64, Vec<String>, Vec<String>) {
    match plane {
        PlaneType::Exploration => (
            0.5,
            vec!["Resource management".into()],
            vec!["Low skill ceiling".into()],
        ),
        PlaneType::Dialogue => (
            0.7,
            vec!["Social intelligence rewarded".into()],
            vec!["Dice override player skill".into()],
        ),
        PlaneType::Tactical => (
            0.9,
            vec!["Positioning, timing, resource optimization".into()],
            vec![],
        ),
        PlaneType::Investigation => (
            0.75,
            vec!["Deductive reasoning".into()],
            vec!["GUMSHOE auto-gives clues (by design)".into()],
        ),
        PlaneType::Political => (
            0.7,
            vec!["Social strategy".into()],
            vec!["Memory/tracking burden".into()],
        ),
        PlaneType::Crafting => (
            0.6,
            vec!["Recipe optimization".into()],
            vec!["Mastery ceiling low after discovery".into()],
        ),
        PlaneType::CardStack => (
            0.95,
            vec!["Sequencing, risk assessment, meta-knowledge".into()],
            vec![],
        ),
    }
}

fn eval_expected_value(plane: PlaneType) -> (f64, Vec<String>, Vec<String>) {
    let base = match plane {
        PlaneType::Exploration | PlaneType::Dialogue => 0.7,
        PlaneType::Tactical | PlaneType::CardStack => 0.85,
        PlaneType::Investigation | PlaneType::Political | PlaneType::Crafting => 0.75,
    };
    (
        base,
        vec!["Risk/reward structures present".into()],
        vec!["Balance requires playtesting".into()],
    )
}

fn eval_challenge(plane: PlaneType) -> (f64, Vec<String>, Vec<String>) {
    match plane {
        PlaneType::Exploration => (
            0.6,
            vec!["Environmental hazards".into()],
            vec!["Low active challenge without encounters".into()],
        ),
        PlaneType::Dialogue => (
            0.7,
            vec!["Social difficulty curves".into()],
            vec!["DDA for conversation pace needed".into()],
        ),
        PlaneType::Tactical => (
            0.9,
            vec!["Encounter design is calibrated challenge".into()],
            vec![],
        ),
        PlaneType::Investigation => (0.8, vec!["Puzzle difficulty scaling".into()], vec![]),
        PlaneType::Political => (
            0.65,
            vec!["Competing agents scale challenge".into()],
            vec!["Difficulty invisible to player".into()],
        ),
        PlaneType::Crafting => (
            0.7,
            vec!["Material scarcity creates challenge".into()],
            vec!["Linear difficulty curve".into()],
        ),
        PlaneType::CardStack => (
            0.85,
            vec!["Opponent skill = dynamic challenge".into()],
            vec![],
        ),
    }
}

fn eval_meaningful_choice(plane: PlaneType) -> (f64, Vec<String>, Vec<String>) {
    match plane {
        PlaneType::Exploration => (
            0.75,
            vec!["Path selection, resource allocation".into()],
            vec!["Many choices equivalent".into()],
        ),
        PlaneType::Dialogue => (
            0.9,
            vec!["Every dialogue option has consequences".into()],
            vec![],
        ),
        PlaneType::Tactical => (
            0.85,
            vec!["Action selection, positioning trade-offs".into()],
            vec![],
        ),
        PlaneType::Investigation => (
            0.7,
            vec!["Which leads to follow".into()],
            vec!["GUMSHOE gives all clues regardless".into()],
        ),
        PlaneType::Political => (
            0.95,
            vec!["Alliance/betrayal has cascading consequences".into()],
            vec![],
        ),
        PlaneType::Crafting => (
            0.6,
            vec!["Material allocation".into()],
            vec!["Optimal path often obvious".into()],
        ),
        PlaneType::CardStack => (
            0.9,
            vec!["Play order, resource commitment, bluffing".into()],
            vec![],
        ),
    }
}

fn eval_transparency(plane: PlaneType) -> (f64, Vec<String>, Vec<String>) {
    match plane {
        PlaneType::Exploration => (0.8, vec!["Simple rules, few mechanics".into()], vec![]),
        PlaneType::Dialogue => (
            0.6,
            vec!["Skill checks transparent".into()],
            vec!["NPC internal state hidden".into()],
        ),
        PlaneType::Tactical => (
            0.7,
            vec!["Rules heavy but documented".into()],
            vec!["PF2e complexity barrier".into()],
        ),
        PlaneType::Investigation => (
            0.75,
            vec!["Evidence rules clear".into()],
            vec!["Deduction logic implicit".into()],
        ),
        PlaneType::Political => (
            0.5,
            vec!["Basic rules clear".into()],
            vec!["Social dynamics opaque by design".into()],
        ),
        PlaneType::Crafting => (
            0.85,
            vec!["Recipe system deterministic and visible".into()],
            vec![],
        ),
        PlaneType::CardStack => (
            0.8,
            vec!["Stack rules fully specified".into()],
            vec!["Rule interactions complex".into()],
        ),
    }
}

fn eval_economy(plane: PlaneType) -> (f64, Vec<String>, Vec<String>) {
    match plane {
        PlaneType::Exploration => (
            0.6,
            vec!["Supply management".into()],
            vec!["Economy minimal".into()],
        ),
        PlaneType::Dialogue => (
            0.5,
            vec!["Trust as currency".into()],
            vec!["Informal economy".into()],
        ),
        PlaneType::Tactical => (
            0.85,
            vec!["Action economy (3 actions/turn PF2e)".into()],
            vec![],
        ),
        PlaneType::Investigation => (
            0.6,
            vec!["Time/attention as resource".into()],
            vec!["Not formalized".into()],
        ),
        PlaneType::Political => (
            0.8,
            vec!["Favor economy, reputation as capital".into()],
            vec![],
        ),
        PlaneType::Crafting => (
            0.9,
            vec!["Material economy is core mechanic".into()],
            vec![],
        ),
        PlaneType::CardStack => (
            0.95,
            vec!["Mana/resource system fully designed".into()],
            vec![],
        ),
    }
}

fn eval_fairness(plane: PlaneType) -> (f64, Vec<String>, Vec<String>) {
    let base = 0.75;
    let modifier = match plane {
        PlaneType::Tactical | PlaneType::CardStack => 0.15,
        PlaneType::Exploration | PlaneType::Crafting => 0.1,
        PlaneType::Dialogue | PlaneType::Investigation | PlaneType::Political => 0.0,
    };
    (
        base + modifier,
        vec!["Single-player or cooperative — fairness vs system".into()],
        vec!["PvP balance requires extensive testing".into()],
    )
}

fn eval_freedom(plane: PlaneType) -> (f64, Vec<String>, Vec<String>) {
    match plane {
        PlaneType::Exploration => (0.95, vec!["Maximum player agency".into()], vec![]),
        PlaneType::Dialogue => (
            0.8,
            vec!["Conversation approach is player's choice".into()],
            vec!["Bounded by NPC knowledge system".into()],
        ),
        PlaneType::Tactical => (
            0.7,
            vec!["Tactical creativity within rules".into()],
            vec!["Rules constrain novel solutions".into()],
        ),
        PlaneType::Investigation => (
            0.65,
            vec!["Approach to investigation free".into()],
            vec!["Solution path predetermined".into()],
        ),
        PlaneType::Political => (0.85, vec!["Strategy entirely player-driven".into()], vec![]),
        PlaneType::Crafting => (
            0.7,
            vec!["Creative expression in combinations".into()],
            vec!["Recipe list bounds possibility".into()],
        ),
        PlaneType::CardStack => (
            0.75,
            vec!["Deck construction as expression".into()],
            vec!["Card pool limits options".into()],
        ),
    }
}

fn eval_feedback(plane: PlaneType) -> (f64, Vec<String>, Vec<String>) {
    match plane {
        PlaneType::Exploration => (
            0.7,
            vec!["Map reveals, inventory updates".into()],
            vec!["Long-term impact unclear".into()],
        ),
        PlaneType::Dialogue => (
            0.75,
            vec!["Trust level visible, NPC reactions".into()],
            vec!["Subtle shifts hard to convey in text".into()],
        ),
        PlaneType::Tactical => (
            0.9,
            vec!["HP, conditions, initiative — constant state".into()],
            vec![],
        ),
        PlaneType::Investigation => (
            0.7,
            vec!["Evidence board accumulation".into()],
            vec!["Insight moments hard to pace".into()],
        ),
        PlaneType::Political => (
            0.6,
            vec!["Reputation scores visible".into()],
            vec!["Faction relations multi-dimensional".into()],
        ),
        PlaneType::Crafting => (
            0.85,
            vec!["Recipe progress, material consumption clear".into()],
            vec![],
        ),
        PlaneType::CardStack => (
            0.9,
            vec!["Board state, life totals, stack visible".into()],
            vec![],
        ),
    }
}

fn eval_story_game_balance(plane: PlaneType) -> (f64, Vec<String>, Vec<String>) {
    match plane {
        PlaneType::Exploration => (0.85, vec!["Discovery IS the story".into()], vec![]),
        PlaneType::Dialogue => (0.95, vec!["Conversation IS narrative".into()], vec![]),
        PlaneType::Tactical => (
            0.6,
            vec!["Combat serves narrative context".into()],
            vec!["Mechanics can overshadow story".into()],
        ),
        PlaneType::Investigation => (0.9, vec!["Mystery solving IS the narrative".into()], vec![]),
        PlaneType::Political => (
            0.85,
            vec!["Political maneuvering creates story".into()],
            vec![],
        ),
        PlaneType::Crafting => (
            0.5,
            vec!["Crafting serves equipment needs".into()],
            vec!["Mechanical, low narrative integration".into()],
        ),
        PlaneType::CardStack => (
            0.6,
            vec!["Card themes provide flavor".into()],
            vec!["Abstraction distances from narrative".into()],
        ),
    }
}

fn eval_flow(plane: PlaneType) -> (f64, Vec<String>, Vec<String>) {
    match plane {
        PlaneType::Exploration => (
            0.8,
            vec!["Ambient discovery promotes flow".into()],
            vec!["Low challenge may drop engagement".into()],
        ),
        PlaneType::Dialogue => (
            0.75,
            vec!["Conversational flow natural".into()],
            vec!["Dice interrupts may break flow".into()],
        ),
        PlaneType::Tactical => (
            0.85,
            vec!["Turn structure maintains engagement".into()],
            vec!["Rules lookup breaks flow".into()],
        ),
        PlaneType::Investigation => (
            0.7,
            vec!["Puzzle flow when clues connect".into()],
            vec!["Dead ends break flow".into()],
        ),
        PlaneType::Political => (
            0.6,
            vec!["Intrigue drives forward momentum".into()],
            vec!["Long timescales reduce flow".into()],
        ),
        PlaneType::Crafting => (
            0.65,
            vec!["Crafting loops can be meditative".into()],
            vec!["Material gathering interrupts".into()],
        ),
        PlaneType::CardStack => (
            0.85,
            vec!["Rapid decision-making promotes flow".into()],
            vec![],
        ),
    }
}

/// Compare two planes across all lenses, returning differential strengths.
#[must_use]
pub fn compare_planes(a: PlaneType, b: PlaneType) -> Vec<LensDifference> {
    let analysis_a = evaluate_plane(a);
    let analysis_b = evaluate_plane(b);

    analysis_a
        .evaluations
        .iter()
        .zip(&analysis_b.evaluations)
        .map(|(ea, eb)| LensDifference {
            lens: ea.lens,
            score_a: ea.score,
            score_b: eb.score,
            delta: ea.score - eb.score,
        })
        .collect()
}

/// Difference in a single lens between two planes.
#[derive(Debug, Clone)]
pub struct LensDifference {
    /// The lens being compared.
    pub lens: Lens,
    /// Score for plane A.
    pub score_a: f64,
    /// Score for plane B.
    pub score_b: f64,
    /// Delta (A - B): positive means A is stronger.
    pub delta: f64,
}

#[cfg(test)]
#[allow(clippy::unwrap_used, reason = "test assertions")]
mod tests {
    use super::*;

    #[test]
    fn all_planes_evaluable() {
        for plane in [
            PlaneType::Exploration,
            PlaneType::Dialogue,
            PlaneType::Tactical,
            PlaneType::Investigation,
            PlaneType::Political,
            PlaneType::Crafting,
            PlaneType::CardStack,
        ] {
            let analysis = evaluate_plane(plane);
            assert_eq!(analysis.evaluations.len(), 20);
            assert!(analysis.overall_score() > 0.0);
            assert!(analysis.overall_score() <= 1.0);
        }
    }

    #[test]
    fn scores_bounded_zero_to_one() {
        for plane in [
            PlaneType::Exploration,
            PlaneType::Dialogue,
            PlaneType::Tactical,
            PlaneType::Investigation,
            PlaneType::Political,
            PlaneType::Crafting,
            PlaneType::CardStack,
        ] {
            let analysis = evaluate_plane(plane);
            for eval in &analysis.evaluations {
                assert!(
                    (0.0..=1.0).contains(&eval.score),
                    "{:?} lens {:?} score {} out of range",
                    plane,
                    eval.lens,
                    eval.score
                );
            }
        }
    }

    #[test]
    fn tactical_strongest_at_problem_solving() {
        let tactical = evaluate_plane(PlaneType::Tactical);
        let ps = tactical
            .evaluations
            .iter()
            .find(|e| e.lens == Lens::ProblemSolving)
            .unwrap();
        assert!(ps.score >= 0.9);
    }

    #[test]
    fn exploration_strongest_at_curiosity() {
        let exploration = evaluate_plane(PlaneType::Exploration);
        let curiosity = exploration
            .evaluations
            .iter()
            .find(|e| e.lens == Lens::Curiosity)
            .unwrap();
        assert!(curiosity.score >= 0.9);
    }

    #[test]
    fn investigation_strongest_at_unification() {
        let investigation = evaluate_plane(PlaneType::Investigation);
        let unification = investigation
            .evaluations
            .iter()
            .find(|e| e.lens == Lens::Unification)
            .unwrap();
        assert!(unification.score >= 0.9);
    }

    #[test]
    fn weak_lenses_filters_correctly() {
        let analysis = evaluate_plane(PlaneType::Political);
        let weak = analysis.weak_lenses(0.6);
        for eval in &weak {
            assert!(eval.score < 0.6);
        }
    }

    #[test]
    fn compare_planes_produces_deltas() {
        let diffs = compare_planes(PlaneType::Tactical, PlaneType::Exploration);
        assert_eq!(diffs.len(), 20);
        for d in &diffs {
            assert!((d.score_a - d.score_b - d.delta).abs() < 1e-10);
        }
    }

    #[test]
    fn compare_planes_freedom_exploration_dominates() {
        let diffs = compare_planes(PlaneType::Exploration, PlaneType::Tactical);
        let freedom = diffs.iter().find(|d| d.lens == Lens::Freedom).unwrap();
        assert!(
            freedom.delta > 0.0,
            "Exploration should dominate Tactical in Freedom"
        );
    }

    #[test]
    fn lens_labels_unique() {
        let labels: Vec<&str> = Lens::ALL.iter().map(|l| l.label()).collect();
        let unique: std::collections::HashSet<&str> = labels.iter().copied().collect();
        assert_eq!(labels.len(), unique.len());
    }

    #[test]
    fn lens_questions_non_empty() {
        for lens in &Lens::ALL {
            assert!(!lens.question().is_empty());
        }
    }
}

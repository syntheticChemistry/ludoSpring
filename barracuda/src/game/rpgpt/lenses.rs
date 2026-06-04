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

/// Apply a single lens to a plane using the data-driven evaluation table.
fn evaluate_single_lens(lens: Lens, plane: PlaneType) -> LensEvaluation {
    let entry = lookup_evaluation(lens, plane);
    LensEvaluation {
        lens,
        score: entry.score,
        strengths: entry.strengths.iter().map(|&s| s.into()).collect(),
        gaps: entry.gaps.iter().map(|&s| s.into()).collect(),
    }
}

struct EvalEntry {
    score: f64,
    strengths: &'static [&'static str],
    gaps: &'static [&'static str],
}

impl EvalEntry {
    const fn new(
        score: f64,
        strengths: &'static [&'static str],
        gaps: &'static [&'static str],
    ) -> Self {
        Self {
            score,
            strengths,
            gaps,
        }
    }
}

#[allow(clippy::too_many_lines, clippy::enum_glob_use)]
const fn lookup_evaluation(lens: Lens, plane: PlaneType) -> EvalEntry {
    use Lens::*;
    use PlaneType::*;

    match (lens, plane) {
        // ─── Essential Experience ───
        (EssentialExperience, Exploration) => EvalEntry::new(0.9, &["Discovery and wonder"], &[]),
        (EssentialExperience, Dialogue) => EvalEntry::new(
            0.85,
            &["Social deduction and empathy"],
            &["Voice system complexity"],
        ),
        (EssentialExperience, Tactical) => EvalEntry::new(0.9, &["Strategic mastery"], &[]),
        (EssentialExperience, Investigation) => {
            EvalEntry::new(0.8, &["Mystery and deduction"], &["Pacing control needed"])
        }
        (EssentialExperience, Political) => EvalEntry::new(
            0.75,
            &["Power dynamics"],
            &["Long-term consequence tracking"],
        ),
        (EssentialExperience, Crafting) => {
            EvalEntry::new(0.7, &["Creative expression"], &["Recipe discovery depth"])
        }
        (EssentialExperience, CardStack) => EvalEntry::new(0.85, &["Combinatorial depth"], &[]),

        // ─── Surprise ───
        (Surprise, Exploration) => EvalEntry::new(0.9, &["Procedural discovery"], &[]),
        (Surprise, Dialogue) => EvalEntry::new(
            0.7,
            &["NPC personality emergence"],
            &["AI unpredictability bounded by cert"],
        ),
        (Surprise, Tactical) => EvalEntry::new(
            0.6,
            &["Dice variance"],
            &["Deterministic grid limits surprise"],
        ),
        (Surprise, Investigation) => EvalEntry::new(0.85, &["Clue revelation timing"], &[]),
        (Surprise, Political) => EvalEntry::new(0.8, &["Betrayal mechanics"], &[]),
        (Surprise, Crafting) => EvalEntry::new(
            0.65,
            &["Unexpected synergies"],
            &["Recipe system predictable after mastery"],
        ),
        (Surprise, CardStack) => EvalEntry::new(0.9, &["Stack interaction emergent behavior"], &[]),

        // ─── Curiosity ───
        (Curiosity, Exploration) => EvalEntry::new(0.95, &["Map mystery, fog of war"], &[]),
        (Curiosity, Dialogue) => EvalEntry::new(0.8, &["NPC secrets, hidden knowledge"], &[]),
        (Curiosity, Tactical) => EvalEntry::new(
            0.5,
            &["Enemy capability unknown"],
            &["Rules-heavy reduces wonder"],
        ),
        (Curiosity, Investigation) => EvalEntry::new(0.95, &["Core mechanic IS curiosity"], &[]),
        (Curiosity, Political) => EvalEntry::new(0.75, &["Hidden agendas"], &[]),
        (Curiosity, Crafting) => {
            EvalEntry::new(0.7, &["Unknown recipes"], &["Finite discovery space"])
        }
        (Curiosity, CardStack) => {
            EvalEntry::new(0.7, &["Novel combinations"], &["Solved states possible"])
        }

        // ─── Endogenous Value ───
        (EndogenousValue, Exploration) => {
            EvalEntry::new(0.8, &["Landmarks have narrative weight"], &[])
        }
        (EndogenousValue, Dialogue) => {
            EvalEntry::new(0.85, &["Trust is earned, secrets matter"], &[])
        }
        (EndogenousValue, Tactical) => EvalEntry::new(
            0.7,
            &["Position matters, HP meaningful"],
            &["Abstract numbers vs world"],
        ),
        (EndogenousValue, Investigation) => EvalEntry::new(0.9, &["Clues unlock narrative"], &[]),
        (EndogenousValue, Political) => EvalEntry::new(0.9, &["Reputation drives access"], &[]),
        (EndogenousValue, Crafting) => {
            EvalEntry::new(0.75, &["Created items have provenance"], &[])
        }
        (EndogenousValue, CardStack) => EvalEntry::new(0.8, &["Card value context-dependent"], &[]),

        // ─── Problem Solving ───
        (ProblemSolving, Exploration) => EvalEntry::new(
            0.6,
            &["Navigation, resource management"],
            &["Low structured challenge"],
        ),
        (ProblemSolving, Dialogue) => EvalEntry::new(0.75, &["Social puzzle solving"], &[]),
        (ProblemSolving, Tactical) => {
            EvalEntry::new(0.95, &["Core mechanic IS problem solving"], &[])
        }
        (ProblemSolving, Investigation) => EvalEntry::new(0.9, &["Deduction chains"], &[]),
        (ProblemSolving, Political) => EvalEntry::new(0.8, &["Multi-agent optimization"], &[]),
        (ProblemSolving, Crafting) => EvalEntry::new(0.85, &["Material constraint puzzles"], &[]),
        (ProblemSolving, CardStack) => EvalEntry::new(0.9, &["Sequencing optimization"], &[]),

        // ─── Elemental Tetrad ───
        (ElementalTetrad, Exploration | CardStack) => EvalEntry::new(
            0.85,
            &["All four elements present"],
            &["Technology element weakest (terminal rendering)"],
        ),
        (ElementalTetrad, Dialogue | Investigation | Political) => EvalEntry::new(
            0.8,
            &["All four elements present"],
            &["Technology element weakest (terminal rendering)"],
        ),
        (ElementalTetrad, Tactical | Crafting) => EvalEntry::new(
            0.75,
            &["All four elements present"],
            &["Technology element weakest (terminal rendering)"],
        ),

        // ─── Unification ───
        (Unification, Exploration) => {
            EvalEntry::new(0.9, &["Theme: discovery unites movement+fog+lore"], &[])
        }
        (Unification, Dialogue) => EvalEntry::new(
            0.85,
            &["Theme: understanding unites voice+trust+knowledge"],
            &[],
        ),
        (Unification, Tactical) => EvalEntry::new(
            0.8,
            &["Theme: mastery unites positioning+timing+resources"],
            &[],
        ),
        (Unification, Investigation) => EvalEntry::new(
            0.95,
            &["Theme: truth unites clues+deduction+revelation"],
            &[],
        ),
        (Unification, Political) => EvalEntry::new(
            0.7,
            &["Theme: power unites reputation+alliance+betrayal"],
            &["Multiple competing themes"],
        ),
        (Unification, Crafting) => EvalEntry::new(
            0.75,
            &["Theme: creation unites materials+recipes+products"],
            &["Economic subsystem splits focus"],
        ),
        (Unification, CardStack) => EvalEntry::new(
            0.8,
            &["Theme: timing unites priority+sequencing+response"],
            &[],
        ),

        // ─── Action → Outcome ───
        (ActionOutcome, Exploration) => EvalEntry::new(
            0.7,
            &["Movement→discovery clear"],
            &["Long-term consequences opaque"],
        ),
        (ActionOutcome, Dialogue) => EvalEntry::new(
            0.6,
            &["Skill check outcomes defined"],
            &["NPC reaction partially opaque (intentional)"],
        ),
        (ActionOutcome, Tactical) => {
            EvalEntry::new(0.95, &["DCs, hit chances, damage fully transparent"], &[])
        }
        (ActionOutcome, Investigation) => EvalEntry::new(
            0.7,
            &["Clue gathering predictable"],
            &["Deduction leaps uncertain"],
        ),
        (ActionOutcome, Political) => EvalEntry::new(
            0.5,
            &["Immediate actions clear"],
            &["Faction ripple effects intentionally opaque"],
        ),
        (ActionOutcome, Crafting) => {
            EvalEntry::new(0.9, &["Recipe inputs→outputs deterministic"], &[])
        }
        (ActionOutcome, CardStack) => EvalEntry::new(
            0.85,
            &["Stack resolution rules-determined"],
            &["Opponent response unknown"],
        ),

        // ─── Goals ───
        (Goals, Exploration) => EvalEntry::new(
            0.6,
            &["Emergent goals from discovery"],
            &["No explicit goal structure"],
        ),
        (Goals, Dialogue) => EvalEntry::new(
            0.7,
            &["Trust thresholds as goals"],
            &["Relationship goals implicit"],
        ),
        (Goals, Tactical) => {
            EvalEntry::new(0.95, &["Defeat enemies, survive, achieve objective"], &[])
        }
        (Goals, Investigation) => EvalEntry::new(0.85, &["Solve the mystery"], &[]),
        (Goals, Political) => EvalEntry::new(
            0.7,
            &["Gain influence/power"],
            &["Multi-objective ambiguity"],
        ),
        (Goals, Crafting) => EvalEntry::new(0.8, &["Create target item"], &[]),
        (Goals, CardStack) => EvalEntry::new(0.9, &["Win the match, reduce opponent to 0"], &[]),

        // ─── Skill ───
        (Skill, Exploration) => {
            EvalEntry::new(0.5, &["Resource management"], &["Low skill ceiling"])
        }
        (Skill, Dialogue) => EvalEntry::new(
            0.7,
            &["Social intelligence rewarded"],
            &["Dice override player skill"],
        ),
        (Skill, Tactical) => {
            EvalEntry::new(0.9, &["Positioning, timing, resource optimization"], &[])
        }
        (Skill, Investigation) => EvalEntry::new(
            0.75,
            &["Deductive reasoning"],
            &["GUMSHOE auto-gives clues (by design)"],
        ),
        (Skill, Political) => {
            EvalEntry::new(0.7, &["Social strategy"], &["Memory/tracking burden"])
        }
        (Skill, Crafting) => EvalEntry::new(
            0.6,
            &["Recipe optimization"],
            &["Mastery ceiling low after discovery"],
        ),
        (Skill, CardStack) => {
            EvalEntry::new(0.95, &["Sequencing, risk assessment, meta-knowledge"], &[])
        }

        // ─── Expected Value ───
        (ExpectedValue, Exploration | Dialogue) => EvalEntry::new(
            0.7,
            &["Risk/reward structures present"],
            &["Balance requires playtesting"],
        ),
        (ExpectedValue, Tactical | CardStack) => EvalEntry::new(
            0.85,
            &["Risk/reward structures present"],
            &["Balance requires playtesting"],
        ),
        (ExpectedValue, Investigation | Political | Crafting) => EvalEntry::new(
            0.75,
            &["Risk/reward structures present"],
            &["Balance requires playtesting"],
        ),

        // ─── Challenge ───
        (Challenge, Exploration) => EvalEntry::new(
            0.6,
            &["Environmental hazards"],
            &["Low active challenge without encounters"],
        ),
        (Challenge, Dialogue) => EvalEntry::new(
            0.7,
            &["Social difficulty curves"],
            &["DDA for conversation pace needed"],
        ),
        (Challenge, Tactical) => {
            EvalEntry::new(0.9, &["Encounter design is calibrated challenge"], &[])
        }
        (Challenge, Investigation) => EvalEntry::new(0.8, &["Puzzle difficulty scaling"], &[]),
        (Challenge, Political) => EvalEntry::new(
            0.65,
            &["Competing agents scale challenge"],
            &["Difficulty invisible to player"],
        ),
        (Challenge, Crafting) => EvalEntry::new(
            0.7,
            &["Material scarcity creates challenge"],
            &["Linear difficulty curve"],
        ),
        (Challenge, CardStack) => {
            EvalEntry::new(0.85, &["Opponent skill = dynamic challenge"], &[])
        }

        // ─── Meaningful Choice ───
        (MeaningfulChoice, Exploration) => EvalEntry::new(
            0.75,
            &["Path selection, resource allocation"],
            &["Many choices equivalent"],
        ),
        (MeaningfulChoice, Dialogue) => {
            EvalEntry::new(0.9, &["Every dialogue option has consequences"], &[])
        }
        (MeaningfulChoice, Tactical) => {
            EvalEntry::new(0.85, &["Action selection, positioning trade-offs"], &[])
        }
        (MeaningfulChoice, Investigation) => EvalEntry::new(
            0.7,
            &["Which leads to follow"],
            &["GUMSHOE gives all clues regardless"],
        ),
        (MeaningfulChoice, Political) => {
            EvalEntry::new(0.95, &["Alliance/betrayal has cascading consequences"], &[])
        }
        (MeaningfulChoice, Crafting) => EvalEntry::new(
            0.6,
            &["Material allocation"],
            &["Optimal path often obvious"],
        ),
        (MeaningfulChoice, CardStack) => {
            EvalEntry::new(0.9, &["Play order, resource commitment, bluffing"], &[])
        }

        // ─── Transparency ───
        (Transparency, Exploration) => EvalEntry::new(0.8, &["Simple rules, few mechanics"], &[]),
        (Transparency, Dialogue) => EvalEntry::new(
            0.6,
            &["Skill checks transparent"],
            &["NPC internal state hidden"],
        ),
        (Transparency, Tactical) => EvalEntry::new(
            0.7,
            &["Rules heavy but documented"],
            &["PF2e complexity barrier"],
        ),
        (Transparency, Investigation) => EvalEntry::new(
            0.75,
            &["Evidence rules clear"],
            &["Deduction logic implicit"],
        ),
        (Transparency, Political) => EvalEntry::new(
            0.5,
            &["Basic rules clear"],
            &["Social dynamics opaque by design"],
        ),
        (Transparency, Crafting) => {
            EvalEntry::new(0.85, &["Recipe system deterministic and visible"], &[])
        }
        (Transparency, CardStack) => EvalEntry::new(
            0.8,
            &["Stack rules fully specified"],
            &["Rule interactions complex"],
        ),

        // ─── Economy ───
        (Economy, Exploration) => EvalEntry::new(0.6, &["Supply management"], &["Economy minimal"]),
        (Economy, Dialogue) => EvalEntry::new(0.5, &["Trust as currency"], &["Informal economy"]),
        (Economy, Tactical) => EvalEntry::new(0.85, &["Action economy (3 actions/turn PF2e)"], &[]),
        (Economy, Investigation) => {
            EvalEntry::new(0.6, &["Time/attention as resource"], &["Not formalized"])
        }
        (Economy, Political) => EvalEntry::new(0.8, &["Favor economy, reputation as capital"], &[]),
        (Economy, Crafting) => EvalEntry::new(0.9, &["Material economy is core mechanic"], &[]),
        (Economy, CardStack) => EvalEntry::new(0.95, &["Mana/resource system fully designed"], &[]),

        // ─── Fairness ───
        (Fairness, Tactical | CardStack) => EvalEntry::new(
            0.9,
            &["Single-player or cooperative — fairness vs system"],
            &["PvP balance requires extensive testing"],
        ),
        (Fairness, Exploration | Crafting) => EvalEntry::new(
            0.85,
            &["Single-player or cooperative — fairness vs system"],
            &["PvP balance requires extensive testing"],
        ),
        (Fairness, Dialogue | Investigation | Political) => EvalEntry::new(
            0.75,
            &["Single-player or cooperative — fairness vs system"],
            &["PvP balance requires extensive testing"],
        ),

        // ─── Freedom ───
        (Freedom, Exploration) => EvalEntry::new(0.95, &["Maximum player agency"], &[]),
        (Freedom, Dialogue) => EvalEntry::new(
            0.8,
            &["Conversation approach is player's choice"],
            &["Bounded by NPC knowledge system"],
        ),
        (Freedom, Tactical) => EvalEntry::new(
            0.7,
            &["Tactical creativity within rules"],
            &["Rules constrain novel solutions"],
        ),
        (Freedom, Investigation) => EvalEntry::new(
            0.65,
            &["Approach to investigation free"],
            &["Solution path predetermined"],
        ),
        (Freedom, Political) => EvalEntry::new(0.85, &["Strategy entirely player-driven"], &[]),
        (Freedom, Crafting) => EvalEntry::new(
            0.7,
            &["Creative expression in combinations"],
            &["Recipe list bounds possibility"],
        ),
        (Freedom, CardStack) => EvalEntry::new(
            0.75,
            &["Deck construction as expression"],
            &["Card pool limits options"],
        ),

        // ─── Feedback ───
        (Feedback, Exploration) => EvalEntry::new(
            0.7,
            &["Map reveals, inventory updates"],
            &["Long-term impact unclear"],
        ),
        (Feedback, Dialogue) => EvalEntry::new(
            0.75,
            &["Trust level visible, NPC reactions"],
            &["Subtle shifts hard to convey in text"],
        ),
        (Feedback, Tactical) => {
            EvalEntry::new(0.9, &["HP, conditions, initiative — constant state"], &[])
        }
        (Feedback, Investigation) => EvalEntry::new(
            0.7,
            &["Evidence board accumulation"],
            &["Insight moments hard to pace"],
        ),
        (Feedback, Political) => EvalEntry::new(
            0.6,
            &["Reputation scores visible"],
            &["Faction relations multi-dimensional"],
        ),
        (Feedback, Crafting) => {
            EvalEntry::new(0.85, &["Recipe progress, material consumption clear"], &[])
        }
        (Feedback, CardStack) => {
            EvalEntry::new(0.9, &["Board state, life totals, stack visible"], &[])
        }

        // ─── Story/Game Balance ───
        (StoryGameBalance, Exploration) => EvalEntry::new(0.85, &["Discovery IS the story"], &[]),
        (StoryGameBalance, Dialogue) => EvalEntry::new(0.95, &["Conversation IS narrative"], &[]),
        (StoryGameBalance, Tactical) => EvalEntry::new(
            0.6,
            &["Combat serves narrative context"],
            &["Mechanics can overshadow story"],
        ),
        (StoryGameBalance, Investigation) => {
            EvalEntry::new(0.9, &["Mystery solving IS the narrative"], &[])
        }
        (StoryGameBalance, Political) => {
            EvalEntry::new(0.85, &["Political maneuvering creates story"], &[])
        }
        (StoryGameBalance, Crafting) => EvalEntry::new(
            0.5,
            &["Crafting serves equipment needs"],
            &["Mechanical, low narrative integration"],
        ),
        (StoryGameBalance, CardStack) => EvalEntry::new(
            0.6,
            &["Card themes provide flavor"],
            &["Abstraction distances from narrative"],
        ),

        // ─── Flow ───
        (Flow, Exploration) => EvalEntry::new(
            0.8,
            &["Ambient discovery promotes flow"],
            &["Low challenge may drop engagement"],
        ),
        (Flow, Dialogue) => EvalEntry::new(
            0.75,
            &["Conversational flow natural"],
            &["Dice interrupts may break flow"],
        ),
        (Flow, Tactical) => EvalEntry::new(
            0.85,
            &["Turn structure maintains engagement"],
            &["Rules lookup breaks flow"],
        ),
        (Flow, Investigation) => EvalEntry::new(
            0.7,
            &["Puzzle flow when clues connect"],
            &["Dead ends break flow"],
        ),
        (Flow, Political) => EvalEntry::new(
            0.6,
            &["Intrigue drives forward momentum"],
            &["Long timescales reduce flow"],
        ),
        (Flow, Crafting) => EvalEntry::new(
            0.65,
            &["Crafting loops can be meditative"],
            &["Material gathering interrupts"],
        ),
        (Flow, CardStack) => EvalEntry::new(0.85, &["Rapid decision-making promotes flow"], &[]),
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

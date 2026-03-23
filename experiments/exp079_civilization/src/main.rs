// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
//! exp079 — Civilization (minimal): strategy as systems thinking.
//!
//! Validates the models that underpin 4X strategy games (eXplore, eXpand,
//! eXploit, eXterminate):
//!
//! 1. **Fog of war** as information asymmetry: the map exists but knowledge
//!    is partial. Uses `TileWorld::reveal_radius` and `visibility_mask`. The
//!    same partial-observability model as sensor fusion in biomeOS.
//! 2. **Tech tree as DAG**: technologies form a directed acyclic graph.
//!    Prerequisites must be researched before dependents. The same DAG
//!    structure as rhizoCrypt provenance graphs.
//! 3. **Symbiont factions**: Lotka-Volterra interaction between civilizations.
//!    Resource competition, trade, war — the same population dynamics
//!    validated in the Lysogeny Symbiont target (exp057).
//! 4. **Turn-based economy**: gold per turn, production queues, resource
//!    accumulation. Tests the accounting model that every strategy game uses.
//! 5. **Hick's law on strategic decisions**: each turn presents bounded
//!    choices (research, build, move, diplomacy). Hick time stays manageable.
//!
//! Cross-spring: fog of war = partial observability in sensor networks.
//! Tech trees = dependency DAGs in build systems and scientific workflows.
//! Faction dynamics = Lotka-Volterra population ecology.

use std::collections::{HashMap, HashSet};
use std::process;

use ludospring_barracuda::game::engine::world::{Terrain, TileWorld};
use ludospring_barracuda::interaction::flow::{FlowState, evaluate_flow};
use ludospring_barracuda::interaction::input_laws::hick_reaction_time;
use ludospring_barracuda::metrics::engagement::{EngagementSnapshot, compute_engagement};
use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "N/A (analytical — Meier 1991, Lotka-Volterra, DAG theory)",
    commit: "N/A",
    date: "2026-03-18",
    command: "N/A (analytical — Civilization first principles)",
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
// Tech tree as DAG
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
struct Tech {
    name: String,
    cost: u32,
    prerequisites: Vec<String>,
}

fn dag_visit<'a>(
    tech: &'a str,
    map: &HashMap<&str, &'a Tech>,
    visited: &mut HashSet<&'a str>,
    stack: &mut HashSet<&'a str>,
) -> bool {
    if stack.contains(tech) {
        return false;
    }
    if visited.contains(tech) {
        return true;
    }
    stack.insert(tech);
    if let Some(t) = map.get(tech) {
        for prereq in &t.prerequisites {
            if !dag_visit(prereq, map, visited, stack) {
                return false;
            }
        }
    }
    stack.remove(tech);
    visited.insert(tech);
    true
}

struct TechTree {
    techs: Vec<Tech>,
}

impl TechTree {
    fn new() -> Self {
        Self {
            techs: vec![
                Tech {
                    name: "Agriculture".into(),
                    cost: 10,
                    prerequisites: vec![],
                },
                Tech {
                    name: "Writing".into(),
                    cost: 15,
                    prerequisites: vec![],
                },
                Tech {
                    name: "Bronze Working".into(),
                    cost: 20,
                    prerequisites: vec!["Agriculture".into()],
                },
                Tech {
                    name: "Pottery".into(),
                    cost: 12,
                    prerequisites: vec!["Agriculture".into()],
                },
                Tech {
                    name: "Mathematics".into(),
                    cost: 25,
                    prerequisites: vec!["Writing".into()],
                },
                Tech {
                    name: "Iron Working".into(),
                    cost: 30,
                    prerequisites: vec!["Bronze Working".into()],
                },
                Tech {
                    name: "Currency".into(),
                    cost: 35,
                    prerequisites: vec!["Mathematics".into(), "Pottery".into()],
                },
                Tech {
                    name: "Engineering".into(),
                    cost: 40,
                    prerequisites: vec!["Mathematics".into(), "Iron Working".into()],
                },
                Tech {
                    name: "Navigation".into(),
                    cost: 50,
                    prerequisites: vec!["Engineering".into(), "Currency".into()],
                },
            ],
        }
    }

    fn can_research(&self, tech_name: &str, researched: &HashSet<String>) -> bool {
        self.techs
            .iter()
            .find(|t| t.name == tech_name)
            .is_some_and(|t| {
                t.prerequisites.iter().all(|p| researched.contains(p))
                    && !researched.contains(tech_name)
            })
    }

    fn available(&self, researched: &HashSet<String>) -> Vec<&Tech> {
        self.techs
            .iter()
            .filter(|t| self.can_research(&t.name, researched))
            .collect()
    }

    fn is_dag(&self) -> bool {
        let mut visited = HashSet::new();
        let mut stack = HashSet::new();
        let name_map: HashMap<&str, &Tech> =
            self.techs.iter().map(|t| (t.name.as_str(), t)).collect();

        for tech in &self.techs {
            if !dag_visit(&tech.name, &name_map, &mut visited, &mut stack) {
                return false;
            }
        }
        true
    }

    fn topological_order(&self) -> Vec<&str> {
        let mut result = Vec::new();
        let mut researched = HashSet::new();
        loop {
            let available: Vec<&str> = self
                .techs
                .iter()
                .filter(|t| {
                    !researched.contains(t.name.as_str())
                        && t.prerequisites
                            .iter()
                            .all(|p| researched.contains(p.as_str()))
                })
                .map(|t| t.name.as_str())
                .collect();
            if available.is_empty() {
                break;
            }
            for name in available {
                researched.insert(name);
                result.push(name);
            }
        }
        result
    }
}

// ---------------------------------------------------------------------------
// Faction economy (Symbiont dynamics)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
#[expect(dead_code, reason = "name used for identification in debug output")]
struct Faction {
    name: String,
    population: f64,
    gold: f64,
    production: f64,
    growth_rate: f64,
    researched: HashSet<String>,
}

impl Faction {
    fn new(name: &str, pop: f64, gold: f64) -> Self {
        Self {
            name: name.into(),
            population: pop,
            gold,
            production: pop * 2.0,
            growth_rate: 0.03,
            researched: HashSet::new(),
        }
    }

    fn income(&self) -> f64 {
        self.population.mul_add(1.5, self.production * 0.5)
    }

    fn tick_economy(&mut self) {
        self.gold += self.income();
        self.population *= 1.0 + self.growth_rate;
        self.production = self.population * 2.0;
    }
}

/// Lotka-Volterra interaction between two factions sharing resources.
///
/// Returns (delta_a, delta_b) population changes.
fn symbiont_interaction(a: &Faction, b: &Faction, carrying_capacity: f64) -> (f64, f64) {
    let alpha = 0.5;
    let da = a.growth_rate
        * a.population
        * (1.0 - (a.population + alpha * b.population) / carrying_capacity);
    let db = b.growth_rate
        * b.population
        * (1.0 - (b.population + alpha * a.population) / carrying_capacity);
    (da, db)
}

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

fn cmd_validate() -> ! {
    let mut h = ValidationHarness::new("exp079_civilization");
    h.print_provenance(&[&PROVENANCE]);

    validate_fog_of_war(&mut h);
    validate_tech_tree_dag(&mut h);
    validate_tech_research_flow(&mut h);
    validate_faction_economy(&mut h);
    validate_symbiont_factions(&mut h);
    validate_hick_strategic_decisions(&mut h);
    validate_engagement(&mut h);

    h.finish();
}

/// Validate fog of war: partial observability, reveal radius, visibility mask.
fn validate_fog_of_war<S: ludospring_barracuda::validation::ValidationSink>(
    h: &mut ValidationHarness<S>,
) {
    let mut world = TileWorld::new(20, 20, "test_map", Terrain::Open);
    let mask_before = world.visibility_mask();
    let visible_before = mask_before.iter().filter(|&&v| v).count();
    h.check_bool("fog_starts_hidden", visible_before == 0);

    world.reveal_radius(10, 10, 3);
    let mask_after = world.visibility_mask();
    let visible_after = mask_after.iter().filter(|&&v| v).count();
    h.check_bool("fog_reveals_tiles", visible_after > 0);
    h.check_bool("fog_partial_reveal", visible_after < 400);

    world.reveal_radius(5, 5, 2);
    let mask_two = world.visibility_mask();
    let visible_two = mask_two.iter().filter(|&&v| v).count();
    h.check_bool("fog_cumulative_reveal", visible_two > visible_after);

    let mut full_world = TileWorld::new(10, 10, "small", Terrain::Open);
    full_world.reveal_radius(5, 5, 20);
    let full_mask = full_world.visibility_mask();
    let all_visible = full_mask.iter().all(|&v| v);
    h.check_bool("fog_full_reveal_possible", all_visible);
}

/// Validate tech tree is a proper DAG: no cycles, topological order exists.
fn validate_tech_tree_dag<S: ludospring_barracuda::validation::ValidationSink>(
    h: &mut ValidationHarness<S>,
) {
    let tree = TechTree::new();
    h.check_bool("tech_tree_is_dag", tree.is_dag());

    let order = tree.topological_order();
    h.check_bool(
        "topological_order_complete",
        order.len() == tree.techs.len(),
    );

    let mut seen = HashSet::new();
    let mut valid_order = true;
    for name in &order {
        if let Some(tech) = tree.techs.iter().find(|t| t.name == *name) {
            for prereq in &tech.prerequisites {
                if !seen.contains(prereq.as_str()) {
                    valid_order = false;
                }
            }
        }
        seen.insert(*name);
    }
    h.check_bool("topological_order_valid", valid_order);
}

/// Validate tech research flow: prerequisites enforce ordering.
fn validate_tech_research_flow<S: ludospring_barracuda::validation::ValidationSink>(
    h: &mut ValidationHarness<S>,
) {
    let tree = TechTree::new();
    let mut researched = HashSet::new();

    h.check_bool(
        "cannot_research_iron_first",
        !tree.can_research("Iron Working", &researched),
    );

    let initial_available = tree.available(&researched);
    let initial_names: Vec<&str> = initial_available.iter().map(|t| t.name.as_str()).collect();
    h.check_bool(
        "agriculture_available_initially",
        initial_names.contains(&"Agriculture"),
    );
    h.check_bool(
        "writing_available_initially",
        initial_names.contains(&"Writing"),
    );

    researched.insert("Agriculture".into());
    h.check_bool(
        "bronze_after_agriculture",
        tree.can_research("Bronze Working", &researched),
    );

    researched.insert("Writing".into());
    researched.insert("Bronze Working".into());
    researched.insert("Pottery".into());
    researched.insert("Mathematics".into());
    h.check_bool(
        "currency_needs_math_and_pottery",
        tree.can_research("Currency", &researched),
    );

    let mut partial = HashSet::new();
    partial.insert("Mathematics".into());
    h.check_bool(
        "currency_blocked_without_pottery",
        !tree.can_research("Currency", &partial),
    );
}

/// Validate faction economy: gold accumulation, population growth, production.
fn validate_faction_economy<S: ludospring_barracuda::validation::ValidationSink>(
    h: &mut ValidationHarness<S>,
) {
    let mut faction = Faction::new("Rome", 100.0, 0.0);
    let initial_income = faction.income();
    h.check_bool("income_positive", initial_income > 0.0);

    faction.tick_economy();
    h.check_bool("gold_accumulates", faction.gold > 0.0);
    h.check_bool("population_grows", faction.population > 100.0);
    h.check_bool("production_scales_with_pop", faction.production > 200.0);

    let gold_after_1 = faction.gold;
    for _ in 0..9 {
        faction.tick_economy();
    }
    h.check_bool(
        "gold_grows_over_10_turns",
        faction.gold > gold_after_1 * 5.0,
    );
    h.check_bool("population_compound_growth", faction.population > 130.0);
}

/// Validate Lotka-Volterra faction dynamics: competition, carrying capacity.
fn validate_symbiont_factions<S: ludospring_barracuda::validation::ValidationSink>(
    h: &mut ValidationHarness<S>,
) {
    let carrying_cap = 1000.0;
    let faction_a = Faction::new("Rome", 100.0, 0.0);
    let faction_b = Faction::new("Greece", 100.0, 0.0);

    let (da, db) = symbiont_interaction(&faction_a, &faction_b, carrying_cap);
    h.check_bool("both_grow_below_capacity", da > 0.0 && db > 0.0);

    let big_a = Faction::new("Empire", 900.0, 0.0);
    let small_b = Faction::new("City-State", 50.0, 0.0);
    let (da_big, db_small) = symbiont_interaction(&big_a, &small_b, carrying_cap);
    h.check_bool("big_faction_slows_near_cap", da_big < da);
    h.check_bool("small_faction_still_grows", db_small > 0.0);

    let mut a = Faction::new("A", 100.0, 0.0);
    let mut b = Faction::new("B", 100.0, 0.0);
    for _ in 0..200 {
        let (growth_a, growth_b) = symbiont_interaction(&a, &b, carrying_cap);
        a.population += growth_a;
        b.population += growth_b;
    }
    h.check_bool(
        "populations_bounded",
        a.population < carrying_cap && b.population < carrying_cap,
    );
    h.check_bool("both_survive", a.population > 10.0 && b.population > 10.0);

    let total = a.population + b.population;
    h.check_bool(
        "total_near_capacity",
        total > carrying_cap * 0.5 && total < carrying_cap * 1.5,
    );
}

/// Validate Hick's law: strategic decisions per turn are bounded.
fn validate_hick_strategic_decisions<S: ludospring_barracuda::validation::ValidationSink>(
    h: &mut ValidationHarness<S>,
) {
    let tree = TechTree::new();
    let researched = HashSet::new();
    let available = tree.available(&researched);
    let research_choices = available.len();

    let build_choices = 5usize;
    let move_choices = 4usize;
    let diplomacy_choices = 3usize;
    let total_choices = research_choices + build_choices + move_choices + diplomacy_choices;

    let hick_time = hick_reaction_time(total_choices, 200.0, 300.0);
    h.check_bool("hick_time_bounded", hick_time < 5000.0);

    let hick_minimal = hick_reaction_time(2, 200.0, 300.0);
    h.check_bool("hick_scales_with_choices", hick_time > hick_minimal);

    let hick_per_category = hick_reaction_time(research_choices, 200.0, 300.0);
    h.check_bool("hick_subcategory_faster", hick_per_category < hick_time);
}

/// Validate engagement from a simulated Civ session.
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "income * 0.3 is small positive; tech costs ≤ 50"
)]
fn validate_engagement<S: ludospring_barracuda::validation::ValidationSink>(
    h: &mut ValidationHarness<S>,
) {
    let tree = TechTree::new();
    let mut faction = Faction::new("Player", 100.0, 50.0);
    let turns = 50u32;
    let mut techs_researched = 0u32;
    let mut research_progress = 0u32;

    for _ in 0..turns {
        faction.tick_economy();
        let available = tree.available(&faction.researched);
        if let Some(tech) = available.first() {
            research_progress += (faction.income() * 0.3) as u32;
            if research_progress >= tech.cost {
                faction.researched.insert(tech.name.clone());
                techs_researched += 1;
                research_progress = 0;
            }
        }
    }

    h.check_bool("session_researches_techs", techs_researched > 0);

    let challenge = (f64::from(techs_researched) * 0.1).min(0.8);
    let flow = evaluate_flow(challenge, 0.5, 0.2);
    h.check_bool("flow_not_anxiety", flow != FlowState::Anxiety);

    let snap = EngagementSnapshot {
        session_duration_s: f64::from(turns) * 30.0,
        action_count: u64::from(turns * 3),
        exploration_breadth: 4,
        challenge_seeking: techs_researched,
        retry_count: 0,
        deliberate_pauses: turns / 5,
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
    fn civilization_validation_passes() {
        let mut h = ValidationHarness::with_sink("exp079_civilization", BufferSink::default());
        validate_fog_of_war(&mut h);
        validate_tech_tree_dag(&mut h);
        validate_tech_research_flow(&mut h);
        validate_faction_economy(&mut h);
        validate_symbiont_factions(&mut h);
        validate_hick_strategic_decisions(&mut h);
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

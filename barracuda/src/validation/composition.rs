// SPDX-License-Identifier: AGPL-3.0-or-later
//! Composition validation — exercising multi-model integration.
//!
//! This module validates that game science models compose correctly through
//! the full pipeline: theory → metrics → visualization → IPC wire format.
//!
//! Validation escalation levels:
//! 1. **Unit**: Each model (Bartle, Gamification, MDA) tested in isolation
//! 2. **Integration**: Models compose with each other (Bartle → NPC → MDA)
//! 3. **Wire**: Composed data serializes to valid scene payloads
//! 4. **E2E**: Full pipeline from model creation to scene rendering intent
//!
//! This is the "modern composition and validation escalation" pattern:
//! individual models are proven correct, then their compositions are
//! validated at increasingly broad scope.

use crate::game::rpgpt::knowledge::KnowledgeBounds;
use crate::game::rpgpt::npc::{MaslowNeeds, NeedState, NpcPersonality, NpcVoice};
use crate::game::rpgpt::personality_dynamics::{
    behavioral_summary, derive_bartle_profile, personality_drift, predict_npc_interaction,
};
use crate::game::rpgpt::trust::TrustModel;
use crate::metrics::gamification::{
    ComputationCredit, GameElement, GamificationProfile, MotivationType,
};
use crate::metrics::mda::{Aesthetic, AestheticDistribution};
use crate::metrics::player_types::{BartleProfile, population_dynamics, population_engagement};
use crate::visualization::scene::{SceneData, ScenePayload};

/// Composition scenario result.
#[derive(Debug)]
pub struct CompositionResult {
    /// Scenario name.
    pub name: &'static str,
    /// Whether validation passed.
    pub passed: bool,
    /// Detailed observations.
    pub observations: Vec<String>,
}

/// Run the full composition validation suite.
///
/// Exercises cross-model interactions and validates that outputs
/// compose into valid scene payloads for the visualization pipeline.
#[must_use]
pub fn run_composition_suite() -> Vec<CompositionResult> {
    vec![
        validate_bartle_npc_mda_pipeline(),
        validate_gamification_population_dynamics(),
        validate_npc_social_graph_scene(),
        validate_personality_drift_visualization(),
        validate_games_at_home_credit_pipeline(),
    ]
}

/// Validates: Bartle Profile → NPC Personality → MDA Aesthetic.
///
/// A player type distribution should predict which MDA aesthetics
/// the player population finds engaging.
fn validate_bartle_npc_mda_pipeline() -> CompositionResult {
    let mut observations = Vec::new();

    let achiever_pop = BartleProfile::new(8.0, 1.0, 1.0, 0.5);
    let explorer_pop = BartleProfile::new(1.0, 8.0, 1.0, 0.5);
    let socializer_pop = BartleProfile::new(1.0, 1.0, 8.0, 0.5);

    let achiever_aesthetics = mda_from_bartle(&achiever_pop);
    let explorer_aesthetics = mda_from_bartle(&explorer_pop);
    let socializer_aesthetics = mda_from_bartle(&socializer_pop);

    observations.push(format!(
        "Achiever → Challenge weight: {:.3}",
        achiever_aesthetics.weight(Aesthetic::Challenge)
    ));
    observations.push(format!(
        "Explorer → Discovery weight: {:.3}",
        explorer_aesthetics.weight(Aesthetic::Discovery)
    ));
    observations.push(format!(
        "Socializer → Fellowship weight: {:.3}",
        socializer_aesthetics.weight(Aesthetic::Fellowship)
    ));

    let achiever_valid = achiever_aesthetics.weight(Aesthetic::Challenge)
        > achiever_aesthetics.weight(Aesthetic::Fellowship);
    let explorer_valid = explorer_aesthetics.weight(Aesthetic::Discovery)
        > explorer_aesthetics.weight(Aesthetic::Challenge);
    let socializer_valid = socializer_aesthetics.weight(Aesthetic::Fellowship)
        > socializer_aesthetics.weight(Aesthetic::Submission);

    let passed = achiever_valid && explorer_valid && socializer_valid;
    if !passed {
        observations.push("FAIL: aesthetic alignment violated".into());
    }

    CompositionResult {
        name: "bartle_npc_mda_pipeline",
        passed,
        observations,
    }
}

/// Validates: Gamification → Population Dynamics → Equilibrium.
///
/// A heavily extrinsic gamification system should cause engagement decay
/// that correlates with Bartle population dynamics (killer surge → exodus).
fn validate_gamification_population_dynamics() -> CompositionResult {
    let mut observations = Vec::new();

    let elements = vec![
        GameElement {
            name: "pvp_leaderboard".into(),
            motivation: MotivationType::Extrinsic,
            engagement_uplift: 0.8,
            decay_rate: 0.1,
        },
        GameElement {
            name: "kill_rewards".into(),
            motivation: MotivationType::Extrinsic,
            engagement_uplift: 0.7,
            decay_rate: 0.08,
        },
    ];

    let profile = GamificationProfile::new(elements);
    let half_life = profile.half_life();
    observations.push(format!("Gamification half-life: {half_life:.2} time units"));

    let killer_heavy = BartleProfile::new(0.2, 0.1, 0.2, 0.5);
    let after_dynamics = population_dynamics(&killer_heavy, 0.3);

    observations.push(format!(
        "Pre-dynamics socializer: {:.3}, post-dynamics socializer: {:.3}",
        killer_heavy.socializer, after_dynamics.socializer
    ));

    let socializers_flee = after_dynamics.socializer < killer_heavy.socializer;
    let engagement_decays = half_life < 20.0;
    let passed = socializers_flee && engagement_decays;

    if !passed {
        observations.push("FAIL: expected socializer exodus and engagement decay".into());
    }

    CompositionResult {
        name: "gamification_population_dynamics",
        passed,
        observations,
    }
}

/// Validates: NPC Social Graph → Interaction Valences → Scene Graph.
///
/// Multiple NPCs with derived Bartle profiles should produce a valid
/// interaction graph that serializes to a GameScene payload.
fn validate_npc_social_graph_scene() -> CompositionResult {
    let mut observations = Vec::new();

    let npcs = vec![
        make_test_npc("merchant", 0.8, 0.2, 0.3, 0.1),
        make_test_npc("sage", 0.1, 0.2, 0.1, 0.9),
        make_test_npc("socialite", 0.1, 0.9, 0.1, 0.2),
        make_test_npc("warrior", 0.4, 0.1, 0.8, 0.1),
    ];

    let mut nodes = Vec::new();
    let mut edges = Vec::new();

    for npc in &npcs {
        let summary = behavioral_summary(npc);
        nodes.push((
            npc.id.clone(),
            npc.name.clone(),
            format!("{:?}", summary.dominant_type),
        ));
    }

    for (i, npc_a) in npcs.iter().enumerate() {
        for npc_b in npcs.iter().skip(i + 1) {
            let valence = predict_npc_interaction(npc_a, npc_b);
            if valence.abs() > 0.1 {
                edges.push((npc_a.id.clone(), npc_b.id.clone()));
            }
        }
    }

    observations.push(format!("Nodes: {}, Edges: {}", nodes.len(), edges.len()));

    let scene = ScenePayload {
        binding_type: "GameScene",
        id: "npc_social_graph".into(),
        label: "NPC Interaction Graph".into(),
        data: SceneData::GameScene { nodes, edges },
    };

    let valid_binding = scene.binding_type == "GameScene";
    let has_nodes = matches!(&scene.data, SceneData::GameScene { nodes, .. } if !nodes.is_empty());

    let passed = valid_binding && has_nodes;
    if !passed {
        observations.push("FAIL: invalid scene composition".into());
    }

    CompositionResult {
        name: "npc_social_graph_scene",
        passed,
        observations,
    }
}

/// Validates: Personality Drift → TimeSeries Scene Payload.
///
/// Simulates NPC personality evolution over time and produces a
/// time-series visualization of the drift.
fn validate_personality_drift_visualization() -> CompositionResult {
    let mut observations = Vec::new();

    let npc = make_test_npc("evolving_npc", 0.5, 0.3, 0.3, 0.4);
    let social_pressure = make_test_npc("social_env", 0.1, 0.9, 0.1, 0.1);

    let mut drift_history = Vec::new();
    let mut current_profile = derive_bartle_profile(&npc);

    for step in 0..10_i32 {
        drift_history.push((f64::from(step), current_profile.socializer));
        let drifted = personality_drift(&npc, &[(&social_pressure, 0.8)]);
        current_profile = drifted;
    }

    let x_values: Vec<f64> = drift_history.iter().map(|(x, _)| *x).collect();
    let y_values: Vec<f64> = drift_history.iter().map(|(_, y)| *y).collect();

    observations.push(format!(
        "Drift range: {:.3} → {:.3}",
        y_values.first().unwrap_or(&0.0),
        y_values.last().unwrap_or(&0.0)
    ));

    let scene = ScenePayload {
        binding_type: "TimeSeries",
        id: "personality_drift".into(),
        label: "NPC Personality Drift (Socializer weight)".into(),
        data: SceneData::TimeSeries {
            x_label: "Time Step".into(),
            y_label: "Socializer Weight".into(),
            unit: "normalized".into(),
            x_values,
            y_values: y_values.clone(),
        },
    };

    let valid_binding = scene.binding_type == "TimeSeries";
    let has_data =
        matches!(&scene.data, SceneData::TimeSeries { y_values, .. } if y_values.len() == 10);

    let all_bounded = y_values.iter().all(|v| (0.0..=1.0).contains(v));

    let passed = valid_binding && has_data && all_bounded;
    if !passed {
        observations.push("FAIL: invalid time series composition".into());
    }

    CompositionResult {
        name: "personality_drift_visualization",
        passed,
        observations,
    }
}

/// Validates: Games@Home Credit → Population Engagement → Gauge Payload.
///
/// Models the feedback loop where computation credits drive player engagement
/// which drives more computation participation.
fn validate_games_at_home_credit_pipeline() -> CompositionResult {
    let mut observations = Vec::new();

    let mut credit = ComputationCredit::new(5.0, 50.0, 1.3);

    for _ in 0..8 {
        credit.contribute(2.0);
    }

    let progress = credit.progress();
    observations.push(format!("Credit progress: {progress:.3}"));

    let population = vec![
        BartleProfile::new(3.0, 2.0, 1.0, 0.5),
        BartleProfile::new(2.0, 3.0, 2.0, 1.0),
        BartleProfile::new(1.0, 1.0, 4.0, 0.5),
    ];

    let content = vec![
        (
            crate::metrics::player_types::MechanicCategory::Progression,
            0.4,
        ),
        (
            crate::metrics::player_types::MechanicCategory::Collection,
            0.3,
        ),
        (
            crate::metrics::player_types::MechanicCategory::Discovery,
            0.3,
        ),
    ];

    let engagement = population_engagement(&population, &content);
    observations.push(format!("Population engagement: {engagement:.3}"));

    let scene = ScenePayload {
        binding_type: "Gauge",
        id: "games_at_home_engagement".into(),
        label: "Games@Home Engagement Score".into(),
        data: SceneData::Gauge {
            value: engagement,
            min: 0.0,
            max: 1.0,
            unit: "engagement".into(),
        },
    };

    let valid_binding = scene.binding_type == "Gauge";
    let valid_value =
        matches!(&scene.data, SceneData::Gauge { value, .. } if *value > 0.0 && *value <= 1.0);
    let credit_progressing = progress > 0.5;

    let passed = valid_binding && valid_value && credit_progressing;
    if !passed {
        observations.push("FAIL: engagement pipeline broken".into());
    }

    CompositionResult {
        name: "games_at_home_credit_pipeline",
        passed,
        observations,
    }
}

/// Map a Bartle profile to MDA aesthetic weights.
///
/// Achievers → Challenge, Explorers → Discovery, Socializers → Fellowship,
/// Killers → Submission (dominance over others).
fn mda_from_bartle(profile: &BartleProfile) -> AestheticDistribution {
    AestheticDistribution::from_weights(&[
        (
            Aesthetic::Sensation,
            profile.explorer.mul_add(0.3, profile.achiever * 0.1),
        ),
        (
            Aesthetic::Fantasy,
            profile.explorer.mul_add(0.4, profile.socializer * 0.2),
        ),
        (
            Aesthetic::Narrative,
            profile.socializer.mul_add(0.3, profile.explorer * 0.3),
        ),
        (
            Aesthetic::Challenge,
            profile.achiever.mul_add(0.6, profile.killer * 0.3),
        ),
        (
            Aesthetic::Fellowship,
            profile.socializer.mul_add(0.6, profile.explorer * 0.1),
        ),
        (
            Aesthetic::Discovery,
            profile.explorer.mul_add(0.6, profile.achiever * 0.1),
        ),
        (
            Aesthetic::Expression,
            profile.socializer.mul_add(0.3, profile.explorer * 0.2),
        ),
        (
            Aesthetic::Submission,
            profile.killer.mul_add(0.5, profile.achiever * 0.2),
        ),
    ])
}

fn make_test_npc(
    id: &str,
    survival: f64,
    belonging: f64,
    esteem: f64,
    actualization: f64,
) -> NpcPersonality {
    NpcPersonality {
        id: id.into(),
        name: id.into(),
        role: "test".into(),
        appearance: String::new(),
        mannerisms: vec![],
        motivations: MaslowNeeds {
            survival: NeedState::new(survival, ""),
            safety: NeedState::new(0.3, ""),
            belonging: NeedState::new(belonging, ""),
            esteem: NeedState::new(esteem, ""),
            self_actualization: NeedState::new(actualization, ""),
        },
        motivation_conflicts: vec![],
        knowledge: KnowledgeBounds {
            knows: vec![],
            suspects: vec![],
            lies_about: vec![],
            does_not_know: vec![],
        },
        voice: NpcVoice {
            speech_patterns: vec![],
            vocabulary_level: String::new(),
            emotional_baseline: String::new(),
            catchphrases: vec![],
        },
        secrets: vec![],
        relationships: vec![],
        arc: vec![],
        trust: TrustModel::new(5),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn composition_suite_all_pass() {
        let results = run_composition_suite();
        for result in &results {
            assert!(
                result.passed,
                "Composition scenario '{}' failed: {:?}",
                result.name, result.observations
            );
        }
    }

    #[test]
    fn composition_suite_count() {
        let results = run_composition_suite();
        assert_eq!(results.len(), 5);
    }

    #[test]
    fn bartle_to_mda_mapping_ranges() {
        use crate::metrics::mda::Aesthetic;
        let balanced = BartleProfile::new(1.0, 1.0, 1.0, 1.0);
        let aesthetics = mda_from_bartle(&balanced);
        assert!(aesthetics.weight(Aesthetic::Challenge) >= 0.0);
        assert!(aesthetics.weight(Aesthetic::Fellowship) >= 0.0);
        assert!(aesthetics.weight(Aesthetic::Discovery) >= 0.0);
    }

    #[test]
    fn npc_social_graph_produces_scene() {
        let result = validate_npc_social_graph_scene();
        assert!(result.passed, "{:?}", result.observations);
    }

    #[test]
    fn drift_visualization_produces_bounded_output() {
        let result = validate_personality_drift_visualization();
        assert!(result.passed, "{:?}", result.observations);
    }
}

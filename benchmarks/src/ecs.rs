// SPDX-License-Identifier: AGPL-3.0-or-later
//! Entity-component tick benchmarks (BM-001).
//!
//! Measures entity iteration and game-state update throughput.
//! This benchmarks ludoSpring's tick model against what an ECS like Bevy
//! achieves for equivalent workloads.
//!
//! **Benchmark targets** (from `OPEN_SYSTEMS_BENCHMARK_SPECIFICATION.md`):
//! - 10K entities with 5 systems at 60 Hz (< 3ms per tick)
//! - Match Bevy ECS throughput within 1.5x for equivalent iteration
//!
//! **Open system baselines**:
//! - Bevy `bevy_ecs` (MIT/Apache-2.0): Rust-native ECS
//!
//! ludoSpring does not use a traditional ECS — it uses a tick-based state
//! machine with replay buffer. These benchmarks measure the overhead of
//! our model vs pure ECS iteration to inform architecture decisions.

use ludospring_barracuda::interaction::difficulty::{PerformanceWindow, suggest_adjustment};
use ludospring_barracuda::interaction::flow::evaluate_flow;
use ludospring_barracuda::metrics::engagement::{EngagementSnapshot, compute_engagement};
use ludospring_barracuda::tolerances::{DDA_TARGET_SUCCESS_RATE, FLOW_CHANNEL_WIDTH};

/// Simulated entity with position and velocity for tick benchmark.
#[derive(Clone)]
pub struct Entity {
    /// X position.
    pub x: f64,
    /// Y position.
    pub y: f64,
    /// X velocity.
    pub vx: f64,
    /// Y velocity.
    pub vy: f64,
    /// Challenge level for this entity's interaction.
    pub challenge: f64,
    /// Skill level of the player interacting with this entity.
    pub skill: f64,
}

/// Run a single game-logic tick: move entities, evaluate flow, adjust difficulty.
///
/// This simulates the full `game_logic` node workload per tick.
#[expect(
    clippy::cast_precision_loss,
    reason = "entity counts ≤ 100_000; len fits in f64 mantissa"
)]
pub fn tick_game_logic(entities: &mut [Entity], dt: f64) -> TickResult {
    let mut flow_count = [0u32; 5]; // boredom, relaxation, flow, arousal, anxiety
    let mut total_adjustment = 0.0;

    for entity in entities.iter_mut() {
        entity.x += entity.vx * dt;
        entity.y += entity.vy * dt;

        let state = evaluate_flow(entity.challenge, entity.skill, FLOW_CHANNEL_WIDTH);
        let idx = match state {
            ludospring_barracuda::interaction::flow::FlowState::Boredom => 0,
            ludospring_barracuda::interaction::flow::FlowState::Relaxation => 1,
            ludospring_barracuda::interaction::flow::FlowState::Flow => 2,
            ludospring_barracuda::interaction::flow::FlowState::Arousal => 3,
            ludospring_barracuda::interaction::flow::FlowState::Anxiety => 4,
        };
        flow_count[idx] += 1;

        let mut window = PerformanceWindow::new(10);
        window.record(entity.skill);
        let adj = suggest_adjustment(&window, DDA_TARGET_SUCCESS_RATE);
        total_adjustment += adj;

        entity.challenge = (entity.challenge + adj * 0.01).clamp(0.0, 1.0);
    }

    TickResult {
        flow_distribution: flow_count,
        mean_adjustment: total_adjustment / entities.len().max(1) as f64,
    }
}

/// Run the metrics node: compute engagement for the session.
#[must_use]
pub fn tick_metrics(action_count: u64, session_s: f64) -> f64 {
    let snap = EngagementSnapshot {
        session_duration_s: session_s,
        action_count,
        exploration_breadth: 10,
        challenge_seeking: 5,
        retry_count: 3,
        deliberate_pauses: 2,
    };
    compute_engagement(&snap).composite
}

/// Result of a single game-logic tick.
pub struct TickResult {
    /// Count of entities in each flow state.
    pub flow_distribution: [u32; 5],
    /// Mean difficulty adjustment across all entities.
    pub mean_adjustment: f64,
}

/// Create a batch of test entities for benchmarking.
#[must_use]
#[expect(
    clippy::cast_precision_loss,
    reason = "entity counts ≤ 100_000; usize fits in f64 mantissa"
)]
pub fn spawn_entities(count: usize) -> Vec<Entity> {
    (0..count)
        .map(|i| {
            let t = i as f64 / count as f64;
            Entity {
                x: t * 100.0,
                y: (1.0 - t) * 100.0,
                vx: t.sin() * 2.0,
                vy: t.cos() * 2.0,
                challenge: 0.3 + t * 0.4,
                skill: 0.4 + (1.0 - t) * 0.3,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tick_updates_positions() {
        let mut entities = spawn_entities(100);
        let x_before = entities[50].x;
        tick_game_logic(&mut entities, 1.0 / 60.0);
        assert!((entities[50].x - x_before).abs() > f64::EPSILON);
    }

    #[test]
    fn tick_10k_entities_completes() {
        let mut entities = spawn_entities(10_000);
        let result = tick_game_logic(&mut entities, 1.0 / 60.0);
        let total: u32 = result.flow_distribution.iter().sum();
        assert_eq!(total, 10_000);
    }

    #[test]
    fn metrics_returns_finite() {
        let composite = tick_metrics(500_u64, 300.0);
        assert!(composite.is_finite());
        assert!(composite >= 0.0);
        assert!(composite <= 1.0);
    }
}

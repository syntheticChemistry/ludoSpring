// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
//! exp076 — Pong: the foundational game.
//!
//! Validates the bedrock of all electronic games against published HCI models:
//!
//! 1. **Newtonian ball physics**: `position += velocity * dt`, elastic reflection
//! 2. **Fitts's law** (Fitts 1954): paddle interception cost — target is the ball,
//!    width is the paddle. Predicts interception difficulty.
//! 3. **Flow** (Csikszentmihalyi 1990): ball speed ramp creates the original DDA.
//!    Challenge rises with score. The game invented Flow maintenance before the
//!    theory was published.
//! 4. **DDA** (Hunicke 2005): speed increase on rally IS dynamic difficulty
//!    adjustment. Validated by tracking performance window across rallies.
//! 5. **Real-time game loop**: 60 Hz input → update → render — the pattern every
//!    game inherits.
//!
//! The game logic is modality-agnostic. TUI, GUI, VR, headless — the physics
//! is `position += velocity * dt` in every case. Only the renderer changes.
//! This experiment validates the math in headless mode.
//!
//! Cross-spring: ball physics is particle simulation. hotSpring MD uses the same
//! Verlet integration for atom positions. Same math, different particle.

use std::process;

use ludospring_barracuda::interaction::difficulty::{PerformanceWindow, suggest_adjustment};
use ludospring_barracuda::interaction::flow::{DifficultyCurve, FlowState, evaluate_flow};
use ludospring_barracuda::interaction::input_laws::fitts_movement_time;
use ludospring_barracuda::metrics::engagement::{EngagementSnapshot, compute_engagement};
use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "N/A (analytical — Newtonian mechanics, Fitts 1954, Csikszentmihalyi 1990)",
    commit: "19e402c0",
    date: "2026-03-18",
    command: "N/A (analytical — Pong first principles)",
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

/// Ball state in 2D continuous space.
#[derive(Debug, Clone, Copy)]
struct Ball {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    radius: f64,
}

/// Paddle state — vertical position on one side of the court.
#[derive(Debug, Clone, Copy)]
struct Paddle {
    y: f64,
    height: f64,
    side_x: f64,
}

/// Court dimensions.
struct Court {
    width: f64,
    height: f64,
}

/// Simulation result for one rally.
struct RallyResult {
    bounces: u32,
    duration_s: f64,
    final_speed: f64,
    scored_by: Side,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Side {
    Left,
    Right,
}

const DT: f64 = 1.0 / 60.0;
const INITIAL_SPEED: f64 = 200.0;
const SPEED_INCREMENT: f64 = 15.0;
const COURT: Court = Court {
    width: 800.0,
    height: 600.0,
};
const PADDLE_HEIGHT: f64 = 80.0;
const BALL_RADIUS: f64 = 8.0;

impl Ball {
    fn new() -> Self {
        Self {
            x: COURT.width / 2.0,
            y: COURT.height / 2.0,
            vx: INITIAL_SPEED,
            vy: INITIAL_SPEED * 0.5,
            radius: BALL_RADIUS,
        }
    }

    fn speed(&self) -> f64 {
        self.vx.hypot(self.vy)
    }

    fn step(&mut self) {
        self.x += self.vx * DT;
        self.y += self.vy * DT;
    }

    fn bounce_top_bottom(&mut self) {
        if self.y - self.radius < 0.0 {
            self.y = self.radius;
            self.vy = self.vy.abs();
        }
        if self.y + self.radius > COURT.height {
            self.y = COURT.height - self.radius;
            self.vy = -self.vy.abs();
        }
    }

    fn check_paddle(&mut self, paddle: &Paddle) -> bool {
        let on_paddle_side = if self.vx < 0.0 {
            self.x - self.radius <= paddle.side_x
        } else {
            self.x + self.radius >= paddle.side_x
        };

        if !on_paddle_side {
            return false;
        }

        let paddle_top = paddle.y - paddle.height / 2.0;
        let paddle_bottom = paddle.y + paddle.height / 2.0;
        if self.y >= paddle_top && self.y <= paddle_bottom {
            self.vx = -self.vx;
            let offset = (self.y - paddle.y) / (paddle.height / 2.0);
            self.vy += offset * 30.0;
            let speed = self.speed();
            let new_speed = speed + SPEED_INCREMENT;
            let scale = new_speed / speed;
            self.vx *= scale;
            self.vy *= scale;
            true
        } else {
            false
        }
    }
}

/// Simple AI: tracks ball y with bounded speed.
fn ai_track(paddle: &mut Paddle, ball: &Ball, skill: f64) {
    let max_speed = 300.0 * skill;
    let diff = ball.y - paddle.y;
    let movement = diff.clamp(-max_speed * DT, max_speed * DT);
    paddle.y += movement;
    paddle.y = paddle
        .y
        .clamp(paddle.height / 2.0, COURT.height - paddle.height / 2.0);
}

/// Simulate one rally and return metrics.
#[expect(
    clippy::cast_precision_loss,
    reason = "tick counts ≤ 7200 fit in f64 mantissa"
)]
fn simulate_rally(left_skill: f64, right_skill: f64) -> RallyResult {
    let mut ball = Ball::new();
    let mut left = Paddle {
        y: COURT.height / 2.0,
        height: PADDLE_HEIGHT,
        side_x: 20.0,
    };
    let mut right = Paddle {
        y: COURT.height / 2.0,
        height: PADDLE_HEIGHT,
        side_x: COURT.width - 20.0,
    };

    let mut bounces = 0u32;
    let mut ticks = 0u64;
    let max_ticks = 60 * 120; // 2 minutes max

    loop {
        ticks += 1;
        if ticks > max_ticks {
            break;
        }

        ai_track(&mut left, &ball, left_skill);
        ai_track(&mut right, &ball, right_skill);
        ball.step();
        ball.bounce_top_bottom();

        if ball.check_paddle(&left) || ball.check_paddle(&right) {
            bounces += 1;
        }

        if ball.x - ball.radius < 0.0 {
            return RallyResult {
                bounces,
                duration_s: ticks as f64 * DT,
                final_speed: ball.speed(),
                scored_by: Side::Right,
            };
        }
        if ball.x + ball.radius > COURT.width {
            return RallyResult {
                bounces,
                duration_s: ticks as f64 * DT,
                final_speed: ball.speed(),
                scored_by: Side::Left,
            };
        }
    }

    RallyResult {
        bounces,
        duration_s: ticks as f64 * DT,
        final_speed: ball.speed(),
        scored_by: Side::Left,
    }
}

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

fn cmd_validate() -> ! {
    let mut h = ValidationHarness::new("exp076_pong");
    h.print_provenance(&[&PROVENANCE]);

    validate_ball_physics(&mut h);
    validate_fitts_interception(&mut h);
    validate_flow_speed_ramp(&mut h);
    validate_dda_from_rallies(&mut h);
    validate_engagement_session(&mut h);
    validate_modality_agnostic(&mut h);

    h.finish();
}

/// Validate Newtonian ball mechanics: position integration, elastic reflection.
fn validate_ball_physics<S: ludospring_barracuda::validation::ValidationSink>(
    h: &mut ValidationHarness<S>,
) {
    let mut ball = Ball::new();
    let initial_x = ball.x;
    let initial_speed = ball.speed();

    ball.step();
    let expected_x = ball.vx.mul_add(DT, initial_x);
    h.check_abs("ball_x_after_step", ball.x, expected_x, 1e-10);

    let mut wall_ball = Ball {
        x: COURT.width / 2.0,
        y: 5.0,
        vx: 100.0,
        vy: -200.0,
        radius: BALL_RADIUS,
    };
    wall_ball.step();
    wall_ball.bounce_top_bottom();
    h.check_bool("ball_bounces_off_top", wall_ball.vy > 0.0);
    h.check_bool("ball_stays_in_court", wall_ball.y >= wall_ball.radius);

    let mut floor_ball = Ball {
        x: COURT.width / 2.0,
        y: COURT.height - 3.0,
        vx: 100.0,
        vy: 200.0,
        radius: BALL_RADIUS,
    };
    floor_ball.step();
    floor_ball.bounce_top_bottom();
    h.check_bool("ball_bounces_off_bottom", floor_ball.vy < 0.0);

    let speed_after = ball.speed();
    h.check_abs(
        "speed_conserved_free_flight",
        speed_after,
        initial_speed,
        1e-10,
    );
}

/// Validate Fitts's law: paddle interception difficulty scales with distance/width ratio.
fn validate_fitts_interception<S: ludospring_barracuda::validation::ValidationSink>(
    h: &mut ValidationHarness<S>,
) {
    let paddle_width = PADDLE_HEIGHT;
    let near_distance = 50.0;
    let far_distance = 300.0;

    let near_time = fitts_movement_time(near_distance, paddle_width, 50.0, 150.0);
    let far_time = fitts_movement_time(far_distance, paddle_width, 50.0, 150.0);
    h.check_bool("fitts_far_harder_than_near", far_time > near_time);

    let small_paddle = 20.0;
    let large_paddle = 120.0;
    let small_time = fitts_movement_time(200.0, small_paddle, 50.0, 150.0);
    let large_time = fitts_movement_time(200.0, large_paddle, 50.0, 150.0);
    h.check_bool("fitts_small_paddle_harder", small_time > large_time);

    h.check_bool("fitts_near_time_positive", near_time > 0.0);
    h.check_bool("fitts_far_time_positive", far_time > 0.0);

    let id_near = (near_distance / paddle_width + 1.0).log2();
    let id_far = (far_distance / paddle_width + 1.0).log2();
    h.check_bool("fitts_index_of_difficulty_scales", id_far > id_near);
}

/// Validate Flow: ball speed ramp creates a difficulty curve that matches
/// Csikszentmihalyi's Flow channel.
fn validate_flow_speed_ramp<S: ludospring_barracuda::validation::ValidationSink>(
    h: &mut ValidationHarness<S>,
) {
    let curve = DifficultyCurve::linear(0.2, 0.9);

    let early_challenge = curve.sample(0.0);
    let mid_challenge = curve.sample(0.5);
    let late_challenge = curve.sample(1.0);
    h.check_bool(
        "flow_difficulty_increases",
        late_challenge > early_challenge,
    );
    h.check_bool(
        "flow_mid_between",
        mid_challenge > early_challenge && mid_challenge < late_challenge,
    );

    let beginner_skill = 0.3;
    let early_flow = evaluate_flow(early_challenge, beginner_skill, 0.15);
    h.check_bool(
        "flow_beginner_early_not_anxiety",
        early_flow != FlowState::Anxiety,
    );

    let expert_skill = 0.85;
    let late_flow = evaluate_flow(late_challenge, expert_skill, 0.15);
    h.check_bool(
        "flow_expert_late_not_boredom",
        late_flow != FlowState::Boredom,
    );

    let matched_flow = evaluate_flow(0.5, 0.5, 0.15);
    h.check_bool("flow_matched_is_flow", matched_flow == FlowState::Flow);

    let rally = simulate_rally(0.7, 0.7);
    h.check_bool("rally_speed_increases", rally.final_speed > INITIAL_SPEED);

    let speed_ratio = rally.final_speed / INITIAL_SPEED;
    let min_expected = 1.0 + (f64::from(rally.bounces) * SPEED_INCREMENT / INITIAL_SPEED);
    h.check_bool(
        "speed_ramp_at_least_bounce_increment",
        speed_ratio >= min_expected * 0.8,
    );
}

/// Validate DDA: performance window tracks skill and suggests adjustments.
fn validate_dda_from_rallies<S: ludospring_barracuda::validation::ValidationSink>(
    h: &mut ValidationHarness<S>,
) {
    let mut perf = PerformanceWindow::new(20);

    for _ in 0..10 {
        let result = simulate_rally(0.9, 0.5);
        let outcome = if result.scored_by == Side::Left {
            1.0
        } else {
            0.0
        };
        perf.record(outcome);
    }

    let strong_skill = perf.estimated_skill();
    h.check_bool("dda_strong_player_high_skill", strong_skill > 0.5);

    let adj = suggest_adjustment(&perf, 0.65);
    h.check_bool("dda_suggests_increase_for_strong", adj > 0.0);

    let mut weak_perf = PerformanceWindow::new(20);
    for _ in 0..10 {
        let result = simulate_rally(0.3, 0.9);
        let outcome = if result.scored_by == Side::Left {
            1.0
        } else {
            0.0
        };
        weak_perf.record(outcome);
    }

    let weak_adj = suggest_adjustment(&weak_perf, 0.65);
    h.check_bool("dda_suggests_decrease_for_weak", weak_adj < 0.0);

    let mut mixed_perf = PerformanceWindow::new(20);
    for i in 0..20 {
        let (l, r) = if i % 2 == 0 { (0.7, 0.6) } else { (0.6, 0.7) };
        let result = simulate_rally(l, r);
        let outcome = if result.scored_by == Side::Left {
            1.0
        } else {
            0.0
        };
        mixed_perf.record(outcome);
    }
    let mixed_skill = mixed_perf.estimated_skill();
    h.check_bool(
        "dda_mixed_skill_in_range",
        mixed_skill > 0.0 && mixed_skill < 1.0,
    );
}

/// Validate engagement metrics from a simulated Pong session.
fn validate_engagement_session<S: ludospring_barracuda::validation::ValidationSink>(
    h: &mut ValidationHarness<S>,
) {
    let num_rallies = 20u32;
    let mut total_bounces = 0u32;
    let mut total_duration = 0.0f64;

    for _ in 0..num_rallies {
        let result = simulate_rally(0.7, 0.7);
        total_bounces += result.bounces;
        total_duration += result.duration_s;
    }

    let snap = EngagementSnapshot {
        session_duration_s: total_duration,
        action_count: u64::from(total_bounces),
        exploration_breadth: 1,
        challenge_seeking: num_rallies,
        retry_count: num_rallies,
        deliberate_pauses: 0,
    };

    let metrics = compute_engagement(&snap);
    h.check_bool("engagement_positive", metrics.composite > 0.0);
    h.check_bool("engagement_bounded", metrics.composite <= 1.0);

    let action_density = f64::from(total_bounces) / total_duration;
    h.check_bool("action_density_positive", action_density > 0.0);
}

/// Validate that game logic is modality-independent: same inputs → same outputs
/// regardless of (absent) renderer.
fn validate_modality_agnostic<S: ludospring_barracuda::validation::ValidationSink>(
    h: &mut ValidationHarness<S>,
) {
    let mut ball_a = Ball::new();
    let mut ball_b = Ball::new();

    for _ in 0..100 {
        ball_a.step();
        ball_a.bounce_top_bottom();
        ball_b.step();
        ball_b.bounce_top_bottom();
    }

    h.check_abs("modality_x_identical", ball_a.x, ball_b.x, 1e-10);
    h.check_abs("modality_y_identical", ball_a.y, ball_b.y, 1e-10);
    h.check_abs("modality_vx_identical", ball_a.vx, ball_b.vx, 1e-10);
    h.check_abs("modality_vy_identical", ball_a.vy, ball_b.vy, 1e-10);

    let rally_1 = simulate_rally(0.7, 0.7);
    let rally_2 = simulate_rally(0.7, 0.7);
    h.check_bool(
        "deterministic_same_inputs",
        rally_1.bounces == rally_2.bounces,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use ludospring_barracuda::validation::BufferSink;

    #[test]
    fn pong_validation_passes() {
        let mut h = ValidationHarness::with_sink("exp076_pong", BufferSink::default());
        validate_ball_physics(&mut h);
        validate_fitts_interception(&mut h);
        validate_flow_speed_ramp(&mut h);
        validate_dda_from_rallies(&mut h);
        validate_engagement_session(&mut h);
        validate_modality_agnostic(&mut h);
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

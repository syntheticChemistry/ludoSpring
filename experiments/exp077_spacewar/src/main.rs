// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
//! exp077 — Spacewar/Asteroids: continuous 2D physics and wrap topology.
//!
//! Validates the bridge from grid-based games (Pong, Tetris) to continuous
//! vector physics — the math that underlies every 2D action game since 1962.
//!
//! 1. **Newtonian thrust**: `velocity += thrust_direction * magnitude * dt`.
//!    Continuous acceleration on a 2D plane. The same F=ma that hotSpring
//!    validates for molecular dynamics, applied to spaceships.
//! 2. **Wrap topology** (toroidal space): position modulo field dimensions.
//!    Objects leaving one edge appear on the opposite side. This is a torus —
//!    the same periodic boundary conditions used in MD simulations.
//! 3. **Angular mechanics**: rotation + directional thrust. The ship has
//!    heading (angle), and thrust applies in the heading direction.
//! 4. **Projectile ballistics**: bullets travel in straight lines at fixed
//!    speed + ship velocity. Collision = distance < sum of radii.
//! 5. **N-body gravity** (Spacewar's central star): gravitational attraction
//!    toward a point mass. `F = G * m1 * m2 / r^2`. Same inverse-square law
//!    as astrophysical N-body simulation.
//! 6. **Flow via survival pressure**: asteroids increase over time, creating
//!    escalating challenge without explicit speed ramp.
//!
//! Cross-spring: toroidal wrap = periodic boundary conditions in MD. Thrust
//! integration = Verlet/Euler integration of particle positions. Gravity =
//! same inverse-square force as hotSpring N-body. Same math, different scale.

use std::f64::consts::PI;
use std::process;

use ludospring_barracuda::interaction::flow::{FlowState, evaluate_flow};
use ludospring_barracuda::metrics::engagement::{EngagementSnapshot, compute_engagement};
use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "N/A (analytical — Newton 1687, Euler integration, toroidal topology)",
    commit: "19e402c0",
    date: "2026-03-18",
    command: "N/A (analytical — Spacewar/Asteroids first principles)",
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
// Core types
// ---------------------------------------------------------------------------

/// 2D vector for positions and velocities.
#[derive(Debug, Clone, Copy)]
struct Vec2 {
    x: f64,
    y: f64,
}

impl Vec2 {
    const ZERO: Self = Self { x: 0.0, y: 0.0 };

    fn magnitude(self) -> f64 {
        self.x.hypot(self.y)
    }

    fn distance_wrapped(self, other: Self, field: &Field) -> f64 {
        let dx = wrap_delta(self.x - other.x, field.width);
        let dy = wrap_delta(self.y - other.y, field.height);
        dx.hypot(dy)
    }

    fn from_angle(angle: f64) -> Self {
        Self {
            x: angle.cos(),
            y: angle.sin(),
        }
    }
}

impl std::ops::Add for Vec2 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl std::ops::Sub for Vec2 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl std::ops::Mul<f64> for Vec2 {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

/// Wrap a coordinate delta to the shortest path on a torus.
fn wrap_delta(delta: f64, size: f64) -> f64 {
    let mut d = delta % size;
    if d > size / 2.0 {
        d -= size;
    }
    if d < -size / 2.0 {
        d += size;
    }
    d
}

/// Wrap position into [0, size).
fn wrap_coord(val: f64, size: f64) -> f64 {
    let mut v = val % size;
    if v < 0.0 {
        v += size;
    }
    v
}

/// The game field (toroidal space).
struct Field {
    width: f64,
    height: f64,
}

impl Field {
    fn wrap_position(&self, pos: Vec2) -> Vec2 {
        Vec2 {
            x: wrap_coord(pos.x, self.width),
            y: wrap_coord(pos.y, self.height),
        }
    }
}

// ---------------------------------------------------------------------------
// Ship
// ---------------------------------------------------------------------------

struct Ship {
    pos: Vec2,
    vel: Vec2,
    heading: f64,
    radius: f64,
    alive: bool,
}

impl Ship {
    fn thrust(&mut self, magnitude: f64, dt: f64) {
        let dir = Vec2::from_angle(self.heading);
        self.vel = self.vel + dir * (magnitude * dt);
    }

    fn rotate(&mut self, angular_vel: f64, dt: f64) {
        self.heading += angular_vel * dt;
    }

    fn integrate(&mut self, dt: f64, field: &Field) {
        self.pos = field.wrap_position(self.pos + self.vel * dt);
    }
}

// ---------------------------------------------------------------------------
// Projectile
// ---------------------------------------------------------------------------

struct Bullet {
    pos: Vec2,
    vel: Vec2,
    alive: bool,
    ttl: f64,
}

impl Bullet {
    fn integrate(&mut self, dt: f64, field: &Field) {
        self.pos = field.wrap_position(self.pos + self.vel * dt);
        self.ttl -= dt;
        if self.ttl <= 0.0 {
            self.alive = false;
        }
    }
}

// ---------------------------------------------------------------------------
// Asteroid
// ---------------------------------------------------------------------------

struct Asteroid {
    pos: Vec2,
    vel: Vec2,
    radius: f64,
    alive: bool,
}

impl Asteroid {
    fn integrate(&mut self, dt: f64, field: &Field) {
        self.pos = field.wrap_position(self.pos + self.vel * dt);
    }
}

// ---------------------------------------------------------------------------
// Gravity
// ---------------------------------------------------------------------------

/// Gravitational acceleration toward a point mass at `center`.
///
/// Returns acceleration vector. Clamps minimum distance to avoid singularity.
fn gravity_accel(pos: Vec2, center: Vec2, gm: f64, field: &Field) -> Vec2 {
    let dx = wrap_delta(center.x - pos.x, field.width);
    let dy = wrap_delta(center.y - pos.y, field.height);
    let r_sq = dx.mul_add(dx, dy * dy).max(100.0);
    let r = r_sq.sqrt();
    let a = gm / r_sq;
    Vec2 {
        x: a * dx / r,
        y: a * dy / r,
    }
}

// ---------------------------------------------------------------------------
// Simulation
// ---------------------------------------------------------------------------

const DT: f64 = 1.0 / 60.0;
const FIELD: Field = Field {
    width: 800.0,
    height: 600.0,
};
const BULLET_SPEED: f64 = 400.0;
const BULLET_TTL: f64 = 2.0;
const SHIP_THRUST: f64 = 200.0;
const SHIP_ROTATION: f64 = 3.0;
const GM: f64 = 50000.0;

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

fn cmd_validate() -> ! {
    let mut h = ValidationHarness::new("exp077_spacewar");
    h.print_provenance(&[&PROVENANCE]);

    validate_thrust_integration(&mut h);
    validate_wrap_topology(&mut h);
    validate_angular_mechanics(&mut h);
    validate_projectile_ballistics(&mut h);
    validate_gravity(&mut h);
    validate_collision_detection(&mut h);
    validate_energy_conservation(&mut h);
    validate_asteroid_session(&mut h);

    h.finish();
}

/// Validate Newtonian thrust: F=ma integration in 2D.
fn validate_thrust_integration<S: ludospring_barracuda::validation::ValidationSink>(
    h: &mut ValidationHarness<S>,
) {
    let mut ship = Ship {
        pos: Vec2 { x: 400.0, y: 300.0 },
        vel: Vec2::ZERO,
        heading: 0.0,
        radius: 10.0,
        alive: true,
    };

    ship.thrust(SHIP_THRUST, DT);
    h.check_bool("thrust_increases_vx", ship.vel.x > 0.0);
    h.check_abs("thrust_vy_zero", ship.vel.y, 0.0, 1e-10);

    let thrust_vx = SHIP_THRUST * DT;
    h.check_abs("thrust_magnitude_correct", ship.vel.x, thrust_vx, 1e-10);

    let initial_x = ship.pos.x;
    ship.integrate(DT, &FIELD);
    let pos_after = thrust_vx.mul_add(DT, initial_x);
    h.check_abs("position_integration_correct", ship.pos.x, pos_after, 1e-10);

    ship.heading = PI / 2.0;
    ship.vel = Vec2::ZERO;
    ship.thrust(SHIP_THRUST, DT);
    h.check_abs("thrust_90deg_vx_zero", ship.vel.x, 0.0, 1e-10);
    h.check_bool("thrust_90deg_vy_positive", ship.vel.y > 0.0);
}

/// Validate toroidal wrap: leaving one edge appears on the opposite.
fn validate_wrap_topology<S: ludospring_barracuda::validation::ValidationSink>(
    h: &mut ValidationHarness<S>,
) {
    let mut ship = Ship {
        pos: Vec2 { x: 799.0, y: 300.0 },
        vel: Vec2 { x: 200.0, y: 0.0 },
        heading: 0.0,
        radius: 10.0,
        alive: true,
    };

    ship.integrate(DT, &FIELD);
    h.check_bool("wrap_right_to_left", ship.pos.x < 100.0);

    ship.pos = Vec2 { x: 1.0, y: 300.0 };
    ship.vel = Vec2 { x: -200.0, y: 0.0 };
    ship.integrate(DT, &FIELD);
    h.check_bool("wrap_left_to_right", ship.pos.x > 700.0);

    ship.pos = Vec2 { x: 400.0, y: 599.0 };
    ship.vel = Vec2 { x: 0.0, y: 200.0 };
    ship.integrate(DT, &FIELD);
    h.check_bool("wrap_bottom_to_top", ship.pos.y < 100.0);

    ship.pos = Vec2 { x: 400.0, y: 1.0 };
    ship.vel = Vec2 { x: 0.0, y: -200.0 };
    ship.integrate(DT, &FIELD);
    h.check_bool("wrap_top_to_bottom", ship.pos.y > 500.0);

    let a = Vec2 { x: 10.0, y: 300.0 };
    let b = Vec2 { x: 790.0, y: 300.0 };
    let wrapped_dist = a.distance_wrapped(b, &FIELD);
    h.check_bool("wrapped_distance_shorter", wrapped_dist < 100.0);
    let naive_dist = (a.x - b.x).abs();
    h.check_bool("naive_distance_longer", naive_dist > 700.0);
}

/// Validate angular mechanics: rotation changes heading, thrust direction follows.
fn validate_angular_mechanics<S: ludospring_barracuda::validation::ValidationSink>(
    h: &mut ValidationHarness<S>,
) {
    let mut ship = Ship {
        pos: Vec2 { x: 400.0, y: 300.0 },
        vel: Vec2::ZERO,
        heading: 0.0,
        radius: 10.0,
        alive: true,
    };

    let full_rotation_time = (2.0 * PI) / SHIP_ROTATION;
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "rotation steps ≈ 125, fits in u32"
    )]
    let steps = (full_rotation_time / DT).round() as u32;
    for _ in 0..steps {
        ship.rotate(SHIP_ROTATION, DT);
    }
    let heading_mod = ship.heading % (2.0 * PI);
    h.check_abs("full_rotation_returns", heading_mod, 0.0, 0.1);

    ship.heading = PI;
    ship.vel = Vec2::ZERO;
    ship.thrust(SHIP_THRUST, DT);
    h.check_bool("thrust_pi_vx_negative", ship.vel.x < 0.0);
    h.check_abs("thrust_pi_vy_near_zero", ship.vel.y, 0.0, 1e-10);
}

/// Validate projectile physics: bullets travel at speed + ship velocity, have TTL.
fn validate_projectile_ballistics<S: ludospring_barracuda::validation::ValidationSink>(
    h: &mut ValidationHarness<S>,
) {
    let ship_vel = Vec2 { x: 100.0, y: 0.0 };
    let heading = 0.0;
    let dir = Vec2::from_angle(heading);

    let mut bullet = Bullet {
        pos: Vec2 { x: 400.0, y: 300.0 },
        vel: ship_vel + dir * BULLET_SPEED,
        alive: true,
        ttl: BULLET_TTL,
    };

    h.check_abs(
        "bullet_initial_speed",
        bullet.vel.magnitude(),
        BULLET_SPEED + 100.0,
        1e-10,
    );

    let initial_x = bullet.pos.x;
    bullet.integrate(DT, &FIELD);
    let pos_after = (BULLET_SPEED + 100.0).mul_add(DT, initial_x);
    h.check_abs("bullet_position_correct", bullet.pos.x, pos_after, 1e-10);

    h.check_bool("bullet_alive_initially", bullet.alive);
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "TTL/DT ≈ 120, fits in u32"
    )]
    let expire_steps = (BULLET_TTL / DT).ceil() as u32;
    for _ in 0..expire_steps {
        bullet.integrate(DT, &FIELD);
    }
    h.check_bool("bullet_expires_after_ttl", !bullet.alive);
}

/// Validate inverse-square gravity toward a central mass.
fn validate_gravity<S: ludospring_barracuda::validation::ValidationSink>(
    h: &mut ValidationHarness<S>,
) {
    let center = Vec2 { x: 400.0, y: 300.0 };

    let near = Vec2 { x: 420.0, y: 300.0 };
    let far = Vec2 { x: 500.0, y: 300.0 };
    let accel_near = gravity_accel(near, center, GM, &FIELD);
    let accel_far = gravity_accel(far, center, GM, &FIELD);

    h.check_bool(
        "gravity_near_stronger",
        accel_near.magnitude() > accel_far.magnitude(),
    );
    h.check_bool("gravity_pulls_toward_center", accel_near.x < 0.0);

    let accel_20 = gravity_accel(near, center, GM, &FIELD).magnitude();
    let accel_100 = gravity_accel(far, center, GM, &FIELD).magnitude();
    let ratio = accel_20 / accel_100;
    let expected_ratio = (100.0f64 / 20.0).powi(2);
    h.check_abs("gravity_inverse_square", ratio, expected_ratio, 0.5);

    let left = Vec2 { x: 10.0, y: 300.0 };
    let accel_wrap = gravity_accel(left, center, GM, &FIELD);
    h.check_bool("gravity_wraps_shortest_path", accel_wrap.x > 0.0);
}

/// Validate circular orbit energy conservation (approximate).
fn validate_energy_conservation<S: ludospring_barracuda::validation::ValidationSink>(
    h: &mut ValidationHarness<S>,
) {
    let center = Vec2 { x: 400.0, y: 300.0 };
    let orbit_radius = 150.0;
    let orbital_speed = (GM / orbit_radius).sqrt();

    let mut ship = Ship {
        pos: Vec2 {
            x: center.x + orbit_radius,
            y: center.y,
        },
        vel: Vec2 {
            x: 0.0,
            y: orbital_speed,
        },
        heading: 0.0,
        radius: 5.0,
        alive: true,
    };

    let kinetic = |v: Vec2| 0.5 * v.x.mul_add(v.x, v.y * v.y);
    let potential = |p: Vec2| {
        let r = p.distance_wrapped(center, &FIELD);
        -GM / r.max(1.0)
    };

    let initial_energy = kinetic(ship.vel) + potential(ship.pos);
    let orbit_steps = 3600;
    for _ in 0..orbit_steps {
        let accel = gravity_accel(ship.pos, center, GM, &FIELD);
        ship.vel = ship.vel + accel * DT;
        ship.integrate(DT, &FIELD);
    }

    let final_energy = kinetic(ship.vel) + potential(ship.pos);
    let energy_drift = ((final_energy - initial_energy) / initial_energy.abs()).abs();
    h.check_bool("energy_drift_under_5pct", energy_drift < 0.05);

    let final_radius = ship.pos.distance_wrapped(center, &FIELD);
    let radius_drift = ((final_radius - orbit_radius) / orbit_radius).abs();
    h.check_bool("orbit_radius_stable", radius_drift < 0.15);
}

/// Validate collision detection between objects.
fn validate_collision_detection<S: ludospring_barracuda::validation::ValidationSink>(
    h: &mut ValidationHarness<S>,
) {
    let ship = Ship {
        pos: Vec2 { x: 100.0, y: 100.0 },
        vel: Vec2::ZERO,
        heading: 0.0,
        radius: 10.0,
        alive: true,
    };

    let near_asteroid = Asteroid {
        pos: Vec2 { x: 115.0, y: 100.0 },
        vel: Vec2::ZERO,
        radius: 10.0,
        alive: true,
    };

    let far_asteroid = Asteroid {
        pos: Vec2 { x: 200.0, y: 200.0 },
        vel: Vec2::ZERO,
        radius: 10.0,
        alive: true,
    };

    let near_dist = ship.pos.distance_wrapped(near_asteroid.pos, &FIELD);
    let collides_near = near_dist < ship.radius + near_asteroid.radius;
    h.check_bool("collision_near_detected", collides_near);

    let far_dist = ship.pos.distance_wrapped(far_asteroid.pos, &FIELD);
    let collides_far = far_dist < ship.radius + far_asteroid.radius;
    h.check_bool("collision_far_not_detected", !collides_far);

    let wrap_ship = Ship {
        pos: Vec2 { x: 5.0, y: 300.0 },
        vel: Vec2::ZERO,
        heading: 0.0,
        radius: 10.0,
        alive: true,
    };
    let wrap_asteroid = Asteroid {
        pos: Vec2 { x: 795.0, y: 300.0 },
        vel: Vec2::ZERO,
        radius: 10.0,
        alive: true,
    };
    let wrap_dist = wrap_ship.pos.distance_wrapped(wrap_asteroid.pos, &FIELD);
    h.check_bool("collision_wraps_around", wrap_dist < 20.0);
}

struct SessionResult {
    destroyed: u32,
    score: u32,
    wave: u32,
    total_ticks: i32,
}

/// Simulate an Asteroids session with AI-controlled ship.
#[expect(
    clippy::cast_precision_loss,
    reason = "asteroid/bullet counts ≤ 1000 fit in f64 mantissa"
)]
fn simulate_asteroids_session() -> SessionResult {
    let mut ship = Ship {
        pos: Vec2 { x: 400.0, y: 300.0 },
        vel: Vec2::ZERO,
        heading: 0.0,
        radius: 10.0,
        alive: true,
    };

    let mut asteroids: Vec<Asteroid> = Vec::new();
    let mut bullets: Vec<Bullet> = Vec::new();
    let mut score = 0u32;
    let mut seed = 42u64;
    let mut destroyed = 0u32;
    let mut wave = 0u32;

    let lcg = |s: &mut u64| -> f64 {
        *s = s.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
        (*s >> 33) as f64 / (1u64 << 31) as f64
    };

    let total_ticks: i32 = 60 * 90;
    let mut fire_cooldown = 0u32;

    for tick in 0..total_ticks {
        spawn_wave_if_needed(&mut asteroids, &mut wave, &mut seed, &ship, &lcg);
        ai_control_ship(&mut ship, &mut bullets, &mut fire_cooldown, &asteroids);
        fire_cooldown = fire_cooldown.saturating_sub(1);
        step_physics(&mut ship, &mut asteroids, &mut bullets);
        resolve_collisions(
            &mut ship,
            &mut asteroids,
            &mut bullets,
            &mut score,
            &mut destroyed,
        );
        bullets.retain(|b| b.alive);

        if !ship.alive && tick < total_ticks - 60 {
            ship.alive = true;
            ship.pos = Vec2 { x: 400.0, y: 300.0 };
            ship.vel = Vec2::ZERO;
            ship.heading = 0.0;
        }
    }

    SessionResult {
        destroyed,
        score,
        wave,
        total_ticks,
    }
}

fn spawn_wave_if_needed(
    asteroids: &mut Vec<Asteroid>,
    wave: &mut u32,
    seed: &mut u64,
    ship: &Ship,
    lcg: &dyn Fn(&mut u64) -> f64,
) {
    if asteroids.iter().any(|a| a.alive) {
        return;
    }
    *wave += 1;
    let count = (*wave * 2 + 2).min(12);
    for _ in 0..count {
        let angle = lcg(seed) * 2.0 * PI;
        let vel = lcg(seed).mul_add(100.0, 50.0);
        let spawn_dist = lcg(seed).mul_add(100.0, 200.0);
        asteroids.push(Asteroid {
            pos: FIELD.wrap_position(Vec2 {
                x: angle.cos().mul_add(spawn_dist, ship.pos.x),
                y: angle.sin().mul_add(spawn_dist, ship.pos.y),
            }),
            vel: Vec2::from_angle(lcg(seed) * 2.0 * PI) * vel,
            radius: lcg(seed).mul_add(20.0, 20.0),
            alive: true,
        });
    }
}

fn ai_control_ship(
    ship: &mut Ship,
    bullets: &mut Vec<Bullet>,
    fire_cooldown: &mut u32,
    asteroids: &[Asteroid],
) {
    let nearest = asteroids.iter().filter(|a| a.alive).min_by(|a, b| {
        let da = ship.pos.distance_wrapped(a.pos, &FIELD);
        let db = ship.pos.distance_wrapped(b.pos, &FIELD);
        da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
    });

    if let Some(target) = nearest {
        let dx = wrap_delta(target.pos.x - ship.pos.x, FIELD.width);
        let dy = wrap_delta(target.pos.y - ship.pos.y, FIELD.height);
        let target_angle = dy.atan2(dx);
        let angle_diff = ((target_angle - ship.heading + PI) % (2.0 * PI)) - PI;
        ship.rotate(angle_diff.clamp(-SHIP_ROTATION, SHIP_ROTATION), DT);

        let dist = ship.pos.distance_wrapped(target.pos, &FIELD);
        if dist < 150.0 {
            let flee_angle = target_angle + PI;
            ship.thrust(SHIP_THRUST * 0.5, DT);
            ship.heading = flee_angle;
        } else {
            ship.thrust(SHIP_THRUST * 0.3, DT);
        }

        if *fire_cooldown == 0 && angle_diff.abs() < 0.5 {
            let dir = Vec2::from_angle(ship.heading);
            bullets.push(Bullet {
                pos: ship.pos,
                vel: ship.vel + dir * BULLET_SPEED,
                alive: true,
                ttl: BULLET_TTL,
            });
            *fire_cooldown = 10;
        }
    }
}

fn step_physics(ship: &mut Ship, asteroids: &mut [Asteroid], bullets: &mut [Bullet]) {
    let center = Vec2 { x: 400.0, y: 300.0 };
    let grav = gravity_accel(ship.pos, center, GM * 0.1, &FIELD);
    ship.vel = ship.vel + grav * DT;
    ship.integrate(DT, &FIELD);

    for asteroid in asteroids.iter_mut() {
        if asteroid.alive {
            asteroid.integrate(DT, &FIELD);
        }
    }
    for bullet in bullets.iter_mut() {
        if bullet.alive {
            bullet.integrate(DT, &FIELD);
        }
    }
}

fn resolve_collisions(
    ship: &mut Ship,
    asteroids: &mut [Asteroid],
    bullets: &mut [Bullet],
    score: &mut u32,
    destroyed: &mut u32,
) {
    for asteroid in asteroids.iter_mut() {
        if !asteroid.alive {
            continue;
        }
        for bullet in bullets.iter_mut() {
            if !bullet.alive {
                continue;
            }
            let dist = bullet.pos.distance_wrapped(asteroid.pos, &FIELD);
            if dist < asteroid.radius {
                asteroid.alive = false;
                bullet.alive = false;
                *score += 1;
                *destroyed += 1;
            }
        }
        let dist = ship.pos.distance_wrapped(asteroid.pos, &FIELD);
        if dist < ship.radius + asteroid.radius {
            ship.alive = false;
        }
    }
}

/// Validate a simulated Asteroids session: escalating asteroid count, engagement.
fn validate_asteroid_session<S: ludospring_barracuda::validation::ValidationSink>(
    h: &mut ValidationHarness<S>,
) {
    let result = simulate_asteroids_session();

    h.check_bool("session_destroyed_asteroids", result.destroyed > 0);
    h.check_bool("session_score_positive", result.score > 0);
    h.check_bool("session_waves_progressed", result.wave > 1);

    let challenge = (f64::from(result.wave) * 0.15).min(1.0);
    let flow = evaluate_flow(challenge, 0.5, 0.2);
    h.check_bool("flow_wave_based", flow != FlowState::Anxiety);

    let snap = EngagementSnapshot {
        session_duration_s: f64::from(result.total_ticks) * DT,
        action_count: u64::from(result.destroyed),
        exploration_breadth: 1,
        challenge_seeking: result.wave,
        retry_count: 0,
        deliberate_pauses: 0,
    };
    let metrics = compute_engagement(&snap);
    h.check_bool("engagement_positive", metrics.composite > 0.0);
}

#[cfg(test)]
mod tests {
    use super::*;
    use ludospring_barracuda::validation::BufferSink;

    #[test]
    fn spacewar_validation_passes() {
        let mut h = ValidationHarness::with_sink("exp077_spacewar", BufferSink::default());
        validate_thrust_integration(&mut h);
        validate_wrap_topology(&mut h);
        validate_angular_mechanics(&mut h);
        validate_projectile_ballistics(&mut h);
        validate_gravity(&mut h);
        validate_collision_detection(&mut h);
        validate_energy_conservation(&mut h);
        validate_asteroid_session(&mut h);
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

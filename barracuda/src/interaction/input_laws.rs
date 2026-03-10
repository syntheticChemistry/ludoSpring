// SPDX-License-Identifier: AGPL-3.0-or-later
//! Empirical laws of human input — Fitts's law, Hick's law, steering law.
//!
//! These are the physics of interaction: validated mathematical models that
//! predict how long humans take to perform input tasks. They're used to
//! evaluate UI layouts, predict interaction costs, and compare game
//! interfaces quantitatively.
//!
//! # References
//! - Fitts, P.M. (1954). "The information capacity of the human motor system
//!   in controlling the amplitude of movement." J. Exp. Psychology, 47(6).
//! - Hick, W.E. (1952). "On the rate of gain of information." Q. J. Exp.
//!   Psychology, 4(1).
//! - Accot, J. & Zhai, S. (1997). "Beyond Fitts' law: models for trajectory-
//!   based HCI tasks." CHI '97.

/// Fitts's law: movement time to acquire a target.
///
/// `MT = a + b * log2(2D / W)` where:
/// - `D` = distance to target center
/// - `W` = target width along the axis of movement
/// - `a` = intercept (device-dependent, ~50ms for mouse)
/// - `b` = slope (device-dependent, ~150ms for mouse)
///
/// Returns movement time in milliseconds.
#[must_use]
pub fn fitts_movement_time(distance: f64, target_width: f64, a: f64, b: f64) -> f64 {
    if target_width <= 0.0 || distance <= 0.0 {
        return a;
    }
    let id = (2.0 * distance / target_width + 1.0).log2();
    a + b * id
}

/// Fitts's index of difficulty (bits).
#[must_use]
pub fn fitts_index_of_difficulty(distance: f64, target_width: f64) -> f64 {
    if target_width <= 0.0 || distance <= 0.0 {
        return 0.0;
    }
    (2.0 * distance / target_width + 1.0).log2()
}

/// Hick's law: decision time given N equally probable choices.
///
/// `RT = a + b * log2(N + 1)` where:
/// - `N` = number of choices
/// - `a` = base reaction time (~200ms)
/// - `b` = processing time per bit (~150ms)
///
/// Returns reaction time in milliseconds.
#[must_use]
pub fn hick_reaction_time(n_choices: usize, a: f64, b: f64) -> f64 {
    if n_choices == 0 {
        return a;
    }
    a + b * ((n_choices + 1) as f64).log2()
}

/// Steering law: time to navigate through a tunnel.
///
/// `T = a + b * (D / W)` where:
/// - `D` = tunnel length
/// - `W` = tunnel width
///
/// Models tasks like moving through menus, navigating corridors, or
/// tracing a path between obstacles.
#[must_use]
pub fn steering_time(tunnel_length: f64, tunnel_width: f64, a: f64, b: f64) -> f64 {
    if tunnel_width <= 0.0 {
        return f64::INFINITY;
    }
    a + b * (tunnel_length / tunnel_width)
}

/// Evaluate the "interaction cost" of a UI element at a given distance and size.
///
/// Combines Fitts's law (acquisition) + Hick's law (decision among options)
/// to give total predicted time in milliseconds.
#[must_use]
pub fn interaction_cost(
    distance: f64,
    target_width: f64,
    n_options: usize,
    fitts_a: f64,
    fitts_b: f64,
    hick_a: f64,
    hick_b: f64,
) -> f64 {
    fitts_movement_time(distance, target_width, fitts_a, fitts_b)
        + hick_reaction_time(n_options, hick_a, hick_b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fitts_larger_target_is_faster() {
        let small = fitts_movement_time(100.0, 10.0, 50.0, 150.0);
        let large = fitts_movement_time(100.0, 50.0, 50.0, 150.0);
        assert!(large < small);
    }

    #[test]
    fn fitts_closer_target_is_faster() {
        let far = fitts_movement_time(200.0, 20.0, 50.0, 150.0);
        let near = fitts_movement_time(50.0, 20.0, 50.0, 150.0);
        assert!(near < far);
    }

    #[test]
    fn hick_more_choices_is_slower() {
        let few = hick_reaction_time(2, 200.0, 150.0);
        let many = hick_reaction_time(16, 200.0, 150.0);
        assert!(many > few);
    }

    #[test]
    fn steering_narrower_is_slower() {
        let wide = steering_time(100.0, 20.0, 0.0, 10.0);
        let narrow = steering_time(100.0, 5.0, 0.0, 10.0);
        assert!(narrow > wide);
    }

    #[test]
    fn index_of_difficulty_matches_expected() {
        let id = fitts_index_of_difficulty(100.0, 10.0);
        let expected = (2.0 * 100.0 / 10.0 + 1.0_f64).log2();
        assert!((id - expected).abs() < 1e-10);
    }
}

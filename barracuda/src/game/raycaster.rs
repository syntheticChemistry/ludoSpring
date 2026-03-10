// SPDX-License-Identifier: AGPL-3.0-or-later
//! Raycaster — reference implementation for first-person spatial navigation.
//!
//! This is a study implementation, not a production renderer. It exists to
//! validate petalTongue's `FirstPersonController` against a known-correct
//! reference (Doom-style raycasting) and to measure interaction metrics
//! (input latency, frame budget, look sensitivity) in a controlled environment.
//!
//! # References
//! - Carmack, J. (1993). Doom engine: BSP traversal + column rendering
//! - Permadi, F. (1996). "Ray-Casting Tutorial"
//! - Lodev (2004). "Raycasting" tutorial (DDA algorithm)

use std::f64::consts::PI;

/// Player state in a raycasted world.
#[derive(Debug, Clone)]
pub struct RayPlayer {
    /// World-space X position.
    pub x: f64,
    /// World-space Y position.
    pub y: f64,
    /// Look angle in radians (0 = east, PI/2 = north).
    pub angle: f64,
    /// Field of view in radians.
    pub fov: f64,
    /// Movement speed (units per second).
    pub speed: f64,
    /// Rotation speed (radians per second).
    pub turn_speed: f64,
}

impl Default for RayPlayer {
    fn default() -> Self {
        Self {
            x: 1.5,
            y: 1.5,
            angle: 0.0,
            fov: PI / 3.0,
            speed: 3.0,
            turn_speed: PI,
        }
    }
}

impl RayPlayer {
    /// Move forward (positive) or backward (negative) by `amount * dt`.
    pub fn move_forward(&mut self, amount: f64, dt: f64) {
        self.x += self.angle.cos() * amount * self.speed * dt;
        self.y += self.angle.sin() * amount * self.speed * dt;
    }

    /// Strafe right (positive) or left (negative) by `amount * dt`.
    pub fn strafe(&mut self, amount: f64, dt: f64) {
        let perp = self.angle + PI / 2.0;
        self.x += perp.cos() * amount * self.speed * dt;
        self.y += perp.sin() * amount * self.speed * dt;
    }

    /// Rotate by `amount * dt` radians.
    pub fn rotate(&mut self, amount: f64, dt: f64) {
        self.angle += amount * self.turn_speed * dt;
        self.angle = self.angle.rem_euclid(2.0 * PI);
    }
}

/// Result of a single ray cast against a grid map.
#[derive(Debug, Clone)]
pub struct RayHit {
    /// Distance from player to wall.
    pub distance: f64,
    /// Which cell was hit (grid coordinates).
    pub cell_x: usize,
    /// Which cell was hit (grid coordinates).
    pub cell_y: usize,
    /// Whether the hit was on a vertical (NS) wall.
    pub vertical_hit: bool,
    /// Exact hit position along the wall (0.0–1.0) for texture mapping.
    pub wall_offset: f64,
}

/// Cast a single ray using DDA (Digital Differential Analyzer).
///
/// `map` is a row-major grid where `true` means solid wall.
/// Returns `None` if ray escapes the map bounds.
#[must_use]
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    reason = "raycaster grid coordinates are small integers; truncation/sign are checked by bounds"
)]
pub fn cast_ray(
    player: &RayPlayer,
    ray_angle: f64,
    map: &[Vec<bool>],
    max_depth: f64,
) -> Option<RayHit> {
    let (height, width) = (map.len(), map.first().map_or(0, Vec::len));
    if width == 0 {
        return None;
    }

    let dir_x = ray_angle.cos();
    let dir_y = ray_angle.sin();

    let delta_x = if dir_x.abs() < 1e-12 {
        f64::MAX
    } else {
        (1.0 / dir_x).abs()
    };
    let delta_y = if dir_y.abs() < 1e-12 {
        f64::MAX
    } else {
        (1.0 / dir_y).abs()
    };

    let mut map_x = player.x as isize;
    let mut map_y = player.y as isize;

    let (step_x, mut side_x) = if dir_x < 0.0 {
        (-1_isize, (player.x - map_x as f64) * delta_x)
    } else {
        (1, (map_x as f64 + 1.0 - player.x) * delta_x)
    };
    let (step_y, mut side_y) = if dir_y < 0.0 {
        (-1_isize, (player.y - map_y as f64) * delta_y)
    } else {
        (1, (map_y as f64 + 1.0 - player.y) * delta_y)
    };

    let mut vertical_hit;
    loop {
        if side_x < side_y {
            side_x += delta_x;
            map_x += step_x;
            vertical_hit = true;
        } else {
            side_y += delta_y;
            map_y += step_y;
            vertical_hit = false;
        }

        if map_x < 0 || map_y < 0 || map_x as usize >= width || map_y as usize >= height {
            return None;
        }

        let distance = if vertical_hit {
            side_x - delta_x
        } else {
            side_y - delta_y
        };
        if distance > max_depth {
            return None;
        }

        if map[map_y as usize][map_x as usize] {
            let wall_offset = if vertical_hit {
                (player.y + distance * dir_y).fract()
            } else {
                (player.x + distance * dir_x).fract()
            };
            return Some(RayHit {
                distance,
                cell_x: map_x as usize,
                cell_y: map_y as usize,
                vertical_hit,
                wall_offset,
            });
        }
    }
}

/// Cast all rays for a screen of given width.
#[must_use]
pub fn cast_screen(
    player: &RayPlayer,
    screen_width: usize,
    map: &[Vec<bool>],
    max_depth: f64,
) -> Vec<Option<RayHit>> {
    (0..screen_width)
        .map(|col| {
            let camera_x = 2.0 * col as f64 / screen_width as f64 - 1.0;
            let ray_angle = player.angle + (camera_x * player.fov / 2.0).atan2(1.0);
            cast_ray(player, ray_angle, map, max_depth)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_map() -> Vec<Vec<bool>> {
        vec![
            vec![true, true, true, true, true],
            vec![true, false, false, false, true],
            vec![true, false, false, false, true],
            vec![true, false, false, false, true],
            vec![true, true, true, true, true],
        ]
    }

    #[test]
    fn player_facing_east_hits_east_wall() {
        let player = RayPlayer {
            x: 2.5,
            y: 2.5,
            angle: 0.0,
            ..Default::default()
        };
        let map = test_map();
        let hit = cast_ray(&player, 0.0, &map, 20.0);
        assert!(hit.is_some());
        let hit = hit.unwrap();
        assert_eq!(hit.cell_x, 4);
        assert!(hit.distance > 1.0 && hit.distance < 2.0);
    }

    #[test]
    fn movement_changes_position() {
        let mut player = RayPlayer::default();
        let x0 = player.x;
        player.move_forward(1.0, 1.0);
        assert!(player.x > x0);
    }

    #[test]
    fn cast_screen_returns_correct_count() {
        let player = RayPlayer {
            x: 2.5,
            y: 2.5,
            angle: 0.0,
            ..Default::default()
        };
        let map = test_map();
        let hits = cast_screen(&player, 320, &map, 20.0);
        assert_eq!(hits.len(), 320);
    }
}

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

/// Flat grid map for raycasting — GPU-promotion ready.
///
/// Row-major `bool` slice with stride, replacing `Vec<Vec<bool>>`.
/// Flat layout maps directly to GPU buffer for Tier A shader promotion.
#[derive(Debug, Clone)]
pub struct GridMap {
    data: Vec<bool>,
    /// Grid width (number of columns).
    pub width: usize,
    /// Grid height (number of rows).
    pub height: usize,
}

impl GridMap {
    /// Create a grid map from a flat bool slice.
    ///
    /// # Panics
    ///
    /// Panics if `data.len() != width * height`.
    #[must_use]
    pub fn new(width: usize, height: usize, data: Vec<bool>) -> Self {
        assert_eq!(
            data.len(),
            width * height,
            "data length must match width*height"
        );
        Self {
            data,
            width,
            height,
        }
    }

    /// Create from nested vecs (migration helper).
    #[must_use]
    pub fn from_nested(rows: &[Vec<bool>]) -> Self {
        let height = rows.len();
        let width = rows.first().map_or(0, Vec::len);
        let data: Vec<bool> = rows.iter().flat_map(|r| r.iter().copied()).collect();
        Self {
            data,
            width,
            height,
        }
    }

    /// Query a cell. Out-of-bounds returns `false`.
    #[must_use]
    pub fn get(&self, x: usize, y: usize) -> bool {
        if x < self.width && y < self.height {
            self.data[y * self.width + x]
        } else {
            false
        }
    }

    /// Raw slice for GPU upload.
    #[must_use]
    pub fn as_slice(&self) -> &[bool] {
        &self.data
    }
}

impl From<&super::engine::world::TileWorld> for GridMap {
    /// Convert a `TileWorld` into a raycaster grid.
    ///
    /// Tiles that block sight (walls, closed doors) become `true` (solid).
    /// All other terrain becomes `false` (open).
    fn from(world: &super::engine::world::TileWorld) -> Self {
        let w = world.width();
        let h = world.height();
        let data: Vec<bool> = (0..h)
            .flat_map(|y| {
                (0..w).map(move |x| {
                    world
                        .get(x, y)
                        .is_some_and(|t| t.terrain.blocks_sight())
                })
            })
            .collect();
        Self {
            data,
            width: w as usize,
            height: h as usize,
        }
    }
}

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
    /// Spawn position and viewing parameters for a classic FPS raycaster.
    ///
    /// Provenance: Doom/Wolfenstein-style defaults — centered in first cell (1.5, 1.5),
    /// 60° `FoV` (`PI`/3), walk speed ~3 units/s, turn speed `PI` rad/s.
    /// Source: Fabien Sanglard, "Game Engine Black Book: Wolfenstein 3D" (2017), Ch. 5.
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
    map: &GridMap,
    max_depth: f64,
) -> Option<RayHit> {
    let (height, width) = (map.height, map.width);
    if width == 0 {
        return None;
    }

    let dir_x = ray_angle.cos();
    let dir_y = ray_angle.sin();

    let delta_x = if dir_x.abs() < crate::tolerances::DDA_NEAR_ZERO {
        f64::MAX
    } else {
        (1.0 / dir_x).abs()
    };
    let delta_y = if dir_y.abs() < crate::tolerances::DDA_NEAR_ZERO {
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

        if map.get(map_x as usize, map_y as usize) {
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
#[expect(
    clippy::cast_precision_loss,
    reason = "screen widths are small (≤8192); col and screen_width fit in f64 mantissa"
)]
pub fn cast_screen(
    player: &RayPlayer,
    screen_width: usize,
    map: &GridMap,
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
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn test_map() -> GridMap {
        GridMap::from_nested(&[
            vec![true, true, true, true, true],
            vec![true, false, false, false, true],
            vec![true, false, false, false, true],
            vec![true, false, false, false, true],
            vec![true, true, true, true, true],
        ])
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
        let Some(hit) = cast_ray(&player, 0.0, &map, 20.0) else {
            panic!("ray should hit enclosed wall");
        };
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

    #[test]
    fn ray_hits_nothing_in_empty_region() {
        let empty_map = GridMap::new(5, 5, vec![false; 25]);
        let player = RayPlayer {
            x: 2.5,
            y: 2.5,
            angle: 0.0,
            ..Default::default()
        };
        let hit = cast_ray(&player, 0.0, &empty_map, 20.0);
        assert!(hit.is_none());
    }

    #[test]
    fn ray_north_small_map_hits_wall() {
        let map = GridMap::from_nested(&[
            vec![true, false, true],
            vec![true, false, true],
            vec![true, true, true],
        ]);
        let player = RayPlayer {
            x: 1.5,
            y: 1.5,
            angle: std::f64::consts::PI / 2.0,
            ..Default::default()
        };
        let hit = cast_ray(&player, std::f64::consts::PI / 2.0, &map, 20.0);
        assert!(hit.is_some());
        let hit = hit.unwrap();
        assert_eq!(hit.cell_y, 2);
    }

    #[test]
    fn ray_north_hits_wall() {
        let map = test_map();
        let player = RayPlayer {
            x: 2.5,
            y: 2.5,
            angle: std::f64::consts::PI / 2.0,
            ..Default::default()
        };
        let hit = cast_ray(&player, std::f64::consts::PI / 2.0, &map, 20.0);
        assert!(hit.is_some());
        let hit = hit.unwrap();
        assert_eq!(hit.cell_y, 4);
        assert!(hit.distance > 1.0);
    }

    #[test]
    fn ray_south_hits_wall() {
        let map = test_map();
        let player = RayPlayer {
            x: 2.5,
            y: 2.5,
            angle: -std::f64::consts::PI / 2.0,
            ..Default::default()
        };
        let hit = cast_ray(&player, -std::f64::consts::PI / 2.0, &map, 20.0);
        assert!(hit.is_some());
        let hit = hit.unwrap();
        assert_eq!(hit.cell_y, 0);
    }

    #[test]
    fn ray_west_hits_wall() {
        let map = test_map();
        let player = RayPlayer {
            x: 2.5,
            y: 2.5,
            angle: std::f64::consts::PI,
            ..Default::default()
        };
        let hit = cast_ray(&player, std::f64::consts::PI, &map, 20.0);
        assert!(hit.is_some());
        let hit = hit.unwrap();
        assert_eq!(hit.cell_x, 0);
    }

    #[test]
    fn max_depth_returns_none() {
        let map = test_map();
        let player = RayPlayer {
            x: 2.5,
            y: 2.5,
            angle: 0.0,
            ..Default::default()
        };
        let hit = cast_ray(&player, 0.0, &map, 0.5);
        assert!(hit.is_none());
    }

    #[test]
    fn zero_width_map_returns_none() {
        let map = GridMap::new(0, 5, vec![]);
        let player = RayPlayer::default();
        let hit = cast_ray(&player, 0.0, &map, 20.0);
        assert!(hit.is_none());
    }

    #[test]
    fn player_at_boundary_still_casts() {
        let map = test_map();
        let player = RayPlayer {
            x: 0.5,
            y: 2.5,
            angle: 0.0,
            ..Default::default()
        };
        let hit = cast_ray(&player, 0.0, &map, 20.0);
        assert!(hit.is_some());
    }

    #[test]
    fn strafe_and_rotate() {
        let mut player = RayPlayer::default();
        let y0 = player.y;
        player.strafe(1.0, 1.0);
        assert!((player.y - y0).abs() > 0.01, "strafe at angle 0 moves in y");
        player.rotate(1.0, 1.0);
        assert!((player.angle - std::f64::consts::PI).abs() < 0.01);
    }

    #[test]
    fn grid_get_out_of_bounds() {
        let map = test_map();
        assert!(!map.get(10, 10));
        assert!(!map.get(5, 0));
    }
}

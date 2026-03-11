// SPDX-License-Identifier: AGPL-3.0-or-later
//! Raycaster throughput benchmarks (BM-003).
//!
//! Measures DDA screen-cast throughput for first-person spatial navigation.
//!
//! **Benchmark targets** (from `OPEN_SYSTEMS_BENCHMARK_SPECIFICATION.md`):
//! - Match C reference (Lodev DDA) within 1.5x for 320-column screen cast
//! - Sustain 60 Hz for standard map sizes (64x64, 128x128)
//!
//! **Open system baselines**:
//! - Lodev raycaster tutorial (C, public domain)
//! - raylib built-in raycaster (C, zlib)

use ludospring_barracuda::game::raycaster::{GridMap, RayPlayer, cast_screen};

/// Build a standard benchmark map: walled perimeter with open interior.
#[must_use]
pub fn arena_map(size: usize) -> GridMap {
    let mut data = vec![false; size * size];
    for y in 0..size {
        for x in 0..size {
            if x == 0 || x == size - 1 || y == 0 || y == size - 1 {
                data[y * size + x] = true;
            }
        }
    }
    GridMap::new(size, size, data)
}

/// Build a map with internal walls for complex raycasting paths.
#[must_use]
pub fn maze_map() -> GridMap {
    let size = 16;
    let mut data = vec![false; size * size];
    for y in 0..size {
        for x in 0..size {
            let wall = x == 0
                || x == size - 1
                || y == 0
                || y == size - 1
                || (x == 4 && y < 12)
                || (x == 8 && y > 3)
                || (x == 12 && y < 14 && y > 2);
            data[y * size + x] = wall;
        }
    }
    GridMap::new(size, size, data)
}

/// Cast a full screen of rays and return the hit distances.
#[must_use]
#[expect(
    clippy::cast_precision_loss,
    reason = "map dimensions ≤ 1024; usize fits in f64 mantissa"
)]
pub fn cast_full_screen(map: &GridMap, columns: usize) -> Vec<Option<f64>> {
    let player = RayPlayer {
        x: map.width as f64 / 2.0,
        y: map.height as f64 / 2.0,
        angle: 0.0,
        fov: std::f64::consts::FRAC_PI_3, // 60 degrees
        speed: 3.0,
        turn_speed: 2.0,
    };
    let hits = cast_screen(&player, columns, map, 64.0);
    hits.into_iter()
        .map(|h| h.map(|hit| hit.distance))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arena_map_has_walls() {
        let map = arena_map(32);
        assert!(map.get(0, 0));
        assert!(!map.get(16, 16));
    }

    #[test]
    fn cast_returns_correct_column_count() {
        let map = arena_map(32);
        let hits = cast_full_screen(&map, 320);
        assert_eq!(hits.len(), 320);
    }

    #[test]
    fn all_rays_hit_walls_in_arena() {
        let map = arena_map(32);
        let hits = cast_full_screen(&map, 320);
        for (i, h) in hits.iter().enumerate() {
            assert!(h.is_some(), "ray {i} should hit a wall in closed arena");
        }
    }
}

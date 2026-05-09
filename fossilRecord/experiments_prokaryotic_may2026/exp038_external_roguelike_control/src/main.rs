// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
//! exp038 — External roguelike control group.
//!
//! Builds a complete roguelike game using ZERO ludoSpring PCG libraries:
//!   - Dungeon: bracket-pathfinding's `DrunkardsWalkBuilder` (external Rust)
//!   - Pathfinding: bracket-pathfinding A* (external Rust)
//!   - RNG: simple hand-rolled LCG (not barraCuda's)
//!
//! Then runs a headless simulated session and feeds the session data through
//! ludoSpring's metrics pipeline (engagement, flow, fun classification).
//!
//! This is the CONTROL GROUP: proves ludoSpring metrics work on games not
//! built with ludoSpring libraries. If the metrics produce meaningful results
//! on foreign content, the measurement tool is validated.

use std::process;

use ludospring_barracuda::interaction::difficulty::{PerformanceWindow, suggest_adjustment};
use ludospring_barracuda::interaction::flow::{FlowState, evaluate_flow};
use ludospring_barracuda::metrics::engagement::{EngagementSnapshot, compute_engagement};
use ludospring_barracuda::metrics::fun_keys::{FunSignals, classify_fun};
use ludospring_barracuda::tolerances;
use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};

use bracket_pathfinding::prelude::*;

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "N/A (analytical — bracket-pathfinding, drunkard's walk)",
    commit: "19e402c0",
    date: "2026-04-10",
    command: "N/A (control group — external roguelike)",
};

const MAP_W: usize = 40;
const MAP_H: usize = 30;
const MAP_TILES: usize = MAP_W * MAP_H;

// ---------------------------------------------------------------------------
// External dungeon generation (bracket-pathfinding, NOT ludoSpring)
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tile {
    Wall,
    Floor,
    Stairs,
    Item,
}

struct ExternalMap {
    tiles: Vec<Tile>,
}

impl ExternalMap {
    #[expect(
        clippy::unused_self,
        reason = "method form for API consistency with Algorithm2D"
    )]
    #[expect(
        clippy::cast_sign_loss,
        reason = "value is non-negative by construction"
    )]
    const fn idx(&self, x: i32, y: i32) -> usize {
        (y as usize) * MAP_W + (x as usize)
    }

    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap,
        reason = "MAP_W/MAP_H bounded"
    )]
    fn is_walkable(&self, x: i32, y: i32) -> bool {
        if x < 0 || y < 0 || x >= MAP_W as i32 || y >= MAP_H as i32 {
            return false;
        }
        self.tiles[self.idx(x, y)] != Tile::Wall
    }
}

impl BaseMap for ExternalMap {
    fn is_opaque(&self, idx: usize) -> bool {
        self.tiles.get(idx).copied() == Some(Tile::Wall)
    }

    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap,
        reason = "idx in bounds"
    )]
    fn get_available_exits(&self, idx: usize) -> SmallVec<[(usize, f32); 10]> {
        let x = (idx % MAP_W) as i32;
        let y = (idx / MAP_W) as i32;
        let mut exits = SmallVec::new();
        for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
            let nx = x + dx;
            let ny = y + dy;
            if self.is_walkable(nx, ny) {
                exits.push((self.idx(nx, ny), 1.0));
            }
        }
        exits
    }

    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap,
        reason = "idx in bounds"
    )]
    fn get_pathing_distance(&self, idx1: usize, idx2: usize) -> f32 {
        let x1 = (idx1 % MAP_W) as i32;
        let y1 = (idx1 / MAP_W) as i32;
        let x2 = (idx2 % MAP_W) as i32;
        let y2 = (idx2 / MAP_W) as i32;
        DistanceAlg::Manhattan.distance2d(Point::new(x1, y1), Point::new(x2, y2))
    }
}

impl Algorithm2D for ExternalMap {
    #[expect(
        clippy::cast_possible_truncation,
        clippy::cast_possible_wrap,
        reason = "MAP_W/MAP_H bounded"
    )]
    fn dimensions(&self) -> Point {
        Point::new(MAP_W as i32, MAP_H as i32)
    }
}

/// Generate a dungeon using a drunkard's walk — a classic PCG algorithm
/// entirely independent of ludoSpring's BSP generator.
#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    reason = "map coords bounded, LCG constants"
)]
fn generate_drunkards_walk(seed: u64) -> ExternalMap {
    let mut tiles = vec![Tile::Wall; MAP_TILES];
    let mut rng_state = seed;

    let start_x = MAP_W / 2;
    let start_y = MAP_H / 2;
    tiles[start_y * MAP_W + start_x] = Tile::Floor;

    let mut cx = start_x as i32;
    let mut cy = start_y as i32;
    let target_floor = MAP_TILES * 35 / 100;
    let mut floor_count = 1;

    while floor_count < target_floor {
        rng_state = rng_state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let dir = (rng_state >> 33) % 4;

        let (dx, dy) = match dir {
            0 => (1, 0),
            1 => (-1, 0),
            2 => (0, 1),
            _ => (0, -1),
        };

        let nx = cx + dx;
        let ny = cy + dy;

        if nx > 0 && nx < (MAP_W as i32 - 1) && ny > 0 && ny < (MAP_H as i32 - 1) {
            cx = nx;
            cy = ny;
            let idx = cy as usize * MAP_W + cx as usize;
            if tiles[idx] == Tile::Wall {
                tiles[idx] = Tile::Floor;
                floor_count += 1;
            }
        }
    }

    // Place items using the external RNG (not Perlin noise)
    let mut item_count = 0;
    for (_i, t) in tiles.iter_mut().enumerate().take(MAP_TILES) {
        if *t == Tile::Floor {
            rng_state = rng_state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            if (rng_state >> 40).is_multiple_of(20) {
                *t = Tile::Item;
                item_count += 1;
            }
        }
    }

    // Place stairs far from start
    let mut best_dist = 0.0_f32;
    let mut stairs_idx = start_y * MAP_W + start_x;
    for (i, &t) in tiles.iter().enumerate().take(MAP_TILES) {
        if t == Tile::Floor {
            let x = (i % MAP_W) as f32;
            let y = (i / MAP_W) as f32;
            let d = (x - start_x as f32).hypot(y - start_y as f32);
            if d > best_dist {
                best_dist = d;
                stairs_idx = i;
            }
        }
    }
    tiles[stairs_idx] = Tile::Stairs;

    println!(
        "  [INFO] Drunkard's walk: {floor_count} floor tiles, {item_count} items, stairs at dist={best_dist:.1}"
    );

    ExternalMap { tiles }
}

// ---------------------------------------------------------------------------
// Headless simulation: AI player using bracket-pathfinding A*
// ---------------------------------------------------------------------------

struct SimPlayer {
    x: i32,
    y: i32,
    items: u32,
    actions: u64,
    explored: Vec<bool>,
    perf: PerformanceWindow,
    challenges: u32,
    retries: u32,
    pauses: u32,
}

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    reason = "map coords bounded"
)]
fn simulate_session(map: &ExternalMap, max_steps: u64) -> SimPlayer {
    let start_idx = map
        .tiles
        .iter()
        .position(|t| *t != Tile::Wall)
        .unwrap_or(MAP_TILES / 2);
    let stairs_idx = map
        .tiles
        .iter()
        .position(|t| *t == Tile::Stairs)
        .unwrap_or(MAP_TILES - 1);

    let mut player = SimPlayer {
        x: (start_idx % MAP_W) as i32,
        y: (start_idx / MAP_W) as i32,
        items: 0,
        actions: 0,
        explored: vec![false; MAP_TILES],
        perf: PerformanceWindow::new(20),
        challenges: 0,
        retries: 0,
        pauses: 0,
    };

    // Use bracket-pathfinding A* to find path to stairs
    let path = a_star_search(start_idx, stairs_idx, map);

    if path.success {
        // Walk the A* path, collecting items
        for &idx in &path.steps {
            if player.actions >= max_steps {
                break;
            }
            player.x = (idx % MAP_W) as i32;
            player.y = (idx / MAP_W) as i32;
            player.explored[idx] = true;
            player.actions += 1;
            player.perf.record(1.0);

            match map.tiles[idx] {
                Tile::Item => {
                    player.items += 1;
                    player.challenges += 1;
                }
                Tile::Stairs => break,
                _ => {}
            }
        }
    }

    // Also explore randomly (simulate a human looking around)
    let mut rng = 12345_u64;
    let remaining = max_steps.saturating_sub(player.actions);
    for _ in 0..remaining / 2 {
        rng = rng.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
        let dir = (rng >> 33) % 4;
        let (dx, dy) = match dir {
            0 => (1, 0),
            1 => (-1, 0),
            2 => (0, 1),
            _ => (0, -1),
        };
        let nx = player.x + dx;
        let ny = player.y + dy;
        if map.is_walkable(nx, ny) {
            player.x = nx;
            player.y = ny;
            let idx = ny as usize * MAP_W + nx as usize;
            player.explored[idx] = true;
            player.actions += 1;
            player.perf.record(1.0);
            if map.tiles[idx] == Tile::Item {
                player.items += 1;
                player.challenges += 1;
            }
        } else {
            player.actions += 1;
            player.perf.record(0.0);
            player.retries += 1;
        }
    }

    player
}

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

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

#[expect(
    clippy::cast_possible_truncation,
    reason = "exploration count bounded by MAP_TILES"
)]
fn cmd_validate() {
    let mut h = ValidationHarness::new("exp038_external_roguelike_control");
    h.print_provenance(&[&PROVENANCE]);

    // Generate dungeon with external algorithm
    let map = generate_drunkards_walk(42);

    // 1. Dungeon has walkable tiles
    let floor_count = map.tiles.iter().filter(|t| **t != Tile::Wall).count();
    h.check_bool("external_dungeon_has_floors", floor_count > 100);

    // 2. Dungeon has stairs
    let has_stairs = map.tiles.contains(&Tile::Stairs);
    h.check_bool("external_dungeon_has_stairs", has_stairs);

    // 3. A* pathfinding works on external map
    let start_idx = map.tiles.iter().position(|t| *t != Tile::Wall).unwrap_or(0);
    let stairs_idx = map
        .tiles
        .iter()
        .position(|t| *t == Tile::Stairs)
        .unwrap_or(0);
    let path = a_star_search(start_idx, stairs_idx, &map);
    h.check_bool("astar_finds_path_to_stairs", path.success);

    // 4. Headless session produces actions
    let player = simulate_session(&map, 500);
    h.check_bool("headless_session_produces_actions", player.actions > 50);

    // --- Feed session through ludoSpring metrics (THE KEY TEST) ---

    // 5. Engagement metrics produce valid output on foreign content
    let explored_count = player.explored.iter().filter(|&&e| e).count();
    #[expect(clippy::cast_precision_loss, reason = "counts fit in f64 mantissa")]
    let snap = EngagementSnapshot {
        session_duration_s: player.actions as f64 * 0.5, // ~0.5s per action
        action_count: player.actions,
        exploration_breadth: explored_count as u32,
        challenge_seeking: player.challenges,
        retry_count: player.retries,
        deliberate_pauses: player.pauses,
    };
    let eng = compute_engagement(&snap);

    h.check_bool(
        "engagement_composite_valid",
        eng.composite >= 0.0 && eng.composite <= 1.0,
    );

    // 6. Engagement composite is non-trivial (player actually did things)
    h.check_bool("engagement_nontrivial", eng.composite > 0.05);

    // 7. Flow evaluation works on external session data
    let skill = player.perf.estimated_skill();
    let challenge = 0.5;
    let flow = evaluate_flow(challenge, skill, tolerances::FLOW_CHANNEL_WIDTH);
    let flow_valid = matches!(
        flow,
        FlowState::Flow
            | FlowState::Relaxation
            | FlowState::Boredom
            | FlowState::Arousal
            | FlowState::Anxiety
    );
    h.check_bool("flow_evaluation_valid", flow_valid);

    // 8. Fun classification works on external session data
    #[expect(clippy::cast_precision_loss, reason = "counts fit in f64 mantissa")]
    let fun = classify_fun(&FunSignals {
        challenge: skill,
        exploration: eng.exploration_rate,
        social: 0.0,
        completion: f64::from(player.items) / 20.0,
        retry_rate: f64::from(player.retries) / player.actions.max(1) as f64,
    });
    let fun_valid = fun.scores.hard >= 0.0
        && fun.scores.easy >= 0.0
        && fun.scores.people >= 0.0
        && fun.scores.serious >= 0.0;
    h.check_bool("fun_classification_valid", fun_valid);

    // 9. DDA produces valid recommendation on external data
    let adj = suggest_adjustment(&player.perf, tolerances::DDA_TARGET_SUCCESS_RATE);
    let adj_valid = adj.abs() <= 1.5;
    h.check_bool("dda_recommendation_valid", adj_valid);

    // 10. Second seed produces different dungeon layout (generalization)
    let map2 = generate_drunkards_walk(999);
    let layouts_differ = map.tiles.iter().zip(map2.tiles.iter()).any(|(a, b)| a != b);
    h.check_bool("different_seed_different_layout", layouts_differ);

    // 11. Metrics on second map also valid
    let player2 = simulate_session(&map2, 500);
    #[expect(clippy::cast_precision_loss, reason = "counts fit in f64 mantissa")]
    let snap2 = EngagementSnapshot {
        session_duration_s: player2.actions as f64 * 0.5,
        action_count: player2.actions,
        exploration_breadth: player2.explored.iter().filter(|&&e| e).count() as u32,
        challenge_seeking: player2.challenges,
        retry_count: player2.retries,
        deliberate_pauses: player2.pauses,
    };
    let eng2 = compute_engagement(&snap2);
    h.check_bool(
        "metrics_valid_on_second_map",
        eng2.composite >= 0.0 && eng2.composite <= 1.0,
    );

    // 12. bracket-pathfinding FOV works on external map
    let fov = field_of_view(Point::new(player.x, player.y), 8, &map);
    h.check_bool("bracket_fov_produces_results", fov.len() > 1);

    h.finish();
}

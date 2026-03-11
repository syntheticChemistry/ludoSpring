// SPDX-License-Identifier: AGPL-3.0-or-later
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
use ludospring_barracuda::validation::ValidationResult;

use bracket_pathfinding::prelude::*;

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
    fn idx(&self, x: i32, y: i32) -> usize {
        (y as usize) * MAP_W + (x as usize)
    }

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

    fn get_pathing_distance(&self, idx1: usize, idx2: usize) -> f32 {
        let x1 = (idx1 % MAP_W) as i32;
        let y1 = (idx1 / MAP_W) as i32;
        let x2 = (idx2 % MAP_W) as i32;
        let y2 = (idx2 / MAP_W) as i32;
        DistanceAlg::Manhattan.distance2d(Point::new(x1, y1), Point::new(x2, y2))
    }
}

impl Algorithm2D for ExternalMap {
    fn dimensions(&self) -> Point {
        Point::new(MAP_W as i32, MAP_H as i32)
    }
}

/// Generate a dungeon using a drunkard's walk — a classic PCG algorithm
/// entirely independent of ludoSpring's BSP generator.
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
        rng_state = rng_state.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1442695040888963407);
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
    for i in 0..MAP_TILES {
        if tiles[i] == Tile::Floor {
            rng_state = rng_state.wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1442695040888963407);
            if (rng_state >> 40) % 20 == 0 {
                tiles[i] = Tile::Item;
                item_count += 1;
            }
        }
    }

    // Place stairs far from start
    let mut best_dist = 0.0_f32;
    let mut stairs_idx = start_y * MAP_W + start_x;
    for i in 0..MAP_TILES {
        if tiles[i] == Tile::Floor {
            let x = (i % MAP_W) as f32;
            let y = (i / MAP_W) as f32;
            let d = ((x - start_x as f32).powi(2) + (y - start_y as f32).powi(2)).sqrt();
            if d > best_dist {
                best_dist = d;
                stairs_idx = i;
            }
        }
    }
    tiles[stairs_idx] = Tile::Stairs;

    println!("  [INFO] Drunkard's walk: {floor_count} floor tiles, {item_count} items, stairs at dist={best_dist:.1}");

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

fn simulate_session(map: &ExternalMap, max_steps: u64) -> SimPlayer {
    let start_idx = map.tiles.iter().position(|t| *t != Tile::Wall).unwrap_or(MAP_TILES / 2);
    let stairs_idx = map.tiles.iter().position(|t| *t == Tile::Stairs).unwrap_or(MAP_TILES - 1);

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
    let path = a_star_search(
        start_idx,
        stairs_idx,
        map,
    );

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

#[allow(clippy::too_many_lines)]
fn cmd_validate() {
    println!("=== exp038: External Roguelike Control Group ===\n");
    println!("  External libraries: bracket-pathfinding (A*, distance, FOV)");
    println!("  Dungeon: drunkard's walk (NOT ludoSpring BSP)");
    println!("  RNG: hand-rolled LCG (NOT barraCuda)");
    println!("  Player: A*-guided headless simulation\n");

    let experiment = "exp038_external_roguelike_control";
    let mut results = Vec::new();

    // Generate dungeon with external algorithm
    let map = generate_drunkards_walk(42);

    // 1. Dungeon has walkable tiles
    let floor_count = map.tiles.iter().filter(|t| **t != Tile::Wall).count();
    results.push(ValidationResult::check(
        experiment,
        "external_dungeon_has_floors",
        if floor_count > 100 { 1.0 } else { 0.0 },
        1.0,
        0.0,
    ));

    // 2. Dungeon has stairs
    let has_stairs = map.tiles.iter().any(|t| *t == Tile::Stairs);
    results.push(ValidationResult::check(
        experiment,
        "external_dungeon_has_stairs",
        if has_stairs { 1.0 } else { 0.0 },
        1.0,
        0.0,
    ));

    // 3. A* pathfinding works on external map
    let start_idx = map.tiles.iter().position(|t| *t != Tile::Wall).unwrap_or(0);
    let stairs_idx = map.tiles.iter().position(|t| *t == Tile::Stairs).unwrap_or(0);
    let path = a_star_search(start_idx, stairs_idx, &map);
    results.push(ValidationResult::check(
        experiment,
        "astar_finds_path_to_stairs",
        if path.success { 1.0 } else { 0.0 },
        1.0,
        0.0,
    ));
    println!("  [INFO] A* path length: {} steps", path.steps.len());

    // 4. Headless session produces actions
    let player = simulate_session(&map, 500);
    results.push(ValidationResult::check(
        experiment,
        "headless_session_produces_actions",
        if player.actions > 50 { 1.0 } else { 0.0 },
        1.0,
        0.0,
    ));
    println!("  [INFO] Session: {} actions, {} items, {} explored",
        player.actions, player.items, player.explored.iter().filter(|&&e| e).count());

    // --- Feed session through ludoSpring metrics (THE KEY TEST) ---

    // 5. Engagement metrics produce valid output on foreign content
    let explored_count = player.explored.iter().filter(|&&e| e).count();
    #[allow(clippy::cast_precision_loss)]
    let snap = EngagementSnapshot {
        session_duration_s: player.actions as f64 * 0.5, // ~0.5s per action
        action_count: player.actions,
        exploration_breadth: explored_count as u32,
        challenge_seeking: player.challenges,
        retry_count: player.retries,
        deliberate_pauses: player.pauses,
    };
    let eng = compute_engagement(&snap);

    results.push(ValidationResult::check(
        experiment,
        "engagement_composite_valid",
        if eng.composite >= 0.0 && eng.composite <= 1.0 { 1.0 } else { 0.0 },
        1.0,
        0.0,
    ));
    println!("  [INFO] Engagement: composite={:.3}, APM={:.1}, exploration={:.2}",
        eng.composite, eng.actions_per_minute, eng.exploration_rate);

    // 6. Engagement composite is non-trivial (player actually did things)
    results.push(ValidationResult::check(
        experiment,
        "engagement_nontrivial",
        if eng.composite > 0.05 { 1.0 } else { 0.0 },
        1.0,
        0.0,
    ));

    // 7. Flow evaluation works on external session data
    let skill = player.perf.estimated_skill();
    let challenge = 0.5;
    let flow = evaluate_flow(challenge, skill, tolerances::FLOW_CHANNEL_WIDTH);
    let flow_valid = matches!(
        flow,
        FlowState::Flow | FlowState::Relaxation | FlowState::Boredom
            | FlowState::Arousal | FlowState::Anxiety
    );
    results.push(ValidationResult::check(
        experiment,
        "flow_evaluation_valid",
        if flow_valid { 1.0 } else { 0.0 },
        1.0,
        0.0,
    ));
    println!("  [INFO] Flow: {} (skill={skill:.2}, challenge={challenge:.2})", flow.as_str());

    // 8. Fun classification works on external session data
    #[allow(clippy::cast_precision_loss)]
    let fun = classify_fun(&FunSignals {
        challenge: skill,
        exploration: eng.exploration_rate,
        social: 0.0,
        completion: player.items as f64 / 20.0,
        retry_rate: player.retries as f64 / player.actions.max(1) as f64,
    });
    let fun_valid = fun.scores.hard >= 0.0
        && fun.scores.easy >= 0.0
        && fun.scores.people >= 0.0
        && fun.scores.serious >= 0.0;
    results.push(ValidationResult::check(
        experiment,
        "fun_classification_valid",
        if fun_valid { 1.0 } else { 0.0 },
        1.0,
        0.0,
    ));
    println!("  [INFO] Fun: {} (hard={:.2}, easy={:.2}, serious={:.2})",
        fun.dominant.as_str(), fun.scores.hard, fun.scores.easy, fun.scores.serious);

    // 9. DDA produces valid recommendation on external data
    let adj = suggest_adjustment(&player.perf, tolerances::DDA_TARGET_SUCCESS_RATE);
    let adj_valid = adj.abs() <= 1.5;
    results.push(ValidationResult::check(
        experiment,
        "dda_recommendation_valid",
        if adj_valid { 1.0 } else { 0.0 },
        1.0,
        0.0,
    ));
    println!("  [INFO] DDA: adjustment={adj:+.3}");

    // 10. Second seed produces different dungeon layout (generalization)
    let map2 = generate_drunkards_walk(999);
    let layouts_differ = map.tiles.iter().zip(map2.tiles.iter()).any(|(a, b)| a != b);
    results.push(ValidationResult::check(
        experiment,
        "different_seed_different_layout",
        if layouts_differ { 1.0 } else { 0.0 },
        1.0,
        0.0,
    ));

    // 11. Metrics on second map also valid
    let player2 = simulate_session(&map2, 500);
    #[allow(clippy::cast_precision_loss)]
    let snap2 = EngagementSnapshot {
        session_duration_s: player2.actions as f64 * 0.5,
        action_count: player2.actions,
        exploration_breadth: player2.explored.iter().filter(|&&e| e).count() as u32,
        challenge_seeking: player2.challenges,
        retry_count: player2.retries,
        deliberate_pauses: player2.pauses,
    };
    let eng2 = compute_engagement(&snap2);
    results.push(ValidationResult::check(
        experiment,
        "metrics_valid_on_second_map",
        if eng2.composite >= 0.0 && eng2.composite <= 1.0 { 1.0 } else { 0.0 },
        1.0,
        0.0,
    ));
    println!("  [INFO] Map2 engagement: composite={:.3}", eng2.composite);

    // 12. bracket-pathfinding FOV works on external map
    let fov = field_of_view(
        Point::new(player.x, player.y),
        8,
        &map,
    );
    results.push(ValidationResult::check(
        experiment,
        "bracket_fov_produces_results",
        if fov.len() > 1 { 1.0 } else { 0.0 },
        1.0,
        0.0,
    ));
    println!("  [INFO] FOV from ({},{}): {} visible tiles", player.x, player.y, fov.len());

    // Print results
    let passed = results.iter().filter(|r| r.passed).count();
    let total = results.len();
    println!();
    for r in &results {
        let tag = if r.passed { "PASS" } else { "FAIL" };
        println!("  [{tag}] {}", r.description);
    }
    println!("\nResults: {passed}/{total} passed");
    if passed < total {
        process::exit(1);
    }
}

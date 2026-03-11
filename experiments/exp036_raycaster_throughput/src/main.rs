// SPDX-License-Identifier: AGPL-3.0-or-later
//! exp036 — BM-003: Raycaster throughput benchmark.
//!
//! From OPEN_SYSTEMS_BENCHMARK_SPECIFICATION.md:
//!   "320-column screen cast at 60 Hz. Match C reference (Lodev DDA) within 1.5x."
//!
//! Validates:
//!   - DDA raycaster sustains 60 Hz for standard map sizes
//!   - All rays hit walls in closed arenas
//!   - Throughput scales with map complexity

use std::process;
use std::time::Instant;

use ludospring_barracuda::validation::ValidationResult;
use ludospring_benchmarks::raycaster::{arena_map, cast_full_screen, maze_map};

fn main() {
    let arg = std::env::args().nth(1).unwrap_or_default();
    match arg.as_str() {
        "bench" => cmd_bench(),
        "validate" | "" => cmd_validate(),
        other => {
            eprintln!("Unknown command: {other}");
            eprintln!("Usage: exp036_raycaster_throughput [validate|bench]");
            process::exit(1);
        }
    }
}

fn cmd_validate() {
    println!("=== exp036: BM-003 Raycaster Throughput Validation ===\n");

    let experiment = "exp036_raycaster_throughput";
    let mut results = Vec::new();

    // 1. 320-column cast on 64x64 arena completes
    let map64 = arena_map(64);
    let t = Instant::now();
    let hits = cast_full_screen(&map64, 320);
    let cast64_us = t.elapsed().as_micros();
    results.push(ValidationResult::check(
        experiment,
        "cast_320col_64x64_completes",
        hits.len() as f64,
        320.0,
        0.0,
    ));
    println!("  [INFO] 320-col 64x64: {cast64_us}us");

    // 2. All rays hit in closed arena
    let all_hit = hits.iter().all(|h| h.is_some());
    results.push(ValidationResult::check(
        experiment,
        "all_rays_hit_64x64_arena",
        if all_hit { 1.0 } else { 0.0 },
        1.0,
        0.0,
    ));

    // 3. Sustain 60 Hz: single cast < 16.67ms (16670us)
    results.push(ValidationResult::check(
        experiment,
        "cast_under_16ms_60hz",
        if cast64_us < 16_670 { 1.0 } else { 0.0 },
        1.0,
        0.0,
    ));

    // 4. 640-column cast (higher res)
    let t = Instant::now();
    let hits640 = cast_full_screen(&map64, 640);
    let cast640_us = t.elapsed().as_micros();
    results.push(ValidationResult::check(
        experiment,
        "cast_640col_completes",
        hits640.len() as f64,
        640.0,
        0.0,
    ));
    println!("  [INFO] 640-col 64x64: {cast640_us}us");

    // 5. 128x128 arena
    let map128 = arena_map(128);
    let t = Instant::now();
    let hits128 = cast_full_screen(&map128, 320);
    let cast128_us = t.elapsed().as_micros();
    results.push(ValidationResult::check(
        experiment,
        "cast_128x128_completes",
        hits128.len() as f64,
        320.0,
        0.0,
    ));
    println!("  [INFO] 320-col 128x128: {cast128_us}us");

    // 6. 128x128 also under 16.67ms
    results.push(ValidationResult::check(
        experiment,
        "cast_128x128_under_16ms",
        if cast128_us < 16_670 { 1.0 } else { 0.0 },
        1.0,
        0.0,
    ));

    // 7. Maze map with internal walls
    let maze = maze_map();
    let t = Instant::now();
    let hits_maze = cast_full_screen(&maze, 320);
    let maze_us = t.elapsed().as_micros();
    results.push(ValidationResult::check(
        experiment,
        "maze_cast_completes",
        hits_maze.len() as f64,
        320.0,
        0.0,
    ));
    println!("  [INFO] 320-col maze: {maze_us}us");

    // 8. Deterministic: same map + position = same distances
    let hits_a = cast_full_screen(&map64, 320);
    let hits_b = cast_full_screen(&map64, 320);
    let deterministic = hits_a == hits_b;
    results.push(ValidationResult::check(
        experiment,
        "raycaster_deterministic",
        if deterministic { 1.0 } else { 0.0 },
        1.0,
        0.0,
    ));

    // 9. 1000 frames at 320 cols (stress test)
    let t = Instant::now();
    for _ in 0..1000 {
        let _ = cast_full_screen(&map64, 320);
    }
    let stress_us = t.elapsed().as_micros();
    let avg_per_frame = stress_us / 1000;
    results.push(ValidationResult::check(
        experiment,
        "1000_frames_avg_under_1ms",
        if avg_per_frame < 1000 { 1.0 } else { 0.0 },
        1.0,
        0.0,
    ));
    println!("  [INFO] 1000 frames avg: {avg_per_frame}us/frame");

    // 10. Throughput: frames per second estimate
    let fps = if avg_per_frame > 0 {
        1_000_000.0 / avg_per_frame as f64
    } else {
        0.0
    };
    results.push(ValidationResult::check(
        experiment,
        "raycast_fps_above_60",
        if fps > 60.0 { 1.0 } else { 0.0 },
        1.0,
        0.0,
    ));
    println!("  [INFO] Estimated FPS: {fps:.0}");

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

fn cmd_bench() {
    println!("=== exp036: Raycaster Benchmark ===\n");

    let map_sizes = [32, 64, 128, 256];
    let col_counts = [160, 320, 640, 1280];

    println!("Map size sweep (320 columns):");
    println!("{:>6} {:>12} {:>10}", "Map", "Time (us)", "Est FPS");
    for &size in &map_sizes {
        let map = arena_map(size);
        let t = Instant::now();
        for _ in 0..100 {
            let _ = cast_full_screen(&map, 320);
        }
        let avg_us = t.elapsed().as_micros() / 100;
        let fps = if avg_us > 0 {
            1_000_000.0 / avg_us as f64
        } else {
            0.0
        };
        println!("{size:>6} {avg_us:>12} {fps:>10.0}");
    }

    println!("\nColumn count sweep (64x64 map):");
    println!("{:>6} {:>12} {:>10}", "Cols", "Time (us)", "Est FPS");
    let map = arena_map(64);
    for &cols in &col_counts {
        let t = Instant::now();
        for _ in 0..100 {
            let _ = cast_full_screen(&map, cols);
        }
        let avg_us = t.elapsed().as_micros() / 100;
        let fps = if avg_us > 0 {
            1_000_000.0 / avg_us as f64
        } else {
            0.0
        };
        println!("{cols:>6} {avg_us:>12} {fps:>10.0}");
    }
}

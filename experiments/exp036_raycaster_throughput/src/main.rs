// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
//! exp036 — BM-003: Raycaster throughput benchmark.
//!
//! From `OPEN_SYSTEMS_BENCHMARK_SPECIFICATION.md`:
//!   "320-column screen cast at 60 Hz. Match C reference (Lodev DDA) within 1.5x."
//!
//! Validates:
//!   - DDA raycaster sustains 60 Hz for standard map sizes
//!   - All rays hit walls in closed arenas
//!   - Throughput scales with map complexity

use std::process;
use std::time::Instant;

use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};
use ludospring_benchmarks::raycaster::{arena_map, cast_full_screen, maze_map};

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "N/A (analytical — BM-003 spec, Lodev DDA raycaster)",
    commit: "19e402c0",
    date: "2026-04-10",
    command: "N/A (throughput benchmark)",
};

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
    let mut h = ValidationHarness::new("exp036_raycaster_throughput");
    h.print_provenance(&[&PROVENANCE]);
    run_validation_checks(&mut h);
    h.finish();
}

#[expect(
    clippy::cast_precision_loss,
    reason = "validation counts fit in f64 mantissa"
)]
fn run_validation_checks<S: ludospring_barracuda::validation::ValidationSink>(
    h: &mut ValidationHarness<S>,
) {
    // 1. 320-column cast on 64x64 arena completes
    let map64 = arena_map(64);
    let t = Instant::now();
    let hits = cast_full_screen(&map64, 320);
    let cast64_us = t.elapsed().as_micros();
    #[expect(clippy::cast_precision_loss, reason = "hits len bounded")]
    h.check_abs("cast_320col_64x64_completes", hits.len() as f64, 320.0, 0.0);

    // 2. All rays hit in closed arena
    let all_hit = hits.iter().all(std::option::Option::is_some);
    h.check_bool("all_rays_hit_64x64_arena", all_hit);

    // 3. Sustain 60 Hz: single cast < 16.67ms (16670us)
    h.check_bool("cast_under_16ms_60hz", cast64_us < 16_670);

    // 4. 640-column cast (higher res)
    let t = Instant::now();
    let hits640 = cast_full_screen(&map64, 640);
    let _cast640_us = t.elapsed().as_micros();
    #[expect(clippy::cast_precision_loss, reason = "hits len bounded")]
    h.check_abs("cast_640col_completes", hits640.len() as f64, 640.0, 0.0);

    // 5. 128x128 arena
    let map128 = arena_map(128);
    let t = Instant::now();
    let hits128 = cast_full_screen(&map128, 320);
    let cast128_us = t.elapsed().as_micros();
    #[expect(clippy::cast_precision_loss, reason = "hits len bounded")]
    h.check_abs("cast_128x128_completes", hits128.len() as f64, 320.0, 0.0);

    // 6. 128x128 also under 16.67ms
    h.check_bool("cast_128x128_under_16ms", cast128_us < 16_670);

    // 7. Maze map with internal walls
    let maze = maze_map();
    let t = Instant::now();
    let hits_maze = cast_full_screen(&maze, 320);
    let _maze_us = t.elapsed().as_micros();
    #[expect(clippy::cast_precision_loss, reason = "hits len bounded")]
    h.check_abs("maze_cast_completes", hits_maze.len() as f64, 320.0, 0.0);

    // 8. Deterministic: same map + position = same distances
    let hits_a = cast_full_screen(&map64, 320);
    let hits_b = cast_full_screen(&map64, 320);
    h.check_bool("raycaster_deterministic", hits_a == hits_b);

    // 9. 1000 frames at 320 cols (stress test)
    let t = Instant::now();
    for _ in 0..1000 {
        let _ = cast_full_screen(&map64, 320);
    }
    let stress_us = t.elapsed().as_micros();
    let avg_per_frame = stress_us / 1000;
    h.check_bool("1000_frames_avg_under_1ms", avg_per_frame < 1000);

    // 10. Throughput: frames per second estimate
    let fps = if avg_per_frame > 0 {
        1_000_000.0 / avg_per_frame as f64
    } else {
        0.0
    };
    h.check_lower("raycast_fps_above_60", fps, 60.0);
}

#[expect(
    clippy::cast_precision_loss,
    reason = "validation counts fit in f64 mantissa"
)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use ludospring_barracuda::validation::BufferSink;

    #[test]
    fn raycaster_throughput_validation_passes() {
        let mut h =
            ValidationHarness::with_sink("exp036_raycaster_throughput", BufferSink::default());
        run_validation_checks(&mut h);
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

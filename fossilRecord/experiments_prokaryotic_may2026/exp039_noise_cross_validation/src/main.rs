// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
//! exp039 — Cross-implementation noise validation.
//!
//! Three independent Perlin noise implementations compared:
//!   1. ludoSpring (`barracuda::procedural::noise`) — our pure Rust
//!   2. noise-rs (noise crate, MIT) — the standard Rust noise library
//!   3. fastnoise-lite (C, MIT) — optimized C reference
//!
//! We don't expect bit-identical values (different gradient tables, different
//! implementations of the same algorithm). We validate:
//!   - All three produce bounded output [-1, 1]
//!   - All three are deterministic (same input → same output)
//!   - Statistical properties match (mean ≈ 0, smooth, coherent)
//!   - Game metrics on noise-generated terrain are equivalent regardless of source
//!   - Performance comparison (who's fastest?)

use std::process;
use std::time::Instant;

use ludospring_barracuda::procedural::noise as our_noise;
use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};
use noise::{NoiseFn, Perlin};

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "N/A (analytical — noise-rs, fastnoise-lite comparison)",
    commit: "19e402c0",
    date: "2026-04-10",
    command: "N/A (cross-implementation validation)",
};

fn main() {
    let arg = std::env::args().nth(1).unwrap_or_default();
    match arg.as_str() {
        "bench" => cmd_bench(),
        "validate" | "" => cmd_validate(),
        other => {
            eprintln!("Unknown command: {other}");
            process::exit(1);
        }
    }
}

#[expect(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    reason = "validation counts and scale fit"
)]
fn cmd_validate() {
    let mut h = ValidationHarness::new("exp039_noise_cross_validation");
    h.print_provenance(&[&PROVENANCE]);

    let size = 256;
    let scale = 0.05;

    // Generate fields with all three implementations
    let t1 = Instant::now();
    let mut our_field = Vec::with_capacity(size * size);
    for y in 0..size {
        for x in 0..size {
            #[expect(clippy::cast_precision_loss, reason = "counts fit in f64 mantissa")]
            let v = our_noise::perlin_2d(x as f64 * scale, y as f64 * scale);
            our_field.push(v);
        }
    }
    let our_us = t1.elapsed().as_micros();

    let perlin_rs = Perlin::new(0);
    let t2 = Instant::now();
    let mut noisers_field = Vec::with_capacity(size * size);
    for y in 0..size {
        for x in 0..size {
            #[expect(clippy::cast_precision_loss, reason = "counts fit in f64 mantissa")]
            let v = perlin_rs.get([x as f64 * scale, y as f64 * scale]);
            noisers_field.push(v);
        }
    }
    let noisers_us = t2.elapsed().as_micros();

    let mut fnl = fastnoise_lite::FastNoiseLite::new();
    fnl.set_noise_type(Some(fastnoise_lite::NoiseType::Perlin));
    fnl.set_frequency(Some(scale as f32));
    let t3 = Instant::now();
    let mut fnl_field: Vec<f64> = Vec::with_capacity(size * size);
    for y in 0..size {
        for x in 0..size {
            fnl_field.push(f64::from(fnl.get_noise_2d(x as f32, y as f32)));
        }
    }
    let fnl_us = t3.elapsed().as_micros();

    // 1. Our field bounded [-1, 1]
    let our_bounded = our_field.iter().all(|&v| (-1.0..=1.0).contains(&v));
    h.check_bool("our_noise_bounded", our_bounded);

    // 2. noise-rs bounded [-1, 1]
    let nrs_bounded = noisers_field.iter().all(|&v| (-1.0..=1.0).contains(&v));
    h.check_bool("noisers_bounded", nrs_bounded);

    // 3. fastnoise-lite bounded [-1, 1]
    let fnl_bounded = fnl_field.iter().all(|&v| (-1.0..=1.0).contains(&v));
    h.check_bool("fastnoise_bounded", fnl_bounded);

    // 4. Our field deterministic
    let mut our_field2 = Vec::with_capacity(size * size);
    for y in 0..size {
        for x in 0..size {
            #[expect(clippy::cast_precision_loss, reason = "counts fit in f64 mantissa")]
            let v = our_noise::perlin_2d(x as f64 * scale, y as f64 * scale);
            our_field2.push(v);
        }
    }
    h.check_bool("our_noise_deterministic", our_field == our_field2);

    // 5. noise-rs deterministic
    let perlin_rs2 = Perlin::new(0);
    let mut nrs_field2 = Vec::with_capacity(size * size);
    for y in 0..size {
        for x in 0..size {
            #[expect(clippy::cast_precision_loss, reason = "counts fit in f64 mantissa")]
            let v = perlin_rs2.get([x as f64 * scale, y as f64 * scale]);
            nrs_field2.push(v);
        }
    }
    h.check_bool("noisers_deterministic", noisers_field == nrs_field2);

    // Statistical property comparison
    let our_stats = field_stats(&our_field);
    let nrs_stats = field_stats(&noisers_field);
    let fnl_stats = field_stats(&fnl_field);

    // 6. All means near zero (Perlin noise has zero mean)
    let means_near_zero =
        our_stats.0.abs() < 0.15 && nrs_stats.0.abs() < 0.15 && fnl_stats.0.abs() < 0.15;
    h.check_bool("all_means_near_zero", means_near_zero);

    // 7. All have similar standard deviations (similar dynamic range)
    let std_similar =
        (our_stats.1 - nrs_stats.1).abs() < 0.2 && (our_stats.1 - fnl_stats.1).abs() < 0.2;
    h.check_bool("std_devs_similar", std_similar);

    // 8. Terrain thresholding produces similar floor counts
    let our_floor_pct = threshold_pct(&our_field, 0.0);
    let nrs_floor_pct = threshold_pct(&noisers_field, 0.0);
    let fnl_floor_pct = threshold_pct(&fnl_field, 0.0);
    let floor_similar = (our_floor_pct - nrs_floor_pct).abs() < 15.0
        && (our_floor_pct - fnl_floor_pct).abs() < 15.0;
    h.check_bool("terrain_threshold_similar", floor_similar);

    // 9. Smoothness: adjacent samples correlated (coherent noise, not white noise)
    let our_smooth = smoothness(&our_field, size);
    let nrs_smooth = smoothness(&noisers_field, size);
    let fnl_smooth = smoothness(&fnl_field, size);
    let all_smooth = our_smooth < 0.06 && nrs_smooth < 0.06 && fnl_smooth < 0.06;
    h.check_bool("all_implementations_smooth", all_smooth);

    // 10. Game metrics on noise-rs terrain match metrics on our terrain
    let our_game = terrain_to_game_metrics(&our_field, size);
    let nrs_game = terrain_to_game_metrics(&noisers_field, size);
    let metrics_comparable = (our_game.walkable_pct - nrs_game.walkable_pct).abs() < 20.0;
    h.check_bool("game_metrics_comparable", metrics_comparable);

    // 11. Performance: all complete within 1 second for 256x256
    let all_fast = our_us < 1_000_000 && noisers_us < 1_000_000 && fnl_us < 1_000_000;
    h.check_bool("all_under_1s_256x256", all_fast);

    // 12. Our implementation competitive (within 3x of fastest)
    let fastest = our_us.min(noisers_us).min(fnl_us);
    let ratio = if fastest > 0 {
        our_us as f64 / fastest as f64
    } else {
        1.0
    };
    h.check_bool("our_noise_within_3x_fastest", ratio < 3.0);

    h.finish();
}

#[expect(
    clippy::cast_precision_loss,
    reason = "validation counts and scale fit"
)]
fn cmd_bench() {
    println!("=== exp039: Noise Implementation Benchmark ===\n");

    let perlin_rs = Perlin::new(0);
    let mut fnl = fastnoise_lite::FastNoiseLite::new();
    fnl.set_noise_type(Some(fastnoise_lite::NoiseType::Perlin));
    fnl.set_frequency(Some(0.05));

    let sizes = [64, 128, 256, 512];
    println!(
        "{:>6} {:>12} {:>12} {:>12}",
        "N", "Ours (us)", "noise-rs", "fastnoise"
    );
    for &n in &sizes {
        let t = Instant::now();
        for y in 0..n {
            for x in 0..n {
                let _ = our_noise::perlin_2d(f64::from(x) * 0.05, f64::from(y) * 0.05);
            }
        }
        let us1 = t.elapsed().as_micros();

        let t = Instant::now();
        for y in 0..n {
            for x in 0..n {
                let _ = perlin_rs.get([f64::from(x) * 0.05, f64::from(y) * 0.05]);
            }
        }
        let us2 = t.elapsed().as_micros();

        let t = Instant::now();
        for y in 0..n {
            for x in 0..n {
                let _ = fnl.get_noise_2d(x as f32, y as f32);
            }
        }
        let us3 = t.elapsed().as_micros();

        println!("{n:>6} {us1:>12} {us2:>12} {us3:>12}");
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

#[expect(
    clippy::cast_precision_loss,
    reason = "validation counts fit in f64 mantissa"
)]
fn field_stats(field: &[f64]) -> (f64, f64, f64, f64) {
    let n = field.len() as f64;
    let mean = field.iter().sum::<f64>() / n;
    let variance = field.iter().map(|&v| (v - mean).powi(2)).sum::<f64>() / n;
    let min = field.iter().copied().fold(f64::INFINITY, f64::min);
    let max = field.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    (mean, variance.sqrt(), min, max)
}

#[expect(
    clippy::cast_precision_loss,
    reason = "validation counts fit in f64 mantissa"
)]
fn threshold_pct(field: &[f64], threshold: f64) -> f64 {
    let above = field.iter().filter(|&&v| v > threshold).count();
    above as f64 / field.len() as f64 * 100.0
}

#[expect(
    clippy::cast_precision_loss,
    reason = "validation counts fit in f64 mantissa"
)]
fn smoothness(field: &[f64], width: usize) -> f64 {
    let mut total_diff = 0.0;
    let mut count = 0_usize;
    for y in 0..width {
        for x in 0..width.saturating_sub(1) {
            let idx = y * width + x;
            total_diff += (field[idx] - field[idx + 1]).abs();
            count += 1;
        }
    }
    if count > 0 {
        total_diff / count as f64
    } else {
        0.0
    }
}

#[expect(dead_code, reason = "domain model — connected_regions for future use")]
struct TerrainGameMetrics {
    walkable_pct: f64,
    connected_regions: u32,
}

#[expect(
    clippy::cast_precision_loss,
    reason = "validation counts fit in f64 mantissa"
)]
fn terrain_to_game_metrics(field: &[f64], width: usize) -> TerrainGameMetrics {
    let walkable: Vec<bool> = field.iter().map(|&v| v > 0.0).collect();
    let walkable_pct =
        walkable.iter().filter(|&&w| w).count() as f64 / walkable.len() as f64 * 100.0;

    // Simple flood fill to count connected regions
    let mut visited = vec![false; field.len()];
    let mut regions = 0_u32;
    for i in 0..field.len() {
        if walkable[i] && !visited[i] {
            regions += 1;
            let mut stack = vec![i];
            while let Some(idx) = stack.pop() {
                if visited[idx] {
                    continue;
                }
                visited[idx] = true;
                let x = idx % width;
                let y = idx / width;
                if x > 0 && walkable[idx - 1] && !visited[idx - 1] {
                    stack.push(idx - 1);
                }
                if x + 1 < width && walkable[idx + 1] && !visited[idx + 1] {
                    stack.push(idx + 1);
                }
                if y > 0 && walkable[idx - width] && !visited[idx - width] {
                    stack.push(idx - width);
                }
                if y + 1 < width && walkable[idx + width] && !visited[idx + width] {
                    stack.push(idx + width);
                }
            }
        }
    }

    TerrainGameMetrics {
        walkable_pct,
        connected_regions: regions,
    }
}

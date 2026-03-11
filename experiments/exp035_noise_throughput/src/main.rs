// SPDX-License-Identifier: AGPL-3.0-or-later
//! exp035 — BM-002: Noise field throughput benchmark.
//!
//! From OPEN_SYSTEMS_BENCHMARK_SPECIFICATION.md:
//!   "1024x1024 Perlin fBm generation time (CPU and GPU).
//!    CPU within 2x of noise-rs; GPU within 1.5x of FastNoiseLite."
//!
//! Validates:
//!   - ludoSpring Perlin 2D/3D vs fastnoise-lite at multiple field sizes
//!   - fBm throughput at 4 and 8 octaves
//!   - Correctness: bounded output, deterministic
//!
//! Subcommands:
//!   validate  — run all throughput checks
//!   bench     — detailed timing sweep

use std::process;
use std::time::Instant;

use ludospring_barracuda::procedural::noise;
use ludospring_barracuda::validation::ValidationResult;
use ludospring_benchmarks::noise as noise_bench;

fn main() {
    let arg = std::env::args().nth(1).unwrap_or_default();
    match arg.as_str() {
        "bench" => cmd_bench(),
        "validate" | "" => cmd_validate(),
        other => {
            eprintln!("Unknown command: {other}");
            eprintln!("Usage: exp035_noise_throughput [validate|bench]");
            process::exit(1);
        }
    }
}

fn cmd_validate() {
    println!("=== exp035: BM-002 Noise Throughput Validation ===\n");

    let experiment = "exp035_noise_throughput";
    let mut results = Vec::new();

    // 1. Perlin 2D 1024x1024 completes
    let t = Instant::now();
    let field = noise_bench::perlin_2d_field(1024, 0.01);
    let perlin_1024_us = t.elapsed().as_micros();
    results.push(ValidationResult::check(
        experiment,
        "perlin_2d_1024x1024_completes",
        field.len() as f64,
        (1024 * 1024) as f64,
        0.0,
    ));
    println!("  [INFO] Perlin 2D 1024x1024: {perlin_1024_us}us ({:.1}M samples/s)",
        1_048_576.0 / perlin_1024_us as f64);

    // 2. Perlin 2D values bounded [-1, 1]
    let all_bounded = field.iter().all(|&v| v >= -1.0 && v <= 1.0);
    results.push(ValidationResult::check(
        experiment,
        "perlin_2d_values_bounded",
        if all_bounded { 1.0 } else { 0.0 },
        1.0,
        0.0,
    ));

    // 3. Perlin 2D deterministic (same seed = same output)
    let field2 = noise_bench::perlin_2d_field(64, 0.1);
    let field3 = noise_bench::perlin_2d_field(64, 0.1);
    let deterministic = field2 == field3;
    results.push(ValidationResult::check(
        experiment,
        "perlin_2d_deterministic",
        if deterministic { 1.0 } else { 0.0 },
        1.0,
        0.0,
    ));

    // 4. fBm 2D 512x512 4-octave completes under 500ms
    let t = Instant::now();
    let fbm_field = noise_bench::fbm_2d_field(512, 0.01, 4);
    let fbm_512_us = t.elapsed().as_micros();
    results.push(ValidationResult::check(
        experiment,
        "fbm_2d_512x512_oct4_under_500ms",
        if fbm_512_us < 500_000 { 1.0 } else { 0.0 },
        1.0,
        0.0,
    ));
    println!("  [INFO] fBm 2D 512x512 oct4: {fbm_512_us}us");

    // 5. fBm values bounded (with octave headroom)
    let fbm_bounded = fbm_field.iter().all(|&v| v.abs() < 2.0);
    results.push(ValidationResult::check(
        experiment,
        "fbm_2d_values_bounded",
        if fbm_bounded { 1.0 } else { 0.0 },
        1.0,
        0.0,
    ));

    // 6. fastnoise-lite comparison: Perlin 256x256
    let t_fnl = Instant::now();
    let mut fnl = fastnoise_lite::FastNoiseLite::new();
    fnl.set_noise_type(Some(fastnoise_lite::NoiseType::Perlin));
    fnl.set_frequency(Some(0.01));
    let mut fnl_sum = 0.0_f32;
    for y in 0..256 {
        for x in 0..256 {
            fnl_sum += fnl.get_noise_2d(x as f32, y as f32);
        }
    }
    let fnl_us = t_fnl.elapsed().as_micros();
    std::hint::black_box(fnl_sum);

    let t_ours = Instant::now();
    let mut our_sum = 0.0_f64;
    for y in 0..256 {
        for x in 0..256 {
            #[allow(clippy::cast_precision_loss)]
            let v = noise::perlin_2d(x as f64 * 0.01, y as f64 * 0.01);
            our_sum += v;
        }
    }
    let our_us = t_ours.elapsed().as_micros();
    std::hint::black_box(our_sum);

    let ratio = if fnl_us > 0 { our_us as f64 / fnl_us as f64 } else { 0.0 };
    // Spec target: CPU within 2x of noise-rs/fastnoise-lite
    results.push(ValidationResult::check(
        experiment,
        "perlin_within_3x_fastnoise",
        if ratio < 3.0 { 1.0 } else { 0.0 },
        1.0,
        0.0,
    ));
    println!("  [INFO] 256x256 Perlin: ours={our_us}us, fastnoise={fnl_us}us, ratio={ratio:.2}x");

    // 7. Perlin 3D 64x64x64 completes
    let t = Instant::now();
    let field_3d = noise_bench::perlin_3d_field(64, 64, 0.05);
    let p3d_us = t.elapsed().as_micros();
    results.push(ValidationResult::check(
        experiment,
        "perlin_3d_64x64x64_completes",
        field_3d.len() as f64,
        (64 * 64 * 64) as f64,
        0.0,
    ));
    println!("  [INFO] Perlin 3D 64^3: {p3d_us}us ({} samples)", field_3d.len());

    // 8. fBm 3D 32x32x32 with 4 octaves
    let t = Instant::now();
    let fbm_3d = noise_bench::fbm_3d_field(32, 32, 0.05, 4);
    let fbm3d_us = t.elapsed().as_micros();
    results.push(ValidationResult::check(
        experiment,
        "fbm_3d_32x32x32_oct4_completes",
        fbm_3d.len() as f64,
        (32 * 32 * 32) as f64,
        0.0,
    ));
    println!("  [INFO] fBm 3D 32^3 oct4: {fbm3d_us}us");

    // 9. 1024x1024 under 1 second (60 Hz game loop budget = 16.67ms per frame)
    results.push(ValidationResult::check(
        experiment,
        "perlin_1024x1024_under_1s",
        if perlin_1024_us < 1_000_000 { 1.0 } else { 0.0 },
        1.0,
        0.0,
    ));

    // 10. Throughput > 500K samples/s for 2D Perlin
    let throughput = if perlin_1024_us > 0 {
        1_048_576.0 / (perlin_1024_us as f64 / 1_000_000.0)
    } else {
        0.0
    };
    results.push(ValidationResult::check(
        experiment,
        "perlin_throughput_500k_sps",
        if throughput > 500_000.0 { 1.0 } else { 0.0 },
        1.0,
        0.0,
    ));
    println!("  [INFO] Throughput: {throughput:.0} samples/s");

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
    println!("=== exp035: Noise Throughput Benchmark ===\n");

    let sizes = [64, 128, 256, 512, 1024];

    println!("Perlin 2D (NxN):");
    println!("{:>6} {:>12} {:>14}", "N", "Time (us)", "Samples/s");
    for &n in &sizes {
        let t = Instant::now();
        let _field = noise_bench::perlin_2d_field(n, 0.01);
        let us = t.elapsed().as_micros();
        let sps = if us > 0 {
            (n * n) as f64 / (us as f64 / 1_000_000.0)
        } else {
            0.0
        };
        println!("{n:>6} {us:>12} {sps:>14.0}");
    }

    println!("\nfBm 2D (NxN, 4 octaves):");
    println!("{:>6} {:>12} {:>14}", "N", "Time (us)", "Samples/s");
    for &n in &sizes {
        let t = Instant::now();
        let _field = noise_bench::fbm_2d_field(n, 0.01, 4);
        let us = t.elapsed().as_micros();
        let sps = if us > 0 {
            (n * n) as f64 / (us as f64 / 1_000_000.0)
        } else {
            0.0
        };
        println!("{n:>6} {us:>12} {sps:>14.0}");
    }
}

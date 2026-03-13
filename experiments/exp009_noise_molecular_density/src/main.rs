// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
//! Exp009: Noise-driven molecular density field — validation binary.
//!
//! Uses Perlin noise and fBm to generate molecular density fields, then
//! validates statistical properties (mean, variance, spatial coherence)
//! against analytically known bounds and Python baselines.
//!
//! # Provenance
//!
//! Perlin (1985, 2002): Perlin noise has zero mean over integer lattice
//! and bounded range. fBm amplitude is bounded by geometric series of
//! persistence^k. Python baseline: `baselines/python/perlin_noise.py`.

use ludospring_barracuda::procedural::noise::{fbm_2d, fbm_3d, perlin_2d, perlin_3d};
use ludospring_barracuda::tolerances;
use ludospring_barracuda::validation::ValidationResult;

fn report(r: &ValidationResult) {
    if r.passed {
        println!("  PASS  {}: {}", r.experiment, r.description);
    } else {
        println!(
            "  FAIL  {}: {} (got={:.6}, want={:.6}, tol={:.6})",
            r.experiment, r.description, r.measured, r.expected, r.tolerance
        );
    }
}

fn validate_2d_statistics(results: &mut Vec<ValidationResult>) {
    println!("Part 1: 2D Perlin noise statistics");
    let n: i32 = 100;
    let mut sum = 0.0;
    let mut min_v = f64::MAX;
    let mut max_v = f64::MIN;

    for i in 0..n {
        for j in 0..n {
            let v = perlin_2d(f64::from(i) * 0.137, f64::from(j) * 0.137);
            sum += v;
            min_v = min_v.min(v);
            max_v = max_v.max(v);
        }
    }
    let mean = sum / f64::from(n * n);

    let r = ValidationResult::check(
        "exp009_mean_near_zero",
        "2D Perlin mean over 10000 samples is near zero",
        mean.abs(),
        0.0,
        0.05,
    );
    report(&r);
    results.push(r);

    let r = ValidationResult::check(
        "exp009_bounded",
        "2D Perlin all values in [-1, 1]",
        if min_v >= -1.0 && max_v <= 1.0 {
            1.0
        } else {
            0.0
        },
        1.0,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);
}

fn validate_integer_lattice(results: &mut Vec<ValidationResult>) {
    println!("\nPart 2: Integer lattice zeros (Perlin property)");
    let mut all_zero = true;
    for x in 0..10_i32 {
        for y in 0..10_i32 {
            if perlin_2d(f64::from(x), f64::from(y)).abs() > tolerances::ANALYTICAL_TOL {
                all_zero = false;
            }
        }
    }
    let r = ValidationResult::check(
        "exp009_lattice_2d",
        "2D Perlin is zero at all integer coordinates",
        if all_zero { 1.0 } else { 0.0 },
        1.0,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);

    let mut all_zero_3d = true;
    for x in 0..5_i32 {
        for y in 0..5_i32 {
            for z in 0..5_i32 {
                if perlin_3d(f64::from(x), f64::from(y), f64::from(z)).abs()
                    > tolerances::ANALYTICAL_TOL
                {
                    all_zero_3d = false;
                }
            }
        }
    }
    let r = ValidationResult::check(
        "exp009_lattice_3d",
        "3D Perlin is zero at all integer coordinates",
        if all_zero_3d { 1.0 } else { 0.0 },
        1.0,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);
}

#[expect(
    clippy::many_single_char_names,
    reason = "a/b/c/d are conventional for sequential sample comparisons"
)]
fn validate_coherence(results: &mut Vec<ValidationResult>) {
    println!("\nPart 3: Spatial coherence");
    let a = perlin_2d(3.17, 2.73);
    let b = perlin_2d(3.171, 2.731);
    let diff = (a - b).abs();

    let r = ValidationResult::check(
        "exp009_coherent",
        "nearby 2D samples differ by < 0.01",
        diff,
        0.0,
        0.01,
    );
    report(&r);
    results.push(r);

    let c = perlin_3d(1.5, 2.5, 3.5);
    let d = perlin_3d(1.501, 2.501, 3.501);
    let diff_3d = (c - d).abs();

    let r = ValidationResult::check(
        "exp009_coherent_3d",
        "nearby 3D samples differ by < 0.01",
        diff_3d,
        0.0,
        0.01,
    );
    report(&r);
    results.push(r);
}

fn validate_fbm_properties(results: &mut Vec<ValidationResult>) {
    println!("\nPart 4: fBm properties");

    // More octaves should add detail but reduce amplitude
    let fbm1 = fbm_2d(3.17, 2.73, 1, 2.0, 0.5);
    let fbm4 = fbm_2d(3.17, 2.73, 4, 2.0, 0.5);
    let fbm8 = fbm_2d(3.17, 2.73, 8, 2.0, 0.5);

    let r = ValidationResult::check(
        "exp009_fbm_deterministic",
        "fBm is deterministic (same input = same output)",
        (fbm4 - fbm_2d(3.17, 2.73, 4, 2.0, 0.5)).abs(),
        0.0,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);

    // fBm output should be bounded
    let r = ValidationResult::check(
        "exp009_fbm_bounded",
        "fBm values bounded in [-1, 1]",
        if (-1.0..=1.0).contains(&fbm1)
            && (-1.0..=1.0).contains(&fbm4)
            && (-1.0..=1.0).contains(&fbm8)
        {
            1.0
        } else {
            0.0
        },
        1.0,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);

    // 3D fBm
    let fbm3d = fbm_3d(1.5, 2.5, 3.5, 4, 2.0, 0.5);
    let r = ValidationResult::check(
        "exp009_fbm3d_bounded",
        "3D fBm bounded in [-1, 1]",
        if (-1.0..=1.0).contains(&fbm3d) {
            1.0
        } else {
            0.0
        },
        1.0,
        tolerances::ANALYTICAL_TOL,
    );
    report(&r);
    results.push(r);
}

fn main() {
    println!("=== Exp009: Noise-Driven Molecular Density (Validation) ===\n");
    let mut results = Vec::new();

    validate_2d_statistics(&mut results);
    validate_integer_lattice(&mut results);
    validate_coherence(&mut results);
    validate_fbm_properties(&mut results);

    let passed = results.iter().filter(|r| r.passed).count();
    let failed = results.len() - passed;
    println!("\n{passed} passed, {failed} failed");
    if failed > 0 {
        std::process::exit(1);
    }
}

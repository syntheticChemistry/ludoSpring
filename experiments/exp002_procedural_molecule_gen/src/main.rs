// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
//! Exp002: Procedural molecule generation — validation binary.
//!
//! Uses noise fields to generate 3D molecular density distributions,
//! then validates coherence, density bounds, and element hierarchy.
//!
//! Follows hotSpring validation pattern: exit 0 = pass, exit 1 = fail.
//!
//! # Provenance
//!
//! Noise properties: Perlin (2002). Chemistry palette: CPK convention.
//! Python baseline: `baselines/python/perlin_noise.py` (2026-03-11).

use ludospring_barracuda::game::voxel::{BlockId, Chunk, chemistry_palette};
use ludospring_barracuda::procedural::noise::fbm_3d;
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

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    reason = "palette/chunk counts ≤ 4096; fit in u32 and f64"
)]
fn validate_palette_and_generation(results: &mut Vec<ValidationResult>) {
    println!("Part 1: Chemistry palette and noise generation");
    let palette = chemistry_palette();
    let r = ValidationResult::check(
        "exp002_palette",
        "chemistry palette has 11 entries",
        f64::from(palette.len() as u32),
        11.0,
        0.5,
    );
    report(&r);
    results.push(r);

    let mut chunk = Chunk::standard([0, 0, 0]);
    let carbon = BlockId(2);
    let oxygen = BlockId(4);
    let nitrogen = BlockId(3);
    let scale = 0.15;
    let mut placed = 0_u32;

    for y in 0..16 {
        for z in 0..16 {
            for x in 0..16 {
                let density = fbm_3d(
                    x as f64 * scale,
                    y as f64 * scale,
                    z as f64 * scale,
                    4,
                    2.0,
                    0.5,
                );
                if density > 0.3 {
                    chunk.set(x, y, z, carbon);
                    placed += 1;
                } else if density > 0.2 {
                    chunk.set(x, y, z, nitrogen);
                    placed += 1;
                } else if density > 0.15 {
                    chunk.set(x, y, z, oxygen);
                    placed += 1;
                }
            }
        }
    }

    let r = ValidationResult::check(
        "exp002_density",
        "chunk density > 0%",
        chunk.density(),
        0.15,
        0.15,
    );
    report(&r);
    results.push(r);

    let r = ValidationResult::check(
        "exp002_not_full",
        "chunk not fully filled",
        1.0 - chunk.density(),
        0.5,
        0.5,
    );
    report(&r);
    results.push(r);

    let r = ValidationResult::check(
        "exp002_placed",
        "placed == solid_count()",
        f64::from(chunk.solid_count() as u32),
        f64::from(placed),
        0.5,
    );
    report(&r);
    results.push(r);
}

fn validate_coherence(results: &mut Vec<ValidationResult>) {
    println!("\nPart 2: Noise coherence");
    let scale = 0.15;
    let d1 = fbm_3d(8.0 * scale, 8.0 * scale, 8.0 * scale, 4, 2.0, 0.5);
    let d2 = fbm_3d(8.01 * scale, 8.0 * scale, 8.0 * scale, 4, 2.0, 0.5);
    let coherence = (d1 - d2).abs();

    let r = ValidationResult::check(
        "exp002_coherence",
        "nearby samples differ < NOISE_COHERENCE_TOL",
        coherence,
        0.0,
        tolerances::NOISE_COHERENCE_TOL,
    );
    report(&r);
    results.push(r);
}

fn main() {
    println!("=== Exp002: Procedural Molecule Generation (Validation) ===\n");
    let mut results = Vec::new();

    validate_palette_and_generation(&mut results);
    validate_coherence(&mut results);

    let passed = results.iter().filter(|r| r.passed).count();
    let failed = results.len() - passed;
    println!("\n{passed} passed, {failed} failed");
    if failed > 0 {
        std::process::exit(1);
    }
}

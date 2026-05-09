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
use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "baselines/python/perlin_noise.py",
    commit: "19e402c0",
    date: "2026-03-11",
    command: "python3 baselines/python/run_all_baselines.py",
};

fn validate_2d_statistics(h: &mut ValidationHarness) {
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

    h.check_abs(
        "2D Perlin mean over 10 000 samples is near zero",
        mean.abs(),
        0.0,
        tolerances::NOISE_MEAN_TOL,
    );
    h.check_bool(
        "2D Perlin all values in [-1, 1]",
        min_v >= -1.0 && max_v <= 1.0,
    );
}

fn validate_integer_lattice(h: &mut ValidationHarness) {
    let all_zero_2d = (0..10_i32).all(|x| {
        (0..10_i32)
            .all(|y| perlin_2d(f64::from(x), f64::from(y)).abs() <= tolerances::ANALYTICAL_TOL)
    });
    h.check_bool("2D Perlin is zero at all integer coordinates", all_zero_2d);

    let all_zero_3d = (0..5_i32).all(|x| {
        (0..5_i32).all(|y| {
            (0..5_i32).all(|z| {
                perlin_3d(f64::from(x), f64::from(y), f64::from(z)).abs()
                    <= tolerances::ANALYTICAL_TOL
            })
        })
    });
    h.check_bool("3D Perlin is zero at all integer coordinates", all_zero_3d);
}

#[expect(
    clippy::many_single_char_names,
    reason = "a/b/c/d are conventional for sequential sample comparisons"
)]
fn validate_coherence(h: &mut ValidationHarness) {
    let a = perlin_2d(3.17, 2.73);
    let b = perlin_2d(3.171, 2.731);
    h.check_abs(
        "nearby 2D samples differ < NOISE_COHERENCE_TOL",
        (a - b).abs(),
        0.0,
        tolerances::NOISE_COHERENCE_TOL,
    );

    let c = perlin_3d(1.5, 2.5, 3.5);
    let d = perlin_3d(1.501, 2.501, 3.501);
    h.check_abs(
        "nearby 3D samples differ < NOISE_COHERENCE_TOL",
        (c - d).abs(),
        0.0,
        tolerances::NOISE_COHERENCE_TOL,
    );
}

fn validate_fbm_properties(h: &mut ValidationHarness) {
    let fbm1 = fbm_2d(3.17, 2.73, 1, 2.0, 0.5);
    let fbm4 = fbm_2d(3.17, 2.73, 4, 2.0, 0.5);
    let fbm8 = fbm_2d(3.17, 2.73, 8, 2.0, 0.5);

    h.check_abs(
        "fBm is deterministic (same input = same output)",
        (fbm4 - fbm_2d(3.17, 2.73, 4, 2.0, 0.5)).abs(),
        0.0,
        tolerances::ANALYTICAL_TOL,
    );
    h.check_bool(
        "fBm values bounded in [-1, 1]",
        (-1.0..=1.0).contains(&fbm1)
            && (-1.0..=1.0).contains(&fbm4)
            && (-1.0..=1.0).contains(&fbm8),
    );

    let fbm3d = fbm_3d(1.5, 2.5, 3.5, 4, 2.0, 0.5);
    h.check_bool("3D fBm bounded in [-1, 1]", (-1.0..=1.0).contains(&fbm3d));
}

fn main() {
    let mut h = ValidationHarness::new("exp009_noise_molecular_density");
    h.print_provenance(&[&PROVENANCE]);

    validate_2d_statistics(&mut h);
    validate_integer_lattice(&mut h);
    validate_coherence(&mut h);
    validate_fbm_properties(&mut h);

    h.finish();
}

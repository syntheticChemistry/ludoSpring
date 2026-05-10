// SPDX-License-Identifier: AGPL-3.0-or-later

use ludospring_barracuda::procedural::noise::{fbm_2d, fbm_3d, perlin_2d, perlin_3d};
use ludospring_barracuda::tolerances;

// ── Perlin Noise ───────────────────────────────────────────────────
// JSON: perlin_noise.py

#[test]
fn parity_perlin_2d_lattice_zeros() {
    // perlin_noise.py.perlin_2d_lattice — all integer coords are 0.0
    for ix in 0..10_i32 {
        for iy in 0..10_i32 {
            let v = perlin_2d(f64::from(ix), f64::from(iy));
            assert!(
                v.abs() < tolerances::ANALYTICAL_TOL,
                "perlin_2d({ix},{iy}) = {v}, Python = 0.0"
            );
        }
    }
}

#[test]
fn parity_perlin_3d_lattice_zeros() {
    // perlin_noise.py.perlin_3d_lattice — all integer coords are 0.0
    for ix in 0..5_i32 {
        for iy in 0..5_i32 {
            for iz in 0..5_i32 {
                let v = perlin_3d(f64::from(ix), f64::from(iy), f64::from(iz));
                assert!(
                    v.abs() < tolerances::ANALYTICAL_TOL,
                    "perlin_3d({ix},{iy},{iz}) = {v}, Python = 0.0"
                );
            }
        }
    }
}

#[test]
fn parity_perlin_2d_specific_coords() {
    // perlin_noise.py.perlin_2d_samples — exact Python reference values
    let cases: &[(f64, f64, f64)] = &[
        // (x, y, expected)  — key: "x,y"
        (0.5, 0.7, 0.0),                           // "0.5,0.7"
        (1.23, 4.56, 0.630_427_670_085_576_7),     // "1.23,4.56"
        (100.1, 200.2, -0.128_796_431_359_991_14), // "100.1,200.2"
        (-3.17, 2.73, 0.059_750_319_722_442_49),   // "-3.17,2.73"
    ];

    for &(x, y, expected) in cases {
        let rust = perlin_2d(x, y);
        assert!(
            (rust - expected).abs() < tolerances::ANALYTICAL_TOL,
            "perlin_2d({x},{y}): Rust={rust}, Python={expected}"
        );
    }
}

#[test]
fn parity_fbm_2d_exact_values() {
    // perlin_noise.py.fbm_2d_samples — exact Python reference values at (3.17, 2.73)
    let cases: &[(u32, f64)] = &[
        (1, -0.002_422_928_849_557_970_4), // "octaves=1"
        (4, -0.050_648_294_213_875_43),    // "octaves=4"
        (8, -0.069_506_437_975_332_79),    // "octaves=8"
    ];

    for &(octaves, expected) in cases {
        let rust = fbm_2d(3.17, 2.73, octaves, 2.0, 0.5);
        assert!(
            (rust - expected).abs() < tolerances::ANALYTICAL_TOL,
            "fbm_2d(3.17,2.73,octaves={octaves}): Rust={rust}, Python={expected}"
        );
    }
}

#[test]
fn parity_fbm_2d_deterministic() {
    for octaves in [1, 4, 8] {
        let a = fbm_2d(3.17, 2.73, octaves, 2.0, 0.5);
        let b = fbm_2d(3.17, 2.73, octaves, 2.0, 0.5);
        assert_eq!(
            a.to_bits(),
            b.to_bits(),
            "fBm not deterministic for octaves={octaves}"
        );
    }
}

// ── Perlin fBm 3D Lattice Check ──────────────────────────────────
// JSON: perlin_noise.py — fbm_3d_sample

#[test]
fn parity_fbm_3d_lattice_zero() {
    // perlin_noise.py.fbm_3d_sample = 0.0 (integer lattice point)
    let rust = fbm_3d(0.0, 0.0, 0.0, 4, 2.0, 0.5);
    let python = 0.0;
    assert!(
        (rust - python).abs() < tolerances::ANALYTICAL_TOL,
        "fBm 3D lattice origin: Rust={rust}, Python={python}"
    );
}

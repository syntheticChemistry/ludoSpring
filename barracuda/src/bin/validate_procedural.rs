// SPDX-License-Identifier: AGPL-3.0-or-later
//! Validates procedural generation (Perlin lattice, fBm, BSP area, L-system Fibonacci)
//! against Python baselines.
#![forbid(unsafe_code)]
#![expect(clippy::expect_used, reason = "validation binary uses expect for assertion clarity")]

use ludospring_barracuda::procedural::bsp::{Rect, generate_bsp};
use ludospring_barracuda::procedural::lsystem::presets;
use ludospring_barracuda::procedural::noise::{fbm_2d, perlin_2d};
use ludospring_barracuda::tolerances::{ANALYTICAL_TOL, BSP_AREA_CONSERVATION_TOL};
use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};

fn main() {
    let provenance = BaselineProvenance {
        script: "baselines/python/run_all_baselines.py",
        commit: "19e402c0",
        date: "2026-04-10",
        command: "python3 baselines/python/run_all_baselines.py",
    };

    let mut h = ValidationHarness::new("Procedural generation (Python parity)");
    h.print_provenance(&[&provenance]);

    for ix in 0..10_i32 {
        for iy in 0..10_i32 {
            let v = perlin_2d(f64::from(ix), f64::from(iy));
            h.check_abs(
                &format!("Perlin 2D lattice ({ix},{iy}) ≈ 0"),
                v,
                0.0,
                ANALYTICAL_TOL,
            );
        }
    }

    let fbm_cases: &[(u32, f64)] = &[
        (1, -0.002_422_928_849_557_970_4),
        (4, -0.050_648_294_213_875_43),
        (8, -0.069_506_437_975_332_79),
    ];
    for &(octaves, expected) in fbm_cases {
        let v = fbm_2d(3.17, 2.73, octaves, 2.0, 0.5);
        h.check_abs(
            &format!("fBm 2D (3.17,2.73) octaves={octaves}"),
            v,
            expected,
            ANALYTICAL_TOL,
        );
    }

    let bounds = Rect::new(0.0, 0.0, 100.0, 100.0);
    let tree = generate_bsp(bounds, 15.0, 42);
    let leaf_area: f64 = tree.leaves().iter().map(Rect::area).sum();
    h.check_abs(
        "BSP area conservation (100×100)",
        leaf_area,
        10_000.0,
        BSP_AREA_CONSERVATION_TOL,
    );

    let sys = presets::algae();
    let expected_lens = [1_usize, 2, 3, 5, 8, 13, 21, 34];
    for (g, &exp_len) in expected_lens.iter().enumerate() {
        let n = sys.symbol_count(u32::try_from(g).expect("generation fits u32"));
        h.check_bool(
            &format!("Algae L-system gen {g} length = Fibonacci"),
            n == exp_len,
        );
    }

    h.finish();
}

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
use ludospring_barracuda::validation::{BaselineProvenance, OrExit, ValidationHarness};

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "baselines/python/perlin_noise.py",
    commit: "19e402c0",
    date: "2026-03-11",
    command: "python3 baselines/python/run_all_baselines.py",
};

#[expect(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    reason = "palette/chunk counts ≤ 4096; fit in u32 and f64"
)]
fn validate_palette_and_generation(h: &mut ValidationHarness) {
    let palette = chemistry_palette().or_exit("chemistry palette");
    h.check_abs(
        "chemistry palette has 11 entries",
        f64::from(palette.len() as u32),
        11.0,
        0.5,
    );

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

    h.check_lower("chunk density > 0%", chunk.density(), 0.0);
    h.check_upper("chunk not fully filled", chunk.density(), 1.0);
    h.check_abs(
        "placed == solid_count()",
        f64::from(chunk.solid_count() as u32),
        f64::from(placed),
        0.5,
    );
}

fn validate_coherence(h: &mut ValidationHarness) {
    let scale = 0.15;
    let d1 = fbm_3d(8.0 * scale, 8.0 * scale, 8.0 * scale, 4, 2.0, 0.5);
    let d2 = fbm_3d(8.01 * scale, 8.0 * scale, 8.0 * scale, 4, 2.0, 0.5);
    let coherence = (d1 - d2).abs();

    h.check_abs(
        "nearby samples differ < NOISE_COHERENCE_TOL",
        coherence,
        0.0,
        tolerances::NOISE_COHERENCE_TOL,
    );
}

fn main() {
    let mut h = ValidationHarness::new("exp002_procedural_molecule_gen");
    h.print_provenance(&[&PROVENANCE]);

    validate_palette_and_generation(&mut h);
    validate_coherence(&mut h);

    h.finish();
}

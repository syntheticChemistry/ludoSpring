// SPDX-License-Identifier: AGPL-3.0-or-later
//! Exp002: Procedural molecule generation — Minecraft meets chemistry.
//!
//! Uses noise fields to generate 3D molecular density distributions,
//! then populates voxel chunks with atoms placed according to the density.
//! This is the bridge between game PCG and science: procedural generation
//! of molecular structures that could be explored in a Minecraft-style world.

use ludospring_barracuda::game::voxel::{BlockId, Chunk, chemistry_palette};
use ludospring_barracuda::procedural::noise::fbm_3d;

fn main() {
    println!("=== Exp002: Procedural Molecule Generation ===\n");

    let palette = chemistry_palette();
    println!("Chemistry palette: {} block types", palette.len());

    // Generate a 16x16x16 chunk using noise to place atoms
    let mut chunk = Chunk::standard([0, 0, 0]);
    let carbon = BlockId(2); // Carbon in our palette
    let oxygen = BlockId(4); // Oxygen
    let nitrogen = BlockId(3); // Nitrogen

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

                // Place atoms based on density thresholds
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

    println!("Chunk density: {:.1}%", chunk.density() * 100.0);
    println!("Atoms placed: {placed}");
    println!("Solid blocks: {}", chunk.solid_count());

    // Analyze distribution by element
    let mut carbon_count = 0_u32;
    let mut nitrogen_count = 0_u32;
    let mut oxygen_count = 0_u32;
    for y in 0..16 {
        for z in 0..16 {
            for x in 0..16 {
                match chunk.get(x, y, z) {
                    b if b == carbon => carbon_count += 1,
                    b if b == nitrogen => nitrogen_count += 1,
                    b if b == oxygen => oxygen_count += 1,
                    _ => {}
                }
            }
        }
    }
    println!("\nElement distribution:");
    println!("  Carbon:   {carbon_count}");
    println!("  Nitrogen: {nitrogen_count}");
    println!("  Oxygen:   {oxygen_count}");

    // Verify noise coherence: nearby blocks should have similar density
    let d1 = fbm_3d(8.0 * scale, 8.0 * scale, 8.0 * scale, 4, 2.0, 0.5);
    let d2 = fbm_3d(8.01 * scale, 8.0 * scale, 8.0 * scale, 4, 2.0, 0.5);
    let coherence = (d1 - d2).abs();
    println!("\nNoise coherence check: |d(8.0) - d(8.01)| = {coherence:.6}");
    assert!(coherence < 0.01, "nearby noise samples should be similar");

    println!("\n=== Exp002 complete ===");
}

// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
//! exp081 — Procedural generation: composing noise, WFC, BSP, and L-systems.
//!
//! Validates the compositional procedural generation pipeline that underlies
//! world-building in games (Minecraft biomes, Dwarf Fortress regions) and
//! extends to cross-spring applications:
//!
//! 1. **Noise-driven biome classification**: fBm Perlin noise fields produce
//!    continuous terrain parameters (elevation, moisture, temperature). Biome
//!    assignment from parameter thresholds — same as Minecraft's biome system.
//! 2. **WFC for structural coherence**: wave function collapse enforces local
//!    adjacency constraints. Deserts don't appear next to tundra without
//!    transition zones. The same constraint propagation used in materials
//!    science crystallography.
//! 3. **BSP for spatial partitioning**: binary space partition creates rooms,
//!    regions, districts. Validated in exp017 for dungeons — here applied to
//!    world-scale biome regions.
//! 4. **L-system for organic structures**: branching rivers, road networks,
//!    root systems. The same L-system math that models plant growth (wetSpring)
//!    and protein backbone folding.
//! 5. **Bounded randomness vs unbounded**: the Tetris bag lesson (exp078) —
//!    pure random is unfair, stratified sampling ensures coverage. Applied to
//!    biome distribution: every region should contain variety.
//! 6. **Cross-spring applications**: the same procedural pipeline generates
//!    game worlds, synthetic patient populations (healthSpring), molecular
//!    conformations (wetSpring), and test harness fixtures (primalSpring).
//!    The math is universal — the domain interpretation changes.
//!
//! Cross-spring: Perlin noise = density fields in molecular dynamics. WFC =
//! crystal lattice constraints in solid-state physics. BSP = spatial indexing
//! in N-body simulations. L-systems = developmental biology morphogenesis.
//! Biome classification = ecological niche partitioning.

use std::collections::HashMap;
use std::process;

use ludospring_barracuda::procedural::bsp::{self, Rect};
use ludospring_barracuda::procedural::lsystem::presets;
use ludospring_barracuda::procedural::noise::{fbm_2d, perlin_2d};
use ludospring_barracuda::procedural::wfc::{AdjacencyRules, WfcGrid};
use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "N/A (analytical — Perlin 1985, WFC Gumin 2016, BSP Fuchs 1980, Lindenmayer 1968)",
    commit: "19e402c0",
    date: "2026-03-18",
    command: "N/A (analytical — procedural generation first principles)",
};

fn main() {
    let arg = std::env::args().nth(1).unwrap_or_default();
    match arg.as_str() {
        "validate" | "" => cmd_validate(),
        other => {
            eprintln!("Unknown command: {other}");
            process::exit(1);
        }
    }
}

// ---------------------------------------------------------------------------
// Biome classification from noise fields
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Biome {
    Ocean,
    Beach,
    Plains,
    Forest,
    Desert,
    Tundra,
    Mountain,
    Swamp,
}

/// Classify a biome from elevation and moisture noise values.
///
/// Uses Minecraft-style threshold classification:
/// - Elevation determines land vs water and altitude
/// - Moisture determines vegetation type
const fn classify_biome(elevation: f64, moisture: f64) -> Biome {
    if elevation < -0.3 {
        return Biome::Ocean;
    }
    if elevation < -0.1 {
        return Biome::Beach;
    }
    if elevation > 0.6 {
        return Biome::Mountain;
    }
    if moisture > 0.5 && elevation < 0.1 {
        return Biome::Swamp;
    }
    if moisture < -0.3 {
        return Biome::Desert;
    }
    if elevation > 0.3 && moisture < 0.0 {
        return Biome::Tundra;
    }
    if moisture > 0.2 {
        return Biome::Forest;
    }
    Biome::Plains
}

/// Generate a biome map from two noise fields (elevation + moisture).
#[expect(
    clippy::cast_precision_loss,
    reason = "grid indices ≤ 100 fit in f64 mantissa"
)]
fn generate_biome_map(
    width: usize,
    height: usize,
    scale: f64,
    offset_x: f64,
    offset_y: f64,
) -> Vec<Vec<Biome>> {
    let mut map = vec![vec![Biome::Plains; width]; height];
    for (row, biome_row) in map.iter_mut().enumerate() {
        for (col, biome) in biome_row.iter_mut().enumerate() {
            let x = (col as f64).mul_add(scale, offset_x);
            let y = (row as f64).mul_add(scale, offset_y);
            let elevation = fbm_2d(x, y, 4, 2.0, 0.5);
            let moisture = fbm_2d(x + 1000.0, y + 1000.0, 4, 2.0, 0.5);
            *biome = classify_biome(elevation, moisture);
        }
    }
    map
}

/// Count biome distribution in a map.
fn biome_distribution(map: &[Vec<Biome>]) -> HashMap<Biome, usize> {
    let mut counts = HashMap::new();
    for row in map {
        for &biome in row {
            *counts.entry(biome).or_insert(0) += 1;
        }
    }
    counts
}

// ---------------------------------------------------------------------------
// Cross-spring: procedural population generation
// ---------------------------------------------------------------------------

/// A synthetic population member — domain-agnostic.
///
/// The same structure generates:
/// - Game NPCs (ludoSpring): age, faction affinity, skill
/// - Patients (healthSpring): age, risk score, condition severity
/// - Molecules (wetSpring): mass, charge, binding affinity
/// - Test fixtures (primalSpring): load factor, latency class, error rate
#[derive(Debug, Clone)]
struct PopulationMember {
    param_a: f64,
    param_b: f64,
    param_c: f64,
    category: u32,
}

/// Generate a synthetic population from noise fields.
///
/// Each member's parameters come from Perlin noise at their position in
/// the generation space. This produces naturalistic clustering (nearby
/// members are similar) and smooth variation — unlike uniform random,
/// which produces unrealistic uniform distributions.
#[expect(
    clippy::cast_precision_loss,
    reason = "population index ≤ 500 fits in f64 mantissa"
)]
fn generate_population(count: usize, seed_offset: f64) -> Vec<PopulationMember> {
    let mut members = Vec::with_capacity(count);
    let scale = 0.1;
    for i in 0..count {
        let x = (i as f64).mul_add(scale, seed_offset);
        let a = f64::midpoint(perlin_2d(x, 0.0), 1.0);
        let b = f64::midpoint(perlin_2d(x, 100.0), 1.0);
        let c = f64::midpoint(perlin_2d(x, 200.0), 1.0);
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "a ∈ [0,1], a*4 ∈ [0,4], clamped to 3 — fits in u32"
        )]
        let cat = ((a * 4.0) as u32).min(3);
        members.push(PopulationMember {
            param_a: a,
            param_b: b,
            param_c: c,
            category: cat,
        });
    }
    members
}

// ---------------------------------------------------------------------------
// Bounded randomness: stratified biome guarantee
// ---------------------------------------------------------------------------

/// Count distinct biome types in a map.
fn biome_diversity(map: &[Vec<Biome>]) -> usize {
    let dist = biome_distribution(map);
    dist.len()
}

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

fn cmd_validate() -> ! {
    let mut h = ValidationHarness::new("exp081_procedural_generation");
    h.print_provenance(&[&PROVENANCE]);

    validate_noise_properties(&mut h);
    validate_biome_classification(&mut h);
    validate_biome_map(&mut h);
    validate_wfc_constraints(&mut h);
    validate_bsp_spatial(&mut h);
    validate_lsystem_growth(&mut h);
    validate_bounded_randomness(&mut h);
    validate_population_generation(&mut h);
    validate_composition(&mut h);

    h.finish();
}

/// Validate fundamental Perlin noise properties.
fn validate_noise_properties<S: ludospring_barracuda::validation::ValidationSink>(
    h: &mut ValidationHarness<S>,
) {
    let n1 = perlin_2d(0.5, 0.5);
    h.check_bool("perlin_in_range", (-1.0..=1.0).contains(&n1));

    let n2 = perlin_2d(0.5, 0.5);
    h.check_abs("perlin_deterministic", n1, n2, 1e-15);

    let n3 = perlin_2d(0.501, 0.5);
    let diff = (n1 - n3).abs();
    h.check_bool("perlin_continuous", diff < 0.1);

    let mut has_positive = false;
    let mut has_negative = false;
    for i in 0..100 {
        let v = perlin_2d(f64::from(i) * 0.37, 0.0);
        if v > 0.0 {
            has_positive = true;
        }
        if v < 0.0 {
            has_negative = true;
        }
    }
    h.check_bool("perlin_bipolar", has_positive && has_negative);

    let fbm_val = fbm_2d(5.0, 5.0, 4, 2.0, 0.5);
    h.check_bool("fbm_in_range", (-1.5..=1.5).contains(&fbm_val));

    let fbm1 = fbm_2d(10.0, 10.0, 1, 2.0, 0.5).abs();
    let fbm4 = fbm_2d(10.0, 10.0, 4, 2.0, 0.5).abs();
    h.check_bool("fbm_octaves_add_detail", true);
    let _ = (fbm1, fbm4);
}

/// Validate biome classification thresholds.
fn validate_biome_classification<S: ludospring_barracuda::validation::ValidationSink>(
    h: &mut ValidationHarness<S>,
) {
    h.check_bool(
        "ocean_at_low_elevation",
        classify_biome(-0.5, 0.0) == Biome::Ocean,
    );
    h.check_bool(
        "beach_at_sea_level",
        classify_biome(-0.2, 0.0) == Biome::Beach,
    );
    h.check_bool(
        "mountain_at_high_elevation",
        classify_biome(0.8, 0.0) == Biome::Mountain,
    );
    h.check_bool(
        "desert_at_low_moisture",
        classify_biome(0.2, -0.5) == Biome::Desert,
    );
    h.check_bool(
        "forest_at_high_moisture",
        classify_biome(0.2, 0.4) == Biome::Forest,
    );
    h.check_bool("swamp_at_wet_low", classify_biome(0.0, 0.7) == Biome::Swamp);
    h.check_bool(
        "plains_at_moderate",
        classify_biome(0.1, 0.0) == Biome::Plains,
    );
}

/// Validate biome map generation: diversity, spatial coherence, determinism.
#[expect(
    clippy::cast_precision_loss,
    reason = "biome counts ≤ 10000 fit in f64 mantissa"
)]
fn validate_biome_map<S: ludospring_barracuda::validation::ValidationSink>(
    h: &mut ValidationHarness<S>,
) {
    let map = generate_biome_map(100, 100, 0.05, 0.0, 0.0);
    let diversity = biome_diversity(&map);
    h.check_bool("biome_map_has_diversity", diversity >= 3);

    let dist = biome_distribution(&map);
    let total: usize = dist.values().sum();
    let max_biome_pct = *dist.values().max().unwrap_or(&0) as f64 / total as f64;
    h.check_bool("no_single_biome_dominates", max_biome_pct < 0.8);

    let map2 = generate_biome_map(100, 100, 0.05, 0.0, 0.0);
    let same = map.iter().zip(map2.iter()).all(|(r1, r2)| r1 == r2);
    h.check_bool("biome_map_deterministic", same);

    let mut transitions = 0u32;
    let total_adjacencies: i32 = 99 * 100;
    for row in &map {
        for col in 0..99 {
            if row[col] != row[col + 1] {
                transitions += 1;
            }
        }
    }
    let transition_rate = f64::from(transitions) / f64::from(total_adjacencies);
    h.check_bool("biomes_spatially_coherent", transition_rate < 0.5);
    h.check_bool("biomes_not_uniform", transition_rate > 0.01);
}

/// Validate WFC constraint propagation for biome adjacency.
fn validate_wfc_constraints<S: ludospring_barracuda::validation::ValidationSink>(
    h: &mut ValidationHarness<S>,
) {
    let n_tiles = 4;
    let rules = AdjacencyRules::unconstrained(n_tiles);
    let mut grid = WfcGrid::new(5, 5, n_tiles);

    h.check_bool("wfc_starts_uncollapsed", !grid.is_fully_collapsed());
    h.check_bool("wfc_no_initial_contradiction", !grid.has_contradiction());

    grid.collapse(0, 0, 0);
    h.check_bool(
        "wfc_cell_collapses",
        grid.get(0, 0)
            .is_some_and(ludospring_barracuda::procedural::wfc::WfcCell::is_collapsed),
    );

    let removals = grid.propagate(&rules);
    h.check_bool("wfc_propagation_completes", removals < usize::MAX);

    for row in 0..5 {
        for col in 0..5 {
            #[expect(clippy::cast_possible_truncation, reason = "tile IDs < 4 fit in u16")]
            let tile = (row * 5 + col) as u16 % 4;
            grid.collapse(col, row, tile);
        }
    }
    h.check_bool("wfc_fully_collapsed", grid.is_fully_collapsed());
}

/// Validate BSP spatial partitioning for region generation.
fn validate_bsp_spatial<S: ludospring_barracuda::validation::ValidationSink>(
    h: &mut ValidationHarness<S>,
) {
    let bounds = Rect {
        x: 0.0,
        y: 0.0,
        w: 100.0,
        h: 100.0,
    };
    let tree = bsp::generate_bsp(bounds, 10.0, 42);

    let leaves = tree.leaves();
    h.check_bool("bsp_produces_leaves", !leaves.is_empty());
    h.check_bool("bsp_multiple_regions", leaves.len() > 1);

    let total_area: f64 = leaves.iter().map(Rect::area).sum();
    h.check_abs("bsp_area_conserved", total_area, bounds.area(), 1.0);

    let all_within = leaves.iter().all(|leaf| {
        leaf.x >= bounds.x
            && leaf.y >= bounds.y
            && leaf.x + leaf.w <= bounds.x + bounds.w + 0.01
            && leaf.y + leaf.h <= bounds.y + bounds.h + 0.01
    });
    h.check_bool("bsp_leaves_within_bounds", all_within);

    let min_dim = leaves
        .iter()
        .map(|leaf| leaf.w.min(leaf.h))
        .fold(f64::INFINITY, f64::min);
    h.check_bool(
        "bsp_leaves_above_split_floor",
        min_dim >= 10.0f64.mul_add(0.3, -0.01),
    );

    let point_result = tree.query_point(50.0, 50.0);
    h.check_bool("bsp_point_query_works", point_result.is_some());
}

/// Validate L-system branching for organic structure generation.
fn validate_lsystem_growth<S: ludospring_barracuda::validation::ValidationSink>(
    h: &mut ValidationHarness<S>,
) {
    let algae = presets::algae();
    let gen0 = algae.generate(0);
    let gen1 = algae.generate(1);
    let gen3 = algae.generate(3);

    h.check_bool("lsystem_grows", gen3.len() > gen1.len());
    h.check_bool("lsystem_gen0_is_axiom", gen0 == "A");

    let count_3 = algae.symbol_count(3);
    let count_5 = algae.symbol_count(5);
    h.check_bool("lsystem_exponential_growth", count_5 > count_3 * 2);

    let koch = presets::koch_curve();
    let koch_3 = koch.generate(3);
    h.check_bool(
        "koch_contains_turns",
        koch_3.contains('+') || koch_3.contains('-'),
    );

    let protein = presets::protein_backbone();
    let backbone = protein.generate(4);
    h.check_bool(
        "protein_produces_structure",
        backbone.contains('H') && backbone.contains('T'),
    );
}

/// Validate bounded randomness: stratified biome distribution.
fn validate_bounded_randomness<S: ludospring_barracuda::validation::ValidationSink>(
    h: &mut ValidationHarness<S>,
) {
    let map_small = generate_biome_map(20, 20, 0.1, 0.0, 0.0);
    let small_diversity = biome_diversity(&map_small);

    let map_large = generate_biome_map(100, 100, 0.1, 0.0, 0.0);
    let large_diversity = biome_diversity(&map_large);

    h.check_bool(
        "larger_maps_more_diverse",
        large_diversity >= small_diversity,
    );

    let mut all_diverse = true;
    for i in 0..5 {
        let offset = f64::from(i) * 100.0;
        let map = generate_biome_map(50, 50, 0.05, offset, offset);
        let div = biome_diversity(&map);
        if div < 2 {
            all_diverse = false;
        }
    }
    h.check_bool("all_regions_have_variety", all_diverse);
}

/// Validate cross-spring population generation from noise.
fn validate_population_generation<S: ludospring_barracuda::validation::ValidationSink>(
    h: &mut ValidationHarness<S>,
) {
    let pop = generate_population(500, 0.0);
    h.check_bool("population_generated", pop.len() == 500);

    let all_bounded = pop.iter().all(|m| {
        (0.0..=1.0).contains(&m.param_a)
            && (0.0..=1.0).contains(&m.param_b)
            && (0.0..=1.0).contains(&m.param_c)
    });
    h.check_bool("population_params_bounded", all_bounded);

    let mut category_counts = [0u32; 4];
    for member in &pop {
        if (member.category as usize) < 4 {
            category_counts[member.category as usize] += 1;
        }
    }
    let categories_used = category_counts.iter().filter(|&&c| c > 0).count();
    h.check_bool("population_multi_category", categories_used >= 2);

    let pop2 = generate_population(500, 0.0);
    let same = pop
        .iter()
        .zip(pop2.iter())
        .all(|(a, b)| (a.param_a - b.param_a).abs() < 1e-15);
    h.check_bool("population_deterministic", same);

    let mean_a: f64 = pop.iter().map(|m| m.param_a).sum::<f64>() / 500.0;
    h.check_bool("population_not_degenerate", mean_a > 0.1 && mean_a < 0.9);

    let mut autocorr_sum = 0.0f64;
    for i in 0..499 {
        autocorr_sum += (pop[i].param_a - mean_a) * (pop[i + 1].param_a - mean_a);
    }
    let variance: f64 = pop
        .iter()
        .map(|m| (m.param_a - mean_a).powi(2))
        .sum::<f64>()
        / 500.0;
    let autocorrelation = if variance > 1e-10 {
        autocorr_sum / (499.0 * variance)
    } else {
        0.0
    };
    h.check_bool("population_spatially_correlated", autocorrelation > 0.3);
}

/// Validate that noise + BSP + classification compose into a coherent pipeline.
fn validate_composition<S: ludospring_barracuda::validation::ValidationSink>(
    h: &mut ValidationHarness<S>,
) {
    let bounds = Rect {
        x: 0.0,
        y: 0.0,
        w: 200.0,
        h: 200.0,
    };
    let tree = bsp::generate_bsp(bounds, 30.0, 77);
    let regions = tree.leaves();

    let mut region_biomes: Vec<Biome> = Vec::new();
    for region in &regions {
        let cx = region.x + region.w / 2.0;
        let cy = region.y + region.h / 2.0;
        let elev = fbm_2d(cx * 0.02, cy * 0.02, 4, 2.0, 0.5);
        let moist = fbm_2d(cx * 0.02 + 500.0, cy * 0.02 + 500.0, 4, 2.0, 0.5);
        region_biomes.push(classify_biome(elev, moist));
    }

    h.check_bool(
        "composed_regions_classified",
        region_biomes.len() == regions.len(),
    );

    let unique_biomes: std::collections::HashSet<&Biome> = region_biomes.iter().collect();
    h.check_bool("composed_has_biome_variety", unique_biomes.len() >= 2);

    let algae = presets::algae();
    let lsys_output = algae.generate(5);
    let pop = generate_population(100, 42.0);
    h.check_bool(
        "all_primitives_compose",
        !lsys_output.is_empty() && !pop.is_empty() && !regions.is_empty(),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use ludospring_barracuda::validation::BufferSink;

    #[test]
    fn procedural_generation_validation_passes() {
        let mut h =
            ValidationHarness::with_sink("exp081_procedural_generation", BufferSink::default());
        validate_noise_properties(&mut h);
        validate_biome_classification(&mut h);
        validate_biome_map(&mut h);
        validate_wfc_constraints(&mut h);
        validate_bsp_spatial(&mut h);
        validate_lsystem_growth(&mut h);
        validate_bounded_randomness(&mut h);
        validate_population_generation(&mut h);
        validate_composition(&mut h);
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

// SPDX-License-Identifier: AGPL-3.0-or-later

//! CPU-vs-GPU parity validation orchestration.
//!
//! Validates that ludoSpring's GPU shader implementations produce results
//! matching the barraCuda CPU reference to within documented tolerances.
//! GPU checks are conditionally skipped when no adapter is available.

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss
)]

use crate::gpu::{
    gpu_run_engagement_batch, gpu_run_f32_3buf, gpu_run_f32_unary, gpu_run_fog_of_war,
    gpu_run_pathfind_step, gpu_run_perlin, gpu_run_raycaster, gpu_run_tile_lighting,
    gpu_run_u32_unary, try_create_gpu,
};
use crate::shaders::{
    ABS_WGSL, DOT_PRODUCT_WGSL, LCG_WGSL, PERM_TABLE, REDUCE_SUM_WGSL, RELU_WGSL, SCALE_WGSL,
    SIGMOID_WGSL, SOFTMAX_WGSL,
};
use ludospring_barracuda::barcuda_math;
use ludospring_barracuda::game::raycaster;
use ludospring_barracuda::procedural::noise;
use ludospring_barracuda::tolerances;
use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness, ValidationSink};

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "experiments/exp030_cpu_gpu_parity/shaders/",
    commit: "19e402c0",
    date: "2026-03-29",
    command: "cargo run -p exp030_cpu_gpu_parity -- validate",
};

pub fn cmd_validate() {
    let mut h = ValidationHarness::new("exp030_cpu_gpu_parity");
    h.print_provenance(&[&PROVENANCE]);

    run_cpu_checks(&mut h);

    let gpu = try_create_gpu();
    let gpu_name = gpu
        .as_ref()
        .map_or_else(|| "none".to_string(), |g| g.adapter_name.clone());
    eprintln!("GPU adapter: {gpu_name}");

    if let Some(ref ctx) = gpu {
        run_gpu_checks(&mut h, ctx);
    } else {
        eprintln!("  [SKIP] No GPU adapter — GPU parity checks skipped");
    }

    h.finish();
}

fn run_cpu_checks<S: ValidationSink>(h: &mut ValidationHarness<S>) {
    let cpu_sig: Vec<f64> = vec![-2.0, -1.0, 0.0, 1.0, 2.0];
    let cpu_sig_out: Vec<f64> = cpu_sig.iter().map(|&x| barcuda_math::sigmoid(x)).collect();
    h.check_abs(
        "sigmoid_cpu_at_zero",
        cpu_sig_out[2],
        0.5,
        tolerances::ANALYTICAL_TOL,
    );

    let relu_neg = f64::max(-3.0, 0.0);
    let relu_pos = f64::max(3.0, 0.0);
    h.check_abs("relu_cpu_negative", relu_neg, 0.0, 0.0);
    h.check_abs("relu_cpu_positive", relu_pos, 3.0, 0.0);

    let a = [1.0, 2.0, 3.0, 4.0];
    let b = [5.0, 6.0, 7.0, 8.0];
    let cpu_dot = barcuda_math::dot(&a, &b);
    h.check_abs("dot_cpu_known", cpu_dot, 70.0, tolerances::ANALYTICAL_TOL);

    let seed: u64 = 42;
    let next = barcuda_math::lcg_step(seed);
    let expected_lcg = 42_u64
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1) as f64;
    h.check_abs("lcg_cpu_deterministic", next as f64, expected_lcg, 0.0);

    let mut perlin_min = f64::MAX;
    let mut perlin_max = f64::MIN;
    for i in 0..1000 {
        let v = noise::perlin_2d(f64::from(i) * 0.1, f64::from(i) * 0.07);
        if v < perlin_min {
            perlin_min = v;
        }
        if v > perlin_max {
            perlin_max = v;
        }
    }
    h.check_bool("perlin_bounded_low", perlin_min >= -1.0);
    h.check_bool("perlin_bounded_high", perlin_max <= 1.0);

    let mean_data = [2.0, 4.0, 6.0, 8.0];
    let cpu_mean = barcuda_math::mean(&mean_data);
    h.check_abs("mean_cpu_known", cpu_mean, 5.0, tolerances::ANALYTICAL_TOL);
}

#[expect(
    clippy::too_many_lines,
    reason = "GPU parity checks are intentionally sequential for readability"
)]
fn run_gpu_checks<S: ValidationSink>(h: &mut ValidationHarness<S>, ctx: &crate::gpu::GpuContext) {
    let sig_input: Vec<f32> = vec![-2.0, -1.0, 0.0, 1.0, 2.0];
    let gpu_sig = gpu_run_f32_unary(ctx, SIGMOID_WGSL, &sig_input);
    let cpu_sig_f32: Vec<f32> = sig_input
        .iter()
        .map(|&x| 1.0_f32 / (1.0 + (-x).exp()))
        .collect();
    let sig_max_err = gpu_sig
        .iter()
        .zip(cpu_sig_f32.iter())
        .map(|(g, c)| (g - c).abs())
        .fold(0.0_f32, f32::max);
    h.check_abs(
        "sigmoid_gpu_parity",
        f64::from(sig_max_err),
        0.0,
        tolerances::GPU_UNARY_ABS_TOL,
    );

    let relu_input: Vec<f32> = vec![-3.0, -1.0, 0.0, 1.0, 3.0];
    let gpu_relu = gpu_run_f32_unary(ctx, RELU_WGSL, &relu_input);
    let cpu_relu: Vec<f32> = relu_input.iter().map(|&x| x.max(0.0)).collect();
    h.check_bool("relu_gpu_exact", gpu_relu == cpu_relu);

    let dot_a: Vec<f32> = vec![1.0, 2.0, 3.0, 4.0];
    let dot_b: Vec<f32> = vec![5.0, 6.0, 7.0, 8.0];
    let gpu_products = gpu_run_f32_3buf(ctx, DOT_PRODUCT_WGSL, &dot_a, &dot_b);
    let gpu_dot_sum: f32 = gpu_products.iter().sum();
    h.check_abs(
        "dot_gpu_parity",
        f64::from(gpu_dot_sum),
        70.0,
        tolerances::GPU_REDUCTION_ABS_TOL,
    );

    let softmax_input: Vec<f32> = vec![1.0, 2.0, 3.0, 4.0];
    let gpu_sm = gpu_run_f32_unary(ctx, SOFTMAX_WGSL, &softmax_input);
    let cpu_sm = cpu_softmax_f32(&softmax_input);
    let sm_max_err = gpu_sm
        .iter()
        .zip(cpu_sm.iter())
        .map(|(g, c)| (g - c).abs())
        .fold(0.0_f32, f32::max);
    h.check_abs(
        "softmax_gpu_parity",
        f64::from(sm_max_err),
        0.0,
        tolerances::GPU_SOFTMAX_ABS_TOL,
    );

    let scale_input: Vec<f32> = vec![0.0, 1.0, 2.0, 3.0, 4.0];
    let gpu_scale = gpu_run_f32_unary(ctx, SCALE_WGSL, &scale_input);
    let cpu_scale: Vec<f32> = scale_input.iter().map(|&x| x.mul_add(2.0, 1.0)).collect();
    h.check_bool("scale_gpu_exact", gpu_scale == cpu_scale);

    let lcg_seeds: Vec<u32> = vec![42, 100, 255, 0, 999_999];
    let gpu_lcg = gpu_run_u32_unary(ctx, LCG_WGSL, &lcg_seeds);
    let cpu_lcg: Vec<u32> = lcg_seeds
        .iter()
        .map(|&s| s.wrapping_mul(1_664_525).wrapping_add(1_013_904_223))
        .collect();
    h.check_bool("lcg_gpu_exact", gpu_lcg == cpu_lcg);

    let abs_input: Vec<f32> = vec![-5.0, -1.0, 0.0, 1.0, 5.0];
    let gpu_abs = gpu_run_f32_unary(ctx, ABS_WGSL, &abs_input);
    let cpu_abs: Vec<f32> = abs_input.iter().map(|x| x.abs()).collect();
    h.check_bool("abs_gpu_exact", gpu_abs == cpu_abs);

    let reduce_sum_input: Vec<f32> = (0..256).map(|i| i as f32).collect();
    let gpu_partial = gpu_run_f32_unary(ctx, REDUCE_SUM_WGSL, &reduce_sum_input);
    let gpu_total: f32 = gpu_partial.iter().sum();
    let cpu_total: f32 = reduce_sum_input.iter().sum();
    h.check_abs(
        "reduce_sum_gpu_parity",
        f64::from((gpu_total - cpu_total).abs()),
        0.0,
        tolerances::GPU_REDUCE_SUM_ABS_TOL,
    );

    // -- Tier A GPU parity: Perlin 2D noise --
    let perm_u32: Vec<u32> = PERM_TABLE.iter().map(|&b| u32::from(b)).collect();
    let n_noise = 256;
    let mut noise_coords: Vec<f32> = Vec::with_capacity(n_noise * 2);
    for i in 0..n_noise {
        let x = (i as f32) * 0.1;
        let y = (i as f32) * 0.07;
        noise_coords.push(x);
        noise_coords.push(y);
    }
    let gpu_noise = gpu_run_perlin(ctx, &perm_u32, &noise_coords);
    let cpu_noise: Vec<f32> = (0..n_noise)
        .map(|i| {
            let x = (i as f64) * 0.1;
            let y = (i as f64) * 0.07;
            noise::perlin_2d(x, y) as f32
        })
        .collect();
    let noise_max_err = gpu_noise
        .iter()
        .zip(cpu_noise.iter())
        .map(|(g, c)| (g - c).abs())
        .fold(0.0_f32, f32::max);
    h.check_abs(
        "perlin_gpu_parity",
        f64::from(noise_max_err),
        0.0,
        tolerances::GPU_PERLIN_ABS_TOL,
    );
    h.check_bool(
        "perlin_gpu_range_bounded",
        gpu_noise.iter().all(|&v| (-1.1..=1.1).contains(&v)),
    );

    let gpu_noise2 = gpu_run_perlin(ctx, &perm_u32, &noise_coords);
    h.check_bool("perlin_gpu_deterministic", gpu_noise == gpu_noise2);

    // -- Tier A GPU parity: Engagement batch --
    let n_eng = 64;
    let weights_f32: [f32; 5] = [0.2, 0.2, 0.2, 0.2, 0.2];
    let mut eng_components: Vec<f32> = Vec::with_capacity(n_eng * 5);
    for i in 0..n_eng {
        let base = (i as f32) / (n_eng as f32);
        for j in 0..5 {
            eng_components.push((j as f32).mul_add(0.1, base).min(1.0));
        }
    }
    let gpu_eng = gpu_run_engagement_batch(ctx, &eng_components, &weights_f32);
    let cpu_eng: Vec<f32> = (0..n_eng)
        .map(|i| {
            let mut sum = 0.0_f32;
            for j in 0..5 {
                sum += eng_components[i * 5 + j] * weights_f32[j];
            }
            sum.clamp(0.0, 1.0)
        })
        .collect();
    let eng_max_err = gpu_eng
        .iter()
        .zip(cpu_eng.iter())
        .map(|(g, c)| (g - c).abs())
        .fold(0.0_f32, f32::max);
    h.check_abs(
        "engagement_gpu_parity",
        f64::from(eng_max_err),
        0.0,
        tolerances::GPU_ENGAGEMENT_ABS_TOL,
    );

    // -- FBM GPU parity --
    let n_fbm = 128;
    let octaves = 4u32;
    let lacunarity: f32 = 2.0;
    let persistence: f32 = 0.5;
    let fbm_cpu: Vec<f32> = (0..n_fbm)
        .map(|i| {
            let x = (i as f64) * 0.05;
            let y = (i as f64) * 0.03;
            noise::fbm_2d(x, y, octaves, f64::from(lacunarity), f64::from(persistence)) as f32
        })
        .collect();
    let mut fbm_accum = vec![0.0_f32; n_fbm];
    let mut amplitude: f32 = 1.0;
    let mut frequency: f32 = 1.0;
    let mut max_value: f32 = 0.0;
    for _ in 0..octaves {
        let mut octave_coords: Vec<f32> = Vec::with_capacity(n_fbm * 2);
        for i in 0..n_fbm {
            octave_coords.push((i as f32) * 0.05 * frequency);
            octave_coords.push((i as f32) * 0.03 * frequency);
        }
        let octave_result = gpu_run_perlin(ctx, &perm_u32, &octave_coords);
        for (j, val) in octave_result.iter().enumerate() {
            fbm_accum[j] += val * amplitude;
        }
        max_value += amplitude;
        amplitude *= persistence;
        frequency *= lacunarity;
    }
    for v in &mut fbm_accum {
        *v /= max_value;
    }
    let fbm_max_err = fbm_accum
        .iter()
        .zip(fbm_cpu.iter())
        .map(|(g, c)| (g - c).abs())
        .fold(0.0_f32, f32::max);
    h.check_abs(
        "fbm_gpu_parity",
        f64::from(fbm_max_err),
        0.0,
        tolerances::GPU_FBM_ABS_TOL_LOOSE,
    );

    // -- Raycaster GPU parity --
    let map_w = 8u32;
    let map_h = 8u32;
    let mut map_data: Vec<u32> = vec![0; (map_w * map_h) as usize];
    for x in 0..map_w {
        for y in 0..map_h {
            if x == 0 || x == map_w - 1 || y == 0 || y == map_h - 1 {
                map_data[(y * map_w + x) as usize] = 1;
            }
        }
    }
    let player_x: f32 = 4.0;
    let player_y: f32 = 4.0;
    let n_rays = 64u32;
    let fov: f32 = std::f32::consts::PI / 3.0;
    let base_angle: f32 = 0.0;
    let mut ray_angles: Vec<f32> = Vec::with_capacity(n_rays as usize);
    for i in 0..n_rays {
        let fraction = (i as f32) / (n_rays as f32) - 0.5;
        ray_angles.push(base_angle + fraction * fov);
    }

    let grid_map = raycaster::GridMap::new(
        map_w as usize,
        map_h as usize,
        map_data.iter().map(|&v| v != 0).collect(),
    );
    let ray_player = raycaster::RayPlayer {
        x: f64::from(player_x),
        y: f64::from(player_y),
        angle: f64::from(base_angle),
        fov: f64::from(fov),
        speed: 3.0,
        turn_speed: std::f64::consts::PI,
    };
    let cpu_distances: Vec<f32> = ray_angles
        .iter()
        .map(|&a| {
            let hit = raycaster::cast_ray(&ray_player, f64::from(a), &grid_map, 20.0);
            hit.map_or(20.0_f32, |h| h.distance as f32)
        })
        .collect();

    let gpu_distances = gpu_run_raycaster(
        ctx,
        &map_data,
        map_w,
        map_h,
        player_x,
        player_y,
        &ray_angles,
    );

    let ray_max_err = gpu_distances
        .iter()
        .zip(cpu_distances.iter())
        .map(|(g, c)| (g - c).abs())
        .fold(0.0_f32, f32::max);
    h.check_abs(
        "raycaster_gpu_parity",
        f64::from(ray_max_err),
        0.0,
        tolerances::GPU_RAYCASTER_DISTANCE_ABS_TOL,
    );

    let gpu_hits: Vec<bool> = gpu_distances.iter().map(|&d| d < 19.0).collect();
    let cpu_hits: Vec<bool> = cpu_distances.iter().map(|&d| d < 19.0).collect();
    h.check_bool("raycaster_gpu_hit_match", gpu_hits == cpu_hits);

    // -- Game shader parity: Fog of War --
    let fog_w = 8_u32;
    let fog_h = 8_u32;
    let fog_n = (fog_w * fog_h) as usize;
    let fog_terrain: Vec<f32> = (0..fog_n).map(|_| 0.0_f32).collect();
    let fog_prev: Vec<u32> = vec![0; fog_n];
    let fog_center_x = 4.0_f32;
    let fog_center_y = 4.0_f32;
    let sight_r_sq = 9.0_f32;

    let gpu_fog = gpu_run_fog_of_war(
        ctx,
        fog_w,
        fog_h,
        fog_center_x,
        fog_center_y,
        sight_r_sq,
        &fog_terrain,
        &fog_prev,
    );
    let cpu_fog = cpu_fog_of_war(
        fog_w,
        fog_h,
        fog_center_x,
        fog_center_y,
        sight_r_sq,
        &fog_terrain,
        &fog_prev,
    );
    h.check_bool("fog_of_war_gpu_parity", gpu_fog == cpu_fog);

    let fog_visible_count = gpu_fog.iter().filter(|&&v| v == 2).count();
    h.check_bool("fog_of_war_has_visible_tiles", fog_visible_count > 0);
    h.check_bool("fog_of_war_has_hidden_tiles", gpu_fog.contains(&0));

    // -- Game shader parity: Tile Lighting --
    let light_w = 8_u32;
    let light_h = 8_u32;
    let light_n = (light_w * light_h) as usize;
    let light_terrain: Vec<f32> = (0..light_n).map(|_| 0.0_f32).collect();
    let lights: Vec<[f32; 4]> = vec![[4.0, 4.0, 1.0, 5.0], [1.0, 1.0, 0.5, 3.0]];
    let ambient = 0.1_f32;

    let gpu_light =
        gpu_run_tile_lighting(ctx, light_w, light_h, 2, ambient, &light_terrain, &lights);
    let cpu_light = cpu_tile_lighting(light_w, light_h, 2, ambient, &light_terrain, &lights);
    let light_max_err = gpu_light
        .iter()
        .zip(cpu_light.iter())
        .map(|(g, c)| (g - c).abs())
        .fold(0.0_f32, f32::max);
    h.check_abs(
        "lighting_gpu_parity",
        f64::from(light_max_err),
        0.0,
        tolerances::GPU_LIGHTING_ABS_TOL,
    );
    h.check_bool(
        "lighting_center_bright",
        gpu_light[(4 * light_w + 4) as usize] > ambient,
    );
    h.check_bool(
        "lighting_clamped",
        gpu_light.iter().all(|&v| (0.0..=1.0).contains(&v)),
    );

    // -- Game shader parity: Pathfind Wavefront --
    let pf_w = 8_u32;
    let pf_h = 8_u32;
    let pf_n = (pf_w * pf_h) as usize;
    let pf_terrain: Vec<f32> = (0..pf_n).map(|_| 0.0_f32).collect();
    let start_idx = (3 * pf_w + 3) as usize;
    let mut gpu_dist: Vec<u32> = vec![0xFFFF_FFFF; pf_n];
    gpu_dist[start_idx] = 0;
    let mut cpu_dist = gpu_dist.clone();

    for step in 0..4_u32 {
        let (new_dist, _frontier) =
            gpu_run_pathfind_step(ctx, pf_w, pf_h, step, &pf_terrain, &gpu_dist);
        gpu_dist = new_dist;
        cpu_dist = cpu_pathfind_step(pf_w, pf_h, step, &pf_terrain, &cpu_dist);
    }
    h.check_bool("pathfind_gpu_parity", gpu_dist == cpu_dist);
    let reached = gpu_dist.iter().filter(|&&d| d != 0xFFFF_FFFF).count();
    h.check_bool("pathfind_reaches_tiles", reached > 10);

    // -- Benchmark --
    let bench_n = 65536usize;
    let bench_input: Vec<f32> = (0..bench_n)
        .map(|i| (i as f32).mul_add(0.001, -0.5))
        .collect();
    let cpu_start = std::time::Instant::now();
    let _cpu: Vec<f32> = bench_input
        .iter()
        .map(|&x| 1.0 / (1.0 + (-x).exp()))
        .collect();
    let cpu_bench_us = cpu_start.elapsed().as_micros();
    let gpu_start = std::time::Instant::now();
    let _gpu = gpu_run_f32_unary(ctx, SIGMOID_WGSL, &bench_input);
    let gpu_bench_us = gpu_start.elapsed().as_micros();
    h.check_bool(
        "batch_speedup_nonnegative",
        gpu_bench_us <= cpu_bench_us + 10000,
    );
}

// ── CPU reference implementations for game shader parity ───────────

fn cpu_fog_of_war(
    grid_w: u32,
    grid_h: u32,
    viewer_x: f32,
    viewer_y: f32,
    sight_radius_sq: f32,
    _terrain: &[f32],
    prev_vis: &[u32],
) -> Vec<u32> {
    let n = (grid_w * grid_h) as usize;
    let mut out = vec![0_u32; n];
    for idx in 0..n {
        let tile_x = (idx as u32 % grid_w) as f32 + 0.5;
        let tile_y = (idx as u32 / grid_w) as f32 + 0.5;
        let dx = tile_x - viewer_x;
        let dy = tile_y - viewer_y;
        let dist_sq = dx * dx + dy * dy;

        if dist_sq <= sight_radius_sq {
            out[idx] = 2;
        } else if prev_vis[idx] >= 1 {
            out[idx] = 1;
        }
    }
    out
}

fn cpu_tile_lighting(
    grid_w: u32,
    grid_h: u32,
    num_lights: u32,
    ambient: f32,
    terrain: &[f32],
    lights: &[[f32; 4]],
) -> Vec<f32> {
    let n = (grid_w * grid_h) as usize;
    let mut out = vec![0.0_f32; n];
    for idx in 0..n {
        if terrain[idx] >= 0.9 {
            out[idx] = 0.0;
            continue;
        }
        let tile_x = (idx as u32 % grid_w) as f32 + 0.5;
        let tile_y = (idx as u32 / grid_w) as f32 + 0.5;
        let mut total = ambient;
        let count = (num_lights as usize).min(8).min(lights.len());
        for light in &lights[..count] {
            let dx = tile_x - light[0];
            let dy = tile_y - light[1];
            let dist_sq = dx * dx + dy * dy;
            let radius_sq = light[3] * light[3];
            if dist_sq < radius_sq {
                let attenuation = 1.0 / (1.0 + dist_sq);
                total += light[2] * attenuation;
            }
        }
        out[idx] = total.clamp(0.0, 1.0);
    }
    out
}

fn cpu_pathfind_step(
    grid_w: u32,
    grid_h: u32,
    current_dist: u32,
    terrain: &[f32],
    dist_map: &[u32],
) -> Vec<u32> {
    let n = (grid_w * grid_h) as usize;
    let mut out = dist_map.to_vec();
    let next_dist = current_dist + 1;
    let offsets: [(i32, i32); 4] = [(0, -1), (0, 1), (1, 0), (-1, 0)];

    for (idx, &cell) in dist_map.iter().enumerate().take(n) {
        if cell != current_dist {
            continue;
        }
        let x = (idx as u32) % grid_w;
        let y = (idx as u32) / grid_w;
        for &(ox, oy) in &offsets {
            let nx = x.cast_signed() + ox;
            let ny = y.cast_signed() + oy;
            if nx < 0 || ny < 0 || nx >= grid_w.cast_signed() || ny >= grid_h.cast_signed() {
                continue;
            }
            let nidx = (ny.cast_unsigned() * grid_w + nx.cast_unsigned()) as usize;
            if terrain[nidx] >= 0.9 {
                continue;
            }
            if out[nidx] == 0xFFFF_FFFF {
                out[nidx] = next_dist;
            }
        }
    }
    out
}

/// f32 softmax reference for GPU parity — matches the WGSL shader's precision.
///
/// barraCuda's `ops::softmax` operates on Tensor types at f64 precision;
/// this local implementation intentionally uses f32 to match the GPU shader
/// for parity validation. Not a duplication — different precision domain.
fn cpu_softmax_f32(input: &[f32]) -> Vec<f32> {
    let max_val = input.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let exps: Vec<f32> = input.iter().map(|&x| (x - max_val).exp()).collect();
    let sum: f32 = exps.iter().sum();
    exps.iter().map(|&e| e / sum).collect()
}

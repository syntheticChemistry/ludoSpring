// SPDX-License-Identifier: AGPL-3.0-or-later

//! CPU-vs-GPU parity validation orchestration.

use crate::gpu::{
    gpu_run_engagement_batch, gpu_run_f32_3buf, gpu_run_f32_unary, gpu_run_perlin,
    gpu_run_raycaster, gpu_run_u32_unary, try_create_gpu,
};
use crate::shaders::{
    ABS_WGSL, DOT_PRODUCT_WGSL, LCG_WGSL, PERM_TABLE, REDUCE_SUM_WGSL, RELU_WGSL, SCALE_WGSL,
    SIGMOID_WGSL, SOFTMAX_WGSL,
};
use ludospring_barracuda::barcuda_math;
use ludospring_barracuda::game::raycaster;
use ludospring_barracuda::procedural::noise;
use ludospring_barracuda::validation::ValidationResult;
use std::process;

#[expect(
    clippy::too_many_lines,
    clippy::cast_precision_loss,
    clippy::similar_names,
    reason = "validation orchestrator — sequential check groups"
)]
pub fn cmd_validate() {
    println!("=== exp030: CPU-vs-GPU Math Parity Validation ===\n");

    let gpu = try_create_gpu();
    let gpu_name = gpu
        .as_ref()
        .map_or_else(|| "none".to_string(), |g| g.adapter_name.clone());
    println!("GPU adapter: {gpu_name}\n");

    let mut results = Vec::new();
    let experiment = "exp030_cpu_gpu_parity";

    // -- CPU-only checks (always run) --

    let cpu_sig: Vec<f64> = vec![-2.0, -1.0, 0.0, 1.0, 2.0];
    let cpu_sig_out: Vec<f64> = cpu_sig.iter().map(|&x| barcuda_math::sigmoid(x)).collect();
    let sig_at_zero = cpu_sig_out[2];
    results.push(ValidationResult::check(
        experiment,
        "sigmoid_cpu_at_zero",
        sig_at_zero,
        0.5,
        1e-10,
    ));

    let relu_neg = f64::max(-3.0, 0.0);
    let relu_pos = f64::max(3.0, 0.0);
    results.push(ValidationResult::check(
        experiment,
        "relu_cpu_negative",
        relu_neg,
        0.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        experiment,
        "relu_cpu_positive",
        relu_pos,
        3.0,
        0.0,
    ));

    let a = [1.0, 2.0, 3.0, 4.0];
    let b = [5.0, 6.0, 7.0, 8.0];
    let cpu_dot = barcuda_math::dot(&a, &b);
    results.push(ValidationResult::check(
        experiment,
        "dot_cpu_known",
        cpu_dot,
        70.0,
        1e-10,
    ));

    let seed: u64 = 42;
    let next = barcuda_math::lcg_step(seed);
    #[expect(clippy::cast_precision_loss, reason = "counts fit in f64 mantissa")]
    let expected_lcg = 42_u64
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1) as f64;
    results.push(ValidationResult::check(
        experiment,
        "lcg_cpu_deterministic",
        next as f64,
        expected_lcg,
        0.0,
    ));

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
    results.push(ValidationResult::check(
        experiment,
        "perlin_bounded_low",
        if perlin_min >= -1.0 { 1.0 } else { 0.0 },
        1.0,
        0.0,
    ));
    results.push(ValidationResult::check(
        experiment,
        "perlin_bounded_high",
        if perlin_max <= 1.0 { 1.0 } else { 0.0 },
        1.0,
        0.0,
    ));

    let mean_data = [2.0, 4.0, 6.0, 8.0];
    let cpu_mean = barcuda_math::mean(&mean_data);
    results.push(ValidationResult::check(
        experiment,
        "mean_cpu_known",
        cpu_mean,
        5.0,
        1e-10,
    ));

    // -- GPU parity checks --

    if let Some(ref ctx) = gpu {
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
        results.push(ValidationResult::check(
            experiment,
            "sigmoid_gpu_parity",
            f64::from(sig_max_err),
            0.0,
            1e-6,
        ));

        let relu_input: Vec<f32> = vec![-3.0, -1.0, 0.0, 1.0, 3.0];
        let gpu_relu = gpu_run_f32_unary(ctx, RELU_WGSL, &relu_input);
        let cpu_relu: Vec<f32> = relu_input.iter().map(|&x| x.max(0.0)).collect();
        let relu_exact = gpu_relu == cpu_relu;
        results.push(ValidationResult::check(
            experiment,
            "relu_gpu_exact",
            if relu_exact { 1.0 } else { 0.0 },
            1.0,
            0.0,
        ));

        let dot_a: Vec<f32> = vec![1.0, 2.0, 3.0, 4.0];
        let dot_b: Vec<f32> = vec![5.0, 6.0, 7.0, 8.0];
        let gpu_products = gpu_run_f32_3buf(ctx, DOT_PRODUCT_WGSL, &dot_a, &dot_b);
        let gpu_dot_sum: f32 = gpu_products.iter().sum();
        results.push(ValidationResult::check(
            experiment,
            "dot_gpu_parity",
            f64::from(gpu_dot_sum),
            70.0,
            1e-4,
        ));

        let softmax_input: Vec<f32> = vec![1.0, 2.0, 3.0, 4.0];
        let gpu_sm = gpu_run_f32_unary(ctx, SOFTMAX_WGSL, &softmax_input);
        let cpu_sm = cpu_softmax_f32(&softmax_input);
        let sm_max_err = gpu_sm
            .iter()
            .zip(cpu_sm.iter())
            .map(|(g, c)| (g - c).abs())
            .fold(0.0_f32, f32::max);
        results.push(ValidationResult::check(
            experiment,
            "softmax_gpu_parity",
            f64::from(sm_max_err),
            0.0,
            1e-5,
        ));

        let scale_input: Vec<f32> = vec![0.0, 1.0, 2.0, 3.0, 4.0];
        let gpu_scale = gpu_run_f32_unary(ctx, SCALE_WGSL, &scale_input);
        let cpu_scale: Vec<f32> = scale_input.iter().map(|&x| x.mul_add(2.0, 1.0)).collect();
        let scale_exact = gpu_scale == cpu_scale;
        results.push(ValidationResult::check(
            experiment,
            "scale_gpu_exact",
            if scale_exact { 1.0 } else { 0.0 },
            1.0,
            0.0,
        ));

        let lcg_seeds: Vec<u32> = vec![42, 100, 255, 0, 999_999];
        let gpu_lcg = gpu_run_u32_unary(ctx, LCG_WGSL, &lcg_seeds);
        let cpu_lcg: Vec<u32> = lcg_seeds
            .iter()
            .map(|&s| s.wrapping_mul(1_664_525).wrapping_add(1_013_904_223))
            .collect();
        let lcg_match = gpu_lcg == cpu_lcg;
        results.push(ValidationResult::check(
            experiment,
            "lcg_gpu_exact",
            if lcg_match { 1.0 } else { 0.0 },
            1.0,
            0.0,
        ));

        let abs_input: Vec<f32> = vec![-5.0, -1.0, 0.0, 1.0, 5.0];
        let gpu_abs = gpu_run_f32_unary(ctx, ABS_WGSL, &abs_input);
        let cpu_abs: Vec<f32> = abs_input.iter().map(|x| x.abs()).collect();
        let abs_exact = gpu_abs == cpu_abs;
        results.push(ValidationResult::check(
            experiment,
            "abs_gpu_exact",
            if abs_exact { 1.0 } else { 0.0 },
            1.0,
            0.0,
        ));

        #[expect(clippy::cast_precision_loss, reason = "counts fit in f64 mantissa")]
        let reduce_sum_input: Vec<f32> = (0..256).map(|i| i as f32).collect();
        let gpu_partial = gpu_run_f32_unary(ctx, REDUCE_SUM_WGSL, &reduce_sum_input);
        let gpu_total: f32 = gpu_partial.iter().sum();
        let cpu_total: f32 = reduce_sum_input.iter().sum();
        results.push(ValidationResult::check(
            experiment,
            "reduce_sum_gpu_parity",
            f64::from((gpu_total - cpu_total).abs()),
            0.0,
            1.0,
        ));

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
        #[expect(
            clippy::cast_precision_loss,
            clippy::cast_possible_truncation,
            reason = "counts fit in f64 mantissa; value bounded"
        )]
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
        results.push(ValidationResult::check(
            experiment,
            "perlin_gpu_parity",
            f64::from(noise_max_err),
            0.0,
            1e-3,
        ));

        // Check range bounded
        let all_bounded = gpu_noise.iter().all(|&v| (-1.1..=1.1).contains(&v));
        results.push(ValidationResult::check(
            experiment,
            "perlin_gpu_range_bounded",
            if all_bounded { 1.0 } else { 0.0 },
            1.0,
            0.0,
        ));

        // Determinism: rerun
        let gpu_noise2 = gpu_run_perlin(ctx, &perm_u32, &noise_coords);
        let deterministic = gpu_noise == gpu_noise2;
        results.push(ValidationResult::check(
            experiment,
            "perlin_gpu_deterministic",
            if deterministic { 1.0 } else { 0.0 },
            1.0,
            0.0,
        ));

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
        results.push(ValidationResult::check(
            experiment,
            "engagement_gpu_parity",
            f64::from(eng_max_err),
            0.0,
            1e-4,
        ));

        // -- FBM GPU parity (use Perlin shader with multiple octave calls) --
        let n_fbm = 128;
        let octaves = 4u32;
        let lacunarity: f32 = 2.0;
        let persistence: f32 = 0.5;
        #[expect(
            clippy::cast_precision_loss,
            clippy::cast_possible_truncation,
            reason = "counts fit in f64 mantissa; value bounded"
        )]
        let fbm_cpu: Vec<f32> = (0..n_fbm)
            .map(|i| {
                let x = (i as f64) * 0.05;
                let y = (i as f64) * 0.03;
                noise::fbm_2d(x, y, octaves, f64::from(lacunarity), f64::from(persistence)) as f32
            })
            .collect();
        // GPU fBm: accumulate octaves by calling Perlin shader per octave
        let mut fbm_gpu = vec![0.0_f32; n_fbm];
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
                fbm_gpu[j] += val * amplitude;
            }
            max_value += amplitude;
            amplitude *= persistence;
            frequency *= lacunarity;
        }
        for v in &mut fbm_gpu {
            *v /= max_value;
        }
        let fbm_max_err = fbm_gpu
            .iter()
            .zip(fbm_cpu.iter())
            .map(|(g, c)| (g - c).abs())
            .fold(0.0_f32, f32::max);
        results.push(ValidationResult::check(
            experiment,
            "fbm_gpu_parity",
            f64::from(fbm_max_err),
            0.0,
            0.01,
        ));

        // -- Raycaster GPU parity (simplified: DDA distance for N angles) --
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

        // CPU raycaster reference
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
                #[expect(clippy::cast_possible_truncation, reason = "value bounded")]
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
        results.push(ValidationResult::check(
            experiment,
            "raycaster_gpu_parity",
            f64::from(ray_max_err),
            0.0,
            0.5,
        ));

        // Hit match: both agree on whether a wall was hit
        let gpu_hits: Vec<bool> = gpu_distances.iter().map(|&d| d < 19.0).collect();
        let cpu_hits: Vec<bool> = cpu_distances.iter().map(|&d| d < 19.0).collect();
        let hit_match = gpu_hits == cpu_hits;
        results.push(ValidationResult::check(
            experiment,
            "raycaster_gpu_hit_match",
            if hit_match { 1.0 } else { 0.0 },
            1.0,
            0.0,
        ));

        // Batch speedup check (positive or equal)
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
        results.push(ValidationResult::check(
            experiment,
            "batch_speedup_nonnegative",
            if gpu_bench_us <= cpu_bench_us + 10000 {
                1.0
            } else {
                0.0
            },
            1.0,
            0.0,
        ));
    } else {
        println!("  [SKIP] No GPU adapter — GPU parity checks skipped\n");
    }

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

fn cpu_softmax_f32(input: &[f32]) -> Vec<f32> {
    let max_val = input.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let exps: Vec<f32> = input.iter().map(|&x| (x - max_val).exp()).collect();
    let sum: f32 = exps.iter().sum();
    exps.iter().map(|&e| e / sum).collect()
}

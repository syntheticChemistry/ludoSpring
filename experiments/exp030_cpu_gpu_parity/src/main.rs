// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
//! exp030 — CPU-vs-GPU math parity validation.
//!
//! Validates that pure Rust CPU math (barraCuda) matches GPU shader output.
//! Subcommands: validate, probe, bench

mod gpu;
mod shaders;
mod validate;

use std::process;

fn main() {
    let arg = std::env::args().nth(1).unwrap_or_default();
    match arg.as_str() {
        "probe" => cmd_probe(),
        "bench" => cmd_bench(),
        "validate" | "" => validate::cmd_validate(),
        other => {
            eprintln!("Unknown command: {other}");
            eprintln!("Usage: exp030_cpu_gpu_parity [validate|probe|bench]");
            process::exit(1);
        }
    }
}

fn cmd_probe() {
    println!("=== exp030: GPU Adapter Probe ===\n");

    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::all(),
        ..Default::default()
    });

    let adapters: Vec<wgpu::Adapter> =
        pollster::block_on(instance.enumerate_adapters(wgpu::Backends::all()));
    if adapters.is_empty() {
        println!("  No adapters found.");
        return;
    }

    for (i, adapter) in adapters.iter().enumerate() {
        let info = adapter.get_info();
        println!("  Adapter {i}:");
        println!("    Name:    {}", info.name);
        println!("    Vendor:  0x{:04x}", info.vendor);
        println!("    Device:  0x{:04x}", info.device);
        println!("    Type:    {:?}", info.device_type);
        println!("    Backend: {:?}", info.backend);
        println!();
    }
}

fn cmd_bench() {
    println!("=== exp030: CPU-vs-GPU Benchmark ===\n");

    let gpu = gpu::try_create_gpu();
    let gpu_name = gpu
        .as_ref()
        .map_or_else(|| "none".to_string(), |g| g.adapter_name.clone());
    println!("GPU: {gpu_name}\n");

    let sizes = [64, 256, 1024, 4096, 16384, 65536];

    println!(
        "{:>8} {:>12} {:>12} {:>8}",
        "N", "CPU (us)", "GPU (us)", "Speedup"
    );
    println!("{}", "-".repeat(48));

    for &n in &sizes {
        #[expect(clippy::cast_precision_loss, reason = "counts fit in f64 mantissa")]
        let input: Vec<f32> = (0..n).map(|i| (i as f32).mul_add(0.001, -0.5)).collect();

        let cpu_start = std::time::Instant::now();
        let _cpu_out: Vec<f32> = input.iter().map(|&x| 1.0 / (1.0 + (-x).exp())).collect();
        let cpu_us = cpu_start.elapsed().as_micros();

        if let Some(ref ctx) = gpu {
            let gpu_start = std::time::Instant::now();
            let _gpu_out = gpu::gpu_run_f32_unary(ctx, shaders::SIGMOID_WGSL, &input);
            let gpu_us = gpu_start.elapsed().as_micros();

            #[expect(clippy::cast_precision_loss, reason = "counts fit in f64 mantissa")]
            let speedup = if gpu_us > 0 {
                cpu_us as f64 / gpu_us as f64
            } else {
                0.0
            };
            println!("{n:>8} {cpu_us:>12} {gpu_us:>12} {speedup:>8.2}x");
        } else {
            println!("{n:>8} {cpu_us:>12} {:>12} {:>8}", "N/A", "N/A");
        }
    }
}

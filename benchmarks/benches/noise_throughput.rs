// SPDX-License-Identifier: AGPL-3.0-or-later
//! BM-002: Noise field generation throughput.

#![allow(missing_docs)]

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use ludospring_benchmarks::noise;

fn perlin_2d_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("perlin_2d");
    for &size in &[64, 128, 256, 512, 1024] {
        group.bench_function(format!("{size}x{size}"), |b| {
            b.iter(|| noise::perlin_2d_field(black_box(size), 0.1));
        });
    }
    group.finish();
}

fn perlin_3d_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("perlin_3d");
    for &size in &[32, 64, 128] {
        group.bench_function(format!("{size}x{size}x{size}"), |b| {
            b.iter(|| noise::perlin_3d_field(black_box(size), size, 0.1));
        });
    }
    group.finish();
}

fn fbm_2d_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("fbm_2d");
    for &octaves in &[1, 4, 8] {
        group.bench_function(format!("256x256_oct{octaves}"), |b| {
            b.iter(|| noise::fbm_2d_field(256, 0.05, black_box(octaves)));
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    perlin_2d_benchmarks,
    perlin_3d_benchmarks,
    fbm_2d_benchmarks
);
criterion_main!(benches);

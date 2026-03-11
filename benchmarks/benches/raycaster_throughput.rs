// SPDX-License-Identifier: AGPL-3.0-or-later
//! BM-003: Raycaster screen-cast throughput.

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use ludospring_benchmarks::raycaster;

fn arena_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("raycaster_arena");

    for &map_size in &[32, 64, 128] {
        let map = raycaster::arena_map(map_size);
        group.bench_function(format!("map{map_size}_320col"), |b| {
            b.iter(|| raycaster::cast_full_screen(black_box(&map), 320));
        });
        group.bench_function(format!("map{map_size}_640col"), |b| {
            b.iter(|| raycaster::cast_full_screen(black_box(&map), 640));
        });
    }
    group.finish();
}

fn maze_benchmarks(c: &mut Criterion) {
    let map = raycaster::maze_map();
    c.bench_function("raycaster_maze_320col", |b| {
        b.iter(|| raycaster::cast_full_screen(black_box(&map), 320));
    });
}

criterion_group!(benches, arena_benchmarks, maze_benchmarks);
criterion_main!(benches);

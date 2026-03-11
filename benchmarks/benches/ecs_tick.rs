// SPDX-License-Identifier: AGPL-3.0-or-later
//! BM-001: Entity-component tick throughput.

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use ludospring_benchmarks::ecs;

fn game_logic_tick(c: &mut Criterion) {
    let mut group = c.benchmark_group("game_logic_tick");

    for &count in &[100, 1_000, 10_000] {
        let mut entities = ecs::spawn_entities(count);
        group.bench_function(format!("{count}_entities"), |b| {
            b.iter(|| ecs::tick_game_logic(black_box(&mut entities), 1.0 / 60.0));
        });
    }
    group.finish();
}

fn metrics_tick(c: &mut Criterion) {
    c.bench_function("metrics_engagement_snapshot", |b| {
        b.iter(|| ecs::tick_metrics(black_box(500_u64), black_box(300.0)));
    });
}

criterion_group!(benches, game_logic_tick, metrics_tick);
criterion_main!(benches);

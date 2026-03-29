// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]
//! exp037 — Game engine tick budget validation.
//!
//! From `GAME_ENGINE_NICHE_SPECIFICATION.md`:
//!   "Target: 16.67ms per tick (60 Hz)"
//!   "`game_logic`: 3ms, physics: 4ms, scene: 2ms, render: 6ms,
//!    audio: 2ms, metrics: 1ms, input: 1ms"
//!
//! Validates:
//!   - 10K entities ticked at 60 Hz within budget
//!   - `game_logic` node stays within 3ms allocation
//!   - metrics node stays within 1ms allocation
//!   - Combined `game_logic` + metrics under 4ms
//!   - Scales: 1K, 10K, 50K, 100K entities

use std::process;
use std::time::Instant;

use ludospring_barracuda::validation::{BaselineProvenance, ValidationHarness};
use ludospring_benchmarks::ecs::{spawn_entities, tick_game_logic, tick_metrics};

const PROVENANCE: BaselineProvenance = BaselineProvenance {
    script: "N/A (analytical — GAME_ENGINE_NICHE_SPECIFICATION.md)",
    commit: "4b683e3e",
    date: "2026-03-15",
    command: "N/A (tick budget benchmark)",
};

fn main() {
    let arg = std::env::args().nth(1).unwrap_or_default();
    match arg.as_str() {
        "bench" => cmd_bench(),
        "validate" | "" => cmd_validate(),
        other => {
            eprintln!("Unknown command: {other}");
            eprintln!("Usage: exp037_tick_budget [validate|bench]");
            process::exit(1);
        }
    }
}

fn cmd_validate() {
    let mut h = ValidationHarness::new("exp037_tick_budget");
    h.print_provenance(&[&PROVENANCE]);

    let dt = 1.0 / 60.0;

    // 1. game_logic 10K entities within 3ms
    let mut entities = spawn_entities(10_000);
    let t = Instant::now();
    let result = tick_game_logic(&mut entities, dt);
    let logic_us = t.elapsed().as_micros();
    h.check_bool("game_logic_10k_under_3ms", logic_us < 3000);

    // 2. All entities processed (flow distribution sums to entity count)
    let total_flow: u32 = result.flow_distribution.iter().sum();
    h.check_abs(
        "all_entities_processed",
        f64::from(total_flow),
        10_000.0,
        0.0,
    );

    // 3. Flow distribution has entries in flow state (challenge ≈ skill)
    let in_flow = result.flow_distribution[2]; // Flow index
    h.check_bool("some_entities_in_flow", in_flow > 0);

    // 4. metrics node within 1ms
    let t = Instant::now();
    let engagement = tick_metrics(500, 300.0);
    let metrics_us = t.elapsed().as_micros();
    h.check_bool("metrics_under_1ms", metrics_us < 1000);

    // 5. Engagement in valid range [0, 1]
    h.check_bool("engagement_in_range", (0.0..=1.0).contains(&engagement));

    // 6. Combined game_logic + metrics under 4ms (their combined budget)
    let combined_us = logic_us + metrics_us;
    h.check_bool("combined_under_4ms", combined_us < 4000);

    // 7. 1K entities within budget (easy)
    let mut e1k = spawn_entities(1_000);
    let t = Instant::now();
    tick_game_logic(&mut e1k, dt);
    let us_1k = t.elapsed().as_micros();
    h.check_bool("game_logic_1k_under_1ms", us_1k < 1000);

    // 8. 50K entities (stress — may exceed game_logic budget, but should be under total)
    let mut e50k = spawn_entities(50_000);
    let t = Instant::now();
    tick_game_logic(&mut e50k, dt);
    let us_50k = t.elapsed().as_micros();
    h.check_bool("game_logic_50k_under_16ms", us_50k < 16_670);

    // 9. Difficulty adjustment is bounded [-1, 1]
    let adj_bounded = result.mean_adjustment.abs() <= 1.0;
    h.check_bool("difficulty_adjustment_bounded", adj_bounded);

    // 10. 60 consecutive ticks (1 second) of 10K entities under 1 second
    let mut entities = spawn_entities(10_000);
    let t = Instant::now();
    for _ in 0..60 {
        tick_game_logic(&mut entities, dt);
    }
    let second_us = t.elapsed().as_micros();
    h.check_bool("60_ticks_10k_under_1s", second_us < 1_000_000);

    h.finish();
}

#[expect(
    clippy::cast_precision_loss,
    reason = "validation counts fit in f64 mantissa"
)]
fn cmd_bench() {
    println!("=== exp037: Tick Budget Benchmark ===\n");

    let dt = 1.0 / 60.0;
    let counts = [100, 1_000, 5_000, 10_000, 25_000, 50_000, 100_000];

    println!("game_logic tick (single tick, DDA + flow eval):");
    println!(
        "{:>8} {:>12} {:>10} {:>10}",
        "Entities", "Time (us)", "us/entity", "Est FPS"
    );
    for &n in &counts {
        let mut entities = spawn_entities(n);
        let t = Instant::now();
        tick_game_logic(&mut entities, dt);
        let us = t.elapsed().as_micros();
        #[expect(clippy::cast_precision_loss, reason = "counts fit in f64 mantissa")]
        let per_entity = us as f64 / n as f64;
        let fps = if us > 0 { 1_000_000.0 / us as f64 } else { 0.0 };
        println!("{n:>8} {us:>12} {per_entity:>10.3} {fps:>10.0}");
    }

    println!("\nSustained 60-tick (1 second simulation):");
    println!(
        "{:>8} {:>12} {:>10} {:>10}",
        "Entities", "Total (us)", "Avg/tick", "Headroom"
    );
    for &n in &[1_000, 5_000, 10_000, 25_000] {
        let mut entities = spawn_entities(n);
        let t = Instant::now();
        for _ in 0..60 {
            tick_game_logic(&mut entities, dt);
        }
        let total_us = t.elapsed().as_micros();
        let avg = total_us / 60;
        let headroom_pct = if avg < 16_670 {
            (16_670 - avg) as f64 / 16_670.0 * 100.0
        } else {
            -(avg as f64 - 16_670.0) / 16_670.0 * 100.0
        };
        println!("{n:>8} {total_us:>12} {avg:>10} {headroom_pct:>9.1}%");
    }
}

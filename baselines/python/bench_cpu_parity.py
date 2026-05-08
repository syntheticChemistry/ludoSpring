#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""
CPU Performance Parity Baseline — ludoSpring

Measures Python execution time for core algorithms so Rust/barraCuda Criterion
benchmarks can report a speedup ratio. This is NOT a correctness check (see
run_all_baselines.py for that) — it measures wall-clock time for comparison.

Provenance:
  Script: baselines/python/bench_cpu_parity.py
  Created: 2026-05-08
  Command: python3 baselines/python/bench_cpu_parity.py
  Python: 3.10+
  Dependencies: None (stdlib only — matches barraCuda zero-dep philosophy)

Usage:
  python3 bench_cpu_parity.py [--json] [--iterations N]

Output (JSON mode):
  {"benchmarks": [...], "_provenance": {...}}

Each benchmark reports:
  - name: algorithm identifier (matches Criterion bench name)
  - iterations: number of runs
  - total_ms: total wall-clock time
  - per_call_us: microseconds per call (median of iterations)
"""

import argparse
import json
import math
import platform
import statistics
import sys
import time
from datetime import datetime, timezone


def perlin_perm_table():
    """Deterministic permutation table (matches Rust ludospring_barracuda::procedural::noise)."""
    perm = list(range(256))
    seed = 42
    for i in range(255, 0, -1):
        seed = (seed * 6364136223846793005 + 1442695040888963407) & 0xFFFFFFFFFFFFFFFF
        j = (seed >> 33) % (i + 1)
        perm[i], perm[j] = perm[j], perm[i]
    return perm * 2


def fade(t):
    return t * t * t * (t * (t * 6.0 - 15.0) + 10.0)


def lerp(a, b, t):
    return a + t * (b - a)


def grad2d(h, x, y):
    h = h & 3
    if h == 0:
        return x + y
    elif h == 1:
        return -x + y
    elif h == 2:
        return x - y
    else:
        return -x - y


PERM = perlin_perm_table()


def perlin_2d(x, y):
    xi = int(math.floor(x)) & 255
    yi = int(math.floor(y)) & 255
    xf = x - math.floor(x)
    yf = y - math.floor(y)
    u = fade(xf)
    v = fade(yf)
    aa = PERM[PERM[xi] + yi]
    ab = PERM[PERM[xi] + yi + 1]
    ba = PERM[PERM[xi + 1] + yi]
    bb = PERM[PERM[xi + 1] + yi + 1]
    x1 = lerp(grad2d(aa, xf, yf), grad2d(ba, xf - 1, yf), u)
    x2 = lerp(grad2d(ab, xf, yf - 1), grad2d(bb, xf - 1, yf - 1), u)
    return lerp(x1, x2, v)


def fbm_2d(x, y, octaves=6, lacunarity=2.0, gain=0.5):
    total = 0.0
    amplitude = 1.0
    frequency = 1.0
    for _ in range(octaves):
        total += amplitude * perlin_2d(x * frequency, y * frequency)
        frequency *= lacunarity
        amplitude *= gain
    return total


def bench_perlin_field(size=256, iterations=10):
    """Generate a size×size Perlin noise field."""
    times = []
    for _ in range(iterations):
        start = time.perf_counter_ns()
        for row in range(size):
            for col in range(size):
                perlin_2d(col * 0.05, row * 0.05)
        elapsed = time.perf_counter_ns() - start
        times.append(elapsed / 1_000)
    return times


def bench_fbm_field(size=64, iterations=10):
    """Generate a size×size fBm noise field (6 octaves)."""
    times = []
    for _ in range(iterations):
        start = time.perf_counter_ns()
        for row in range(size):
            for col in range(size):
                fbm_2d(col * 0.05, row * 0.05)
        elapsed = time.perf_counter_ns() - start
        times.append(elapsed / 1_000)
    return times


def dda_raycast(pos_x, pos_y, dir_x, dir_y, grid_w, grid_h, max_steps=200):
    """Simple DDA raycaster (matches Rust game::raycaster::cast_ray)."""
    map_x = int(pos_x)
    map_y = int(pos_y)
    delta_x = abs(1.0 / dir_x) if dir_x != 0 else 1e30
    delta_y = abs(1.0 / dir_y) if dir_y != 0 else 1e30
    if dir_x < 0:
        step_x = -1
        side_x = (pos_x - map_x) * delta_x
    else:
        step_x = 1
        side_x = (map_x + 1.0 - pos_x) * delta_x
    if dir_y < 0:
        step_y = -1
        side_y = (pos_y - map_y) * delta_y
    else:
        step_y = 1
        side_y = (map_y + 1.0 - pos_y) * delta_y

    for _ in range(max_steps):
        if side_x < side_y:
            side_x += delta_x
            map_x += step_x
        else:
            side_y += delta_y
            map_y += step_y
        if map_x < 0 or map_x >= grid_w or map_y < 0 or map_y >= grid_h:
            break
    return map_x, map_y


def bench_raycaster(columns=320, grid_size=64, iterations=10):
    """Cast columns rays across a grid (matches Criterion raycaster bench)."""
    times = []
    for _ in range(iterations):
        start = time.perf_counter_ns()
        for col in range(columns):
            angle = -0.5 + col / columns
            dir_x = math.cos(angle)
            dir_y = math.sin(angle)
            dda_raycast(32.0, 32.0, dir_x, dir_y, grid_size, grid_size)
        elapsed = time.perf_counter_ns() - start
        times.append(elapsed / 1_000)
    return times


def bench_fitts_hick(iterations=10000):
    """Fitts + Hick law computation (matches Criterion ecs_tick overhead)."""
    times = []
    a, b = 50.0, 150.0
    for _ in range(iterations):
        start = time.perf_counter_ns()
        for d in range(1, 101):
            for w in range(1, 11):
                _ = a + b * math.log2(d / w + 1)
                _ = 200.0 * math.log2(w + 1)
        elapsed = time.perf_counter_ns() - start
        times.append(elapsed / 1_000)
    return times


def run_all(iterations):
    benchmarks = []

    print("Running perlin_2d 256×256...", file=sys.stderr)
    times = bench_perlin_field(256, iterations)
    benchmarks.append({
        "name": "perlin_2d_256x256",
        "iterations": iterations,
        "total_ms": sum(times) / 1000,
        "per_call_us": statistics.median(times),
        "unit": "field_generation",
    })

    print("Running fbm_2d 64×64...", file=sys.stderr)
    times = bench_fbm_field(64, iterations)
    benchmarks.append({
        "name": "fbm_2d_64x64",
        "iterations": iterations,
        "total_ms": sum(times) / 1000,
        "per_call_us": statistics.median(times),
        "unit": "field_generation",
    })

    print("Running raycaster 320 columns...", file=sys.stderr)
    times = bench_raycaster(320, 64, iterations)
    benchmarks.append({
        "name": "raycaster_320col_64grid",
        "iterations": iterations,
        "total_ms": sum(times) / 1000,
        "per_call_us": statistics.median(times),
        "unit": "screen_cast",
    })

    print("Running fitts_hick 1000×10...", file=sys.stderr)
    times = bench_fitts_hick(iterations)
    benchmarks.append({
        "name": "fitts_hick_1000x10",
        "iterations": iterations,
        "total_ms": sum(times) / 1000,
        "per_call_us": statistics.median(times),
        "unit": "law_batch",
    })

    return benchmarks


def main():
    parser = argparse.ArgumentParser(description="CPU Performance Parity Baseline")
    parser.add_argument("--json", action="store_true", help="Output JSON")
    parser.add_argument("--iterations", type=int, default=10, help="Iterations per bench")
    args = parser.parse_args()

    benchmarks = run_all(args.iterations)

    provenance = {
        "script": "baselines/python/bench_cpu_parity.py",
        "date": datetime.now(timezone.utc).isoformat(),
        "python_version": platform.python_version(),
        "platform": platform.platform(),
        "iterations": args.iterations,
        "purpose": "CPU performance baseline for Rust/barraCuda speedup ratio",
    }

    if args.json:
        print(json.dumps({"benchmarks": benchmarks, "_provenance": provenance}, indent=2))
    else:
        print(f"\n{'='*60}")
        print("ludoSpring CPU Performance Parity Baseline")
        print(f"{'='*60}")
        print(f"Python {provenance['python_version']} on {provenance['platform']}")
        print(f"Iterations: {args.iterations}")
        print(f"{'='*60}\n")
        for b in benchmarks:
            print(f"  {b['name']:30s}  {b['per_call_us']:>12.1f} µs/call  ({b['total_ms']:.1f} ms total)")
        print(f"\n{'='*60}")
        print("Compare with: cargo bench (Criterion output)")
        print(f"{'='*60}")


if __name__ == "__main__":
    main()

# SPDX-License-Identifier: AGPL-3.0-or-later
"""
BSP tree partitioning — Python baseline for ludoSpring.

Reference implementation of Binary Space Partitioning for Doom-style
level generation. Uses the same LCG as barraCuda for determinism.

References:
  - Fuchs, Kedem, Naylor (1980). "On Visible Surface Generation."
  - Carmack (1993). Doom engine BSP builder.

Provenance:
  Script: baselines/python/bsp_partition.py
  Date: 2026-03-11 (created); see combined_baselines.json _provenance for latest run
  Python: CPython 3.10+ (stdlib only)
  Command: python3 baselines/python/bsp_partition.py
"""

import json
import sys


# LCG matching barraCuda::rng::lcg_step
LCG_A = 6364136223846793005
LCG_C = 1442695040888963407
LCG_MOD = 2**64


def lcg_step(state):
    return (state * LCG_A + LCG_C) % LCG_MOD


def state_to_f64(state):
    return (state >> 11) / float(1 << 53)


def generate_bsp(x, y, w, h, min_size, seed, depth=0):
    """Recursively split a rectangle into BSP leaves."""
    horizontal = depth % 2 == 0
    span = h if horizontal else w

    if span < min_size * 2:
        return [{"x": x, "y": y, "w": w, "h": h}]

    next_seed = lcg_step(seed)
    ratio = state_to_f64(next_seed)
    split_ratio = 0.3 + ratio * 0.4

    if horizontal:
        split_pos = y + span * split_ratio
        top_h = split_pos - y
        bot_h = (y + h) - split_pos
        left = generate_bsp(x, y, w, top_h, min_size, next_seed, depth + 1)
        right = generate_bsp(x, split_pos, w, bot_h, min_size,
                             (next_seed + 1) % LCG_MOD, depth + 1)
    else:
        split_pos = x + span * split_ratio
        left_w = split_pos - x
        right_w = (x + w) - split_pos
        left = generate_bsp(x, y, left_w, h, min_size, next_seed, depth + 1)
        right = generate_bsp(split_pos, y, right_w, h, min_size,
                             (next_seed + 1) % LCG_MOD, depth + 1)

    return left + right


def main():
    results = {}

    # Standard test case
    leaves = generate_bsp(0, 0, 100, 100, 15, 42)
    results["leaf_count"] = len(leaves)
    results["total_area"] = sum(l["w"] * l["h"] for l in leaves)

    # Too-small case
    small = generate_bsp(0, 0, 5, 5, 10, 42)
    results["small_leaf_count"] = len(small)

    # Area conservation check
    offset = generate_bsp(10, 20, 80, 60, 12, 99)
    results["offset_area"] = sum(l["w"] * l["h"] for l in offset)
    results["offset_expected"] = 80 * 60

    json.dump(results, sys.stdout, indent=2)
    print()

    checks = [
        ("area_conservation", abs(results["total_area"] - 10000) < 1e-6, True),
        ("small_single_leaf", results["small_leaf_count"], 1),
        ("offset_area", abs(results["offset_area"] - 4800) < 1e-6, True),
    ]

    all_pass = True
    for name, actual, expected in checks:
        ok = actual == expected
        status = "PASS" if ok else "FAIL"
        print(f"{name}: {status} (actual={actual}, expected={expected})",
              file=sys.stderr)
        if not ok:
            all_pass = False

    if not all_pass:
        sys.exit(1)


if __name__ == "__main__":
    main()

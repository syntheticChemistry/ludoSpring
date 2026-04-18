# SPDX-License-Identifier: AGPL-3.0-or-later
"""
Interaction law reference implementations — Python baseline for ludoSpring.

Fitts's law, Hick's law, and steering law with the same parameters as the
Rust implementation. These produce the ground truth for validation.

References:
  - Fitts, P.M. (1954). "The information capacity of the human motor system."
  - MacKenzie, I.S. (1992). "Fitts' law as a research and design tool in HCI."
  - Hick, W.E. (1952). "On the rate of gain of information."
  - Hyman, R. (1953). "Stimulus information as a determinant of reaction time."
  - Accot, J. & Zhai, S. (1997). "Beyond Fitts' law: models for trajectory-based HCI tasks."

Provenance:
  Script: baselines/python/interaction_laws.py
  Date: 2026-03-11 (created); see combined_baselines.json _provenance for latest run
  Python: CPython 3.10+ (stdlib only)
  Command: python3 baselines/python/interaction_laws.py
"""

import json
import math
import sys

# Constants matching tolerances/mod.rs
FITTS_A_MOUSE_MS = 50.0
FITTS_B_MOUSE_MS = 150.0
HICK_A_MS = 200.0
HICK_B_MS = 150.0


def fitts_movement_time(distance: float, width: float,
                        a: float, b: float) -> float:
    """Shannon formulation: MT = a + b * log2(2D/W + 1)"""
    return a + b * math.log2(2.0 * distance / width + 1.0)


def fitts_index_of_difficulty(distance: float, width: float) -> float:
    """ID = log2(2D/W + 1)"""
    return math.log2(2.0 * distance / width + 1.0)


def hick_reaction_time(n_choices: int, a: float, b: float) -> float:
    """RT = a + b * log2(N + 1)"""
    return a + b * math.log2(n_choices + 1)


def steering_time(distance: float, width: float,
                  a: float, b: float) -> float:
    """T = a + b * (D/W)"""
    return a + b * (distance / width)


def main():
    results = {}

    # Fitts's law validation
    results["fitts_mt_D100_W10"] = fitts_movement_time(100.0, 10.0, 50.0, 150.0)
    results["fitts_id_D100_W10"] = fitts_index_of_difficulty(100.0, 10.0)

    # Fitts with default mouse constants
    scenarios = [
        ("close_barrel", 50.0, 30.0),
        ("medium_imp", 150.0, 20.0),
        ("far_cacodemon", 300.0, 15.0),
        ("sniper_far_tiny", 400.0, 5.0),
    ]
    fitts_scenarios = {}
    for name, d, w in scenarios:
        mt = fitts_movement_time(d, w, FITTS_A_MOUSE_MS, FITTS_B_MOUSE_MS)
        fitts_scenarios[name] = {"distance": d, "width": w, "mt_ms": mt}
    results["fitts_doom_scenarios"] = fitts_scenarios

    # Hick's law validation
    results["hick_rt_N7"] = hick_reaction_time(7, 200.0, 150.0)

    # Hick for different choice counts
    hick_results = {}
    for n in [2, 4, 7, 10, 16]:
        rt = hick_reaction_time(n, HICK_A_MS, HICK_B_MS)
        hick_results[f"N={n}"] = rt
    results["hick_choice_sweep"] = hick_results

    # Steering law validation
    results["steering_D100_W20"] = steering_time(100.0, 20.0, 10.0, 5.0)

    json.dump(results, sys.stdout, indent=2)
    print()

    # Cross-check analytical values
    expected_fitts = 50.0 + 150.0 * math.log2(2.0 * 100.0 / 10.0 + 1.0)
    expected_hick = 200.0 + 150.0 * math.log2(8.0)
    expected_steering = 35.0

    checks = [
        ("fitts_mt", results["fitts_mt_D100_W10"], expected_fitts),
        ("hick_rt", results["hick_rt_N7"], expected_hick),
        ("steering", results["steering_D100_W20"], expected_steering),
    ]

    all_pass = True
    for name, actual, expected in checks:
        ok = abs(actual - expected) < 1e-10
        status = "PASS" if ok else "FAIL"
        print(f"{name}: {status} (actual={actual:.10f}, expected={expected:.10f})",
              file=sys.stderr)
        if not ok:
            all_pass = False

    if not all_pass:
        sys.exit(1)


if __name__ == "__main__":
    main()

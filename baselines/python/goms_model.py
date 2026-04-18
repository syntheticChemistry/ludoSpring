# SPDX-License-Identifier: AGPL-3.0-or-later
"""
GOMS / Keystroke-Level Model (KLM) — Python baseline for ludoSpring.

Reference implementation of Card, Moran & Newell (1983) operator times.
Produces ground truth values consumed by Rust parity tests.

References:
  - Card, S.K., Moran, T.P., & Newell, A. (1983). "The Psychology of
    Human-Computer Interaction." Lawrence Erlbaum Associates.
  - Card, Moran & Newell (1980). "The Keystroke-Level Model." CACM 23(7).

Provenance:
  Script: baselines/python/goms_model.py
  Date: 2026-03-11 (created); see combined_baselines.json _provenance for latest run
  Python: CPython 3.10+ (stdlib only)
  Command: python3 baselines/python/goms_model.py
"""

import json
import sys

# Operator times from Card et al. (1983), Table 2
KEYSTROKE_BEST = 0.08
KEYSTROKE_AVG = 0.20
KEYSTROKE_WORST = 0.50
POINT = 1.10
HOME = 0.40
MENTAL = 1.35

# Operator symbols
K = "K"  # keystroke
P = "P"  # point
H = "H"  # home
M = "M"  # mental
# R(t) = response wait of t seconds


def task_time(operators, keystroke_s=KEYSTROKE_AVG):
    """Compute total task time for a KLM operator sequence."""
    total = 0.0
    for op in operators:
        if isinstance(op, tuple) and op[0] == "R":
            total += op[1]
        elif op == K:
            total += keystroke_s
        elif op == P:
            total += POINT
        elif op == H:
            total += HOME
        elif op == M:
            total += MENTAL
    return total


def main():
    results = {}

    # Empty task
    results["empty"] = task_time([])

    # Single keystroke
    results["single_key"] = task_time([K])

    # Menu open: M P K
    menu_open = [M, P, K]
    results["menu_open"] = task_time(menu_open)
    results["menu_open_expected"] = MENTAL + POINT + KEYSTROKE_AVG

    # Drag-drop: M P K P K
    drag_drop = [M, P, K, P, K]
    results["drag_drop"] = task_time(drag_drop)
    results["drag_drop_expected"] = MENTAL + 2 * POINT + 2 * KEYSTROKE_AVG

    # Chat input: M H 6K
    chat = [M, H, K, K, K, K, K, K]
    results["chat"] = task_time(chat)
    results["chat_expected"] = MENTAL + HOME + 6 * KEYSTROKE_AVG

    # Skill levels: 20 keystrokes
    twenty_keys = [K] * 20
    results["best_20k"] = task_time(twenty_keys, KEYSTROKE_BEST)
    results["avg_20k"] = task_time(twenty_keys, KEYSTROKE_AVG)
    results["worst_20k"] = task_time(twenty_keys, KEYSTROKE_WORST)

    json.dump(results, sys.stdout, indent=2)
    print()

    # Cross-check analytical values
    checks = [
        ("empty", results["empty"], 0.0),
        ("single_key", results["single_key"], KEYSTROKE_AVG),
        ("menu_open", results["menu_open"], results["menu_open_expected"]),
        ("drag_drop", results["drag_drop"], results["drag_drop_expected"]),
        ("chat", results["chat"], results["chat_expected"]),
        ("best_20k", results["best_20k"], 20 * KEYSTROKE_BEST),
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

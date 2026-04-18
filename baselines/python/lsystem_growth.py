# SPDX-License-Identifier: AGPL-3.0-or-later
"""
L-system growth — Python baseline for ludoSpring.

Reference implementation of Lindenmayer systems: algae (Fibonacci),
Koch curve, protein backbone, and turtle interpretation.

References:
  - Lindenmayer (1968). "Mathematical models for cellular interactions."
  - Prusinkiewicz & Lindenmayer (1990). "The Algorithmic Beauty of Plants."

Provenance:
  Script: baselines/python/lsystem_growth.py
  Date: 2026-03-11 (created); see combined_baselines.json _provenance for latest run
  Python: CPython 3.10+ (stdlib only)
  Command: python3 baselines/python/lsystem_growth.py
"""

import json
import math
import sys


def lsystem_step(s, rules):
    """Apply one generation of L-system rewriting."""
    return "".join(rules.get(c, c) for c in s)


def lsystem_generate(axiom, rules, generations):
    """Generate L-system string after n generations."""
    current = axiom
    for _ in range(generations):
        current = lsystem_step(current, rules)
    return current


def turtle_interpret(lstring, step_length, angle_degrees):
    """Interpret L-system string as 2D turtle graphics."""
    angle_rad = math.radians(angle_degrees)
    x, y, heading = 0.0, 0.0, 0.0
    stack = []
    points = [(x, y)]

    for ch in lstring:
        if ch in ('F', 'H', 'S', 'L'):
            x += math.cos(heading) * step_length
            y += math.sin(heading) * step_length
            points.append((x, y))
        elif ch == '+':
            heading += angle_rad
        elif ch == '-':
            heading -= angle_rad
        elif ch == 'T':
            heading += angle_rad * 0.5
        elif ch == '[':
            stack.append((x, y, heading))
        elif ch == ']':
            if stack:
                x, y, heading = stack.pop()

    return points


# Preset L-systems matching Rust implementations
ALGAE = {"axiom": "A", "rules": {"A": "AB", "B": "A"}}
KOCH = {"axiom": "F", "rules": {"F": "F+F-F-F+F"}}
PROTEIN = {"axiom": "HLSH", "rules": {"H": "HHL", "S": "SLT", "L": "LS"}}
DRAGON = {"axiom": "FX", "rules": {"X": "X+YF+", "Y": "-FX-Y"}}


def main():
    results = {}

    # Algae: lengths should follow Fibonacci sequence
    algae_lengths = []
    for g in range(8):
        s = lsystem_generate(ALGAE["axiom"], ALGAE["rules"], g)
        algae_lengths.append(len(s))
    results["algae_lengths"] = algae_lengths

    # Koch curve: gen 0 and gen 1 lengths
    results["koch_g0"] = len(lsystem_generate(KOCH["axiom"], KOCH["rules"], 0))
    results["koch_g1"] = len(lsystem_generate(KOCH["axiom"], KOCH["rules"], 1))

    # Protein backbone gen 3: must contain all structural elements
    protein_g3 = lsystem_generate(PROTEIN["axiom"], PROTEIN["rules"], 3)
    results["protein_g3_len"] = len(protein_g3)
    results["protein_g3_has_H"] = "H" in protein_g3
    results["protein_g3_has_S"] = "S" in protein_g3
    results["protein_g3_has_L"] = "L" in protein_g3
    results["protein_g3_has_T"] = "T" in protein_g3

    # Turtle interpretation: two forward steps
    pts = turtle_interpret("FF", 1.0, 90.0)
    results["turtle_FF_end"] = list(pts[-1])

    # Square: F+F+F+F should return near origin
    sq = turtle_interpret("F+F+F+F", 1.0, 90.0)
    end = sq[-1]
    results["turtle_square_end"] = list(end)
    results["turtle_square_dist"] = math.sqrt(end[0]**2 + end[1]**2)

    json.dump(results, sys.stdout, indent=2)
    print()

    # Cross-check
    checks = [
        ("algae_fibonacci", algae_lengths, [1, 2, 3, 5, 8, 13, 21, 34]),
        ("koch_g0", results["koch_g0"], 1),
        ("koch_g1", results["koch_g1"], 9),
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

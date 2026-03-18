#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""
Baseline drift detector — re-runs all Python baselines and compares
against the stored combined_baselines.json. Reports any numerical drift
that would invalidate Rust parity assumptions.

Exit codes:
  0 — no drift
  1 — drift detected or execution error

Usage:
  python3 baselines/python/check_drift.py
"""

import json
import math
import subprocess
import sys
import tempfile
from pathlib import Path


# Matches Rust tolerances::ANALYTICAL_TOL — both sides use the same bound
# so a drift detected here guarantees a Rust parity failure.
TOLERANCE = 1e-10


def flatten(obj, prefix=""):
    """Recursively flatten nested JSON into dot-path → value pairs."""
    items = {}
    if isinstance(obj, dict):
        for k, v in obj.items():
            new_key = f"{prefix}.{k}" if prefix else k
            items.update(flatten(v, new_key))
    elif isinstance(obj, list):
        for i, v in enumerate(obj):
            items.update(flatten(v, f"{prefix}[{i}]"))
    else:
        items[prefix] = obj
    return items


def numeric_diff(a, b, tol=TOLERANCE):
    """Compare two values; return None if equal, else a description."""
    if isinstance(a, (int, float)) and isinstance(b, (int, float)):
        if math.isnan(a) and math.isnan(b):
            return None
        if a == b:
            return None
        delta = abs(a - b)
        if delta <= tol:
            return None
        return f"was {a}, now {b} (Δ={delta:.2e})"
    if a != b:
        return f"was {a!r}, now {b!r}"
    return None


def main():
    base_dir = Path(__file__).parent
    stored_path = base_dir / "combined_baselines.json"

    if not stored_path.exists():
        print("ERROR: combined_baselines.json not found — run run_all_baselines.py first",
              file=sys.stderr)
        sys.exit(1)

    with open(stored_path) as f:
        stored = json.load(f)

    with tempfile.NamedTemporaryFile(suffix=".json", delete=False) as tmp:
        tmp_path = Path(tmp.name)

    try:
        proc = subprocess.run(
            [sys.executable, str(base_dir / "run_all_baselines.py"),
             "--output", str(tmp_path)],
            capture_output=True, text=True,
        )
        if proc.returncode != 0:
            print(f"ERROR: run_all_baselines.py failed (exit {proc.returncode})",
                  file=sys.stderr)
            if proc.stderr:
                print(proc.stderr, file=sys.stderr)
            sys.exit(1)

        with open(tmp_path) as f:
            fresh = json.load(f)
    finally:
        tmp_path.unlink(missing_ok=True)

    stored_flat = flatten(stored)
    fresh_flat = flatten(fresh)

    skip_prefixes = ("_provenance.",)
    drifts = []
    missing = []

    for key, old_val in stored_flat.items():
        if any(key.startswith(p) for p in skip_prefixes):
            continue
        if key not in fresh_flat:
            missing.append(key)
            continue
        diff = numeric_diff(old_val, fresh_flat[key])
        if diff:
            drifts.append((key, diff))

    new_keys = [k for k in fresh_flat if k not in stored_flat
                and not any(k.startswith(p) for p in skip_prefixes)]

    if not drifts and not missing and not new_keys:
        print("✓ No baseline drift detected")
        sys.exit(0)

    if drifts:
        print(f"\n⚠ {len(drifts)} value(s) drifted:")
        for key, desc in sorted(drifts):
            print(f"  {key}: {desc}")

    if missing:
        print(f"\n⚠ {len(missing)} key(s) missing from fresh run:")
        for key in sorted(missing):
            print(f"  {key}")

    if new_keys:
        print(f"\nℹ {len(new_keys)} new key(s) in fresh run:")
        for key in sorted(new_keys):
            print(f"  {key}")

    sys.exit(1)


if __name__ == "__main__":
    main()

# SPDX-License-Identifier: AGPL-3.0-or-later
"""
Run all Python baselines and produce combined output.

Provenance:
  Script: baselines/python/run_all_baselines.py
  Date: 2026-03-11
  Command: python3 baselines/python/run_all_baselines.py
"""

import json
import subprocess
import sys
from pathlib import Path

BASELINES = [
    "perlin_noise.py",
    "interaction_laws.py",
    "flow_engagement.py",
    "goms_model.py",
    "lsystem_growth.py",
    "bsp_partition.py",
    "fun_keys_model.py",
]


def main():
    base_dir = Path(__file__).parent
    all_pass = True
    results = {}

    for script in BASELINES:
        path = base_dir / script
        print(f"Running {script}...", file=sys.stderr)
        proc = subprocess.run(
            [sys.executable, str(path)],
            capture_output=True, text=True,
        )
        if proc.returncode != 0:
            print(f"  FAIL (exit {proc.returncode})", file=sys.stderr)
            if proc.stderr:
                print(f"  stderr: {proc.stderr.strip()}", file=sys.stderr)
            all_pass = False
        else:
            print(f"  PASS", file=sys.stderr)
            if proc.stderr:
                for line in proc.stderr.strip().split("\n"):
                    print(f"    {line}", file=sys.stderr)

        if proc.stdout.strip():
            try:
                results[script] = json.loads(proc.stdout)
            except json.JSONDecodeError:
                results[script] = proc.stdout.strip()

    # Write combined output
    output_path = base_dir / "combined_baselines.json"
    with open(output_path, "w") as f:
        json.dump(results, f, indent=2)
        f.write("\n")
    print(f"\nCombined output: {output_path}", file=sys.stderr)

    if all_pass:
        print("\nAll baselines PASS", file=sys.stderr)
    else:
        print("\nSome baselines FAILED", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()

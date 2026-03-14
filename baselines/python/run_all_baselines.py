# SPDX-License-Identifier: AGPL-3.0-or-later
"""
Run all Python baselines and produce combined output.

Provenance:
  Script: baselines/python/run_all_baselines.py
  Date: 2026-03-11 (initial), updated 2026-03-14
  Command: python3 baselines/python/run_all_baselines.py
  Python: CPython 3.12 (stdlib only — no numpy/scipy)
  Dependencies: math, json, subprocess, sys, pathlib, platform, datetime
"""

import datetime
import json
import platform
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

    git_commit = "unknown"
    try:
        git_proc = subprocess.run(
            ["git", "rev-parse", "HEAD"],
            capture_output=True, text=True, cwd=str(base_dir),
        )
        if git_proc.returncode == 0:
            git_commit = git_proc.stdout.strip()
    except FileNotFoundError:
        pass

    results["_provenance"] = {
        "script": "baselines/python/run_all_baselines.py",
        "date": datetime.datetime.now(datetime.timezone.utc).isoformat(),
        "python_version": platform.python_version(),
        "python_implementation": platform.python_implementation(),
        "command": "python3 baselines/python/run_all_baselines.py",
        "git_commit": git_commit,
        "dependencies": "stdlib only (math, json)",
    }

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

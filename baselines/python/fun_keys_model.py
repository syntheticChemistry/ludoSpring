# SPDX-License-Identifier: AGPL-3.0-or-later
"""
Four Keys to Fun — Python baseline for ludoSpring.

Reference implementation of Lazzaro's (2004) fun taxonomy classification.

References:
  - Lazzaro, N. (2004). "Why We Play Games: Four Keys to More Emotion
    Without Story." GDC '04.

Provenance:
  Script: baselines/python/fun_keys_model.py
  Date: 2026-03-11
  Python: CPython 3.10+ (stdlib only)
  Command: python3 baselines/python/fun_keys_model.py
"""

import json
import math
import sys


def classify_fun(challenge, exploration, social, completion, retry_rate):
    """Classify fun type from behavioral signals."""
    hard = challenge * 0.6 + retry_rate * 0.4
    easy = exploration * 0.8 + (1.0 - challenge) * 0.2
    people = social
    serious = completion * 0.7 + (1.0 - social) * 0.15 + (1.0 - challenge) * 0.15

    scores = {"hard": hard, "easy": easy, "people": people, "serious": serious}
    dominant = max(scores, key=scores.get)
    return dominant, scores


def main():
    results = {}

    scenarios = {
        "dark_souls_boss": (0.95, 0.2, 0.05, 0.3, 0.9),
        "minecraft_creative": (0.1, 0.9, 0.1, 0.3, 0.0),
        "among_us": (0.3, 0.1, 0.95, 0.1, 0.1),
        "animal_crossing": (0.05, 0.3, 0.1, 0.9, 0.0),
        "celeste": (0.9, 0.3, 0.0, 0.4, 0.85),
        "no_mans_sky": (0.15, 0.85, 0.15, 0.2, 0.05),
    }

    for name, signals in scenarios.items():
        dominant, scores = classify_fun(*signals)
        results[name] = {"dominant": dominant, "scores": scores}

    # Zero signals
    _, zero_scores = classify_fun(0, 0, 0, 0, 0)
    results["zero_scores"] = zero_scores

    # Max signals
    _, max_scores = classify_fun(1, 1, 1, 1, 1)
    results["max_scores"] = max_scores

    json.dump(results, sys.stdout, indent=2)
    print()

    expected = {
        "dark_souls_boss": "hard",
        "minecraft_creative": "easy",
        "among_us": "people",
        "animal_crossing": "serious",
        "celeste": "hard",
        "no_mans_sky": "easy",
    }

    all_pass = True
    for name, exp_dominant in expected.items():
        actual = results[name]["dominant"]
        ok = actual == exp_dominant
        status = "PASS" if ok else "FAIL"
        print(f"{name}: {status} (actual={actual}, expected={exp_dominant})",
              file=sys.stderr)
        if not ok:
            all_pass = False

    # Check bounds
    for k, v in max_scores.items():
        ok = v <= 1.0
        if not ok:
            print(f"max_{k}: FAIL (score={v} > 1.0)", file=sys.stderr)
            all_pass = False

    for k, v in zero_scores.items():
        ok = v >= 0.0
        if not ok:
            print(f"zero_{k}: FAIL (score={v} < 0.0)", file=sys.stderr)
            all_pass = False

    if not all_pass:
        sys.exit(1)


if __name__ == "__main__":
    main()

# SPDX-License-Identifier: AGPL-3.0-or-later
"""
Flow state and engagement reference — Python baseline for ludoSpring.

Models Csikszentmihalyi flow channel and Yannakakis engagement scoring.

References:
  - Csikszentmihalyi, M. (1990). "Flow: The Psychology of Optimal Experience."
  - Chen, J. (2007). "Flow in Games." M.S. Thesis, USC.
  - Yannakakis, G.N. & Togelius, J. (2018). "AI and Games." Springer, Ch. 11.
  - Hunicke, R. (2005). "The case for dynamic difficulty adjustment."

Provenance:
  Script: baselines/python/flow_engagement.py
  Date: 2026-03-11 (created); see combined_baselines.json _provenance for latest run
  Python: CPython 3.10+ (stdlib only)
  Command: python3 baselines/python/flow_engagement.py
"""

import json
import sys

FLOW_CHANNEL_WIDTH = 0.15
DDA_TARGET_SUCCESS_RATE = 0.7
ENGAGEMENT_WEIGHT = 0.2
ENGAGEMENT_APM_CEILING = 60.0
ENGAGEMENT_EXPLORATION_CEILING = 5.0
MIN_SESSION_MINUTES = 0.01


def evaluate_flow(challenge: float, skill: float,
                  channel_width: float) -> str:
    diff = challenge - skill
    if abs(diff) <= channel_width:
        return "Flow"
    elif diff > channel_width * 2.0:
        return "Anxiety"
    elif diff > channel_width:
        return "Arousal"
    elif diff < -channel_width * 2.0:
        return "Boredom"
    else:
        return "Relaxation"


def compute_engagement(session_s: float, actions: int, exploration: int,
                       challenge_seeking: int, retries: int,
                       pauses: int) -> dict:
    minutes = session_s / 60.0
    if minutes < MIN_SESSION_MINUTES:
        minutes = MIN_SESSION_MINUTES

    apm = actions / minutes
    exploration_rate = exploration / minutes
    challenge_appetite = challenge_seeking / actions if actions > 0 else 0.0
    persistence = retries / actions if actions > 0 else 0.0
    deliberation = pauses / actions if actions > 0 else 0.0

    w = ENGAGEMENT_WEIGHT
    composite = (
        min(apm / ENGAGEMENT_APM_CEILING, 1.0) * w
        + min(exploration_rate / ENGAGEMENT_EXPLORATION_CEILING, 1.0) * w
        + min(challenge_appetite, 1.0) * w
        + min(persistence, 1.0) * w
        + min(deliberation, 1.0) * w
    )
    composite = max(0.0, min(1.0, composite))

    return {
        "actions_per_minute": apm,
        "exploration_rate": exploration_rate,
        "challenge_appetite": challenge_appetite,
        "persistence": persistence,
        "deliberation": deliberation,
        "composite": composite,
    }


def main():
    results = {}

    # Flow state boundary tests
    flow_tests = {
        "exact_diagonal": evaluate_flow(0.5, 0.5, FLOW_CHANNEL_WIDTH),
        "inside_channel_low": evaluate_flow(
            0.5, 0.5 - FLOW_CHANNEL_WIDTH * 0.9, FLOW_CHANNEL_WIDTH),
        "inside_channel_high": evaluate_flow(
            0.5, 0.5 + FLOW_CHANNEL_WIDTH * 0.9, FLOW_CHANNEL_WIDTH),
        "high_challenge_low_skill": evaluate_flow(0.9, 0.1, FLOW_CHANNEL_WIDTH),
        "low_challenge_high_skill": evaluate_flow(0.1, 0.9, FLOW_CHANNEL_WIDTH),
    }
    results["flow_states"] = flow_tests

    # Engagement for active player
    active = compute_engagement(300.0, 200, 15, 10, 20, 15)
    results["engagement_active"] = active

    # Engagement for idle player
    idle = compute_engagement(300.0, 2, 1, 0, 0, 0)
    results["engagement_idle"] = idle

    # Zero-duration edge case
    zero = compute_engagement(0.0, 0, 0, 0, 0, 0)
    results["engagement_zero"] = zero

    # DDA session simulation (matches exp004)
    dda_session = []
    skill = 0.3
    for round_num in range(20):
        progress = round_num / 19.0
        x = (progress - 0.5) * 8.0
        sigmoid = 1.0 / (1.0 + 2.718281828459045 ** (-x))
        difficulty = 0.2 + (0.9 - 0.2) * sigmoid
        flow = evaluate_flow(difficulty, skill, FLOW_CHANNEL_WIDTH)
        dda_session.append({
            "round": round_num + 1,
            "difficulty": difficulty,
            "skill": skill,
            "flow": flow,
        })
        skill = min(skill + 0.02, 0.95)
    results["dda_session"] = dda_session

    json.dump(results, sys.stdout, indent=2)
    print()

    # Validation
    checks_pass = True
    if flow_tests["exact_diagonal"] != "Flow":
        print("FAIL: exact diagonal should be Flow", file=sys.stderr)
        checks_pass = False
    if flow_tests["high_challenge_low_skill"] != "Anxiety":
        print("FAIL: high challenge/low skill should be Anxiety",
              file=sys.stderr)
        checks_pass = False
    if flow_tests["low_challenge_high_skill"] != "Boredom":
        print("FAIL: low challenge/high skill should be Boredom",
              file=sys.stderr)
        checks_pass = False
    if not (0.0 <= zero["composite"] <= 1.0 and
            zero["composite"] == zero["composite"]):
        print("FAIL: zero session composite not finite/bounded",
              file=sys.stderr)
        checks_pass = False

    print(f"flow_engagement: {'PASS' if checks_pass else 'FAIL'}",
          file=sys.stderr)
    if not checks_pass:
        sys.exit(1)


if __name__ == "__main__":
    main()

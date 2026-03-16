#!/usr/bin/env python3
# SPDX-License-Identifier: AGPL-3.0-or-later
"""
ludoSpring tolerance constants — Python mirror of Rust `barracuda::tolerances`.

Every constant here matches the identically named constant in the Rust codebase
(`barracuda/src/tolerances/`). This module enables Python baselines to import
named constants rather than using magic numbers, following the wetSpring V121
pattern (`scripts/tolerances.py`).

Usage:
    from tolerances import ANALYTICAL_TOL, FITTS_A_MOUSE_MS
    assert abs(computed - expected) < ANALYTICAL_TOL
"""

# === validation.rs ===

ANALYTICAL_TOL = 1e-10
"""Closed-form formula validation (Fitts, Hick, Steering). Source: f64 reassociation bound."""

RAYCASTER_DISTANCE_TOL = 0.5
"""DDA integer-step raycasting distance. Source: sub-cell positioning error."""

NOISE_COHERENCE_TOL = 0.01
"""Perlin noise coherence for Δx = 0.01. Source: smooth C² interpolation."""

UI_DATA_INK_TOL = 0.05
"""Tufte data-ink ratio validation. Source: font-metric / rounding variance."""

UI_COVERAGE_TOL = 0.02
"""HUD screen-coverage validation. Source: width × height / total area rounding."""

RAYCASTER_HIT_RATE_TOL = 5.0
"""DDA ray-wall hit-rate (%). Source: f32 GPU vs f64 CPU step delta on boundary rays."""

# === interaction.rs ===

FITTS_A_MOUSE_MS = 50.0
"""Fitts's law intercept (mouse, desktop). Source: MacKenzie (1992), Table 2."""

FITTS_B_MOUSE_MS = 150.0
"""Fitts's law slope (ms/bit). Source: MacKenzie (1992), Table 2."""

HICK_A_MS = 200.0
"""Hick's law base reaction time (ms). Source: Hick (1952)."""

HICK_B_MS = 150.0
"""Hick's law processing time per bit (ms). Source: Hyman (1953)."""

STEERING_A_MS = 10.0
"""Steering law intercept (ms). Source: Accot & Zhai (1997), mouse condition."""

STEERING_B_MS = 5.0
"""Steering law index coefficient (ms per D/W). Source: Accot & Zhai (1997)."""

FLOW_CHANNEL_WIDTH = 0.15
"""Flow channel half-width. Source: Chen (2007), Figure 3.2."""

DDA_TARGET_SUCCESS_RATE = 0.7
"""DDA target success rate. Source: Hunicke (2005), Section 4 midpoint."""

# === metrics.rs ===

TUFTE_MIN_DATA_INK_RATIO = 0.4
"""Minimum data-ink ratio. Source: Tufte (1983), Chapter 4."""

MAX_HUD_COVERAGE = 0.25
"""Max HUD screen coverage. Source: Fagerholt & Lorentzon (2009), Section 4.3."""

ENGAGEMENT_WEIGHT = 0.2
"""Equal weight per engagement dimension. Source: Yannakakis & Togelius (2018), Ch. 11."""

ENGAGEMENT_APM_CEILING = 60.0
"""Actions per minute normalization ceiling. Source: casual play average."""

ENGAGEMENT_EXPLORATION_CEILING = 5.0
"""Exploration rate ceiling (areas/min). Source: Bartle (1996), Explorer archetype."""

TUFTE_SIMPLIFIABLE_THRESHOLD = 0.5
"""Data-ink ratio below which element is simplifiable. Source: Tufte (1983)."""

TUFTE_SEVERE_DECORATION_THRESHOLD = 0.3
"""Data-ink ratio indicating severe decoration. Source: Tufte (1983)."""

TUFTE_MIN_DATA_VALUES_LARGE_ELEMENT = 3
"""Min data values for large UI element. Source: Fagerholt & Lorentzon (2009)."""

TUFTE_LARGE_ELEMENT_THRESHOLD = 0.05
"""Screen coverage triggering size review. Source: Fagerholt & Lorentzon (2009)."""

MIN_SESSION_MINUTES = 0.01
"""Minimum session duration before computing rates (0.6 seconds)."""

# === game.rs ===

NPC_PROXIMITY_TILES = 3
"""NPC spatial proximity for narration cues (tiles)."""

AREA_DESCRIPTION_RANGE_TILES = 5
"""Extended range for area descriptions (tiles)."""

ITEM_PROXIMITY_TILES = 3
"""Item proximity for area descriptions (tiles)."""

DEFAULT_VERTEX_QUERY_LIMIT = 50
"""Default NPC memory vertex query limit."""

TARGET_FRAME_RATE_HZ = 60.0
"""Target frame rate for budget calculations (Hz)."""

DEFAULT_SIGHT_RADIUS = 5
"""Default fog of war sight radius (tiles)."""

TRIGGER_DETECTION_RANGE = 1
"""Trigger detection range for movement (tiles)."""

GAME_STATE_TOL = 0.01
"""Game-state comparison tolerance (trust, flow, engagement)."""

# === ipc.rs ===

RPC_TIMEOUT_SECS = 5
"""JSON-RPC call timeout (seconds)."""

PROBE_TIMEOUT_MS = 500
"""Socket probe timeout (milliseconds)."""

CONNECT_PROBE_TIMEOUT_MS = 200
"""Quick liveness check timeout (milliseconds)."""

# === procedural.rs ===

DDA_NEAR_ZERO = 1e-12
"""Near-zero threshold for DDA ray direction components."""

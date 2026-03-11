# Expedition 024: Doom-in-a-Terminal — First-Person Navigation from Validated Math

**Date:** March 11, 2026
**Status:** Playable — BSP levels + DDA raycaster + ratatui renderer
**Real game reference:** Doom (1993), Wolfenstein 3D (1992)

---

## The Science of First-Person Navigation

This expedition proves that ludoSpring's validated math produces a playable
first-person walker. Every component maps to published research:

| Component | Module | Research |
|-----------|--------|----------|
| Level layout | `procedural::bsp` | Fuchs, Kedem & Naylor (1980) |
| Ray marching | `game::raycaster` | Carmack (1993), DDA algorithm |
| HUD quality | `metrics::tufte_gaming` | Tufte (1983, 1990) |
| Target acquisition | `interaction::input_laws` | Fitts (1954) |

## What We Built

- **BSP level generation:** Rooms carved from BSP leaves, connected by L-shaped
  corridors. Deterministic (seed=42 always produces the same map).
- **DDA raycasting:** Per-column Digital Differential Analyzer produces distance
  to nearest wall. Column characters scale by distance (█▓▒░).
- **Collision detection:** Grid-based wall check rejects moves into solid cells.
- **Terminal rendering:** ratatui canvas with 30 Hz fixed timestep. Zero GPU
  dependency — runs over SSH.
- **Tufte analysis:** HUD scored at startup for data-ink ratio, info density,
  and screen coverage.

## Why BSP Produces Good Levels

Binary Space Partitioning recursively subdivides space into rooms. The resulting
tree structure guarantees:
1. **Full coverage:** Leaf areas sum to total area (conservation law)
2. **No overlaps:** Tree structure prevents room intersection
3. **Navigable connectivity:** Corridor generation between adjacent leaves creates
   a connected graph

This is the same structure Carmack used in Doom (1993) for front-to-back
rendering without z-buffer. We use it for collision-free level generation.

## Why DDA Is GPU-Friendly

The Digital Differential Analyzer steps along grid boundaries using integer
increments. At each step:
- Compare distances to next horizontal and vertical grid lines
- Step to the nearer one
- Check for wall

No floating-point branching. Each screen column is independent (embarrassingly
parallel). This maps directly to a compute shader where each invocation handles
one column — the pattern documented in the barraCuda/toadStool evolution handoff.

## Extensibility

The same raycaster works for:
- **Molecular cave navigation:** Replace wall texture with electron density field
- **Office layout walkthrough:** BSP partitions match architectural floor plans
- **Museum exhibit navigation:** First-person spatial exploration of any grid world
- **Warehouse routing:** Collision-free path planning in partitioned space

Terminal rendering means:
- Zero GPU dependency
- Runs on any machine with a terminal
- Accessible over SSH
- Testable in CI

## Controls

WASD: move/strafe | Q/E or arrows: rotate | Esc: quit

## Reproducibility

```bash
cargo run --bin exp024_doom_terminal          # default seed
cargo run --bin exp024_doom_terminal -- 99    # custom seed
```

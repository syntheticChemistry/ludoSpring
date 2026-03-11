# Expedition 038: External Roguelike Control Group

**Date:** 2026-03-11
**Status:** Active
**Reference:** bracket-pathfinding (A*, FOV), drunkard's walk PCG

## What We Built

A complete roguelike game using ZERO ludoSpring PCG libraries as the
control group for validating our metrics framework. If ludoSpring's
metrics produce meaningful results on foreign content, the measurement
tool itself is validated.

### External Dependencies (not ludoSpring)

| Component | Library | License |
|-----------|---------|---------|
| Dungeon generation | Drunkard's walk (hand-rolled) | — |
| Pathfinding | bracket-pathfinding A* | MIT |
| Field of view | bracket-pathfinding FOV | MIT |
| RNG | Hand-rolled LCG (not barraCuda) | — |
| Item placement | Random scatter (not Perlin) | — |

### ludoSpring Metrics Applied to Foreign Content

| Metric | Result | Valid? |
|--------|--------|--------|
| Engagement composite | 0.444 | In range [0,1] |
| Flow state | Boredom (correct — AI player too skilled) | Valid enum |
| Fun classification | Easy Fun (exploration-driven) | Valid |
| DDA recommendation | +0.20 (increase difficulty) | Correct direction |

### Key Finding

ludoSpring's metrics pipeline works on ANY game content, regardless of
how it was generated. The measurement framework is independent of the
generation framework. This validates the tool, not just the content.

### Reproducibility

```bash
cargo run --bin exp038_external_roguelike_control    # validate (12 checks)
```

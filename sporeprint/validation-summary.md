+++
title = "ludoSpring Validation Summary"
description = "Game science, HCI, procedural generation — 820+ tests, 13 HCI models validated, 100 experiment crates"
date = 2026-05-08

[taxonomies]
primals = ["barracuda", "toadstool", "petaltongue", "biomeos", "squirrel"]
springs = ["ludospring"]
+++

## Status

- **820+** workspace tests (unit, integration, property, determinism, parity)
- **13 foundational HCI models** validated against published research
- **100 experiment crates** covering game science, RPGPT, provenance, composition
- **2 playable prototypes** (Doom terminal raycaster, roguelike explorer)
- **L5 guideStone** (54/54 checks: Tier 1 bare + Tier 2 IPC + Tier 3 NUCLEUS)
- **Pure composition model** — no spring binary deploys; biomeOS orchestrates primal graph
- Game genres as **interaction architectures** (FPS = molecular explorer, roguelike = parameter space)

## Key Validation Binaries

- `validate_interaction` — Fitts, Hick, Steering, GOMS, Flow, DDA
- `validate_procedural` — Perlin noise, fBm, WFC, L-systems, BSP
- `validate_engagement` — engagement composite, Four Keys, Tufte metrics
- `validate_composition` — IPC golden-value parity (7 game.* methods)
- `validate_primal_proof` — raw IPC to live barraCuda primals
- `validate_all` — aggregates all above + guidestone + composition
- `ludospring_guidestone` — three-tier self-validating NUCLEUS node

## Workload TOMLs

Not yet created — contribute to `projectNUCLEUS/workloads/ludospring/`.
Foundation thread 9 (Gaming/Creative) mapped but not instrumented.

## See Also

- [Spring Catalog](https://primals.eco/architecture/spring-catalog-status-science-and-evolution/) on primals.eco
- [baseCamp Papers 17-22](https://primals.eco/science/)

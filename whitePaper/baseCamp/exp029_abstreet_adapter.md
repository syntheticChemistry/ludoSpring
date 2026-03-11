# Expedition 029: A/B Street Telemetry Adapter

**Date:** 2026-03-11
**Status:** Active
**Reference:** A/B Street (Apache-2.0, github.com/a-b-street/abstreet)

## What We Built

A simulation-to-telemetry adapter that translates A/B Street's
transportation analytics into ludoSpring events, treating city planning
as a game.

### A/B Street Architecture

- Traffic simulation with game-like UI
- Headless mode for programmatic control
- `sim::analytics` with `SlidingWindow` for event counts
- Trip completion, signal changes, throughput metrics

### Event Mapping (Simulation-as-Game)

| A/B Street event | Telemetry event type | Game analogy |
|---|---|---|
| `SimStart` | `session_start` | Match start |
| `InfraChange` | `player_action` | Player move |
| `TripComplete` | `challenge_complete` | Enemy defeated |
| `TripFailed` | `challenge_fail` | Challenge failed |
| `CongestionDetected` | `challenge_encounter` | Boss encounter |
| `RoadThroughput` | `exploration_discover` | Area explored |
| `SimEnd` | `session_end` | Match end |

### Unique Angle: Simulation as Game

This adapter proves ludoSpring's metrics work beyond traditional games.
A city planner's decisions are "player actions." Traffic congestion is
a "challenge." Throughput improvements are "discoveries."

The same mapping works for any decision-making simulation:
- Factory optimization (Factorio-like)
- Ecosystem management
- Supply chain design
- Educational simulations

### Reproducibility

```bash
cargo run -p ludospring-exp029 -- validate  # 8 adapter checks
cargo run -p ludospring-exp029 -- demo      # synthetic simulation analysis
```

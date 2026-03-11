# Expedition 027: Veloren Telemetry Adapter

**Date:** 2026-03-11
**Status:** Active
**Reference:** Veloren (GPL-3.0, gitlab.com/veloren/veloren)

## What We Built

A log parser that translates Veloren's SPECS ECS tracing output into
ludoSpring telemetry events.

### Veloren Architecture

- SPECS ECS with `EventBus` for game events
- `tracing` crate for structured logging
- WASM plugin system for extensibility
- Events: `HealthChangeEvent`, `DeleteEvent`, `CreateItemDropEvent`

### Event Mapping

| Veloren log pattern | Telemetry event type |
|---|---|
| `player_pos x=... y=... z=...` | `player_move` |
| `health_change change=-15 source="wolf"` | `player_damage` |
| `entity_delete cause="killed_by:..."` | `challenge_complete` |
| `region_enter region="..."` | `exploration_discover` |
| `session_start` / `session_end` | `session_start` / `session_end` |

### Science Applied

With the adapter, we can answer questions about Veloren gameplay:
- Is the difficulty curve maintaining flow? (Csikszentmihalyi)
- Are combat encounters well-paced? (DDA, Hunicke 2005)
- What fun type dominates exploration vs combat? (Lazzaro 2004)
- How engaged are players across biomes? (Yannakakis 2018)

### Future: WASM Plugin

Veloren's WASM plugin system allows live in-game telemetry emission
without modifying the game's source code. The adapter pattern demonstrated
here would be packaged as a `.plugin.tar` with `plugin.toml`.

### Reproducibility

```bash
cargo run -p ludospring-exp027 -- validate  # 9 adapter checks
cargo run -p ludospring-exp027 -- demo      # synthetic session analysis
cargo run -p ludospring-exp027 -- parse veloren.log > session.ndjson
cargo run -p ludospring-exp026 -- analyze session.ndjson
```

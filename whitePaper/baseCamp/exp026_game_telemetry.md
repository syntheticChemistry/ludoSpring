# Expedition 026: Portable Game Telemetry Protocol

**Date:** 2026-03-11
**Status:** Active
**Reference:** OpenTelemetry, Unity Analytics, GameAnalytics SDK

## What We Built

A language-agnostic game telemetry protocol (NDJSON) with a full analysis
pipeline that maps events to ludoSpring's validated metrics.

### Components

| Component | Module | Purpose |
|-----------|--------|---------|
| Event schema | `barracuda::telemetry::events` | 13 typed event types + extensible Custom |
| Accumulator | `barracuda::telemetry::mapper` | Event stream -> metric inputs |
| Report generator | `barracuda::telemetry::report` | Full gameplay analysis from events |
| NDJSON parser | `barracuda::telemetry` | File/stdin/reader ingestion |
| CLI analyzer | `exp026_game_telemetry` | Offline analysis, generation, validation |

### Science Mapping

| Event stream | ludoSpring engine | Paper |
|---|---|---|
| Actions + exploration + retries | `engagement::compute_engagement` | Yannakakis 2018 |
| Challenge vs estimated skill | `flow::evaluate_flow` | Csikszentmihalyi 1990 |
| Success/fail window | `difficulty::suggest_adjustment` | Hunicke 2005 |
| Behavioral signals | `fun_keys::classify_fun` | Lazzaro 2004 |
| UI element descriptions | `tufte_gaming::analyze_game_ui` | Tufte 1983 |
| Interaction distances | `input_laws::fitts/hick/steering` | Fitts 1954, Hick 1952 |

### Portability

The protocol is pure JSON. Any language that can write JSON objects can be
a telemetry producer:

- **Rust games:** Direct `use ludospring_barracuda::telemetry` (zero overhead)
- **Unity (C#):** `File.AppendAllText("telemetry.ndjson", JsonUtility.ToJson(evt))`
- **Godot (GDScript):** `file.store_line(JSON.stringify(evt))`
- **Web (JS):** `fetch('/telemetry', {body: JSON.stringify(evt)})`

### Reproducibility

```bash
cargo run -p ludospring-exp026 -- validate     # 13 protocol checks
cargo run -p ludospring-exp026 -- generate -    # synthetic NDJSON to stdout
cargo run -p ludospring-exp026 -- analyze f.ndjson  # full report from file
cargo run -p ludospring-exp026 -- schema        # print event type reference
```

# Expedition 028: Fish Folk / Jumpy Telemetry Adapter

**Date:** 2026-03-11
**Status:** Active
**Reference:** Fish Folk (MIT/Apache-2.0, github.com/fishfolk)

## What We Built

A Bevy plugin pattern adapter that translates Fish Folk's ECS events into
ludoSpring telemetry, demonstrating multiplayer game analysis.

### Fish Folk Architecture

- Bevy ECS for rendering and audio
- Bones framework for deterministic core gameplay
- Rollback networking with snapshot/restore
- Games: Jumpy (2D shooter), Punchy (beat-em-up), Bomby (action)

### Event Mapping

| Fish Folk event | Telemetry event type |
|---|---|
| `MatchStart` | `session_start` |
| `PlayerMove` | `player_move` |
| `WeaponPickup` | `exploration_discover` |
| `PlayerHit` (as target) | `player_damage` |
| `PlayerHit` (as source) | `player_action` |
| `PlayerKill` (as killer) | `challenge_complete` |
| `PlayerKill` (as victim) | `player_death` |
| `MatchEnd` | `session_end` |

### Unique Angle: PvP Analysis

Fish Folk is multiplayer PvP. The adapter demonstrates per-player
perspective analysis from shared match events. The same pattern works
for any Bevy game using `EventReader<T>`.

### Bevy Integration Pattern

```rust
// How a real Bevy plugin would look (not compiled here to avoid dep)
fn telemetry_system(
    mut reader: EventReader<GameEvent>,
    mut writer: ResMut<TelemetryWriter>,
) {
    for event in reader.read() {
        let telemetry = translate_event(event);
        writer.emit(telemetry);
    }
}
```

### Reproducibility

```bash
cargo run -p ludospring-exp028 -- validate  # 7 adapter checks
cargo run -p ludospring-exp028 -- demo      # synthetic match analysis
```

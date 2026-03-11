# Open Systems Benchmark Specification

**Date**: March 10, 2026
**Status**: Active
**License**: AGPL-3.0-or-later

---

## Philosophy

ludoSpring aims for **pure sovereign Rust**. We get there by *scaffolding*
from open systems and *evolving* each component into native primal code.
Every scaffold dependency has an exit plan.

```
Open system (benchmark baseline)
    → Rust binding / FFI scaffold
    → Feature-parity Rust implementation
    → barraCuda GPU evolution
    → Sovereign pipeline (coralReef native)
```

We study open systems for two reasons:

1. **Benchmarks** — they define the bar. Our Rust implementations must meet
   or exceed their latency, throughput, and correctness.
2. **Design archaeology** — understand what decisions they made, why, and
   what we'd do differently with primal composition.

Proprietary systems (Steam, Unity, Discord, Unreal) are studied only as
*capability specifications* — what they do, not how. We never depend on
them. Open systems give us inspectable baselines.

---

## Open System Catalog

### Tier 1: Game Engines (architecture benchmarks)

| System | License | Why We Study It | Benchmark Target | Evolution Plan |
|--------|---------|-----------------|------------------|----------------|
| **Bevy** | MIT/Apache-2.0 | Rust-native ECS, wgpu, modular, data-driven — closest to our primal composition model | ECS throughput (entities/tick), frame time, plugin load | Study ECS patterns → inform `game::state` entity model; no direct dependency |
| **Godot 4** | MIT | Full engine, community-governed, GDScript/C#/GDExtension — reference monolith | Scene tree update time, physics step, GDScript vs native perf | Design archaeology: what a monolith looks like that we're decomposing into primals |
| **Fyrox** | MIT | Rust, scene editor, complete 3D/2D — alternative Rust engine data point | Editor responsiveness, scene serialization | Compare scene representation vs our SceneGraph |
| **Raylib** | zlib | C, minimal, perfect rendering primitive | Draw call overhead, input latency | petalTongue comparison: minimal vs maximal rendering |
| **Love2D** | zlib | Lua, 2D, jam-developer favorite — Track 10 "one-person creation" target | Time-to-prototype, API surface area | Informs indie tooling goals |

### Tier 2: Networking / Multiplayer (Songbird benchmarks)

| System | License | Why We Study It | Benchmark Target | Evolution Plan |
|--------|---------|-----------------|------------------|----------------|
| **Nakama** | Apache-2.0 | Open matchmaking, leaderboards, chat, storage, auth — most complete open game backend | Match join latency, concurrent connections, state sync throughput | Scaffold: use Nakama as reference server → evolve to Songbird+NestGate chimera |
| **Quilkin** | Apache-2.0 | Google's UDP proxy for game servers — latency-aware routing | Relay overhead (μs), packet loss under load | Songbird relay optimization benchmark |
| **Matchbox** | MIT/Apache-2.0 | Rust WebRTC matchmaking for Bevy — closest Rust-native networking | Peer connection time, NAT traversal success rate | Direct Rust reference for Songbird matchmaking |
| **Naia** | MIT/Apache-2.0 | Rust netcode with client prediction, server authority | State sync accuracy, rollback overhead | Inform ToadStool deterministic execution |

### Tier 3: Social / Community (BirdSong benchmarks)

| System | License | Why We Study It | Benchmark Target | Evolution Plan |
|--------|---------|-----------------|------------------|----------------|
| **Matrix (Conduit)** | Apache-2.0 | Federated chat, E2E encrypted — Rust server (Conduit) | Message delivery latency, federation sync time, memory per room | BirdSong protocol comparison; federated room model |
| **Revolt** | AGPL-3.0 | Discord-like, self-hosted — AGPL matches our license | Feature parity checklist vs Discord, UI response time | Direct chimera reference for sovereign Discord |
| **Mumble** | BSD-3 | Low-latency voice, positional audio | Voice latency, spatial audio accuracy | petalTongue SpatialAudioCompiler benchmark |
| **Jitsi** | Apache-2.0 | WebRTC video/voice, self-hosted | Call setup time, bandwidth per participant | Voice chimera baseline |

### Tier 4: Distribution / Storefront (NestGate benchmarks)

| System | License | Why We Study It | Benchmark Target | Evolution Plan |
|--------|---------|-----------------|------------------|----------------|
| **Modrinth** | AGPL-3.0 | Minecraft mod hosting, open API, creator-first, AGPL | Upload/download throughput, search latency, API design | Workshop chimera baseline — perfect ethos match |
| **Flatpak** | LGPL | Sandboxed app distribution, reproducible builds | Install time, sandbox overhead, delta updates | Sovereign distribution via NestGate content-addressed storage |
| **F-Droid** | AGPL-3.0 | Reproducible builds, no tracking, community-audited | Build reproducibility rate, trust verification time | Trust model for primal distribution |

### Tier 5: Anti-Patterns (what to avoid)

| System | Problem | What We Learn |
|--------|---------|---------------|
| **Steam** (Valve) | Vendor lock-in, 30% tax, opaque algorithms, DRM | Capability spec only: what features matter to users |
| **Discord** (proprietary) | Surveillance capitalism, centralized, no data export | Capability spec only: what social features users expect |
| **Unity** (proprietary) | Runtime fee fiasco, license instability | Why sovereignty matters: your tools shouldn't change terms |
| **Unreal** (Epic) | Monolithic, C++ ecosystem lock-in, royalty model | Why composition > monolith |
| **CurseForge** (Overwolf) | Creator-hostile changes, ad injection, data harvesting | Why open workshop federation matters |

---

## Benchmark Methodology

### Categories

Every open system benchmark measures the same five axes:

| Axis | What We Measure | Tool |
|------|----------------|------|
| **Latency** | Time to complete operation (p50, p95, p99) | `criterion` (Rust), custom harness |
| **Throughput** | Operations per second at steady state | `criterion`, sustained load tests |
| **Correctness** | Bit-exact or tolerance-bounded match | `proptest`, analytical validation |
| **Resource** | Memory, CPU, GPU utilization at load | `/proc`, `gpu-allocator`, `jemalloc-ctl` |
| **Sovereignty** | Dependencies, licenses, network requirements | Static analysis, `cargo-deny` |

### Scaffold → Evolve Protocol

For each open system we scaffold from:

```
Phase 0: STUDY     — Read source, document architecture, identify benchmarks
Phase 1: SCAFFOLD  — Use open system as subprocess or FFI for integration tests
Phase 2: BENCHMARK — Measure open system's performance as our baseline target
Phase 3: IMPLEMENT — Build pure Rust equivalent in ludoSpring/barracuda
Phase 4: VALIDATE  — Our implementation meets or exceeds benchmark baseline
Phase 5: SHED      — Remove scaffold dependency; open system becomes test oracle
```

Each phase produces a handoff document in `wateringHole/handoffs/`.

### Benchmark Crate Structure

```
ludoSpring/
  benchmarks/
    Cargo.toml          # workspace member, criterion + optional scaffold deps
    src/
      lib.rs
      bevy_ecs.rs       # ECS throughput comparison
      nakama_match.rs   # Matchmaking latency comparison
      conduit_chat.rs   # Federated chat comparison
      noise_field.rs    # Perlin/fBm vs GPU noise throughput
      raycaster.rs      # Our DDA vs reference implementations
    benches/
      ecs_throughput.rs
      match_latency.rs
      noise_throughput.rs
```

---

## Immediate Benchmarks (Phase 0–2)

### BM-001: ECS Throughput (Bevy comparison)

**What**: Entity iteration, component access, system scheduling overhead.
**Why**: Our `game::state` tick model must handle 10K+ entities at 60 Hz.
**Open baseline**: Bevy `Schedule` with 10K entities, 5 systems.
**Our target**: Match or exceed with primal composition overhead < 1ms.
**Phase**: 0 (study) — read Bevy's `bevy_ecs` crate architecture.

### BM-002: Noise Field Throughput

**What**: 1024x1024 Perlin fBm generation time (CPU and GPU).
**Why**: Procedural generation must not stall the game loop.
**Open baseline**: `noise-rs` crate, `FastNoiseLite` (C), Bevy `bevy_noise`.
**Our target**: CPU within 2x of `noise-rs`; GPU within 1.5x of `FastNoiseLite`.
**Phase**: 2 (benchmark) — we already have a Perlin/fBm implementation.

### BM-003: Raycaster Throughput

**What**: 320-column screen cast at 60 Hz.
**Why**: First-person spatial navigation is our primary genre.
**Open baseline**: Lodev DDA reference (C), `raylib` built-in raycaster.
**Our target**: Match C implementation within 1.5x on CPU.
**Phase**: 2 (benchmark) — we already have a raycaster implementation.

### BM-004: Matchmaking Latency (Nakama comparison)

**What**: Time from queue join to match start, at 100/1000/10000 concurrent.
**Open baseline**: Nakama (Go) matchmaker.
**Our target**: Songbird chimera matches Nakama latency with E2E encryption.
**Phase**: 0 (study) — document Nakama's matchmaker API and protocol.

### BM-005: Federated Chat Latency (Conduit comparison)

**What**: Message delivery time across federated rooms.
**Open baseline**: Conduit (Rust Matrix server).
**Our target**: BirdSong mesh matches Conduit with genetic-lineage auth.
**Phase**: 0 (study) — Conduit is Rust, direct architectural comparison.

---

## Dependency Evolution Tracker

| Scaffold Dep | Current Phase | Pure Rust Target | Blocks |
|-------------|---------------|------------------|--------|
| `noise-rs` | Phase 0 | `procedural::noise` (done) | GPU evolution |
| `bevy_ecs` (study only) | Phase 0 | `game::state` entity model | Architecture decision |
| Nakama (study only) | Phase 0 | Songbird+NestGate chimera | Songbird matchmaking |
| Conduit (study only) | Phase 0 | BirdSong federated rooms | Songbird evolution |
| Modrinth API (study only) | Phase 0 | NestGate workshop chimera | NestGate content-addressed storage |
| `matchbox` | Phase 0 | Songbird WebRTC relay | NAT traversal |

---

## Success Criteria

A benchmark is **passed** when:

1. Pure Rust implementation meets latency/throughput target
2. Correctness validated against open system as oracle
3. Scaffold dependency removed from production `Cargo.toml`
4. Benchmark remains in CI as regression guard
5. Handoff document published to `wateringHole/`

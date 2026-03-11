# Ludology Domain Specification

**Date**: March 10, 2026
**Status**: Active
**License**: AGPL-3.0-or-later

## Domain Definition

**Ludology** (from Latin *ludus*: play, game) is the formal study of games and play as systems. ludoSpring treats game design with the same scientific rigor as the other ecoPrimals springs:

- **wetSpring** validates bioinformatics against published papers
- **hotSpring** validates nuclear physics against experimental data
- **healthSpring** builds health applications from validated science
- **ludoSpring** validates game mechanics against empirical HCI research

## Scope

ludoSpring is a **science spring** that studies games — and the
**game_logic + metrics** primal within the game engine niche. It is
both the researcher and a live participant:

| Role | What ludoSpring Does |
|------|---------------------|
| Science spring | Validates interaction models against empirical research |
| Niche participant | Provides game_logic and metrics nodes in the continuous graph |
| Benchmark oracle | Measures open systems, defines performance targets |
| Evolution driver | Scaffolds from open systems, evolves to pure sovereign Rust |

ludoSpring's outputs feed every primal that has a user interface. petalTongue consumes ludoSpring's interaction models. hotSpring consumes ludoSpring's difficulty curves for physics tutorials. wetSpring consumes ludoSpring's procedural generation for molecular exploration.

## Foundational Research

### Input Science (validated models)

| Model | Source | What it predicts |
|-------|--------|-----------------|
| Fitts's law | Fitts (1954), MacKenzie (1992) | Movement time = a + b·log₂(2D/W + 1) |
| Hick's law | Hick (1952), Hyman (1953) | Decision time = a + b·log₂(N + 1) |
| Steering law | Accot & Zhai (1997) | Tunnel navigation time = a + b·(D/W) |
| GOMS | Card, Moran, Newell (1983) | Task completion time from operator sequence |

### Flow and Engagement

| Model | Source | What it predicts |
|-------|--------|-----------------|
| Flow theory | Csikszentmihalyi (1990) | Optimal experience when challenge ≈ skill |
| Dynamic difficulty | Hunicke (2005) | Player retention under adaptive challenge |
| Four keys to fun | Lazzaro (2004) | Hard fun, easy fun, people fun, serious fun |
| Engagement metrics | Yannakakis & Togelius (2018) | Action density, exploration breadth, persistence |

### Procedural Content Generation

| Algorithm | Source | Application |
|-----------|--------|------------|
| Perlin noise | Perlin (1985, 2002) | Terrain, density fields, molecular distributions |
| Wave function collapse | Gumin (2016) | Valid structure generation under adjacency rules |
| L-systems | Lindenmayer (1968) | Biological growth patterns, protein backbones |
| BSP trees | Fuchs, Kedem, Naylor (1980) | Spatial partitioning, Doom-style level layouts |

### Information Design (Tufte on Games)

| Principle | Tufte source | Game application |
|-----------|-------------|-----------------|
| Data-ink ratio | Tufte (1983) | HUD chrome vs information content |
| Chartjunk | Tufte (1983) | Decorative UI elements that obscure gameplay |
| Small multiples | Tufte (1990) | Minimap as small-multiple of main view |
| Lie factor | Tufte (1983) | Visual size ≠ game-mechanical magnitude |

## Experiment Agenda

### Track 1: Reference Analysis (Exp001–010)

Study existing games as interaction architectures:

- **Exp001**: Doom raycaster — DDA validation, HUD Tufte analysis, targeting cost
- **Exp002**: Procedural molecule generation — noise fields → chemistry voxels
- **Exp003**: Genre UI comparison — FPS vs RTS vs sandbox through Tufte lens
- **Exp004**: Folding adversarial — player vs AI with DDA and flow tracking

### Track 2: Interaction Models (Exp011–020)

Validate and extend HCI models for game-like interactions:

- Fitts's law under different input devices (mouse, gamepad, gaze, voice)
- Hick's law with hierarchical menus vs radial menus vs voice commands
- Flow channel width calibration across age groups and ability levels

### Track 3: Procedural Generation (Exp021–030)

Bridge PCG algorithms to scientific domain generation:

- Noise-driven molecular density fields
- WFC for valid crystal lattice generation
- L-system protein backbone growth
- Hybrid: noise + WFC for chemically valid voxel worlds

### Track 4: Accessibility Science (Exp031–040)

Quantify universal playability:

- Blind-player navigation via spatial audio (HRTF models)
- Motor-limited input (Fitts's law with restricted devices)
- Cognitive load measurement under different UI complexities
- Cross-species interface design (dolphin click-pattern games)

### Track 5: Fun Metrics (Exp041–050)

Measure what makes science exploration engaging:

- Engagement curves across molecular exploration sessions
- Difficulty-skill balance in protein folding challenges
- Tufte constraint sweeps across game UI configurations
- Retention modeling under different reward structures

## Open System References

ludoSpring studies open systems to establish benchmarks before building
sovereign equivalents. Proprietary systems are studied only as capability
specifications — what users need, not how vendors build it.

See `specs/OPEN_SYSTEMS_BENCHMARK_SPECIFICATION.md` for the full catalog.

### Good Design (models to study)

| System | License | Design Lesson |
|--------|---------|---------------|
| **Doom (1993)** | GPL-2.0 | Minimal HUD (high data-ink ratio), sub-frame input, DDA raycasting, BSP spatial partitioning |
| **NetHack** | NGPL | 40 years of procedural generation from simple rules, emergent complexity |
| **Dwarf Fortress** | proprietary | Simulation depth → emergent narrative without graphical budget |
| **Bevy** | MIT/Apache-2.0 | Rust-native ECS, data-driven, modular — philosophical match for primal composition |
| **Godot 4** | MIT | Community-governed full engine — the "good monolith" we decompose |
| **Revolt** | AGPL-3.0 | Sovereign Discord alternative — self-hosted, federated-capable |
| **Matrix/Conduit** | Apache-2.0 | Federated E2E chat — Rust server, aligns with BirdSong model |
| **Modrinth** | AGPL-3.0 | Creator-first mod hosting — AGPL, open API, community-audited |
| **Nakama** | Apache-2.0 | Open game backend — matchmaking, leaderboards, chat, storage |
| **Mumble** | BSD-3 | Low-latency voice with positional audio — spatial audio baseline |
| **Matchbox** | MIT/Apache-2.0 | Rust WebRTC matchmaking for Bevy — closest Rust-native networking |

### Anti-Patterns (what we study and replace)

The anti-patterns are not "platforms charging money." Steam earns its
30% by providing a genuinely high-quality service equitably. The anti-
patterns are extractive mechanics that degrade craft and exploit players.

| Anti-Pattern | Where We See It | Why It Fails | Our Alternative |
|-------------|----------------|-------------|-----------------|
| **Microtransactions** | Mobile games, AAA live-service | Every system becomes a monetization surface; progression designed to frustrate, not engage | Flow science: engagement measures genuine satisfaction |
| **Gambling / loot boxes** | Gacha, "surprise mechanics" | Slot machines targeting vulnerable players; compulsion, not fun | DDA + flow: optimize for Csikszentmihalyi channel, detect compulsion loops |
| **Low-quality churn** | Annual franchises, asset flips | Volume over craft when creators don't own what they make | Radiating Attribution: crafters retain proportional value |
| **Surveillance capitalism** | Discord, most free-to-play | Users are the product, data harvested for ad targeting | BearDog genetic lineage: you control your identity |
| **Creator-hostile platforms** | CurseForge (Overwolf), YouTube content ID | Ad injection, false claims, value extraction from creators | Federated workshop: Modrinth model via NestGate |
| **Chartjunk UI** | Mobile games, bloated AAA HUDs | Decorative noise obscures information; UI becomes ad surface | Tufte constraints: data-ink ratio, lie factor analysis |
| **Monolithic lock-in** | Unity runtime fees, Unreal royalties | Terms change; creators lose sovereignty retroactively | biomeOS graphs: swap any node, own every layer |

### Good Patterns (what we study and federate)

| Pattern | Where We See It | What Works | How We Extend It |
|---------|----------------|-----------|------------------|
| **Equitable curation** | Steam | Indie games compete on quality, not marketing budget | Engagement-based discovery ranking (ludoSpring metrics) |
| **Workshop ecosystem** | Steam Workshop | Modding communities thrive with creator tools | Federate via NestGate so mods survive server shutdowns |
| **Community reviews** | Steam reviews | Player-driven quality signal, "mostly positive" means something | Enhance with engagement science, detect review manipulation |
| **Indie empowerment** | Steam, itch.io | Small teams producing craft that outperforms corporate studios | Radiating Attribution ensures creators retain value long-term |
| **Open modding** | Modrinth, Thunderstore | Community-audited, AGPL, creator-first | Sovereign workshop chimera with provenance tracking |

---

## Integration Points

### petalTongue

ludoSpring provides:
- Fitts/Hick models for UI layout optimization
- Flow models for difficulty-adaptive visualization tutorials
- Tufte-on-games analysis for petalTongue's own panels
- Voxel world representation for 3D molecular exploration
- Engagement metrics for proprioception (how engaged is the user?)

### Other Springs

ludoSpring provides:
- Difficulty curves for any spring's educational scenarios
- Procedural generation algorithms for domain-specific worlds
- Accessibility scoring for any primal's user interface
- Interaction cost models for IPC API design (even CLIs have Hick's law)

### barraCuda

ludoSpring evolves:
- Noise generation shaders (Perlin/simplex on GPU)
- Collision broadphase (spatial hashing)
- Raycasting (parallel ray-grid intersection)
- WFC parallel constraint propagation

### biomeOS

ludoSpring participates in:
- **Deploy graph**: `ludospring_deploy.toml` (Tower → ToadStool → ludoSpring)
- **Continuous niche**: game_logic + metrics nodes at 60 Hz
- **Chimera composition**: Senses, Brain, Gaming-Mesh chimeras
- **Capability registry**: 8 `game.*` methods routed by Neural API

### Live Primal Binaries

During development, ludoSpring runs alongside live primal binaries from
the ecoPrimals build tree. Primals are discovered at runtime via
capability-based routing — no hardcoded paths or names:

```
ludoSpring discovers primals by capability:
  "security"  → BearDog (wherever it lives)
  "discovery" → Songbird (wherever it lives)
  "compute"   → ToadStool (wherever it lives)
  "storage"   → NestGate (wherever it lives)
```

---

## Platform Chimera Roadmap

ludoSpring contributes game-science intelligence to platform chimeras that
replace proprietary platforms with sovereign alternatives:

| Platform | Chimera | ludoSpring Contribution |
|----------|---------|----------------------|
| **Steam** | Storefront + Workshop + Gaming-Mesh | Engagement metrics, difficulty curves, accessibility scoring |
| **Discord** | Social + Voice + Federation | Flow-aware presence, engagement-driven room suggestions |
| **Twitch** | Streaming + Metrics + Social | Real-time engagement analysis, Tufte overlay scoring |

See `specs/PLATFORM_CHIMERA_SPECIFICATION.md` for the full decomposition.

---

## Primal Identity

- **Name**: ludospring
- **Domain**: game science, ludology, interaction design
- **Capabilities**: `game.*` (analyze_ui, evaluate_flow, fitts_cost, engagement, generate_noise, wfc_step, accessibility, difficulty_adjustment)
- **Socket**: `ludospring-{FAMILY_ID}.sock`
- **Bonding**: Covalent within NUCLEUS, Ionic for external API consumers
- **License**: AGPL-3.0-or-later

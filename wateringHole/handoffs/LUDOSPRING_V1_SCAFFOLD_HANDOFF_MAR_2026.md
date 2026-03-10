# ludoSpring V1 Scaffold Handoff — March 2026

## What was built

Initial scaffold of ludoSpring, the seventh ecoPrimals spring.

### Crates
- `ludospring-barracuda` — core library with 4 domain modules
- `ludospring-forge` — hardware dispatch for game science workloads
- 4 experiment crates (Exp001–004)

### Domain Modules
- `game` — state machines, genre taxonomy, raycaster (DDA), voxel worlds with chemistry palette
- `interaction` — Fitts's law, Hick's law, steering law, flow state model, DDA, accessibility scoring
- `procedural` — Perlin noise (2D/3D), fBm, wave function collapse
- `metrics` — Tufte-on-games UI analysis, engagement metrics

### Test Coverage
- 45 tests across all modules
- All passing, zero clippy warnings

### Experiments
- Exp001: Doom raycaster analysis (DDA validation, HUD Tufte, Fitts targeting)
- Exp002: Procedural molecule generation (noise→voxel chemistry world)
- Exp003: Genre UI Tufte comparison (Doom vs Minecraft vs RTS)
- Exp004: Folding adversarial (player vs AI with DDA and flow)

## biomeOS Integration
- Capability domain: `game` with 7 capabilities
- Deploy graph: `ludospring_deploy.toml`
- Capability registry entries added
- petalTongue scenario loader updated with `ludospring` → `game` domain

## Game Engine Niche Architecture (added March 10, 2026)

Following a review of `whitePaper/neuralAPI/`, `whitePaper/RootPulse/`, and
`phase2/biomeOS/` graph deployments, the architectural design for the game
engine as the **first continuous niche** has been documented:

### Key finding

The game engine is NOT a new primal. It is a biomeOS coordination graph that
composes petalTongue, barraCuda, toadStool, ludoSpring, and optionally science
springs into a 60 Hz real-time loop.

### Critical blocker

biomeOS currently supports only run-to-completion coordination modes
(Sequential, Parallel, ConditionalDAG, Pipeline). The game engine requires a
fifth mode: **Continuous** — a graph that ticks repeatedly with feedback edges
between iterations.

### Documents produced

| Document | Location | Purpose |
|----------|----------|---------|
| Niche API Patterns | `whitePaper/neuralAPI/08_NICHE_API_PATTERNS.md` | Whitepaper defining continuous niches, comparing to RootPulse, proposing game engine as first implementation |
| Game Engine Niche Spec | `ludoSpring/specs/GAME_ENGINE_NICHE_SPECIFICATION.md` | ludoSpring's role, interfaces, genre configurations, phased reference implementations, budget allocation |
| biomeOS Handoff | `phase2/biomeOS/wateringHole/handoffs/CONTINUOUS_COORDINATION_REQUIREMENTS_MAR_2026.md` | Exact code changes needed in biomeOS, files to modify/create, dependency chain, testing strategy |

### ludoSpring's role in the niche

ludoSpring occupies two graph nodes:
1. **game_logic** — receives sensor events, updates game state, triggers PCG,
   applies difficulty adjustment
2. **metrics** — computes engagement, evaluates flow, scores accessibility,
   feeds back to game_logic via next-tick edge

### Phased delivery

| Phase | What | Primal count |
|-------|------|-------------|
| A: Walk through atoms | Static molecular world, first-person navigation | 4 |
| B: Hear the molecule | Add spatial audio | 4 (existing primals) |
| C: Touch a reaction | Live wetSpring simulation streaming to scene | 5 |
| D: Curriculum as world | biomeOS-orchestrated learning sequences | 6+ |

### Learning signal

The game engine gives Neural API's Pathway Learner ~5M frames/day of
optimization signal, compared to ~100 ops/day from RootPulse. This 50,000x
richer signal is the strongest argument for building the game engine first.

## What's next
- biomeOS: Implement Continuous coordination mode (critical path)
- ludoSpring: IPC server implementation (JSON-RPC methods)
- ludoSpring: GPU shader evolution (noise generation, raycasting on GPU)
- ludoSpring: Tracks 2–10 experiments (50 planned)
- ludoSpring: exp005 — continuous niche validation experiment
- petalTongue: SensorEvent streaming interface for game input
- Integration: Run game_engine_tick.toml through ContinuousExecutor

# Game Engine Niche Specification

## Overview

The game engine is the first **continuous niche** in the ecoPrimals ecosystem.
It is not a primal. It is a coordination pattern — a biomeOS graph that ticks
at 60 Hz, composing existing primals into an interactive real-time system.

This specification defines ludoSpring's role within that niche and the
interfaces it must provide.

See also: `whitePaper/neuralAPI/08_NICHE_API_PATTERNS.md` for the full
architectural design of continuous niches.

## ludoSpring's Role

ludoSpring is the **game science primal**. Within the game engine niche, it
occupies two graph nodes:

1. **game_logic** — receives sensor events, updates game state, applies
   difficulty adjustment, triggers procedural generation
2. **metrics** — observes player behavior, computes engagement scores,
   evaluates flow state, scores accessibility, feeds back to game_logic
   via a next-tick edge

ludoSpring does NOT:
- Render frames (petalTongue + toadStool)
- Run physics (barraCuda + toadStool)
- Synthesize audio (petalTongue)
- Handle networking (Songbird)

## Required Interfaces

### game_logic node

**Input**: `SensorEvent[]` from petalTongue's interaction engine

**Output**: `GameState`

```rust
pub struct GameState {
    /// Current session phase.
    pub phase: SessionPhase,
    /// Active game entities with positions, types, state.
    pub entities: Vec<EntityUpdate>,
    /// Procedural generation requests (new chunks, structures).
    pub generation_requests: Vec<GenerationRequest>,
    /// Difficulty parameters for this tick.
    pub difficulty: DifficultyParams,
    /// Commands for the physics node.
    pub physics_commands: Vec<PhysicsCommand>,
}
```

**Capabilities called**:
- `game.evaluate_flow` — determine if player is in flow
- `game.difficulty_adjustment` — adapt challenge to skill
- `game.generate_noise` / `game.wfc_step` — when generation_requests
  trigger new world content

### metrics node

**Input**: `SensorEvent[]` (from input node), `GameState` (from game_logic)

**Output**: `EngagementReport`

```rust
pub struct EngagementReport {
    /// Current flow state.
    pub flow: FlowState,
    /// Actions per minute.
    pub actions_per_minute: f64,
    /// Exploration breadth this session.
    pub exploration_breadth: u32,
    /// Suggested difficulty adjustment (-1.0 to 1.0).
    pub difficulty_adjustment: f64,
    /// Accessibility dimension scores.
    pub accessibility: AccessibilityReport,
    /// Frame budget utilization (for Neural API learning).
    pub frame_budget_ms: f64,
}
```

**Feedback edge**: `metrics.difficulty_adjustment` feeds into game_logic's
next tick as an input, closing the DDA loop.

## Genre Configurations

The game engine niche is parameterized by genre. ludoSpring provides the
taxonomy (game::genre) and the metrics appropriate to each:

### First-Person Spatial (molecular explorer, particle cave)

```toml
[genre]
architecture = "FirstPersonSpatial"
default_fov = 60.0
mouse_sensitivity = 2.0
movement_speed = 3.0

[metrics]
primary = ["exploration_rate", "deliberation"]
fitts_device = "mouse"
target_success_rate = 0.75
```

### Top-Down Command (systems biology dashboard)

```toml
[genre]
architecture = "TopDownCommand"
selection_mode = "box"
camera_mode = "isometric"

[metrics]
primary = ["actions_per_minute", "challenge_appetite"]
hick_menu_depth = 3
target_success_rate = 0.65
```

### Sandbox (molecule builder)

```toml
[genre]
architecture = "Sandbox"
block_palette = "chemistry"
generation_algorithm = "noise_wfc_hybrid"

[metrics]
primary = ["exploration_breadth", "deliberation"]
target_success_rate = 0.80
```

## Primal Evolution Requirements

For the game engine niche to be fully operational, each participating primal
needs specific evolutions. ludoSpring's responsibility is to provide the
validated models that inform these evolutions.

### What ludoSpring provides to other primals

| Primal | What ludoSpring provides | Interface |
|--------|-------------------------|-----------|
| petalTongue | Fitts/Hick models for UI layout, accessibility scoring | `game.fitts_cost`, `game.accessibility` |
| petalTongue | Flow models for tutorial pacing | `game.evaluate_flow` |
| barraCuda | Collision broadphase validation data | Experiment results |
| toadStool | Frame budget allocation recommendations | `metrics.frame_budget_ms` |
| biomeOS | Engagement signal for Neural API learning | `EngagementReport` |
| Springs | Difficulty curves for educational scenarios | `game.difficulty_adjustment` |

### What ludoSpring needs from other primals

| Primal | What ludoSpring needs | Interface |
|--------|----------------------|-----------|
| petalTongue | SensorEvents (player input) | JSON-RPC notification stream |
| petalTongue | RenderPlan (what was shown) | For proprioception |
| barraCuda | Physics state (positions, collisions) | JSON-RPC response |
| biomeOS | Tick clock signal | Continuous coordinator |

## Experiment Mapping

Each ludoSpring experiment track maps to a component of the game engine niche:

| Track | Niche component | What it validates |
|-------|----------------|-------------------|
| Track 1: Reference analysis | Genre configurations | Doom, Minecraft, Stardew interaction patterns |
| Track 2: Input science | input node | Fitts/Hick predictions vs observed latency |
| Track 3: Flow & engagement | metrics node | Flow channel calibration, DDA effectiveness |
| Track 4: Procedural generation | game_logic node | Noise/WFC quality for domain worlds |
| Track 5: Accessibility | metrics node | Accessibility scoring vs player outcomes |
| Track 6: Living worlds | game_logic + springs | Mycorrhizal network generation + simulation |
| Track 7: Spatial audio | petalTongue evolution | HRTF models vs localization accuracy |
| Track 8: Education through play | Full niche | KSP/FoldIt engagement vs learning outcomes |
| Track 9: Tufte-on-games | metrics node | UI Tufte scores vs player performance |
| Track 10: Indie tooling | Niche API | What enables one-person creation |

## Reference Implementations

### Phase A: Walk through atoms

The minimum viable game engine niche:

```
input (petalTongue) → game_logic (ludoSpring: first-person controller)
                    → physics (barraCuda: Euler integration)
                    → scene (petalTongue: SceneGraph with molecules)
                    → render (toadStool: wgpu perspective projection)
```

Primal count: 4. Tick rate: 60 Hz. World: static molecular structure.
Player capability: walk, look, inspect.

### Phase B: Hear the molecule

Add spatial audio:

```
scene → audio (petalTongue: SpatialAudioCompiler → toadStool: audio device)
```

Primal count: 4 (audio uses existing primals). New capability: spatial
audio positioning, element-to-timbre mapping.

### Phase C: Touch a reaction

Add live science simulation:

```
game_logic → wetSpring (Gillespie stochastic simulation, streaming)
           → scene (new particles from reactions)
           → metrics (engagement during reaction cascades)
```

Primal count: 5. New capability: live data binding from springs to scene.

### Phase D: Curriculum as world

Add orchestrated learning sequences:

```
biomeOS deploys lesson graph → game_logic (ludoSpring: difficulty curve)
                              → metrics (ludoSpring: engagement + learning)
                              → neuralSpring (validate learner predictions)
```

Primal count: 6+. New capability: biomeOS graph sequencing for educational
progression.

## Budget Allocation

Target: 16.67ms per tick (60 Hz)

| Node | Budget | Rationale |
|------|--------|-----------|
| input | 1ms | Event polling, negligible |
| game_logic | 3ms | State machine + DDA evaluation |
| physics | 4ms | Collision + integration (or offload to GPU) |
| scene | 2ms | Scene graph update, transform composition |
| render | 6ms | GPU submission + present (async) |
| audio | 2ms | HRTF convolution + mix (can overlap render) |
| metrics | 1ms | Engagement snapshot, flow eval |
| **Total** | **19ms** | Over budget — audio overlaps render: **effective 17ms** |

The 0.33ms of slack is where the Neural API's Pathway Learner finds
optimization opportunities over time.

## Relationship to RootPulse

The game engine niche and RootPulse niche share architectural DNA:

| Concept | RootPulse | Game Engine |
|---------|-----------|-------------|
| "Pulse is what primals DO together" | Version control emerges | Real-time simulation emerges |
| Two-tier temporal model | rhizoCrypt (present) + LoamSpine (past) | Tick buffer (present) + replay (past) |
| Dehydration | Ephemeral DAG → permanent log | Session → save file |
| Attribution | SweetGrass semantic braids | Engagement tracking per player |
| Sovereignty | You own your code | You own your world |
| Federation | Songbird discovery | Songbird multiplayer |

Both niches validate the same architectural thesis: complex capabilities
emerge from simple primal coordination, not from monolithic construction.
The game engine tests it under the strictest real-time constraints.

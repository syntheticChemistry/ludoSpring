# Ludology Domain Specification

## Domain Definition

**Ludology** (from Latin *ludus*: play, game) is the formal study of games and play as systems. ludoSpring treats game design with the same scientific rigor as the other ecoPrimals springs:

- **wetSpring** validates bioinformatics against published papers
- **hotSpring** validates nuclear physics against experimental data
- **healthSpring** builds health applications from validated science
- **ludoSpring** validates game mechanics against empirical HCI research

## Scope

ludoSpring is NOT a game engine. It is a **science spring** that studies games. The difference:

| Game engine | ludoSpring |
|-------------|-----------|
| Renders frames | Measures rendering costs |
| Handles input | Models input latency (Fitts, Hick, steering) |
| Plays audio | Quantifies spatial audio accessibility |
| Runs physics | Validates physics integrators |
| Ships games | Ships validated interaction models |

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

## Primal Identity

- **Name**: ludospring
- **Domain**: game science, ludology, interaction design
- **Capabilities**: `game.*` (analyze_ui, evaluate_flow, fitts_cost, engagement, generate_noise, wfc_step, accessibility)
- **Socket**: `ludospring-{FAMILY_ID}.sock`
- **License**: AGPL-3.0-or-later

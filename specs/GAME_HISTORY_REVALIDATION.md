# Game History Revalidation — Rebuilding Games from First Principles

**Date**: March 18, 2026
**Status**: Active — 6 experiments implemented (exp076-081)
**License**: AGPL-3.0-or-later
**Pattern**: hotSpring started with MD (molecular dynamics) before QCD. We start
with Pong before Disco Elysium. Each era validates the math for the next.
**Depends on**: `GAME_QUALITY_PROFILES.md`, `CROSS_DOMAIN_LEARNING.md`,
`LYSOGENY_CATALOG.md`

---

## The Principle

hotSpring replicated the literal history of computational physics:

```
MD (1957) → Monte Carlo (1953) → Lattice QCD (1974) → Full QCD (ongoing)
```

Each step validated the math needed for the next. MD proved force
integration. Monte Carlo proved statistical sampling. Lattice QCD proved
gauge field discretization. You cannot skip to QCD without MD.

ludoSpring does the same with games:

```
Pong (1972) → Tetris (1984) → Doom (1993) → MTG (1993) → Civilization (1991)
→ NetHack (1987) → Diablo (1996) → Half-Life (1998) → Deus Ex (2000)
→ Dwarf Fortress (2006) → Minecraft (2011) → Slay the Spire (2019)
→ Disco Elysium (2019) → RPGPT (our target)
```

Each game is an era. Each era validates a set of models that the next
era depends on. You cannot build RPGPT without validating Pong, because
Pong is where Fitts's law, Flow, and real-time input begin.

---

## The Modality Insight: TUI Pong vs VR Pong

Every game in the history is built modality-agnostic. The game logic is
pure math. The rendering is a swappable petalTongue binding. This proves
that the game IS the math, not the pixels.

**Pong as the proof:**

| Layer | TUI (ratatui) | GUI (egui) | VR (petalTongue 3D) | Headless (test) |
|-------|--------------|-----------|-------------------|----------------|
| Ball physics | `position += velocity * dt` | same | same | same |
| Paddle input | Arrow keys → `paddle.y += dy` | Mouse → `paddle.y = cursor.y` | Controller → `paddle.y = hand.y` | Scripted input |
| Collision | `if ball.x <= paddle.x && ball.y in paddle_range` | same | same + z-depth | same |
| Score | `score += 1` | same | same | same |
| Flow state | `evaluate_flow(challenge, skill)` | same | same | same |
| Fitts cost | `fitts_movement_time(paddle_dist, ball_size)` | same | same + depth | same |
| Rendering | ASCII characters on terminal grid | 2D shapes on canvas | 3D meshes in space | No rendering |

**The game logic is identical across all four modalities.** The only thing
that changes is how petalTongue translates game state to pixels (or
characters, or meshes, or nothing). This is the architectural proof that
games are math, not graphics.

What TUI Pong teaches that VR Pong can't: **you can feel Flow in
ASCII.** If the game math is right, the experience is right. Graphics
are a modality, not a requirement. This is why ludoSpring's TUI games
(exp024, exp025) work — the math creates the experience, not the
renderer.

What VR Pong teaches that TUI Pong can't: **Fitts's law changes in 3D.**
Target acquisition in depth adds a dimension. Steering law through 3D
tunnels has different constants. The HCI models need revalidation per
modality, even though the game logic is unchanged.

---

## Era 0: Pre-Electronic (validated math, no game yet)

These are the scientific foundations. No game exists yet, but the math
that will power every game is already published.

| Year | Model | Publication | What It Becomes |
|------|-------|------------|-----------------|
| 1687 | Newtonian mechanics | Newton, *Principia* | Every physics engine. Ball trajectories. Projectile motion. Gravity |
| 1738 | Expected value / probability | Bernoulli, D. | Every dice system. Every loot table. Every random event |
| 1855 | Diffusion | Fick, A. | Information spread. Fog of war propagation. Heat maps |
| 1906 | Markov chains | Markov, A.A. | State machines. AI behavior. Procedural narrative |
| 1928 | Game theory | von Neumann, J. | Every competitive game. Minimax. Nash equilibria |
| 1948 | Information theory | Shannon, C.E. | Deckbuilding entropy. Compression. Procedural generation |
| 1952 | Hick's law | Hick, W.E. | Menu design. Decision complexity. Information overload |
| 1954 | Fitts's law | Fitts, P.M. | Target acquisition. UI layout. Cursor movement |

**ludoSpring status**: All validated in barraCuda primitives. Python
baselines confirmed (exp005, exp006, exp011).

---

## Era 1: The Origin — Pong (1972)

**What Pong validates:**

| Model | How Pong Uses It | Experiment |
|-------|-----------------|------------|
| Newtonian mechanics | Ball velocity, reflection angle, constant speed | **NEW**: exp076 |
| Fitts's law | Paddle interception — target is the ball, width is the paddle. Fitts predicts interception difficulty | **NEW**: exp076 |
| Flow (Csikszentmihalyi) | Ball speed increases → challenge rises with skill. The original DDA | **NEW**: exp076 |
| DDA (Hunicke) | Speed ramp IS difficulty adaptation. Pong invented DDA before the term existed | **NEW**: exp076 |
| Real-time input loop | 60Hz game loop: read input → update state → render. The pattern every game inherits | **NEW**: exp076 |

**Modalities**: TUI (ratatui), GUI (egui), headless (validation)

**What we learn for downstream**: The game loop. Input → update → render
at fixed timestep. Every game from here to RPGPT inherits this loop.
If we can't get Pong right, nothing downstream works.

**Cross-spring**: Ball physics is particle simulation. hotSpring MD
uses the same Verlet integration for atom positions that Pong uses
for ball position. Same math, different particle.

---

## Era 2: Space — Spacewar! (1962) / Asteroids (1979)

**What space games validate:**

| Model | How It's Used | Experiment |
|-------|-------------|------------|
| 2D vector physics | Thrust, inertia, rotation. No grid — continuous space | **NEW**: exp077 |
| Wrap-around topology | Toroidal space — exit left, enter right. Topology as game mechanic | **NEW**: exp077 |
| Multi-body interaction | Two ships, bullets, asteroids. N-body problem in 2D | **NEW**: exp077 |
| Steering law (Accot & Zhai) | Navigating through asteroid fields is steering through a tunnel | exp007 (validated) |

**What we learn**: Continuous physics vs grid physics. Pong is on a
grid (paddle moves in integer steps). Spacewar is in continuous space
(thrust produces fractional velocities). This distinction matters for
every physics engine downstream.

**Cross-spring**: N-body simulation IS molecular dynamics. Asteroids
bouncing off screen edges IS periodic boundary conditions in MD.
hotSpring validates the same integration scheme.

---

## Era 3: Falling — Tetris (1984)

**What Tetris validates:**

| Model | How It's Used | Experiment |
|-------|-------------|------------|
| Flow | The defining Flow game. Speed increases with level. Challenge matches growing skill. Tetris IS Csikszentmihalyi's diagram | **NEW**: exp078 |
| Hick's law | 7 tetromino shapes. Which one next? Where does it go? Rotation × position = bounded decision space | exp006 (validated) |
| WFC adjacency | Tetris line clearing is constraint satisfaction. A full row satisfies the "complete" constraint | exp008 (connection) |
| Engagement | "One more line" loop. Session duration stretches without player noticing. Time distortion = Flow | exp010 (validated) |
| Procedural generation | Random piece sequence from bag. The 7-bag algorithm guarantees fairness (no drought > 12 pieces) | **NEW**: exp078 |

**Modalities**: TUI is the natural home. Tetris IS a terminal game.
GUI adds color. VR adds depth (3D Tetris = different game). Headless
validates bag fairness and scoring.

**What we learn**: The bag randomizer is the first fair procedural
generator in games. It guarantees bounded drought — you will never go
more than 12 pieces without seeing a specific shape. This is the seed
of procedural generation with fairness guarantees. Same principle as
balanced experimental design in science: ensure coverage across
conditions.

**Cross-spring**: The bag randomizer is sampling without replacement —
the same statistic as card draw (Shannon), sequencing read sampling
(wetSpring), and balanced block randomization in clinical trials
(healthSpring).

---

## Era 4: Dungeon — Rogue (1980) / NetHack (1987)

**What roguelikes validate:**

| Model | How It's Used | Experiment |
|-------|-------------|------------|
| Procedural generation (BSP) | Dungeon layout from binary space partition. Every run is novel | exp017 (validated) |
| Procedural generation (Perlin) | Terrain variation within rooms. Noise as environment | exp009 (validated) |
| Permadeath + meta-learning | Player skill is the meta-progression. No mechanical carry-forward | exp058 (Conjugant — the formalization) |
| Turn-based game loop | No real-time pressure. Deliberation is the game. Infinite time per decision | exp025 (implemented) |
| Engagement via emergence | Stories emerge from system interactions, not authored narrative | exp059 (Quorum) |
| ASCII as UI | The original TUI game. Proof that games are math, not graphics | exp024, exp025 (implemented) |

**What we learn**: Roguelikes invented procedural generation, permadeath,
emergent narrative, and ASCII gaming. Everything in the sandbox/roguelite
genre descends from Rogue. We already have exp025 (roguelike explorer)
— this era is partially validated.

**Cross-spring**: Dungeon generation via BSP is spatial partitioning.
Same algorithm that bioinformatics uses for kd-trees in sequence space
(wetSpring), that physics uses for Barnes-Hut N-body (hotSpring), and
that database indexing uses for R-trees (NestGate).

---

## Era 5: First Person — Wolfenstein 3D (1992) / Doom (1993)

**What FPS games validate:**

| Model | How It's Used | Experiment |
|-------|-------------|------------|
| Raycasting | DDA ray marching through grid. 2D map → 3D illusion | exp001 (validated) |
| BSP rendering | Binary space partition for visibility. Which walls are visible from here? | exp017 (validated) |
| Fitts's law (targeting) | Weapon crosshair → enemy hitbox. Target acquisition cost predicts difficulty | exp001, exp005 (validated) |
| Tufte on HUD | Doom's HUD — health, ammo, face. Data-ink ratio analysis | exp001, exp003 (validated) |
| DDA raycasting | Push-forward combat. Health from kills forces aggression | exp001 (validated) |
| TUI rendering | exp024 already renders Doom in a terminal via ratatui | exp024 (implemented) |

**What we learn**: We jumped here first (exp001). Doom is our MD — the
first complex system we validated. But now we see that Pong → Spacewar
→ Tetris → Rogue should have come first, because each validates a
prerequisite for Doom (real-time loop, continuous physics, procedural
generation, spatial partitioning).

**Cross-spring**: Raycasting is ray tracing simplified. hotSpring uses
ray tracing for radiation transport through materials. Same algorithm,
different medium. The ray doesn't know if it's rendering a wall or
computing neutron flux.

---

## Era 6: Strategy — Civilization (1991) / StarCraft (1998)

**What strategy games validate:**

| Model | How It's Used | Experiment |
|-------|-------------|------------|
| Symbiont interaction matrix | Unit counters, faction relationships, alliance/rivalry | exp057 (validated) |
| Game tree complexity | Decision branching per turn. Civilization's tree rivals chess | exp050 (validated) |
| Fog of war | Incomplete information. Exploration as information gathering | **NEW**: exp079 |
| Tech tree | Directed graph of prerequisites. Innovation unlocks capability | **NEW**: exp079 |
| Economy / carrying capacity | Resource management. Growth bounded by carrying capacity K | Ecology math (CROSS_DOMAIN_LEARNING.md) |
| DDA via difficulty settings | Explicit player-chosen difficulty. The original user-driven DDA | exp020 (validated) |

**Modalities**: TUI Civilization is turn-based — it works perfectly in
ASCII. Each tile is a character. The map is a grid. This is why Dwarf
Fortress works in a terminal.

**What we learn**: Strategy games are the first genre where the game tree
is large enough to be genuinely novel data per session. Civilization
proves that turn-based games generate unique combinatorial states. This
is the foundation for exp050's game tree complexity metric.

**Cross-spring**: Tech trees are phylogenetic trees. Evolutionary
branching in biology (wetSpring) uses the same directed acyclic graph
as technology branching in Civilization. Same topology, different nodes.

---

## Era 7: Loot — Diablo (1996) / Baldur's Gate (1998)

**What ARPGs and CRPGs validate:**

| Model | How It's Used | Experiment |
|-------|-------------|------------|
| Loot tables as probability distributions | Item rarity follows power-law distributions. Drop rates are tunable parameters | **NEW**: exp080 |
| Character progression (Integrase) | XP accumulation → level up → capability unlock. Same as Integrase capture threshold | exp056 (validated) |
| Pathogen (exploitation risk) | Diablo's gambling and loot are the ancestor of modern gacha | exp060 (validated) |
| Ruleset ingestion | Baldur's Gate runs AD&D 2e. Rules as data, not code | exp045 (validated) |
| Dialogue trees | Baldur's Gate's dialogue system. NPCs with branching responses | exp046, exp067 (validated) |

**What we learn**: This era splits games into "systems-driven" (Diablo —
the loot IS the game) and "narrative-driven" (Baldur's Gate — the story
IS the game). Both use the same underlying math (probability, state
machines, DAGs) but optimize for different Fun Keys (Hard Fun vs Serious
Fun). RPGPT must serve both.

**Cross-spring**: Loot table probability distributions are the same
power-law distributions that describe species abundance in ecology
(wetSpring), earthquake magnitudes in geophysics (hotSpring), and
word frequency in natural language (neuralSpring). Zipf's law everywhere.

---

## Era 8: Emergent — The Sims (2000) / Dwarf Fortress (2006)

**What emergence validates:**

| Model | How It's Used | Experiment |
|-------|-------------|------------|
| Agent-based modeling | Individual agents with simple rules → complex collective behavior | exp059 (Quorum, validated) |
| Needs hierarchy (Maslow) | Sims have needs. Hunger, bladder, social, fun. Behavior emerges from need state | RPGPT NPC motivations (exp067-072) |
| Self-organized criticality | Dwarf Fortress tantrum spirals. Local dissatisfaction → global crisis | exp059 (Quorum threshold) |
| Emergent narrative | No authored story. Story emerges from simulation. The player narrates | exp059, RPGPT World 2 |

**What we learn**: This era proves that narrative can emerge from
simulation. You don't need a writer if the system is rich enough.
But (the bounding problem) emergent narrative lacks structure. The
Sims generates anecdotes, not stories. Dwarf Fortress generates
legends, not novels. RPGPT's answer: authored scaffolds with
emergent flesh.

**Cross-spring**: Agent-based modeling IS population dynamics. Every
Dwarf Fortress dwarf is a microbial cell with needs, interactions,
and environmental responses. The Sims is a microbiome with plumbing.

---

## Era 9: Cards — MTG: Online (2002) / Slay the Spire (2019)

**What card games validate:**

| Model | How It's Used | Experiment |
|-------|-------------|------------|
| Combinatorial game tree | 10^358 for MTG Commander. Every game is novel data | exp049, exp050 (validated) |
| Stack resolution | LIFO execution with priority and interrupts | exp048 (validated) |
| Card provenance | Every card is a certificate with history | exp047, exp061 (validated) |
| Deckbuilding as optimization | Alphabet design. Shannon entropy of deck | exp050 (connection) |
| Signed randomness | BearDog-signed shuffles for provably fair games | exp064 (validated) |

**What we learn**: Card games have the deepest validated math in
ludoSpring (exp047-050, exp061). They're our most mature era. This
is because card games are pure combinatorics — no physics, no
real-time, no spatial reasoning. Pure math. Easiest to validate,
hardest to exhaust.

---

## Era 10: Extraction — Tarkov (2017) / Hunt (2019)

**What extraction validates:**

| Model | How It's Used | Experiment |
|-------|-------------|------------|
| Provenance DAG | Every action is a vertex. Chain of custody on every item | exp053, exp054 (validated) |
| Anti-cheat as structure | Cheating = structural impossibility, not behavioral detection | exp053 (8 fraud types) |
| Certificate economy | Every item is a loamSpine cert. No duplication. Atomic transfer | exp053, exp064 (validated) |
| Cross-domain fraud | Same detector works for games, science, medicine | exp065 (validated) |

**What we learn**: We already have deep validation here. exp053-054
are ludoSpring's most complete cross-domain experiments. Tarkov is
our QCD — the complex system that proves the full stack works.

---

## Era 11: Narrative — Disco Elysium (2019) / Esoteric Ebb (2025)

**What narrative CRPGs validate:**

| Model | How It's Used | Experiment |
|-------|-------------|------------|
| Internal voices | Skills as perspectives. Constrained AI calls with personality | exp069, exp070 (validated) |
| NPC knowledge bounds | NPCs know, suspect, lie, and don't know | exp067, exp068 (validated) |
| NPC memory as DAG | Structural memory that survives context windows | exp071 (validated) |
| Trust dynamics | Relationship evolution over multiple encounters | exp072 (validated) |
| Plane transitions | Seamless mode shifts with state continuity | exp075 (validated) |
| Dialogue skill checks | Tabletop mechanics in conversation | exp073 (validated) |
| Flow in dialogue | Challenge/skill balance in persuasion, not combat | exp074 (validated) |

**What we learn**: This is RPGPT Phase 1. Already implemented and
validated (321 checks across exp067-075). The narrative era is our
destination, but we built it before validating the foundations.
The revalidation roadmap fills the gap.

---

## Era 12: The Target — RPGPT

RPGPT is not an era in game history. It's the synthesis. Every era's
validated math composes:

| Era | What RPGPT Inherits |
|-----|-------------------|
| Pong | Real-time game loop, Fitts interception, Flow from speed ramp |
| Spacewar | Continuous physics, multi-body interaction |
| Tetris | Fair procedural generation (bag guarantees), Flow as speed ramp |
| Rogue | Procedural dungeon generation, permadeath, emergent narrative |
| Doom | Raycasting, BSP, FPS interaction model, Tufte HUD |
| Civilization | Strategy tree, fog of war, economy, faction Symbiont |
| Diablo/BG | Loot probability, character progression, ruleset ingestion, dialogue |
| The Sims/DF | Agent needs, emergent narrative, self-organized criticality |
| MTG/StS | Card provenance, stack resolution, combinatorial depth, signed randomness |
| Tarkov | Provenance DAG, certificate economy, anti-cheat as structure |
| Disco/Ebb | Internal voices, NPC depth, dialogue Flow, plane transitions |

**RPGPT = Pong's game loop + Rogue's procedural generation + Doom's
spatial rendering + Civilization's strategy + Diablo's loot math +
BG's ruleset engine + Dwarf Fortress's emergence + MTG's combinatorics
+ Tarkov's provenance + Disco's narrative depth.**

Each era must be validated before we can trust the synthesis.

---

## The Rendering Proof

Every historical game is rendered across all petalTongue modalities to
prove the game IS the math:

| Modality | Technology | What It Proves |
|----------|-----------|---------------|
| **TUI** (ratatui) | Terminal, SSH, any machine | Game works without GPU. Math is sufficient. Accessibility baseline |
| **GUI** (egui) | Desktop window, mouse/keyboard | Standard interactive experience. Visual feedback enriches math |
| **Headless** | No rendering, validation only | Game logic is testable without display. CI/CD integration |
| **SVG export** | Static image, printable | Game state is serializable. Replay and analysis |
| **Audio** (sonification) | Sound only | Accessibility for visually impaired. Game state as sound |

exp024 (Doom TUI) and exp025 (Roguelike TUI) already prove TUI works.
The revalidation extends this to every era.

**The question "how does TUI Pong differ from VR Pong?" has a precise
answer**: the game logic is identical. The Fitts constants change (mouse
vs controller vs gaze tracking). The Steering law constants change (2D
path vs 3D path). The Flow channel width may shift (VR immersion lowers
anxiety threshold). But `position += velocity * dt` is the same in every
modality. That's the point.

---

## Implementation Roadmap

### Phase 1: Foundations (Eras 0-3)

| Experiment | Game | What It Validates | Priority |
|-----------|------|------------------|----------|
| **exp076** | Pong | Real-time loop, Fitts interception, Flow speed ramp, DDA | **DONE** — 35/35 checks |
| **exp077** | Spacewar / Asteroids | Continuous 2D physics, wrap topology, N-body | **DONE** — 33/33 checks |
| **exp078** | Tetris | Flow as speed ramp, bag randomizer fairness, Hick on rotation | **DONE** — 35/35 checks |

### Phase 2: Complexity (Eras 4-6)

| Experiment | Game | What It Validates | Priority |
|-----------|------|------------------|----------|
| exp025 | Roguelike (Rogue/NetHack) | BSP dungeon, turn-based loop, emergence | DONE |
| exp001, exp024 | Doom | Raycasting, BSP rendering, Fitts targeting, Tufte HUD | DONE |
| **exp079** | Civilization (minimal) | Fog of war, tech tree DAG, Symbiont factions, economy | **DONE** — 33/33 checks |

### Phase 3: Systems (Eras 7-9)

| Experiment | Game | What It Validates | Priority |
|-----------|------|------------------|----------|
| **exp080** | Diablo (loot system) | Power-law loot tables, Integrase progression, Pathogen risk | **DONE** — 36/36 checks |
| exp047-050 | MTG / Card games | Combinatorial depth, stack resolution, provenance | DONE |
| exp055-060 | Lysogeny games | Usurper, Integrase, Symbiont, Conjugant, Quorum, Pathogen | DONE |

### Phase 4: Integration (Eras 10-12)

| Experiment | Game | What It Validates | Priority |
|-----------|------|------------------|----------|
| exp053-054 | Tarkov (extraction) | Provenance DAG, anti-cheat, certificate economy | DONE |
| exp067-075 | Disco / RPGPT Phase 1 | Internal voices, NPC depth, plane transitions | DONE |

### Gap Analysis

| Gap | What's Missing | Proposed Experiment | Supporting |
|-----|---------------|-------------------|-----------|
| ~~Real-time game loop~~ | ~~We have turn-based (exp025) but not real-time validated~~ | **exp076 (Pong) — DONE** | exp012 (Flow), exp037 (tick budget) |
| ~~Continuous physics~~ | ~~We have grid-based (raycaster) but not vector physics~~ | **exp077 (Spacewar) — DONE** | exp019 (composite cost) |
| ~~Fair procedural generation~~ | ~~We have BSP/Perlin but not fairness-guaranteed generation~~ | **exp078 (Tetris bag) — DONE** | exp014 (hybrid noise+WFC) |
| ~~Fog of war~~ | ~~Referenced but not validated as an independent model~~ | **exp079 (Civ) — DONE** | exp018 (Four Keys), exp016 (cognitive load) |
| ~~Loot probability~~ | ~~Implicit in Lysogeny but not validated as standalone~~ | **exp080 (Diablo) — DONE** | exp036 (throughput) |
| ~~Compositional procgen~~ | ~~Individual primitives validated but not composed into biome/world pipeline~~ | **exp081 (Procedural Generation) — DONE** | exp009, exp014, exp017 |

---

## Faculty Anchors

- Atari (1972) — Pong: the origin of electronic games
- Russell, S. et al. (1962) — Spacewar!: the first interactive computer game
- Pajitnov, A. (1984) — Tetris: the Flow reference game
- Toy, M. et al. (1980) — Rogue: the origin of procedural generation
- Carmack, J. (1993) — Doom: the FPS that defined an industry
- Meier, S. (1991) — Civilization: strategy as systems thinking
- Brevik, D. et al. (1996) — Diablo: loot as probability engine
- Wright, W. (2000) — The Sims: needs-driven agent simulation
- Adams, T. (2006) — Dwarf Fortress: emergent narrative from simulation
- Garfield, R. (1993) — Magic: The Gathering: combinatorial game design
- Nikishin, B. (2017) — Escape from Tarkov: provenance as game architecture
- Kurvitz, R. (2019) — Disco Elysium: dialogue as gameplay, skills as voices

## License

AGPL-3.0-or-later

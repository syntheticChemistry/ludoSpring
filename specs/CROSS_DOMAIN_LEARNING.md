# Cross-Domain Learning — Games ↔ Science ↔ Systems

**Date**: March 18, 2026
**Status**: Living document — grows as cross-spring validation deepens
**License**: AGPL-3.0-or-later
**Purpose**: The bidirectional bridge. What games teach science about systems.
What science teaches games about mechanics. Why every spring benefits from
every other spring's domain knowledge.
**Depends on**: `GAME_QUALITY_PROFILES.md`, `LYSOGENY_CATALOG.md`,
`RPGPT_DEEP_SYSTEM_DESIGN.md`

---

## The Core Thesis

Games are systems laboratories. They compress complex dynamics into
30-minute sessions where a player must understand, adapt to, and master
the system to survive. Every game mechanic is a system model with
immediate feedback.

Science is full of systems that are hard to understand because feedback
is slow, invisible, or both. Microbial communities evolve over days.
Climate systems shift over decades. Protein folding happens in
nanoseconds but takes years to simulate.

**The bridge works in both directions:**

- **Games → Science**: Game mechanics are intuition pumps for systems
  thinking. A player who understands XCOM's action economy understands
  resource allocation. A player who understands Slay the Spire's
  deckbuilding understands combinatorial optimization. A player who
  understands Tarkov's economy understands supply chain dynamics.

- **Science → Games**: Published scientific models are untapped
  game mechanic reservoirs. Every ecological interaction, every
  epidemiological model, every materials science phase transition
  is a game mechanic waiting to be discovered by a creative designer.

---

## Part 1: What Games Teach Other Springs

### Shooters Teach Systems Engineering

An FPS is a real-time distributed system under adversarial conditions.
Every problem in shooter design has a direct analogue in infrastructure,
bioinformatics pipelines, and scientific computing.

| Shooter Problem | Systems Analogue | Spring Application |
|----------------|-----------------|-------------------|
| **Netcode / desync** | Distributed consensus. Client and server disagree on world state. Who is authoritative? How do you reconcile divergent timelines? | wetSpring: pipeline outputs differ between runs. Provenance DAG reconciles. hotSpring: GPU and CPU disagree on computation result. Which is authoritative? |
| **Hit registration** | Event ordering in distributed systems. Did the bullet hit before the player moved? Causality in concurrent events | primalSpring: IPC message ordering. Did the request arrive before the capability expired? rhizoCrypt DAG enforces causal ordering |
| **Anti-cheat** | Input validation in adversarial environments. How do you trust client-submitted data? | All springs: how do you trust data from an external sensor, an untrusted lab, a third-party API? Provenance chain validates every claim |
| **Matchmaking** | Resource allocation under constraints. Match players of similar skill while minimizing wait time | toadStool: job scheduling. Match compute tasks to appropriate hardware while minimizing queue time. Same optimization problem |
| **Tick rate / budget** | Real-time scheduling. 60 operations per second, each must complete within 16.6ms or the system degrades | biomeOS: primal orchestration tick budget. Each primal must respond within deadline or the deploy graph falls behind |
| **Spatial partitioning (BSP, octree)** | Data structure for efficient range queries. Which entities are near this point? | wetSpring: which genomic features are near this locus? Same spatial indexing, different coordinate system |
| **Weapon balance** | Multi-variable equilibrium. Every weapon must be viable. Buff one, and the meta shifts | wetSpring: antibiotic cocktail design. Every drug affects the target differently. Adjust one dose, and resistance patterns shift. Same Symbiont interaction matrix (exp057) |
| **Bullet physics** | Projectile simulation — ballistic trajectories, wind, gravity, penetration | hotSpring: particle transport in reactor physics. Same differential equations, different projectiles |

**The key insight**: A game developer who solves "hit registration across
200ms latency" has solved the same causal ordering problem as a distributed
systems engineer. The solution (content-addressed DAG with causal parents)
transfers directly.

### Strategy Games Teach Ecology

Turn-based strategy is ecology compressed into turns. Every mechanic has
a published ecological equivalent.

| Strategy Mechanic | Ecological Model | Spring Application |
|------------------|-----------------|-------------------|
| **Unit counter system** (rock-paper-scissors) | Competitive exclusion (Gause 1934). Species A beats B, B beats C, C beats A. Cyclical dominance maintains diversity | wetSpring: antibiotic resistance patterns. Strain A resists drug X, strain B resists drug Y. Cycling antibiotics = cycling unit compositions |
| **Resource economy** (gold, wood, food) | Nutrient cycling. Carbon, nitrogen, phosphorus flow through trophic levels. Bottleneck resources limit growth | airSpring: soil nutrient modeling. Limiting nutrient determines crop yield. Same bottleneck math |
| **Tech tree** | Evolutionary innovation. Mutations unlock new capabilities. Innovation requires prerequisites (oxygen → multicellularity) | wetSpring: gene gain/loss in evolution. New metabolic capabilities require prerequisite genes |
| **Fog of war** | Incomplete information. You know your local environment but not the global state. Decisions under uncertainty | wetSpring: metagenomic sampling. You see what your sequencer captures. The rest is fog. Sampling strategy = scouting strategy |
| **Diplomacy / alliances** | Mutualism and syntrophy. Organisms cooperate when mutual benefit exceeds competition cost | healthSpring: multi-provider care coordination. Specialists cooperate when patient benefit exceeds coordination cost |
| **Attrition warfare** | Red Queen dynamics (Van Valen 1973). Constant arms race. Neither side gains lasting advantage. Both must keep evolving to survive | wetSpring: host-pathogen co-evolution. Immune system and virus in constant escalation |
| **Supply lines** | Metabolic pathways. Substrates must flow from source to consumer. Cutting the supply starves the endpoint | wetSpring: metabolic flux analysis. Block one enzyme, and downstream metabolites starve. Same graph, same vulnerability |
| **Terrain advantage** | Niche differentiation. Some species thrive in specific environments. Environmental heterogeneity maintains diversity | airSpring: microclimate variation across a field. Same crop, different yields based on position. Spatial heterogeneity |

### Roguelikes Teach Evolutionary Biology

A roguelike run IS an evolutionary generation. The Conjugant mechanic
(exp058) proves this formally, but the conceptual mapping is deeper.

| Roguelike Mechanic | Evolutionary Model | Spring Application |
|-------------------|-------------------|-------------------|
| **Permadeath** | Organismal death. Information is lost unless transmitted | wetSpring: culture death in the lab. Protocols must be documented (loamSpine certs) or knowledge dies with the postdoc |
| **Meta-progression** | Horizontal gene transfer. Dead organisms release DNA. Living organisms pick it up. Information accumulates across generations | wetSpring: LTEE (Lenski Long-Term Evolution Experiment). 60,000 generations of accumulated beneficial mutations. exp058 models this exactly |
| **Seed-based RNG** | Deterministic chaos. Same initial conditions → same outcome. Reproducibility from seeds | All springs: reproducible computation. Same input seed → same output. This IS scientific reproducibility |
| **Build diversity** | Genotype diversity. Different gene combinations produce different phenotypes. Some are fit, some aren't | neuralSpring: hyperparameter search. Different model configurations produce different performance. Some converge, some don't |
| **Boss adaptation** | Predator-prey arms race. The prey that survives is the one the predator can't catch. Natural selection in real time | wetSpring: antibiotic dose escalation. The bacteria that survive are the ones the drug can't kill |
| **Procedural levels** | Environmental stochasticity. Each generation faces a different environment. Fitness depends on which challenges appear | airSpring: weather variation across growing seasons. Each year is a "procedural level" for the crop |

### Card Games Teach Combinatorics and Information Theory

MTG Commander has a game tree of 10^358 (exp050). That's not a game —
that's an information theory laboratory.

| Card Game Mechanic | Information Theory Model | Spring Application |
|-------------------|------------------------|-------------------|
| **Deckbuilding** | Alphabet design. Which symbols (cards) do you include in your message (deck)? Shannon entropy of the deck determines information content per draw | neuralSpring: feature selection. Which input features do you include in the model? Same optimization — maximize information per parameter |
| **Card draw** | Sampling without replacement. Each draw changes the distribution of remaining cards. Bayesian updating in real time | wetSpring: sequencing. Each read is a sample from the genome. Coverage depth = draw count. Same statistics |
| **Stack resolution** | LIFO execution. Last in, first out. Interrupts and responses nest. The stack IS a call stack | primalSpring: IPC message handling. Requests nest. Responses resolve in reverse order. Same execution model |
| **Mana curve** | Resource scheduling. Too many expensive cards = dead draws early. Too many cheap cards = no late game. The distribution matters | toadStool: job scheduling. Too many GPU-heavy tasks = queue starvation for small jobs. Same curve optimization |
| **Metagame** | Nash equilibrium seeking. The population of decks evolves toward equilibrium. Counter-strategies emerge. The meta IS evolutionary dynamics | neuralSpring: model competition. Published models compete on benchmarks. New architectures counter old ones. The leaderboard IS a metagame |

### Investigation Games Teach Scientific Method

An investigation game IS the scientific method gamified. Return of the
Obra Dinn is a hypothesis-testing engine.

| Investigation Mechanic | Scientific Method | Spring Application |
|-----------------------|------------------|-------------------|
| **Core clues** (GUMSHOE) | Observable data. Always available. You don't roll to see the evidence — it's there if you look | All springs: raw data is never gated. FASTQ files, sensor readings, experiment results. The data exists. Interpretation is the skill |
| **Deduction chains** | Hypothesis formation. Evidence A + Evidence B → Hypothesis C. Refutable by Evidence D | wetSpring: differential diagnosis in metagenomics. Species A present + metabolite B elevated → hypothesis: syntrophic relationship C |
| **False leads** | Type I errors. Evidence that appears to support a hypothesis but doesn't. The investigation teaches you to distinguish signal from noise | neuralSpring: overfitting. The model appears to learn the pattern but is memorizing noise. Same false lead, different domain |
| **Knowledge bounds** (NPC) | Experimental limitations. Every instrument has a detection limit. Every assay has a sensitivity threshold. The experiment "doesn't know" things below its limit | wetSpring: sequencing depth. Below 10x coverage, you can't reliably call variants. The sequencer has knowledge bounds |
| **The reveal** | Publication. The hypothesis is confirmed, the mechanism is understood, the paper is written | All springs: the experiment is done, the validation passes, the handoff is written |

### Sandbox Games Teach Emergence

Dwarf Fortress is a complex adaptive system simulator. Factorio is a
production system optimizer. Both teach that simple rules produce
complex behavior.

| Sandbox Mechanic | Complexity Science Model | Spring Application |
|-----------------|------------------------|-------------------|
| **Emergent narrative** (Dwarf Fortress) | Self-organized criticality (Bak 1987). Small local interactions produce occasional large-scale events | wetSpring: microbiome assembly. Individual species interactions produce occasional community-level phase transitions (blooms, crashes) |
| **Optimization spirals** (Factorio) | Operations research. Throughput optimization under constraint. The factory MUST grow because inefficiency compounds | toadStool: GPU pipeline optimization. Throughput must increase because batch sizes grow. Same optimization pressure |
| **Emergent economy** (EVE Online) | Market dynamics. Supply, demand, arbitrage, speculation. All from player actions, no designer control | sunCloud: radiating attribution economy. Value flows through the graph based on contribution. Emergent pricing from participation, not decree |
| **Mod ecosystems** | Open source software ecosystems. Community contributors build on shared infrastructure. Forking, merging, compatibility | All springs: the entire ecoPrimals ecosystem. Primals compose. Springs coordinate. Community extends. Same ecosystem dynamics |

---

## Part 2: What Science Teaches Games

The Lysogeny catalog (exp055-060) already documents 6 mechanics derived
from published science. But the reservoir is far deeper. Every scientific
domain contains game mechanics that no designer has discovered yet.

### Microbial Communities → Game Mechanics

Microbial ecology is the richest untapped source of game mechanics because
microbes face the same problems as game characters: survive, compete,
cooperate, evolve, communicate — all under resource constraints with
incomplete information.

| Microbial System | Game Mechanic | How It Plays | Status |
|-----------------|---------------|-------------|--------|
| **Quorum sensing** | Collective event trigger. Individual signals accumulate. At threshold, everyone acts simultaneously | Town revolt, raid boss phase, market crash. When enough NPCs/players reach a threshold, the world changes | VALIDATED (exp059) |
| **Horizontal gene transfer** | Meta-progression. Dead runs transfer capabilities to living ones. Information crosses lineages | Roguelite upgrades, inherited knowledge, cross-character progression | VALIDATED (exp058) |
| **Biofilm formation** | Collective defense structure. Individuals sacrifice mobility for community resilience. The group becomes harder to displace | Fortification mechanics. Players/NPCs build collective structures that sacrifice individual freedom for group survivability | Conceptual — maps to base-building + Quorum |
| **Phage-host dynamics** | Asymmetric predator-prey. The predator is specialized (specific receptor). The prey evolves resistance. The predator evolves to match | Boss evolution. The boss adapts to player strategy. Player must change approach. Neither side reaches stable dominance | Conceptual — extends Usurper (exp055) |
| **Auxotrophy** (metabolic dependency) | Forced cooperation. Organism A cannot produce amino acid X. Organism B cannot produce amino acid Y. They MUST trade to survive | RPG party composition where no class is self-sufficient. The healer can't fight. The fighter can't heal. Cooperation is structural, not optional | Conceptual — maps to RPGPT party system |
| **Persister cells** | Dormancy as survival strategy. A fraction of the population goes dormant during stress. They survive what kills the active population, then reactivate | NPC "sleeper agents." A faction member goes dormant during a purge, survives, reactivates later to rebuild. Mechanically: some NPCs are immune to area effects because they're dormant | Partial — exp055 models persistence |
| **CRISPR adaptive immunity** | Memory-based defense. Organism records past threats. Future encounters with the same threat trigger targeted defense | NPC combat learning. After being hit by fire magic, the NPC develops fire resistance. The defense is specific to the recorded attack, not general | Conceptual — extends Usurper memory |
| **Chemotaxis** | Gradient following. Organism moves toward nutrients, away from toxins. Navigation by local gradient, no global knowledge | NPC pathfinding driven by "desire gradients." NPCs don't know where the treasure is — they follow signals (rumors, smells, sounds) that get stronger near the goal | Conceptual — maps to Investigation plane |
| **Sporulation** | Irreversible commitment. Under extreme stress, organism transforms into a spore — dormant, durable, but unable to act until conditions improve | Player sacrifice mechanics. Commit resources irreversibly for future benefit. The decision cannot be undone. Strategic irreversibility as game tension | Conceptual — relates to Conjugant |
| **Plasmid fitness cost** | Power has a price. Carrying resistance genes is metabolically expensive. The resistant strain grows slower than the susceptible one in the absence of the antibiotic | Equipment weight / ability cooldowns. Carrying powerful gear slows you down. Having powerful abilities means longer cooldowns. Every advantage has a cost that matters when the advantage isn't needed | Conceptual — maps to Tarkov loadout economics |

### Epidemiology → Narrative Mechanics

Disease spread models are narrative spread models. Information,
rumors, fear, and influence propagate through populations the same
way pathogens do.

| Epidemiological Model | Game Mechanic | How It Plays |
|----------------------|---------------|-------------|
| **SIR model** (Susceptible → Infected → Recovered) | Rumor/knowledge propagation. NPCs start uninformed (S), learn a secret (I), and eventually the secret becomes old news (R). Rate of spread depends on social connectivity | Investigation games: you release information and watch it propagate through the NPC network. Strategic leaking |
| **Super-spreader events** | Key NPCs who talk to everyone spread information disproportionately. Targeting them amplifies or suppresses information flow | Political plane: identify the town gossip and feed them misinformation (or truth). Quorum mechanics determine when the information reaches critical mass |
| **R₀ (basic reproduction number)** | Virality metric for in-game events. If R₀ > 1, the rumor spreads exponentially. If R₀ < 1, it dies out. The player can manipulate R₀ by changing social conditions | DDA parameter: if the narrative is "too viral" (every NPC knows everything), reduce R₀ by introducing skepticism. If it's stalling, increase R₀ by adding a super-spreader |
| **Vaccination / herd immunity** | Disinformation resistance. If enough NPCs know the truth, lies can't take hold even if some NPCs are susceptible. The threshold is calculable | Political plane defense: inoculate NPCs against manipulation by preemptively sharing evidence. Mathematical threshold for "narrative immunity" |
| **Antigenic drift** | Evolving disinformation. The lie mutates to evade debunking. Each time you refute a version, a new variant appears | Investigation plane challenge: the conspiracy evolves. You debunk version 1, and version 2 emerges. Same underlying lie, different surface |
| **Contact tracing** | Detective mechanic. Trace the spread of a rumor/poison/curse back to patient zero. The evidence DAG IS a contact tracing graph | Investigation games: "who told whom" is the same algorithm as "who infected whom." rhizoCrypt DAG supports both queries identically |

### Physics → Gameplay Mechanics

Physical systems provide mechanics that feel intuitive because players
live in a physical world.

| Physical System | Game Mechanic | How It Plays |
|----------------|---------------|-------------|
| **Phase transitions** (solid → liquid → gas) | State changes with thresholds. Reputation doesn't degrade linearly — it collapses at a critical point. Alliance → neutral → hostile isn't gradual | Symbiont mechanics with criticality. Faction standing appears stable, then suddenly shifts. Same math as magnetization phase transition (Ising model) |
| **Resonance** | Frequency matching for amplified effect. Hit the enemy at the right rhythm and damage multiplies. Miss the rhythm and it's wasted | Combat mechanics in rhythm games, but also in any game with timing windows. Parry systems (Souls-like) are resonance — match the attack frequency |
| **Conservation laws** | Nothing is created or destroyed, only transformed. Every item in the economy came from somewhere and goes somewhere. Closed system | Extraction shooter economy. No currency injection — all value comes from raids. Same as conservation of energy. exp053 enforces this via DAG |
| **Entropy** | Disorder increases. The world tends toward chaos. Order requires effort. Without maintenance, the fortress crumbles, the garden overgrows, the alliance frays | Sandbox maintenance pressure. Dwarf Fortress: unmaintained rooms decay. Factorio: pollution accumulates. The system degrades without active order |
| **Anderson localization** | Signal gets trapped in disorder. In a disordered medium, waves can't propagate — they localize. More disorder = more localization | Already modeled (exp044). Sanity mechanics: knowledge IS disorder. The more you learn about eldritch truths, the more your ability to function in normal reality localizes — you can't "propagate" socially |
| **Percolation** | Connected paths through random networks. At a critical density, a connected path appears. Below it, no path exists | Map connectivity. At what point does the dungeon become traversable? Procedural generation needs percolation theory to guarantee solvability. Below the threshold, the level is impossible |
| **Diffusion** | Slow spreading without direction. Heat diffuses. Ink diffuses. Information diffuses through a population without deliberate transmission | Passive knowledge spread. NPCs learn things slowly through proximity without anyone telling them. The baker overhears the guard. Information diffuses through spatial proximity |

### Ecology → Economy Design

Game economies are ecosystems. Every economy problem has an ecological
solution, and every ecological model is an economy mechanic.

| Ecological Model | Economy Mechanic | Application |
|-----------------|-----------------|-------------|
| **Carrying capacity (K)** | Maximum market size. The economy can only support N active traders before resource depletion makes trading unprofitable | Any game with a player-driven economy. EVE Online, Tarkov flea market. K determines when inflation becomes deflation |
| **Trophic cascades** | Removing one actor destabilizes the chain. Remove the predator, and the herbivore population explodes, and the plants collapse | Game economy cascades. Ban the whales, and the RMT market collapses, and the item farmers lose income, and the grinders face higher prices |
| **Niche partitioning** | Multiple strategies coexist when they exploit different resources. Generalists compete with everyone. Specialists dominate their niche | Build diversity in RPGs. If every build competes for the same stat, only one is optimal. If builds exploit different niches (tank, healer, DPS), all coexist |
| **Succession** | Pioneer species establish first. Climax community develops later. Each stage creates conditions for the next | Server lifecycle in MMOs. Early adopters (pioneers) establish the economy. Mid-game players build on it. Late-game players inherit a mature ecosystem. Each wave needs different design |
| **Invasive species** | A new entrant with no natural predators disrupts the existing equilibrium. Native species decline or go extinct | Overpowered new content. A new weapon/class/item with no counter destabilizes the meta. Balance patches = restoration ecology |
| **Mutualism vs parasitism continuum** | Bronstein 2001. Every interaction sits on a spectrum from cooperative to exploitative. Context determines which end | Player interaction design. Trade can be mutualistic (both benefit) or parasitic (scam). The system must detect and penalize parasitism. sweetGrass attribution tracks this |

### Materials Science → Crafting Systems

Every crafting system is a materials science simulation. The Crafting
plane in RPGPT should feel like a chemistry lab.

| Materials Science Concept | Crafting Mechanic | How It Plays |
|--------------------------|------------------|-------------|
| **Alloy phase diagrams** | Mixing materials produces different results depending on ratios. 70% copper + 30% tin = bronze. 50/50 = different properties | Crafting recipes with ratio sensitivity. Not just "combine A + B" but "combine 3 parts A with 1 part B at high temperature." The ratio matters |
| **Catalysis** | A catalyst enables a reaction without being consumed. Rare catalyst + common ingredients = valuable product. The catalyst is the bottleneck | Master craftsman tools. The tool doesn't get used up but is required. Rare tools gate access to advanced recipes. The economy revolves around catalyst access |
| **Tempering / annealing** | Controlled heating and cooling produces different properties. Quick cool = brittle and hard. Slow cool = flexible and tough | Crafting quality system. Rush the process (quick cool) and get a fragile but sharp weapon. Take time (slow cool) and get a durable one. Time investment = quality control |
| **Crystal growth** | Slow, controlled process produces perfect crystals. Fast growth produces polycrystalline material with defects | Enchantment system. Slow, deliberate enchanting produces perfect results. Rushing introduces flaws that may be interesting (unexpected enchantments as "defects") |
| **Corrosion** | Materials degrade in hostile environments. Iron rusts. Copper verdigris. Rate depends on environment | Equipment degradation with environmental factors. Desert = sand erosion. Swamp = corrosion. Underground = mineral deposits. Maintenance is environment-dependent |
| **Combinatorial chemistry** | Systematically combining molecular fragments to explore a chemical space. Each fragment contributes a property. Stacking fragments in different orders produces different molecules | **Compositional loot** (the Noita model). Items are mechanic tags, not stat rolls. Combining "Homing + Explosive + Bouncing" is combinatorial chemistry applied to game mechanics. The composition space is bounded (finite fragment vocabulary) but the emergent interactions are vast. This is strictly superior to random stat ranges (Fallout 4) because the player is a system designer, not a lottery player |

### Procedural Generation → Cross-Spring Universality

Procedural generation is the single most transferable technique between
games and science. The same primitives compose into radically different
outputs depending on domain interpretation.

| Procgen Primitive | Game Application | healthSpring | wetSpring | primalSpring |
|-------------------|-----------------|-------------|-----------|-------------|
| **Perlin noise / fBm** | Terrain elevation, moisture maps, biome classification (Minecraft) | Continuous patient parameter fields — age, risk score, severity. Noise produces naturalistic clustering unlike uniform random | Molecular density fields, electrostatic potential landscapes | Load distribution profiles, latency gradients across regions |
| **Wave Function Collapse** | Adjacency-constrained tile maps. Deserts don't appear next to tundra without transitions | Constraint-valid patient cohorts — diabetics have correlated conditions. WFC ensures the synthetic population respects comorbidity rules | Crystal lattice constraints. Valid molecular conformations. Protein secondary structure assignment | Configuration constraint propagation — valid infrastructure topologies |
| **BSP (Binary Space Partition)** | Dungeon rooms, world regions, spatial indexing for collision | Patient triage partitioning — severity zones, ward allocation, hospital floor plans | Spatial indexing for N-body simulations, molecular docking search spaces | Data sharding, service mesh partitioning, load balancer regions |
| **L-systems** | Branching rivers, road networks, vegetation, organic structures | Vascular networks, disease spread topology, organ system modeling | Protein backbone folding, polymer branching, crystal growth patterns | Dependency trees, pipeline DAGs, service discovery topology |
| **Bounded randomness (bag)** | Tetris 7-bag guarantees fairness. Every region has variety | Stratified patient sampling — every test cohort has balanced demographics. The bag guarantees representation | Balanced experimental design — every simulation covers the conformational space | Test matrix coverage — every suite exercises all code paths |

**The key insight (exp081)**: the math is universal. A Perlin noise field
that generates Minecraft elevation is the same function that generates a
patient risk-score distribution for healthSpring. The domain interpretation
changes — the underlying continuity, spatial correlation, and determinism
are identical. This is why barracuda's procedural module serves all springs.

**healthSpring procedural patient pools**: synthetic patient generation
from noise fields produces populations with naturalistic clustering
(nearby patients are demographically similar), smooth parameter variation,
and deterministic reproducibility. Combined with WFC for comorbidity
constraints and BSP for cohort partitioning, this is a complete
synthetic patient pipeline — no real patient data needed for development
and testing, with statistical properties matching real-world distributions.

### Symphony Pipeline → Cross-Spring Concurrent Compute

The symphony architecture (exp082, `metalForge/SYMPHONY_ARCHITECTURE.md`)
replaces synchronous CPU-or-GPU dispatch with a concurrent pipeline where
CPU and GPU operate simultaneously on different aspects of the same frame.
This pattern is universal across all springs — only the domain
interpretation changes.

| Spring | CPU Role (melody) | GPU Compute Role (harmony) | "Render" Role (beat) | Pipeline Depth |
|--------|------------------|--------------------------|---------------------|---------------|
| **ludoSpring** | Game logic, AI decision trees, state machine transitions, DAG ops | Physics batch, noise fBm, pathfinding wavefronts, engagement metrics | Frame render, post-processing, present | 2-3 frames (double/triple buffer) |
| **hotSpring** | Reactor control logic, safety interlocks, parameter scheduling | Neutron transport Monte Carlo, thermal hydraulics CFD, fuel depletion | Visualization, operator dashboard, alarm rendering | 1-2 frames (latency-sensitive for safety) |
| **wetSpring** | Molecular topology management, bond graph updates, constraint propagation | Force evaluation (Lennard-Jones, Coulomb), Verlet integration, energy minimization | Trajectory output, molecular viewer, property time series | 2 frames (double buffer for trajectory streaming) |
| **healthSpring** | Patient state machine, treatment protocol evaluation, consent gates | Population simulation (10K+ synthetic patients), risk scoring batch, comorbidity WFC | Dashboard render, patient timeline, alert visualization | 1 frame (clinical latency requirements) |
| **primalSpring** | Test orchestration, IPC routing, capability discovery | Load generation, data processing batch, coverage analysis | Report generation, CI dashboard, provenance graph render | 2 frames (throughput over latency) |

**The symphony insight**: effective frame time = max(CPU, GPU), not sum.
When CPU takes 4ms and GPU takes 8ms, the sequential model costs 12ms.
The symphony model costs 8ms — a 33% improvement from concurrency alone.
On our local hardware (RTX 4060 + Ryzen 5800X3D), exp082 validates that
a typical mixed workload fits in 60 Hz with 6ms headroom.

**Double buffering is universal**: the zero-stall buffer swap pattern
(CPU writes buffer B while GPU reads buffer A) applies identically to:
- Game frames (ludoSpring)
- Simulation timesteps (hotSpring reactor ticks)
- MD integration steps (wetSpring force evaluation)
- Patient processing batches (healthSpring population updates)

The buffer contents differ — game state vs neutron flux vs atomic
coordinates vs patient records — but the concurrency protocol is identical.
This is why `metalForge/forge` implements `DoubleBuffer` as a domain-agnostic
primitive that all springs can consume.

**Silicon exploitation**: consumer GPUs (RTX 4060) have ~52% of their die
area idle during compute-only dispatch. The symphony model assigns every
functional unit a role — tensor cores for DF64 precision, RT cores for
spatial queries, TMUs for lookup tables, ROPs for histogram reduction.
This is the hotSpring "every piece of silicon" philosophy applied
universally. For springs with precision-critical math (wetSpring molecular
dynamics, hotSpring reactor physics), the DF64 pattern provides 8-16x
throughput improvement over native FP64 on consumer hardware.

**Persistent GPU state**: the biggest win from the symphony model is
amortizing GPU dispatch overhead across frames. exp030 showed GPU dispatch
costs ~1.5ms cold. With persistent state (data stays resident in VRAM/L2
across frames), only deltas transfer over PCIe. Over 60 frames, this saves
>90% of PCIe bandwidth — turning the "CPU always wins for small workloads"
observation into "GPU wins for persistent workloads."

---

## Part 3: New Science Systems → Future Lysogeny Targets

The Lysogeny catalog currently has 7 targets (Usurper, Integrase,
Symbiont, Conjugant, Quorum, Pathogen, Novel Ferment Transcript). These
are all grounded in microbial ecology. The cross-domain learning above
reveals additional targets from other scientific domains.

| Proposed Target | Source Science | Game Mechanic | Prior Art |
|----------------|---------------|---------------|-----------|
| **Cascade** | Trophic cascades (Paine 1966, Estes 1998) | Removing a key actor triggers chain reaction through the system. Economy/faction/ecology destabilization | Ecology: pre-game, pre-patent. Well-published since 1960s |
| **Percolant** | Percolation theory (Broadbent & Hammersley 1957) | Network connectivity at critical thresholds. Map solvability, information reachability, social connectivity | Physics/math: pre-game, pre-patent. Published 1957 |
| **Resonant** | Physical resonance (Euler 1744, Helmholtz 1863) | Frequency-matched actions amplify effects. Timing windows, parry systems, cooperative abilities that "resonate" | Physics: pre-game by centuries |
| **Diffusant** | Fick's laws of diffusion (Fick 1855), SIR model (Kermack & McKendrick 1927) | Information/rumor/influence spread through populations. Controllable propagation rate | Physics + epidemiology: pre-game by over a century |
| **Auxotroph** | Metabolic dependency (Davis 1950, Romine 2017) | Forced cooperation from metabolic incompleteness. No individual is self-sufficient | Microbiology: well-published, not game-patented |
| **Catalyst** | Chemical catalysis (Berzelius 1835, Sabatier 1902) | Rare items that enable transformations without being consumed. Economy bottleneck design | Chemistry: pre-game by two centuries |

Each proposed target follows the Lysogeny provenance requirements:

1. Published paper predating any game patent
2. barraCuda primitive implementing the model
3. ludoSpring experiment validating the game mechanic
4. Cross-domain mapping table proving generality
5. Alternative non-game use case
6. AGPL-3.0-or-later on all code

---

## Part 4: The Cross-Spring Learning Matrix

Every spring both teaches and learns from every other spring. This
matrix captures the bidirectional flow:

### What ludoSpring Teaches Other Springs

| Spring | What We Teach | Mechanism |
|--------|-------------|-----------|
| wetSpring | Engagement science for bioinformatics UIs. Why do researchers abandon tools? Hick's law on CLI interfaces. Flow for long pipeline runs. Tufte on genome browsers | ludoSpring metrics consumed by petalTongue for wetSpring UI |
| neuralSpring | Game tree complexity as model evaluation metric. Training a model = searching a game tree. Hyperparameter optimization = deckbuilding (selecting from combinatorial space) | exp050 game tree math directly applicable to architecture search |
| hotSpring | Real-time budget management. GPU frame time budgets = reactor simulation timestep budgets. Same scheduling problem | exp037 tick budget directly transferable |
| healthSpring | Patient journey as game session. DDA for treatment difficulty. Flow for rehabilitation engagement. Pathogen detection for predatory health apps | Therapeutic game design using validated ludoSpring models |
| airSpring | Growing season as roguelike run. Each season faces procedural weather. Meta-progression = soil health accumulating across seasons. Conjugant mechanic for agricultural knowledge transfer | exp058 Conjugant directly models iterative growing seasons |
| primalSpring | IPC API design through Hick's lens. Too many capabilities = choice paralysis. Capability namespacing = menu hierarchy. The Hick-optimal API has log₂(n+1) depth | exp006 Hick analysis on API surface area |

### What Other Springs Teach ludoSpring

| Spring | What They Teach | Mechanic We Gain |
|--------|----------------|-----------------|
| wetSpring | Microbial community dynamics. Real data on cooperation, competition, quorum sensing, horizontal gene transfer, biofilm formation | Every Lysogeny mechanic (exp055-060) is grounded in wetSpring biology. New microbial models = new game mechanics |
| neuralSpring | Generative AI for NPC dialogue. Diffusion models for procedural art. Reinforcement learning for adaptive opponents | Squirrel integration for RPGPT. Neural models as DM brains, voice generators, adaptive NPCs |
| hotSpring | GPU compute patterns. Shader optimization. Parallel algorithms for physics simulation | barraCuda GPU primitives that ludoSpring consumes for real-time game computation |
| healthSpring | Human psychology models. Addiction science (Pathogen defense). Cognitive load research. Accessibility data from clinical populations | Pathogen catalog (exp060) grounded in clinical addiction research. Accessibility models (exp015) informed by clinical motor data |
| airSpring | Long-term system modeling. Multi-season dynamics. Soil as living system with memory | Sandbox/simulation mechanics where the world has memory spanning long time horizons |
| primalSpring | Composition patterns. How independent agents with self-knowledge compose into larger systems without central coordination | biomeOS deploy graphs, Songbird discovery, capability-based composition = how game systems compose at runtime |

### The Multiplier Effect

The value of cross-spring learning is multiplicative, not additive:

- wetSpring discovers a new quorum sensing pathway →
- ludoSpring models it as a game mechanic (new Lysogeny target) →
- RPGPT uses it for emergent NPC behavior →
- healthSpring applies the same math to patient population dynamics →
- airSpring applies it to soil microbiome management →
- The original wetSpring researcher sees their discovery powering
  5 different domains through one set of validated math

This is why springs never import each other. They coordinate through
shared math (barraCuda), shared provenance (wateringHole handoffs),
and shared principles (ecoBin, SCYBORG). The learning crosses domains
through the math, not through the code.

---

## Part 5: Why Every Game in the Catalog Matters for Science

Revisiting the 14 genre profiles with the cross-learning lens:

| Genre | Core System Lesson | Scientific Application |
|-------|-------------------|----------------------|
| CRPG | Hidden state + knowledge bounds + memory | Any system with agents that have partial information and persistent memory. Researchers, clinicians, sensors |
| Roguelike | Iterative improvement under uncertainty. Failure as data | Experimental design. Failed experiments ARE data. The lab meta-progresses |
| Investigation | Hypothesis formation from incomplete evidence. Deduction DAGs | Scientific method itself. Every investigation game is a scientific method simulator |
| Strategy | Resource allocation under constraint with incomplete information | Grant funding, lab resource allocation, computational budget management |
| FPS | Real-time systems under adversarial conditions. Latency, ordering, authority | Distributed computing, sensor networks, real-time monitoring |
| Sandbox | Emergence from simple rules. Complex behavior from agent interactions | Complex adaptive systems. Microbiomes, ecosystems, economies, cities |
| Card | Combinatorial optimization. Information-theoretic sampling | Feature selection, experimental design, statistical sampling |
| Immersive Sim | Multiple valid solutions to the same problem. Spatial discovery | Exploratory science. Multiple valid hypotheses for the same data. Discovery through investigation |
| JRPG | Time as scarce resource. Forced prioritization between competing goals | Research career management. Limited time, many possible experiments. Which do you run? |
| Horror | Degrading system coherence. Increasing disorder | System failure modes. Understanding how systems degrade helps design resilient ones |
| Extraction | Provenance chains. Every item has history. Trust through verification | Supply chain integrity, sample chain of custody, data provenance |
| MOBA | Team composition optimization. Synergy and counter-play | Multi-drug therapy design. Drug synergies and antagonisms. Same interaction matrix |
| Looter/MMO | Long-term engagement economics. Content vs compulsion | Long-term study design. How do you keep participants engaged for years without coercion? |
| Battle Royale | Spatial compression forcing interaction. Phase transitions | Experimental concentration. Increasing selective pressure forces evolutionary events |

---

## Faculty Anchors

- Gause, G.F. (1934) — competitive exclusion as game balance
- Paine, R.T. (1966) — trophic cascades as economy design
- Van Valen, L. (1973) — Red Queen dynamics as difficulty scaling
- Bak, P. et al. (1987) — self-organized criticality as emergent narrative
- Broadbent, S.R. & Hammersley, J.M. (1957) — percolation as map design
- Kermack, W.O. & McKendrick, A.G. (1927) — SIR as narrative propagation
- Fick, A. (1855) — diffusion laws as information spread
- Anderson, P.W. (1958) — localization as sanity mechanics
- Bronstein, J.L. (2001) — mutualism-parasitism continuum as interaction design
- Shannon, C.E. (1948) — information theory as deckbuilding
- Davis, B.D. (1950) — auxotrophy as forced cooperation
- Sabatier, P. (1902) — catalysis as crafting bottleneck design
- Lenski, R.E. et al. (1991) — LTEE as roguelite meta-progression
- Carmack, J. (1999) — frame pipelining as concurrent CPU/GPU symphony
- Dekker, T.J. (1971) — double-float arithmetic as consumer GPU precision
- NVIDIA (2024) — async concurrent execution as pipeline concurrency model

## License

AGPL-3.0-or-later

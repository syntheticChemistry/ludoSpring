# Game Quality Profiles — What Makes Games Good and Bad

**Date**: March 18, 2026
**Status**: Living document — profiles evolve as RPGPT design matures
**License**: AGPL-3.0-or-later
**Purpose**: Structured catalog of exemplar and anti-pattern games across genres, mapped to ludoSpring's validated HCI models. Informs RPGPT's quality targets and DDA tuning.
**Methodology**: Each profile maps observable game qualities to the 13 validated models (Fitts, Hick, Steering, GOMS, Flow, DDA, Four Keys, Engagement, Perlin, WFC, L-systems, BSP, Tufte). Quality discrimination via exp040.

---

## Why This Matters

Every spring needs a target. wetSpring mines bioinformatics literature and Reddit
threads for the real problems scientists face (broken pipelines, irreproducible
analyses, vendor lock-in). neuralSpring aims for PyTorch parity on validated AI
models. hotSpring characterizes real GPU hardware behavior.

ludoSpring needs both:
- **The science of games** — validated HCI models, reproducible experiments
- **What makes a good game** — curated exemplar profiles that define quality targets

RPGPT cannot aim at "good" without knowing what "good" looks like across genres.
These profiles are the training data for design intuition.

### The Lysogeny Purpose: Tools for Creatives, Not Publishers

The Lysogeny catalog exists so that **one person with the right tools can
make Stardew Valley**. Not so that a publisher can extract more microtransactions
from a live-service game.

Every exemplar in this document shares a trait: they were made by small teams
or individuals with creative vision. Disco Elysium was a novel that became a
game. Stardew Valley was one person's four-year labor of love. Into the Breach
was a three-person team. Hades was a studio that chose to tell a story about
failure as growth. Return of the Obra Dinn was one developer who invented a
visual style to solve a design problem.

Every anti-pattern shares a different trait: the creative team was subordinate
to the business model. Anthem had hundreds of talented developers locked into
a game designed by committee and marketing mandate. The mobile gacha industry
employs brilliant engineers to implement Skinner boxes. Live-service shooters
have art teams capable of masterpieces, constrained to cosmetic skin factories.

**The tools we build are for the creatives.** The Lysogeny mechanics (Usurper,
Integrase, Symbiont, Conjugant, Quorum) are not proprietary gates — they are
open math that anyone can use. The Pathogen catalog exists to detect and expose
exploitation, not to enable it. RPGPT exists so that a single narrative
designer with a vision can produce bounded adventures with the depth of Disco
Elysium, without needing a AAA budget.

The games are good because of the teams, the creatives, the effort. Not the
marketing department. Not the publishers. That way lies the microtransactions.

---

## Measurement Framework

Every profile is evaluated against the same validated models:

| Model | What It Measures | Key Metric | Experiment |
|-------|-----------------|------------|------------|
| Flow (Csikszentmihalyi 1990) | Challenge/skill balance → optimal experience | FlowState enum | exp012, exp040 |
| Engagement (Yannakakis & Togelius 2018) | Activity density, exploration, persistence | Composite 0–1 | exp010, exp040 |
| Four Keys (Lazzaro 2004) | Fun type: Hard/Easy/People/Serious | FunKey enum | exp018, exp040 |
| DDA (Hunicke 2005) | Difficulty adaptation responsiveness | Adjustment magnitude | exp020, exp040 |
| Hick's law (Hick 1952) | Choice complexity / information overload | Decision time ∝ log₂(n+1) | exp006, exp016 |
| Fitts's law (Fitts 1954) | UI target acquisition cost | Movement time = a + b·log₂(D/W+1) | exp005, exp015 |
| Tufte data-ink (Tufte 1983) | Information density / visual noise | Data-ink ratio | exp003, exp016 |
| Steering law (Accot & Zhai 1997) | Path-constrained navigation cost | Time ∝ A/W | exp007 |
| GOMS (Card, Moran, Newell 1983) | Cognitive task decomposition | KLM time sum | exp011 |

Anti-pattern detection uses:

| Model | What It Detects | Source |
|-------|----------------|--------|
| Pathogen (Skinner 1938, Kahneman & Tversky 1979) | Exploitative monetization, operant conditioning loops | exp060 |
| Quorum (Nealson & Hastings 1979) | Emergent collective behavior / phase transitions | exp059 |
| Usurper (Fisher 1930, Maynard Smith 1982) | Persistent adaptive NPC hierarchies | exp055 |

---

## Genre 1: CRPG / Narrative RPG

### Exemplar Tier — The Pinnacle

**Disco Elysium** (ZA/UM, 2019)

Why it's exceptional, mapped to our models:

| Model | Observation | ludoSpring Metric |
|-------|------------|-------------------|
| Flow | Challenge/skill balance in *conversation*, not combat. The "combat" is persuasion, deception, introspection | Dialogue plane Flow — exp012 channel width calibrated for non-combat challenge |
| Hick's law | Dialogue options are curated (typically 3–6). Never overwhelming. Each option meaningfully distinct | Hick time stays low — information overload avoided |
| Four Keys | Dominant: Serious Fun (narrative meaning) + People Fun (NPC relationships). Hard Fun absent — no "difficulty" in traditional sense | FunKey::SeriousFun primary |
| Engagement | Exploration breadth extreme — every object inspectable, every thought pursuable. But action density low — deliberation dominant | High exploration_breadth, high deliberate_pauses, moderate action_count |
| DDA | No traditional DDA. Difficulty is self-selected via skill allocation. The game trusts the player | Flat DDA curve — player agency over system adjustment |
| Tufte | UI is information-dense but clean. Thought Cabinet as spatial map of internal state. Minimal chrome | High data-ink ratio |
| Internal voices | **The defining innovation.** Skills are perspectives, not numbers. Each skill has personality, opinions, interjections. This is what separates Disco from every other RPG | RPGPT exp069-070 directly models this |
| NPC quality | NPCs have knowledge bounds, secrets, lies. Kim Kitsuragi remembers everything. Evrart Claire lies constantly with mechanical tells | RPGPT exp067-068 directly models this |

**Key design insight**: Flow state is achievable in pure dialogue. The challenge is
*understanding people*, not killing them. This proves Flow theory applies beyond
action games — confirming Csikszentmihalyi's original domain-agnostic formulation.

**Esoteric Ebb** (Cliche Studio, 2025)

Why it matters for RPGPT:

| Model | Observation | ludoSpring Metric |
|-------|------------|-------------------|
| Flow | Disco Elysium's internal voice model WITH real tabletop mechanics underneath. Skill checks have mechanical weight | Flow in both Dialogue and Tactical planes |
| Four Keys | Hard Fun (real mechanical challenge) + Serious Fun (narrative) + People Fun (NPC depth). The trifecta | Multiple FunKey hits per session |
| DDA | Tabletop-style difficulty — transparent dice, visible DCs, player can assess risk before committing | Visible DDA — player has agency over difficulty perception |
| Lysogeny | Uses tabletop mechanics that are prime SCYBORG/ORC targets. The math (dice pools, degree-of-success, skill trees) is replicable from published game design math | RulesetCert ingestible |

**Key design insight**: You can have Disco Elysium's narrative depth AND mechanical
crunch. The internal monologue system works with real dice rolls. This is RPGPT's
exact target — the Investigation + Dialogue + Tactical plane combination.

**Planescape: Torment** (Black Isle, 1999)

| Model | Observation | RPGPT Connection |
|-------|------------|-----------------|
| Flow | "What can change the nature of a man?" — a single question sustains 40+ hours because the answer branches based on character build | DAG branching depth — story tree as design metric (exp050) |
| Engagement | ~80% of content is optional. Player chooses depth. Completionists find 3x the content of speedrunners | Exploration breadth as engagement driver |
| Four Keys | Serious Fun dominant. The game is about meaning, not mastery | FunKey::SeriousFun |
| NPC quality | Morte lies to you for the entire game. Dak'kon has a secret that changes everything. Fall-from-Grace is a succubus who chose celibacy | Knowledge bounds + secrets + arc (exp067, exp072) |

**Baldur's Gate 3** (Larian, 2023)

| Model | Observation | RPGPT Connection |
|-------|------------|-----------------|
| Flow | D&D 5e mechanics create legitimate tactical challenge. Long rest economy forces resource management | Tactical plane Flow + resource tracking via DAG |
| Plane transitions | Seamless: Exploration → Dialogue → Tactical → back. World state persists. NPCs react to combat outcomes | RPGPT plane transitions (exp075) — this is the reference implementation |
| DDA | Difficulty modes exist but the real DDA is encounter design — multiple valid approaches to every fight | Multi-path resolution — DAG branching |
| Engagement | 200+ hour campaigns with sustained engagement. Replayability from build variety and narrative branching | Game tree complexity (exp050) |

### Anti-Pattern Tier — What Goes Wrong

**Generic "AI NPC" games (various 2024-2025 demos)**

| Model | Failure | ludoSpring Detection |
|-------|---------|---------------------|
| Flow | No challenge/skill balance. AI generates plausible text but there's no game loop | Flow = Boredom (challenge ≈ 0) |
| NPC quality | "Chatbot swill" — NPCs have no knowledge bounds, no secrets, no memory. They agree with everything, contradict nothing, reveal everything | RPGPT spec Part 2 exists specifically to prevent this |
| Hick's law | Infinite open-ended prompts → choice paralysis. "What do you want to say?" is worse than 4 curated options | Hick time → ∞ as options → ∞ |
| Four Keys | No fun type registers. The "game" is a conversation with an LLM wearing a costume | FunKey::None — no classification possible |

**Procedural narrative games with unbounded scope**

The Gumshoe/investigation game problem — Shadows of Doubt (ColePowered, 2023) and
similar procedurally generated detective games:

| Model | Observation | Bounding Solution |
|-------|------------|-------------------|
| Flow | Excellent premise — procedural murder mysteries. But cases feel samey because the generation grammar is shallow | DAG templates as story scaffolds — not fully procedural, but bounded branching within authored structure |
| Engagement | Early sessions high engagement. Repetition detection kicks in by hour 10–15 as patterns emerge | Quorum mechanic (exp059) — emergent events that break repetition |
| Hick's law | Open-world investigation with minimal guidance → information overload. Where do I even start? | GUMSHOE principle in RPGPT Investigation plane: core clues are automatic. Progress is guaranteed. Bonus rewards exploration |
| NPC quality | Procedurally generated NPCs lack depth. They have schedules and relationships but no personality, no secrets, no contradictions | NPC personality certs cannot be procedurally generated without a personality grammar — this is an open research problem |

**Key insight for RPGPT**: Pure procedural generation produces *breadth without
depth*. The solution is **authored scaffolds with procedural flesh** — the plot
skeleton is designed (like Macbeth's five acts), the specific details are
generated (which NPC plays which role, how events unfold within each act). The
DAG enforces the scaffold while allowing freedom within it.

---

## Genre 2: Roguelike / Roguelite

### Exemplar Tier

**Hades** (Supergiant, 2020)

| Model | Observation | ludoSpring Metric |
|-------|------------|-------------------|
| Flow | Near-perfect challenge/skill curve. Each run teaches something. Death is progress, not punishment | Flow maintained across difficulty ramp — DDA is implicit in player skill growth |
| Conjugant (exp058) | **This is the Conjugant mechanic.** Dead runs release "genes" (Mirror of Night upgrades). Survivors carry forward. Meta-progression = horizontal gene transfer | Roguelite progression mapped 1:1 to HGT + Price equation |
| Four Keys | Hard Fun dominant (mastery), with Serious Fun (narrative reason to keep dying) and People Fun (NPC relationships in the House) | Multiple FunKey types sustained simultaneously |
| Engagement | 20-minute run loops. High action density. Each run feels fresh due to boon combinations | Short session_duration, high action_count, high retry_count |
| DDA | God Mode (optional): +2% damage resistance per death. Elegant — player self-selects, no shame | Transparent DDA — player retains agency |

**Slay the Spire** (MegaCrit, 2019)

| Model | Observation | ludoSpring Metric |
|-------|------------|-------------------|
| Flow | Decision quality matters more than execution speed. Every card pick, every path choice, every relic interaction | Flow in decision-making, not reflexes |
| Hick's law | 3 card choices per reward. 2–3 path options per floor. Information is bounded at every decision point | Hick time stays manageable even as complexity grows |
| Game tree | ~10^80 estimated game tree per run (deck × relics × potions × paths × enemies). Every run genuinely novel | Game tree complexity (exp050) — validates "every game is novel data" |
| Four Keys | Hard Fun dominant. The joy is in the optimization puzzle | FunKey::HardFun |

**Caves of Qud** (Freehold Games, ongoing)

| Model | Observation | ludoSpring Metric |
|-------|------------|-------------------|
| Engagement | Exploration breadth is effectively infinite. Procedural history, factions, artifacts, mutations | Extreme exploration_breadth, extreme challenge_seeking |
| Flow | Difficulty is brutal and unforgiving. Flow state is narrow — experts only | Flow = Anxiety for new players, Flow for experienced |
| Four Keys | Serious Fun (world lore is extraordinary) + Hard Fun (survival challenge) | FunKey::SeriousFun + FunKey::HardFun |
| NPC quality | Procedurally generated NPCs with faction relationships, history, grudges. Best-in-class for procedural depth | Model for what NPC personality grammar could look like |

### Anti-Pattern Tier

**Procedural roguelikes with "content exhaustion"**

| Model | Failure | Detection |
|-------|---------|-----------|
| Flow | After learning the meta, challenge drops. Same optimal strategy every run | Flow → Boredom as skill outpaces challenge |
| Engagement | Exploration breadth exhausted — no new content to discover | exploration_breadth plateaus while action_count remains high |
| Four Keys | Only Hard Fun remains. When mastery is achieved, nothing is left | Single FunKey → zero FunKey transition |

---

## Genre 3: Investigation / Detective

### Exemplar Tier

**Return of the Obra Dinn** (Lucas Pope, 2018)

| Model | Observation | ludoSpring Metric |
|-------|------------|-------------------|
| Flow | Perfect challenge curve. Each deduction builds on the last. The "aha" moments are earned | Flow maintained through escalating deductive challenge |
| Hick's law | 60 fates to determine, but the UI chunks them into manageable groups. Temporal/spatial navigation constrains the possibility space | Hick time bounded by UI design despite massive solution space |
| Investigation plane | **The reference implementation.** Core information is always accessible (you can replay any death scene). Bonus deductions reward pattern recognition | GUMSHOE principle: core clues automatic, bonus rewards skill |
| Tufte | Monochromatic, 1-bit aesthetic forces focus on spatial relationships and body language. Maximum signal, minimum noise | Extreme data-ink ratio — every pixel is information |
| Four Keys | Hard Fun (deduction puzzle) + Serious Fun (uncovering the narrative) | FunKey::HardFun + FunKey::SeriousFun |

**The Case of the Golden Idol** (Color Gray Games, 2022)

| Model | Observation | RPGPT Connection |
|-------|------------|-----------------|
| Flow | Each scene is a bounded deduction puzzle. Scope is always clear | Bounded adventure — the investigation has walls |
| Investigation plane | Evidence DAG is explicit — you drag words into sentences to form deductions. The mechanic IS the DAG | Evidence DAG as first-class UI element |
| Hick's law | Word bank per scene is bounded (30-50 words). Possible deductions are constrained by grammar | Bounded generation — grammar constrains output |

**Her Story** (Sam Barlow, 2015)

| Model | Observation | RPGPT Connection |
|-------|------------|-----------------|
| Flow | The player is the search engine. Challenge is in knowing what to search for | Investigation as self-directed discovery |
| NPC quality | One NPC, one actress, but the knowledge bounds are deeply layered. She lies, she omits, she contradicts | Knowledge bounds at their most effective |
| Engagement | No explicit objectives. No completion meter. Yet players stay for hours because the mystery is intrinsically compelling | Intrinsic motivation > extrinsic reward |

### Anti-Pattern Tier

**Walking simulators with false investigation**

| Model | Failure | Detection |
|-------|---------|-----------|
| Flow | No challenge. Walking between narrative triggers. Engagement is passive | Flow = Relaxation → Boredom (no challenge axis) |
| Hick's law | One option at each "decision point." The investigation is on rails | Hick time ≈ 0 — no actual decisions |
| Engagement | action_count near zero. deliberate_pauses dominate. Player is a passenger | Engagement composite bottoms out |

---

## Genre 4: Strategy / Tactics

### Exemplar Tier

**Into the Breach** (Subset Games, 2018)

| Model | Observation | ludoSpring Metric |
|-------|------------|-------------------|
| Flow | Perfect information. Every enemy move shown before your turn. Challenge is pure optimization | Flow via decision quality, not information gathering |
| Hick's law | Typically 3 mechs × 2 actions = 6 decisions per turn. Bounded. Yet combinatorial depth is massive | Low Hick time, high game tree depth |
| Tufte | Every tile is information. No decorative elements. Grid is the game | Data-ink ratio approaching 1.0 |
| DDA | Island difficulty + optional objectives = player-controlled challenge ramp | Player-driven DDA — the gold standard |
| Four Keys | Hard Fun pure. The puzzle is the entire experience | FunKey::HardFun |

**XCOM 2** (Firaxis, 2016)

| Model | Observation | ludoSpring Metric |
|-------|------------|-------------------|
| Flow | Imperfect information + probability creates tension. "95% hit chance → miss" is the most discussed moment in gaming | Flow from uncertainty — Flow channel widens |
| DDA | Strategic layer (Avatar Project timer) creates macro-pressure. Tactical layer has per-mission difficulty | Multi-scale DDA — session and campaign level |
| Four Keys | Hard Fun (tactical mastery) + Serious Fun (soldier attachment → permadeath grief) | FunKey transition: HardFun → SeriousFun when a soldier dies |
| Usurper (exp055) | ADVENT's adaptive response to player strategy mirrors Usurper dynamics. They counter your specialization | Persistent adaptive opposition — NPC hierarchy |

### Anti-Pattern Tier

**Information-overloaded grand strategy**

| Model | Failure | Detection |
|-------|---------|-----------|
| Hick's law | Hundreds of decisions per turn, poor hierarchical grouping. Players optimize one system and ignore the rest | Hick time → decision paralysis. exploration_breadth collapses to narrow optimization |
| Tufte | UI drowns in chrome. Important information buried under decorative elements | Low data-ink ratio. Critical state hidden |
| Flow | Veterans in Flow, new players in Anxiety. No middle ground | Bimodal Flow distribution — bad sign |

---

## Genre 5: Action / FPS

### Exemplar Tier

**DOOM (2016)** (id Software)

| Model | Observation | ludoSpring Metric |
|-------|------------|-------------------|
| Flow | "Push forward combat" — health drops from Glory Kills force aggression. The game pushes you into Flow by design | DDA mechanism: low health → more aggressive → more Glory Kills → health restored. Self-correcting loop |
| Fitts's law | Large targets (demons), generous aim assist. Weapon switching is fast (GOMS task time minimized) | Low Fitts cost per target acquisition |
| Engagement | Extreme action_count. Zero deliberate_pauses. Session is pure kinetic energy | Maximum action density |
| Four Keys | Hard Fun pure. The Fun is the challenge of surviving the arena | FunKey::HardFun at maximum intensity |

**Titanfall 2** (Respawn, 2016)

| Model | Observation | ludoSpring Metric |
|-------|------------|-------------------|
| Flow | Wallrunning + combat + puzzle rooms + time travel. Constant mode-shifting prevents adaptation plateau | Multi-plane transitions maintaining Flow — never stays in one mode long enough for Boredom or Anxiety |
| Steering law | Movement through narrow spaces at high speed is Steering law in action. The game is a steering task | Steering cost as difficulty parameter |
| DDA | Single-player campaign has implicit DDA — each level introduces one mechanic, masters it, then adds another | Sequential mechanic introduction = controlled challenge ramp |

### Anti-Pattern Tier

**Generic military shooters**

| Model | Failure | Detection |
|-------|---------|-----------|
| Flow | Difficulty is binary: too easy (aim assist) or too hard (competitive multiplayer). No channel | Bimodal: Boredom or Anxiety, rarely Flow |
| Four Keys | Hard Fun only, and only in PvP. Campaign is tutorial | FunKey variety = zero |
| Engagement | High action_count but low exploration_breadth. Corridor design | action_count / exploration_breadth ratio extremely high |

---

## Genre 6: Sandbox / Emergent

### Exemplar Tier

**Dwarf Fortress** (Bay 12, 2006–ongoing)

| Model | Observation | ludoSpring Metric |
|-------|------------|-------------------|
| Quorum (exp059) | **The Quorum mechanic in action.** When enough dwarves reach a mood threshold, tantrum spirals emerge. Collective phase transition = emergent narrative | Emergent events from agent-based threshold dynamics |
| Engagement | Infinite emergent stories. No two fortresses alike. Maximum exploration_breadth | Engagement from emergence, not authored content |
| Usurper (exp055) | Noble hierarchy, power struggles, assassination. Persistent NPCs with grudges and relationships | Adaptive NPC hierarchy = Usurper dynamics |
| Hick's law | The barrier to entry. Hundreds of systems, thousands of options. Legendary complexity | Hick time is the primary accessibility barrier |

**Factorio** (Wube Software, 2020)

| Model | Observation | ludoSpring Metric |
|-------|------------|-------------------|
| Flow | Sustained Flow for 10+ hour sessions. Challenge scales with ambition — player sets their own complexity | Self-directed difficulty → persistent Flow |
| Engagement | "One more production line" loop. Time distortion is a signature of Flow state | Session duration extreme, deliberate_pauses minimal |
| Four Keys | Hard Fun (optimization) + Easy Fun (the factory grows visually) | FunKey::HardFun + FunKey::EasyFun |
| Crafting plane | The entire game is the Crafting plane. Recipe discovery, material transformation, optimization | RPGPT Crafting plane reference |

### Anti-Pattern Tier

**Sandbox survival games with no emergent narrative**

| Model | Failure | Detection |
|-------|---------|-----------|
| Engagement | First 10 hours high (explore, build, survive). Then exploration_breadth exhausts | engagement.composite drops after initial peak |
| Flow | Survival mechanics create Anxiety early, Boredom late. No Flow channel | Flow state never achieved — wrong difficulty curve |
| Four Keys | Only Easy Fun (building) survives. No Hard, Serious, or People Fun | FunKey collapses to single type |

---

## Genre 7: Card / Deckbuilding

### Exemplar Tier

**Magic: The Gathering — Commander format**

| Model | Observation | ludoSpring Metric |
|-------|------------|-------------------|
| Game tree | ~10^358 game tree (exp049 — proven). Every game genuinely novel data | Maximum possible game tree complexity |
| Four Keys | All four simultaneously: Hard (deckbuilding optimization), Easy (spectacle combos), People (4-player politics), Serious (self-expression through deck identity) | Only game that reliably hits all four FunKeys |
| Pathogen (exp060) | **Critical warning.** Pre-built commander decks and "staples" culture contract the game tree ×0.036 (exp050). Monetization of power creep is the Pathogen | Pathogen detection: game tree contraction rate |
| Quorum (exp059) | 4-player politics. "When do we stop the leader?" is a quorum threshold decision | Political plane dynamics |

### Anti-Pattern Tier

**Pay-to-win CCGs / Gacha**

| Model | Failure | Detection |
|-------|---------|-----------|
| Pathogen (exp060) | Operant conditioning loops (random reward schedules, loss aversion, sunk cost). The Pathogen catalog documents these precisely | Pathogen score high — exploitation detected |
| Flow | Pay-to-win eliminates challenge for whales and creates despair for F2P. No one is in Flow | Flow = Boredom (whales) or Anxiety (F2P) |
| Game tree | Net-decking + meta-solving contract the tree. Games are solved before they begin | Game tree complexity drops below roguelike levels |
| Four Keys | Only Easy Fun (spectacle) and compulsion (not a Fun Key — that's the point) | fun_composite near zero despite high playtime |

---

## Genre 8: Immersive Sim

### Exemplar Tier

**Deus Ex** (Ion Storm, 2000) / **Prey** (Arkane, 2017)

| Model | Observation | ludoSpring Metric |
|-------|------------|-------------------|
| Flow | Multiple valid approaches to every problem (stealth, combat, hacking, social). Player finds their own Flow channel | Multi-path Flow — skill diversity widens the channel |
| Engagement | Extreme exploration breadth. Every vent, every terminal, every NPC conversation is a potential path forward | Exploration breadth as primary engagement driver |
| Four Keys | Hard Fun (system mastery) + Serious Fun (narrative discovery) + Easy Fun (powers/gadgets spectacle) | 3 simultaneous FunKeys |
| Hick's law | High option count per encounter, but options are discovered organically through exploration, not presented all at once | Hick time bounded by spatial discovery, not menu presentation |

**Key design insight**: The Immersive Sim proves that high option counts do not
cause Hick paralysis IF options are discovered spatially rather than presented
simultaneously. This maps to RPGPT's Investigation plane — clues are found in the
environment, not listed in a menu.

### Anti-Pattern Tier

**Illusion-of-choice Immersive Sims**

| Model | Failure | Detection |
|-------|---------|-----------|
| Engagement | Multiple paths exist but all converge to the same outcome. Exploration breadth is cosmetic | Exploration breadth / narrative branching ratio near zero |
| Flow | Only one approach is actually viable at any difficulty level. "Choose stealth or combat" but combat is suicide | Single-path Flow — skill diversity doesn't widen the channel |

---

## Genre 9: JRPG / Turn-Based RPG

### Exemplar Tier

**Persona 5** (Atlus, 2016)

| Model | Observation | ludoSpring Metric |
|-------|------------|-------------------|
| Flow | Dual loop: social sim (time management) + dungeon crawl (turn-based combat). Each supports the other's Flow | Multi-plane Flow — Exploration + Dialogue + Tactical |
| Symbiont (exp057) | Social links ARE the Symbiont mechanic. Relationships with NPCs directly power combat abilities | Faction/reputation system directly integrated with mechanical progression |
| Engagement | Calendar system creates urgency. Every day matters. Opportunity cost for every action | Time pressure as engagement amplifier |
| Four Keys | People Fun (social links) + Hard Fun (boss fights) + Easy Fun (stylish presentation) + Serious Fun (societal commentary) | All four FunKeys |
| Plane transitions | Seamless rotation: School (Exploration) → Social Link (Dialogue) → Palace (Tactical) → Mementos (Investigation) | RPGPT plane transition reference (exp075) |

**Key design insight**: The JRPG model proves that Plane transitions can be
calendar-driven. Time as a resource forces meaningful choices about which plane
to engage with. This is a bounding mechanism — you cannot do everything.

### Anti-Pattern Tier

**Grind-focused JRPGs**

| Model | Failure | Detection |
|-------|---------|-----------|
| Flow | Random encounters with trivially easy enemies. Flow = Boredom for 80% of playtime, then sudden Anxiety at boss gates | Bimodal Flow — long Boredom stretches punctuated by Anxiety spikes |
| Engagement | High action_count but zero exploration_breadth. Player walks in circles killing the same enemies | action_count / exploration_breadth ratio extreme |
| DDA | No DDA. Level = power. If you're stuck, grind more | DDA flat — difficulty is a number, not a system |

---

## Genre 10: Horror / Survival Horror

### Exemplar Tier

**Resident Evil 4** (Capcom, 2005/2023 Remake)

| Model | Observation | ludoSpring Metric |
|-------|------------|-------------------|
| Flow | Dynamic difficulty is invisible and aggressive. Dying lowers enemy count, damage, and aggression. Succeeding increases them | DDA at its most effective — player never notices |
| DDA | Internal "rank" system: player performance continuously adjusts 15+ parameters. Published analysis proves it | Hidden DDA as design reference (Hunicke 2005) |
| Four Keys | Hard Fun (combat tension) + Easy Fun (spectacle violence) + Serious Fun (horror atmosphere) | Three FunKeys |
| Engagement | Resource scarcity creates deliberation. Every bullet matters. High deliberate_pauses relative to action_count | Deliberation as engagement signal |

**Silent Hill 2** (Konami, 2001)

| Model | Observation | RPGPT Connection |
|-------|------------|-----------------|
| Internal voices | The town IS the internal voice system. Every monster, every location, every NPC is a projection of James's psychology | Internal voices as world design, not just character mechanic |
| NPC quality | Every NPC is lying. Every NPC has hidden motivations. Every NPC knows something James doesn't — including about James | Knowledge bounds at their most thematically rich |
| Engagement | Low action density. High deliberation. Exploration is dread | Deliberation and exploration as horror engagement |
| Anderson W | **Sanity as disorder parameter.** As understanding increases, the world becomes less coherent. This IS the Anderson localization insight applied to narrative | World 3 (Lovecraftian Investigation) directly uses this model |

### Anti-Pattern Tier

**Jump-scare horror games**

| Model | Failure | Detection |
|-------|---------|-----------|
| Flow | Tension is binary: walking (Boredom) → jump scare (Anxiety). No sustained channel | Flow never achieved — oscillation between Boredom and Anxiety |
| Engagement | After 5 scares, habituation kills engagement. Predictable rhythm = zero surprise | Engagement decay rate accelerates — opposite of Flow maintenance |
| Four Keys | Only Easy Fun (startle response). No Hard, Serious, or People Fun | Single FunKey, and it's the weakest type |

---

## Genre 11: Extraction Shooter

### Exemplar Tier

**Escape from Tarkov** (Battlestate Games, 2017)

Tarkov is already modeled — exp053 and exp054 use it as the prototype for the
provenance trio's anti-cheat system. The game's design is the reason.

| Model | Observation | ecoPrimals Solution |
|-------|------------|-------------------|
| Flow | Every raid is a Flow generator. High stakes (gear loss), high skill demand, time pressure, spatial awareness. The extraction timer is a Flow anchor — skill must match challenge before time runs out | Flow monitoring per-raid (exp074 pattern) |
| Engagement | 20-45 minute raid loops. Extreme deliberation (every corner could be death). Low action density, high consequence per action | Engagement from consequence, not frequency |
| DDA | No DDA. Matchmaking is loose. New players face veterans. This is both the appeal (earned mastery) and the barrier (punishing early experience) | DDA could be applied to loot scarcity, not combat — a design opportunity |
| Four Keys | Hard Fun dominant (survival mastery) + Serious Fun (gear attachment, loss grief) | Permadeath attachment maps to Conjugant — except runs don't transfer value, which is a design choice |
| Pathogen risk | Flea market economy, RMT (real-money trading), carry services. The economic layer invites exploitation | exp060 Pathogen detection directly applicable |

**How ecoPrimals solves Tarkov's core problems:**

| Problem | Current (Tarkov) | ecoPrimals Solution | Experiment |
|---------|-----------------|-------------------|------------|
| Cheating (aimbot, ESP, speed) | BattlEye (proprietary, cat-and-mouse) | Provenance DAG — every action is a content-addressed vertex. Impossible actions have no valid parent. Cheating is a structural impossibility, not a detection problem | exp053 (8 fraud types) |
| Item duplication | Server-side inventory checks | loamSpine certificates — every item is a unique cert. Duplication requires forging a certificate, which requires BearDog's private key | exp053 + exp064 |
| RMT / carry services | Ban waves (reactive) | sweetGrass attribution — every transfer is recorded. Anomalous transfer patterns (always giving, never receiving) are detectable | exp066 |
| Desync / "netcode" | Proprietary server authority | rhizoCrypt DAG — both client and server maintain DAGs. Divergence is detectable and reconcilable. The DAG IS the netcode | exp053 (honest raid DAG) |
| Bullet tracking | Server-side ballistics | Every round is a loamSpine cert. Every shot is a DAG vertex referencing the round cert. Phantom rounds (shot without possessing the round) are structurally impossible | exp053 (phantom round detection) |

**Key insight**: Tarkov's anti-cheat problem is a provenance problem. If every
action, every item, every bullet has an immutable chain of custody, cheating
becomes structurally impossible rather than behaviorally detectable. This is
why exp053 exists — it's not about games, it's about proving that provenance
DAGs eliminate entire categories of fraud.

### Anti-Pattern Tier

**Extraction shooters with pay-to-win loadouts**

| Model | Failure | Detection |
|-------|---------|-----------|
| Flow | Purchased gear eliminates the gear progression loop. Challenge/skill balance collapses when money substitutes for skill | Flow = Boredom (whale) or Despair (F2P) |
| Pathogen | "Buy a loadout" is a Pathogen pattern. Loss aversion (you might lose the gear you paid for) drives further spending | exp060 directly detects paid-advantage patterns |
| Conjugant | Meta-progression corrupted — instead of runs teaching you, money teaches you nothing. The Conjugant loop is broken | Run value drops to zero when progression is purchasable |

---

## Genre 12: MOBA (Multiplayer Online Battle Arena)

### Exemplar Tier

**Dota 2** (Valve, 2013)

| Model | Observation | ecoPrimals Solution |
|-------|------------|-------------------|
| Flow | 5v5 with extreme mechanical + strategic depth. Flow channel is narrow but deep — when you're in it, nothing else exists | Flow monitoring for team-based games — aggregate team Flow, not just individual |
| Game tree | ~10^200 estimated per match (hero picks × item builds × lane strategies × team coordination). Rivals MTG in combinatorial depth | Game tree complexity (exp050) — validates "every match is novel data" |
| Four Keys | Hard Fun (mechanical mastery) + People Fun (teamwork, communication) + Serious Fun (strategic depth). All three simultaneously for 40+ minutes | Multi-Fun-Key sustained in multiplayer context |
| DDA | MMR-based matchmaking IS the DDA. External to the game itself — players of similar skill face each other. The game never adjusts internally | External DDA via Songbird matchmaking — skill rating as capability discovery |
| Quorum (exp059) | Team fights are Quorum events. Individual positioning decisions aggregate until a threshold triggers engagement. The "teamfight" is an emergent phase transition | Agent threshold dynamics → collective combat event |
| Symbiont (exp057) | Hero synergies and counters ARE the Symbiont faction graph. Interaction coefficients between heroes determine team composition viability | Multi-species Lotka-Volterra as hero balance math |

**How ecoPrimals solves MOBA:**

| Capability | Primal | What It Does |
|------------|--------|-------------|
| Match DAG | rhizoCrypt | Every action (spell cast, item purchase, ward placed, kill) is a vertex. Full match reconstruction from DAG. Replay is DAG traversal |
| Hero balance | ludoSpring + barraCuda | Symbiont interaction matrix (exp057) as balance tool — hero winrates modeled as Lotka-Volterra equilibria. Balance patches = adjusting interaction coefficients |
| Team formation | Songbird | Capability-based matchmaking. "Find 4 players whose hero pools complement mine" is a capability discovery query |
| Anti-cheat | provenance trio | Scripting detection: action frequency analysis (exp053 speed violation). Map hack: information access without vision vertex |
| Spectator/coaching | petalTongue | Real-time game state visualization. DAG-powered replay with branching "what if" analysis |

### Anti-Pattern Tier

**MOBAs with aggressive monetization / hero paywalls**

| Model | Failure | Detection |
|-------|---------|-----------|
| Symbiont | Locking heroes behind paywalls breaks the Symbiont matrix — players can't access counters without paying. Balance becomes pay-to-compete | Symbiont completeness metric: what % of the interaction matrix is accessible to F2P? |
| Pathogen | Skin gambling, lootboxes, battle passes with FOMO timers. The game is free but the Pathogen score is extreme | exp060 detects all four Pathogen patterns (variable ratio, artificial scarcity, loss aversion, FOMO) |
| Flow | New player experience is catastrophic. 100+ heroes, each with 4+ abilities, complex item system. Hick paralysis before Flow is even possible | Hick time for new players → decision paralysis. No curated onboarding |

---

## Genre 13: Extraction / Looter / MMO Shooter

### Exemplar Tier

**Hunt: Showdown** (Crytek, 2019)

| Model | Observation | ecoPrimals Solution |
|-------|------------|-------------------|
| Flow | PvPvE creates unpredictable tension. AI enemies are noise that masks player movement. Flow from environmental awareness, not just aim | Multi-source threat Flow — richer than pure PvP |
| Engagement | Audio design IS the engagement system. Every sound is information. Footsteps, crows, gunshots at distance. Tufte principle applied to audio — every sound is data | Tufte data-ink applied to spatial audio as information channel |
| Four Keys | Hard Fun (gunplay mastery) + Serious Fun (bounty hunter atmosphere) + People Fun (duo/trio coordination) | Three Fun Keys in 30-minute loops |
| Quorum (exp059) | Server-wide events. When enough teams converge on a boss lair, the "banish" phase triggers a server-wide PvP convergence event. This IS quorum sensing — individual hunter movements aggregate to a collective threshold | Boss banish as quorum threshold event |

**Destiny 2** (Bungie, 2017) — mixed exemplar/anti-pattern

| Model | Observation | Warning |
|-------|------------|---------|
| Flow | Gunplay is best-in-class. The "30 seconds of fun" loop Bungie pioneered. Moment-to-moment gameplay is peak Flow | Mechanical Flow is excellent |
| Pathogen | But: Eververse store, season passes, expansion paywalls, FOMO seasonal content that gets "vaulted" (removed). The business model is a textbook Pathogen | Pathogen corrupts a mechanically excellent game |
| Engagement | Players report "playing out of obligation, not fun." Season pass completion deadlines create anxiety, not Flow | Engagement score high but fun_composite low = compulsion, not enjoyment |

**Key insight**: Destiny 2 is the cautionary tale. The game underneath is
exceptional. The business model on top is parasitic. This is what happens
when marketing overrides design. The Pathogen doesn't care how good your
gunplay is.

### Anti-Pattern Tier

**Live-service looter shooters that die within a year**

| Model | Failure | Examples |
|-------|---------|---------|
| Flow | Content treadmill. Same activities repeated at higher numbers. Challenge doesn't evolve, it scales | Anthem, Marvel's Avengers, Babylon's Fall |
| Engagement | "Endgame" is grinding the same content for better numbers. exploration_breadth drops to zero | Engagement collapse after campaign completion |
| Conjugant | Meta-progression is infinite but meaningless. Higher numbers don't change how you play | Conjugant loop without behavioral change = empty progression |
| Four Keys | Only Hard Fun (if the combat is good). No Serious, People, or Easy Fun in the endgame | Single Fun Key in a genre that needs three |

---

## Genre 14: Battle Royale

### Exemplar Tier

**Apex Legends** (Respawn, 2019)

| Model | Observation | ecoPrimals Solution |
|-------|------------|-------------------|
| Flow | Titanfall's movement system in a BR. Slides, climbs, ziplines. The movement creates Flow even when not fighting | Flow from traversal, not just combat — Steering law applies |
| Four Keys | Hard Fun (gunplay + movement mastery) + People Fun (squad communication, legend synergies) + Easy Fun (spectacle) | Three Fun Keys maintained |
| Quorum (exp059) | Ring closure forces convergence. The ring IS a quorum mechanism — it compresses the play space until agent density triggers engagement. Predictable phase transitions from exploration to combat | Spatial compression as quorum forcing function |
| DDA | Skill-based matchmaking (SBMM) is the DDA. Controversial because transparent — players know they're being matched. Invisible DDA (RE4) feels better | Visible vs invisible DDA as design parameter |

### Anti-Pattern Tier

**Battle royales with excessive RNG**

| Model | Failure | Detection |
|-------|---------|-----------|
| Flow | Loot RNG determines outcomes more than skill. Landing on a shotgun vs landing on nothing. Challenge/skill balance is random, not designed | Flow variance too high — some matches Flow, most don't |
| DDA | No DDA. 100 players, 1 winner. 99% of sessions end in loss. The genre has a structural Flow problem that only squad play partially addresses | Win rate too low for sustained Flow. Squad play (People Fun) compensates |

---

## Cross-Genre Patterns

### What Universally Makes Games Good

1. **Flow state is achievable** — challenge matches skill, with a channel wide enough for variance. Every exemplar above achieves this.

2. **Information is bounded at each decision point** — Hick's law is respected. Options are curated, not infinite. Even in complex games (Into the Breach, Slay the Spire), each individual decision is tractable.

3. **Multiple Fun Keys fire simultaneously** — Great games hit 2-3 Fun Keys. Legendary games hit all four. Single-Fun-Key games eventually exhaust.

4. **NPCs (if present) have hidden state** — Knowledge bounds, secrets, lies, arcs. The NPC knows things you don't. The NPC doesn't know things you do. The NPC lies about things you can detect.

5. **Death/failure is progress, not punishment** — Hades (Conjugant), roguelikes (meta-progression), Disco Elysium (conversation failure reveals new information). The best games transform failure into a different kind of success.

6. **The game trusts the player** — Player-driven DDA (Into the Breach optional objectives, DOOM's push-forward loop, Slay the Spire's path choices) outperforms system-driven DDA. The player should feel in control of their challenge level.

### What Universally Makes Games Bad

1. **Flow is structurally impossible** — bimodal difficulty (too easy / too hard), or no challenge axis at all (walking simulators, chatbot NPCs).

2. **Hick paralysis** — unbounded choice without curated guidance. "What do you want to do?" with no context is worse than 4 specific options.

3. **Single Fun Key exhaustion** — game relies on one type of fun that eventually depletes. Hard Fun alone runs out when mastery is achieved. Easy Fun alone runs out when novelty fades.

4. **Pathogen patterns** — random reward schedules, loss aversion exploitation, social pressure mechanics, artificial scarcity. These are parasitic, not mutualistic. exp060 catalogs them.

5. **Procedural breadth without authored depth** — infinite content that feels the same. The bounding problem. Generated NPCs without personality grammars. Generated quests without narrative scaffolds.

6. **Context window memory** — NPCs that forget, worlds that don't react, choices that don't matter. The antithesis of DAG-based structural memory.

7. **Random stats as loot system** — items differentiated only by number ranges (Fallout 4 Legendary prefixes, Diablo 3 vanilla Auction House era, generic looter shooters). The player's decision is "is this number bigger?" which is not a decision at all. Compositional mechanics (Noita, New Vegas unique weapons, Slay the Spire relic synergies) where items change *how you play* are strictly superior to items that change *how big your numbers are*.

---

## The Composition Principle: Mechanics > Numbers

### The Insight

Random stats within a set range are the boring loot solution. A sword with
+12 to +18 damage is a slot machine, not a game. The player isn't making
decisions — they're pulling a lever and hoping for a high roll. This is
the Pathogen pattern applied to item systems: variable-ratio reinforcement
schedule dressed up as "gameplay."

**The real fun happens when bounded mechanics stack compositionally.**
The emergent space of mechanic interactions is where absurd, memorable,
player-driven gameplay lives. The item isn't a number — it's a *system
contribution* that changes how other systems behave.

### Case Study: Fallout New Vegas vs Fallout 4

**Fallout: New Vegas** (Obsidian, 2010) — weapons are hand-crafted systems.
The Ratslayer is a unique varmint rifle with a night scope, silencer, and
extended mags. It's not "Varmint Rifle +5" — it's a specific tool that
enables a specific playstyle (stealth sniper at low levels). Finding it is
a discovery. Using it changes how you approach the Mojave.

**Fallout 4** (Bethesda, 2015) — the Legendary prefix system generates random
stat modifiers on random weapons. "Explosive Minigun" is memorable not because
Bethesda designed it, but because two random rolls accidentally composed into
something absurd. The system stumbles into fun by accident. Most of the time
it produces "Nocturnal Rolling Pin" — meaningless noise.

| Dimension | New Vegas | Fallout 4 |
|-----------|-----------|-----------|
| Item identity | Each unique weapon has a name, a story, a location | Random prefix + random base = noise |
| Decision space | "Do I use this weapon's strengths?" (bounded, interesting) | "Is this number higher than my current number?" (trivial) |
| Exploration reward | Finding a unique weapon is a discovery | Finding a Legendary is a dice roll |
| Flow model | Fitts + Hick: weapon choice affects engagement strategy | Integrase ratchet only: power number goes up |
| Engagement type | Serious Fun (weapon has personality) + Hard Fun (master its quirks) | Compulsion loop (check if number is bigger) |

### Exemplar: Noita (Nolla Games, 2019)

Noita is the proof that compositional mechanics in a bounded space create
emergent absurdity that no random stat system can match.

| Model | Observation | Why It Works |
|-------|------------|-------------|
| **Composition** | Wand = ordered spell sequence. Modifiers stack. "Homing + Drilling + Exploding + Bouncing" is not a random stat — it's a player-authored system. Each modifier is a mechanic, not a number | The player is a *system designer*, not a lottery player |
| **Bounded space** | Finite modifier pool (~400 spells). Finite wand slots. The space is large but bounded. The combinatorial explosion is the fun, not the numbers | Hick's law is respected at each assembly step: "which spell next?" |
| **Emergent absurdity** | Physics simulation + spell composition = every run discovers new interactions. "Turns out polymorphine + fire = chain reaction that destroys half the level." The game didn't design this. The systems composed | Emergence from rules, not from random numbers |
| **Flow** | Every wand assembly is a risk/reward decision. Overpowered wands can kill you. The challenge isn't "bigger number" — it's "can I control this system I built?" | Challenge scales with player ambition, not with stat inflation |
| **Four Keys** | Hard Fun (master spell interactions) + Easy Fun (spectacle of physics) + Serious Fun (discovery of hidden mechanics) | Three Fun Keys from composition alone |

### The Principle for ecoPrimals

**Items are certificates with mechanic tags, not stat rolls.**

In the ecoPrimals model:
- Every item is a loamSpine certificate with a set of mechanic tags
- Mechanic tags compose according to documented rules (RulesetCert)
- The composition space is bounded (finite tag vocabulary)
- Emergent interactions arise from tag combinations, not random numbers
- The DAG tracks which compositions have been discovered (rhizoCrypt)
- New discoveries ARE the progression — not bigger numbers

This directly maps to the Lysogeny model: the 7 validated mechanics
(Usurper, Integrase, Symbiont, Conjugant, Quorum, Pathogen, plus the
6 proposed targets) are *compositional building blocks*. A game item that
combines Symbiont + Resonant + Cascade creates a mutualistic weapon that
amplifies allies and chain-reacts through enemy groups. That's not a
random stat roll — it's a system contribution that the player chose,
assembled, and now must learn to control.

### What This Means for exp080

exp080 currently validates the Integrase ratchet (power number goes up)
and Pathogen risk-reward (higher risk = higher numbers). This is the
*foundation* — power-law distributions and diminishing returns are real.
But the *evolution* is from "number goes up" to "composition space expands":

```
Phase 1 (exp080 today): Power-law rarity × stat ranges → Integrase ratchet
    Validates: basic probability, diminishing returns, risk-reward

Phase 2 (future): Mechanic tag composition × bounded modifier pool → Noita model
    Validates: combinatorial explosion, emergent interactions, system mastery
    Needs: compositional item certificates (loamSpine), interaction rules (RulesetCert),
           discovery tracking (rhizoCrypt DAG of found compositions)

Phase 3 (target): Player-assembled systems that interact with world physics → full emergence
    Validates: Dwarf Fortress/Noita-level emergent gameplay from bounded rules
    Needs: physics integration (exp077 Spacewar math), composition validation
```

The Diablo power ramp is the floor, not the ceiling.

---

## How ecoPrimals Solves Each Genre

The primal stack isn't genre-specific. The same primitives compose differently
for different game types:

| Genre | Primary Primals | Key Experiments | Core Pattern |
|-------|----------------|-----------------|-------------|
| **CRPG** | Squirrel (AI narration), rhizoCrypt (story DAG), loamSpine (NPC certs) | exp067-075 | Personality certs + knowledge bounds + internal voices |
| **Roguelike** | rhizoCrypt (run DAG), loamSpine (meta-progression certs) | exp058 (Conjugant) | HGT = death transfers value. Price equation = information accumulation |
| **Investigation** | rhizoCrypt (evidence DAG), Squirrel (passive checks) | exp068, exp069 | GUMSHOE core clues as mandatory DAG vertices. Passive checks fire voices |
| **Strategy** | ludoSpring (DDA + Flow), barraCuda (Symbiont balance math) | exp057, exp040 | Lotka-Volterra interaction matrices as unit/faction balance |
| **Action/FPS** | ludoSpring (Flow + Fitts), toadStool (real-time compute) | exp005, exp012 | Sub-frame input, Fitts target acquisition, push-forward Flow |
| **Sandbox** | rhizoCrypt (world state DAG), ludoSpring (Quorum events) | exp059 | Agent-based emergence from threshold dynamics |
| **Card** | loamSpine (card certs), rhizoCrypt (stack DAG), BearDog (signed shuffles) | exp047-050, exp061 | Every card is a cert. Every game action is a DAG vertex. Signed randomness |
| **Extraction** | Provenance trio (rhizoCrypt + loamSpine + sweetGrass) | **exp053, exp054** | Every item cert, every bullet cert, every action vertex. Cheating = structural impossibility |
| **MOBA** | Songbird (matchmaking), ludoSpring (Symbiont balance), rhizoCrypt (match DAG) | exp057, exp059 | Hero interaction matrix. Quorum teamfights. Capability-based matchmaking |
| **Battle Royale** | Songbird (matchmaking), ludoSpring (Quorum ring), rhizoCrypt (match DAG) | exp059 | Ring as quorum forcing function. Spatial compression → phase transition |
| **MMO Shooter** | Provenance trio + Songbird (persistent world) | exp053, exp066 | Tarkov pattern scaled. Item economy as certificate graph. RMT = detectable transfer anomaly |
| **Horror** | Squirrel (internal voices as world), ludoSpring (Anderson W for sanity) | exp069, exp044 | Disorder parameter drives voice priority. World coherence degrades with knowledge |
| **JRPG** | Squirrel (plane transitions), ludoSpring (Symbiont social links) | exp057, exp075 | Calendar as bounding mechanism. Social links = Symbiont reputation graph |
| **Immersive Sim** | rhizoCrypt (multi-path DAG), ludoSpring (Hick spatial discovery) | exp046 | Multiple valid DAG paths through same space. Options discovered, not presented |
| **Compositional** (Noita) | loamSpine (mechanic tag certs), RulesetCert (composition rules), rhizoCrypt (discovery DAG) | exp080 + future | Items as mechanic certificates. Composition space bounded. Emergence from rules, not numbers |
| **Procedural World** (Minecraft) | barraCuda (noise+WFC+BSP+L-sys), rhizoCrypt (world state DAG) | exp081, exp009, exp014, exp017 | Noise fields → biome classification. WFC → adjacency constraints. BSP → spatial partitioning. L-systems → organic structures. Same pipeline generates patient populations (healthSpring) |

**The composable insight**: A game that combines CRPG + Extraction (imagine
a narrative extraction shooter) composes the relevant primals: provenance trio
for anti-cheat + personality certs for NPCs + story DAG for narrative. The
primals don't know what genre they're serving. They discover each other by
capability and compose.

---

## Implications for RPGPT

### The Target Profile

RPGPT should aim for the Disco Elysium / Esoteric Ebb intersection:

- **Internal voices** (Disco Elysium's defining innovation) — VALIDATED (exp069-070)
- **Real tabletop mechanics** (Esoteric Ebb's contribution) — SPEC'D (RulesetCert, exp045)
- **Investigation as core loop** (Obra Dinn's bounded deduction) — SPEC'D (Investigation plane)
- **NPC depth** (Planescape's secrets and lies) — VALIDATED (exp067-068, exp072)
- **Bounded branching** (authored scaffold + procedural flesh) — DESIGNED (DAG templates)
- **Multi-plane transitions** (BG3's seamless mode shifts) — VALIDATED (exp075)
- **Emergent collective events** (Dwarf Fortress spirals) — VALIDATED (exp059 Quorum)
- **Death as progress** (Hades loop) — VALIDATED (exp058 Conjugant)
- **Anti-exploitation** (no Pathogen patterns) — VALIDATED (exp060)

### The Bounding Solution

The problem Shadows of Doubt reveals: pure procedural generation produces
breadth without depth. The RPGPT answer:

```
Adventure Template (authored scaffold)
├── Act structure (Macbeth's 5 acts, or custom)
├── Key revelations (GUMSHOE core clues — always found)
├── Mandatory scenes (plot skeleton vertices in DAG)
├── NPC role slots (which personality cert fills which role)
└── Win/lose conditions (DAG terminus vertices)

Within each scene (procedural flesh):
├── Dialogue options generated by Squirrel within RulesetCert constraints
├── Skill checks resolved by tabletop mechanics
├── Internal voices fire on passive checks
├── NPC reactions driven by personality cert + memory DAG
└── Quorum events trigger when agent thresholds met
```

The scaffold bounds. The DAG enforces. The AI generates within walls.

"Macbeth in space" becomes:
- Act 1: The Prophecy (Investigation plane — receive the prophecy, evaluate its meaning)
- Act 2: The Murder (Dialogue plane — convince your partner, overcome doubt, voices scream)
- Act 3: The Crown (Political plane — manage factions, paranoia escalates)
- Act 4: The Unraveling (Investigation + Dialogue — evidence accumulates, NPCs suspect)
- Act 5: The Reckoning (Tactical + Dialogue — final confrontation, consequences)

The story is always Macbeth. The specific path through it is always yours.

---

## Connection to Other Springs

Every spring needs a "what good looks like" and "what bad looks like" catalog
in its domain. The pattern is universal:

| Spring | Good Profiles | Bad Profiles | Mining Source |
|--------|--------------|-------------|--------------|
| wetSpring | Reproducible pipelines (Snakemake, Nextflow), clean NCBI submissions, good FastQC reports | Broken Galaxy workflows, irreproducible analyses, vendor-locked pipelines, "works on my machine" | Reddit r/bioinformatics, r/CompChem, PubMed reproducibility studies, GitHub issues on bioinformatics tools |
| neuralSpring | PyTorch reference implementations with known accuracy, ONNX model zoo, papers with code | Models that don't converge, gradient vanishing/exploding, mode collapse, benchmark gaming | Papers With Code leaderboards, model evaluation suites, ML reproducibility challenges |
| hotSpring | Well-characterized GPU behavior (NVIDIA's published ISA docs, AMD's open register specs) | Undocumented firmware quirks, thermal throttling edge cases, driver bugs, clock domain crossing hazards | GPU hardware debugging forums, driver release notes, HPC mailing lists, NVIDIA developer forums |
| ludoSpring | **This document** — exemplar games with validated model mappings | Anti-pattern games with failure mode analysis | Published game design research (GDC talks, academic HCI), player community analysis, game postmortems |
| healthSpring | Well-validated clinical workflows, HIPAA-compliant data pipelines, reproducible EHR analyses | Data silos, vendor lock-in, interoperability failures, consent violations | Clinical informatics literature, EHR vendor comparison studies, HIPAA violation databases |
| primalSpring | Clean IPC contracts, capability-based discovery patterns, graceful degradation | Hardcoded primal names, brittle socket paths, missing capability advertisements, IPC race conditions | Cross-primal integration test failures, wateringHole handoff gap analysis |

Each spring's profile catalog is the domain knowledge that makes the spring
useful beyond validated math. The math proves the tools work. The profiles
show what the tools should aim at.

### The Co-Evolution Pattern

Primals and primalSpring handle composition. Each spring owns its domain science.
The profiles co-evolve:

1. ludoSpring validates Flow, DDA, Engagement models → profiles map them to real games
2. Profiles reveal gaps → "we can detect Flow but not narrative coherence" → new experiment
3. New experiments may need new barraCuda primitives → handoff to toadStool/barraCuda
4. Validated primitives become available to RPGPT → RPGPT gets closer to the profile targets
5. Playing RPGPT generates new data → profiles update with empirical results

The profiles are not static targets. They are a living evaluation framework
that sharpens as the system matures.

---

## Narrative Structure Templates (DAG Scaffolds)

The bounding problem — how to constrain procedural generation without killing
emergence — is solved by **authored scaffolds with procedural flesh**. Different
story structures produce different DAG topologies:

### Linear (e.g., Half-Life 2, Titanfall 2)

```
Act 1 → Act 2 → Act 3 → Act 4 → Act 5
```

DAG: chain of mandatory vertices. Zero branching. Maximum authorial control,
minimum player agency. Works for FPS campaigns where Flow comes from execution,
not choice. Not suitable for RPGPT.

### Branching Tree (e.g., Detroit: Become Human, The Witcher 3)

```
       ┌→ B1 → C1
A → B ─┤
       └→ B2 → C2 → D1
                    ↘ D2
```

DAG: tree with exclusive branches. Exponential authoring cost. Each branch
needs full content. Works for authored narrative RPGs with finite scope.
Suitable for bounded RPGPT adventures (Macbeth in Space).

### Hub and Spoke (e.g., Mass Effect, Baldur's Gate 3)

```
          ┌→ Quest A →┐
Hub ──────┤→ Quest B →├──→ Hub' ──→ ...
          └→ Quest C →┘
```

DAG: hub vertices with optional spoke subgraphs. Spokes can be
completed in any order. Hub transitions are mandatory. Moderate authoring
cost, high player agency within bounds. The sweet spot for RPGPT.

### Scaffold + Procedural (RPGPT Target)

```
Authored Act Structure (Macbeth):
  Act 1: Prophecy   ← mandatory hub
  Act 2: Murder     ← mandatory hub
  Act 3: Crown      ← mandatory hub
  Act 4: Unraveling ← mandatory hub
  Act 5: Reckoning  ← mandatory hub

Within each Act (procedural):
  - Which NPCs fill which roles (from personality cert pool)
  - How scenes unfold (Squirrel generates within ruleset constraints)
  - What side investigations emerge (Quorum events)
  - Which internal voices speak (passive check triggers)
  - How difficulty adapts (DDA within act parameters)
```

DAG: mandatory hub chain (authored) with procedural spoke generation
(Squirrel + ruleset certs). The story always arrives at the next hub.
The path between hubs is always different. This is how you get
replayable bounded adventures.

### Kishōtenketsu (Four-Act, no conflict)

```
Ki (Introduction) → Shō (Development) → Ten (Twist) → Ketsu (Reconciliation)
```

Japanese/Chinese narrative structure without Western conflict. The twist
(Ten) is not a conflict but a shift in perspective. Suitable for
Exploration-heavy RPGPT campaigns, investigation mysteries, and
introspective character studies. Maps well to the Inland Empire voice.

### Episodic (e.g., Life is Strange, Telltale games)

```
Episode 1 ──→ Episode 2 ──→ Episode 3
   ↕ save        ↕ save        ↕ save
(persistent state carries forward)
```

DAG: episodes as self-contained sub-DAGs with a shared persistent state
DAG underneath. Each episode has its own scaffold. State from previous
episodes constrains future scaffolds. Suitable for serialized RPGPT
campaigns (weekly tabletop sessions).

---

## Steam Catalog as Source Reference

A ~15-year Steam library with 2,000-3,000 games is a curated source catalog
of real games to model against. Not every game needs a full profile, but the
library gives us a concrete corpus: real titles, real genres, real developers,
real release dates. Instead of inventing hypothetical games to analyze, we
model against the actual landscape.

### Steam Web API as Source Index

Steam exposes a public JSON API (free API key from Valve):

```
GET /IPlayerService/GetOwnedGames/v1/
  → { game_count, games: [{ appid, playtime_forever, playtime_2weeks }] }

GET /api/appdetails?appids={id}
  → { name, type, genres, categories, metacritic, release_date, developers, publishers }
```

All JSON. All parseable with `serde_json`. Zero C deps. Pure Rust. This gives
us a machine-readable index of 2,000+ real games with genre tags, developer
attribution, and release metadata — a structured starting point for building
and expanding the profile catalog above.

### Privacy

- **Local only.** The analysis binary runs locally, outputs to `LUDOSPRING_OUTPUT_DIR`.
- Steam API key is yours. Results stay on your machine.
- rhizoCrypt DAG records analysis provenance if desired.

---

## Future Work

- **Player profiles**: Map Bartle types (Achiever, Explorer, Socializer, Killer) to Four Keys + Flow
- **Session arc profiles**: How good games structure a 2-hour session vs a 20-minute session
- **Accessibility profiles**: How exemplar games handle motor/cognitive/visual accessibility (exp015 connection)
- **Multiplayer dynamics**: How Songbird peer discovery changes quality profiles (co-op vs competitive vs asymmetric)
- **Steam source catalog experiment**: Implement the API integration to index real games for modeling
- **Developer-side profiles**: Invert the analysis — given a game concept, predict which quality models to optimize for
- **Additional genres**: Racing, Sports, Rhythm, City Builder, Colony Sim, Visual Novel, Auto-Battler
- **Anti-cheat as provenance**: Expand exp053 pattern to cover all multiplayer genres (MOBA scripting, BR wallhacks, MMORPG botting)

## Faculty Anchors

- Csikszentmihalyi, M. (1990) — Flow in games: Games as optimal experience generators
- Lazzaro, N. (2004) — Four Keys: Multi-dimensional fun measurement
- Hunicke, R. (2005) — DDA: System vs player-driven difficulty
- Yannakakis, G. & Togelius, J. (2018) — Computational game science: Engagement as measurable
- Hick, W.E. (1952) — Choice complexity: Bounded decisions in good design
- Fitts, P.M. (1954) — Target acquisition: UI as game mechanic
- Tufte, E.R. (1983) — Data-ink: Information density in game UI
- Laws, R. (2007) — GUMSHOE: Core clue principle for investigation design
- Bartle, R. (1996) — Player types: Motivation taxonomy

## License

AGPL-3.0-or-later

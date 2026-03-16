# RPGPT Deep System Design — The Planes Architecture

**Status**: Design specification + implementation (Phase 1 complete, petalTongue wired)
**Date**: March 15, 2026
**Supersedes**: `RPGPT_ARCHITECTURE_SKETCH.md` (preserved as historical record)
**Springs**: ludoSpring (game science), rhizoCrypt (ephemeral DAG), sweetGrass (attribution), loamSpine (permanence), biomeOS (orchestration), Squirrel (AI/MCP), toadStool (GPU), petalTongue (visualization), BearDog (signing), Songbird (discovery), NestGate (data), sunCloud (economics)
**Open Systems**: Pathfinder 2e (ORC), FATE Core (CC-BY), Cairn (CC-BY-SA), PbtA (CC-BY), Cypher (Open License)
**License**: AGPL-3.0-or-later

---

## The Core Insight

A game is not a single mode of play. Disco Elysium is pure dialogue and
skill checks. A tabletop session flows from exploration to combat to
roleplay to investigation. JRPGs transition between overworld, dialogue,
and battle. The magic is in the transitions — and in the continuity that
survives them.

RPGPT is not a game. It is a **game substrate** — a system where the
story DAG is continuous and universal, but the rules governing how the
DAG grows change as you move between planes of play. Your character,
your NPCs, your world state, your choices all persist. Only the
interaction architecture shifts.

---

## Part 1: The Planes — Game Modes as Swappable Rulesets

The rhizoCrypt DAG is the spine. It never stops. What changes is the
**active ruleset cert** (loamSpine) governing how new vertices are added.

### The Seven Planes

Each plane is a loamSpine `RulesetCert` — a machine-readable constraint
document governing how Squirrel adds vertices to the DAG.

| Plane | Interaction Model | Ruleset Source | ludoSpring Metric | AI Narration Style |
|-------|------------------|----------------|-------------------|-------------------|
| **Exploration** | Open world, free movement, ambient discovery | Cairn (CC-BY-SA) — minimal rules, inventory-as-HP | Engagement (action density, exploration ratio) | Atmospheric, descriptive, sensory |
| **Dialogue** | Skill-as-voice, persuasion, deception, empathy | Custom (Disco Elysium model, open math) | Flow (challenge/skill balance in conversation) | In-character voices, internal monologue |
| **Tactical** | Grid/zone combat, action economy, positioning | Pathfinder 2e (ORC) — 3-action, 4-degree success | DDA (dynamic difficulty, encounter CR) | Tense, blow-by-blow, consequences-focused |
| **Investigation** | Clue gathering, evidence DAG, deduction chains | Custom (GUMSHOE-inspired, open logic) | Hick's law (information overload detection) | Suggestive, atmospheric, Socratic |
| **Political** | Faction reputation, alliances, betrayal, negotiation | FATE Core (CC-BY) — Aspects as faction dynamics | Four Keys (People Fun — social interaction scoring) | Intrigue, subtext, strategic |
| **Crafting** | Material transformation, recipe discovery, alchemy | Custom (reaction kinetics — validated math) | Engagement (creative flow, discovery satisfaction) | Instructive, experimental, rewarding |
| **Card/Stack** | Zone management, stack resolution, resource economy | MTG rules (game actions as DAG, exp047/048 validated) | Game tree complexity (exp050 metrics) | Precise, mechanical, strategic |

### Plane Transitions Are DAG Vertices

When the party enters combat, a `PlaneTransition` vertex is written:

```
PlaneTransition {
    from: Dialogue,
    to: Tactical,
    trigger: "guard_alerted",
    world_state_snapshot: <hash>,
}
```

The AI narration style shifts. The available actions change. But the
world state — NPC dispositions, inventory, knowledge, consequences —
carries forward seamlessly.

A JRPG-style campaign might rotate: Exploration (overworld) ->
Dialogue (town) -> Tactical (dungeon) -> Investigation (mystery) ->
Political (court) -> back to Exploration. The DAG records the entire
journey. The story is never lost.

### The RulesetCert Structure

Building on the existing `Ruleset` trait in `barracuda/src/game/ruleset.rs`:

```rust
struct RulesetCert {
    plane: PlaneType,
    dice_system: DiceSystem,
    action_economy: ActionEconomy,
    resolution: ResolutionMethod,
    available_actions: Vec<ActionTemplate>,
    passive_checks: Vec<PassiveCheck>,
    constraints: Vec<RuleConstraint>,
    narration_style: NarrationGuide,
}
```

See `RPGPT_PLANES_SCHEMA.md` for the full schema of each plane.

---

## Part 2: NPCs That Are Not Chatbot Swill

The difference between a chatbot NPC and a *character* is **internal
state that the player cannot see but can sense**. A chatbot generates
plausible responses to prompts. A character has motivations, secrets,
contradictions, a past, and a future they are working toward — and all
of this exists whether or not the player asks about it.

### The NPC Personality Architecture

Every NPC is defined by a loamSpine `NpcPersonality` certificate
containing seven layers:

1. **Identity** — name, role, appearance, mannerisms
2. **Motivations** — Maslow hierarchy (numeric state, not flavor text)
3. **Knowledge Bounds** — what they know, suspect, lie about, and do not know
4. **Voice** — speech patterns, vocabulary level, emotional range
5. **Secrets** — hidden motivations, private knowledge
6. **Relationships** — Symbiont faction graph (who they like/hate, and why)
7. **Character Arc** — where they are going (goal state for each motivation)

See `RPGPT_NPC_PERSONALITY_SPEC.md` for the full certificate schema.

### Motivations as Math (Maslow + Lotka-Volterra)

Every NPC has a hierarchy of needs (public domain — Maslow 1943):

```
Survival > Safety > Belonging > Esteem > Self-Actualization
```

These are numeric state that drives behavior:

- A guard whose Survival need is threatened (player threatens their
  family) behaves differently from one whose Esteem need is active
  (player challenges their honor).
- NPC motivations interact via the Symbiont math (exp057 — multi-species
  Lotka-Volterra). The blacksmith's need for Safety (guild protection)
  conflicts with their need for Esteem (recognition for forbidden
  techniques).
- The Quorum mechanic (exp059) governs collective NPC events: when
  enough NPCs reach a threshold on the same need, emergent events
  trigger. The town revolts. The guild fractures. The festival happens
  spontaneously.

### Knowledge Bounds — What Makes NPCs Not Chatbots

NPCs know things the AI knows, but they also DON'T know things, and
they LIE about things. This is enforced by the loamSpine cert:

```yaml
knowledge_bounds:
  knows:
    - "The king is ill"
    - "The northern pass is blocked by snow"
  suspects:
    - "The advisor may be poisoning the king"
  lies_about:
    - topic: "The advisor"
      reason: "fears retaliation"
      tell: "slight hesitation, avoids eye contact"
  does_not_know:
    - "The dragon's weakness"
    - "What happened to the missing prince"
```

When Squirrel voices this NPC, the knowledge bounds are a hard
constraint. The AI cannot reveal what the NPC doesn't know. It must
actively obfuscate what the NPC lies about. And the *tells* — the
subtle behavioral cues — are where passive checks come in.

### NPC Memory as DAG (Not Context Window)

The critical failure of LLM NPCs: they forget. Context windows are
finite. Previous conversations evaporate.

RPGPT NPCs never forget because memory is structural:

- Every interaction is a rhizoCrypt vertex.
- When voicing an NPC, Squirrel receives a **curated context** built
  from the DAG: recent vertices + relationship vertices + promise
  vertices + secret-relevant vertices.
- This is **graph-aware retrieval**: walk the NPC's memory subgraph,
  extract relevant vertices, summarize older ones, present recent ones
  in full.
- NestGate caches NPC memory summaries for fast retrieval.

---

## Part 3: Internal Voices — The Disco Elysium Model

Your character has skills. But skills are not just numbers — they are
**perspectives**. Each skill is a constrained Squirrel call with its
own personality cert.

| Voice | Personality | What It Notices | When It Speaks |
|-------|-------------|-----------------|----------------|
| Logic | Cold, analytical | Contradictions, patterns | Evidence doesn't add up |
| Empathy | Warm, intuitive | Emotional states, subtext | NPCs hiding feelings |
| Rhetoric | Silver-tongued, political | Leverage, argument weaknesses | Opening to persuade |
| Perception | Hyper-aware, detail-obsessed | Physical details, tells | Something out of place |
| Endurance | Stoic, physical | Threats, fatigue, danger | Body is at risk |
| Authority | Commanding, bold | Power dynamics | Dominance could shift |
| Composure | Cool, controlled | Others reading you | Facade matters |
| Electrochemistry | Hedonistic, risk-seeking | Temptations, shortcuts | Dangerous easy option |
| Encyclopedia | Pedantic, academic | Historical parallels, lore | World context applies |
| Inland Empire | Dreamy, mystical | Hidden connections, hunches | Rational explanation fails |

Each voice runs as a separate Squirrel inference call with a distinct
system prompt derived from the voice's personality cert. **Passive
checks** trigger voices without the player asking — Logic interrupts
when someone contradicts themselves; Empathy whispers when an NPC is
afraid; Inland Empire mutters when something supernatural is near.

These are short, focused calls (50-100 tokens) to a local 7B model
via Ollama. At local inference speeds, passive checks fire in under
200ms.

See `RPGPT_INTERNAL_VOICES_SPEC.md` for the full voice system design.

---

## Part 4: The Primal Orchestra

Every ecoPrimal has a role in RPGPT:

| Primal | RPGPT Role | Specific Capability |
|--------|-----------|---------------------|
| **Squirrel** | DM brain | `ai.chat` with ruleset constraints, `ai.summarize` for NPC memory, multi-model routing for different voices |
| **ludoSpring** | Session quality gauge | Flow (pacing), DDA (encounters), engagement (boredom), Four Keys (fun type), Hick's law (choice paralysis) |
| **rhizoCrypt** | Living game state | Session DAG with branching, NPC memory subgraphs, condition tracking, phase transitions |
| **loamSpine** | Permanent records | Ruleset certs, character sheets, NPC personalities, world lore, item certs, achievements |
| **sweetGrass** | Creative attribution | Player world-building (Creation), AI narration (Implementation), NPC drift (Extension) |
| **BearDog** | Dice integrity + anti-cheat | Ed25519-signed rolls, no phantom items, provably fair randomness |
| **Songbird** | Multiplayer discovery | Co-op campaigns, remote tabletop, spectator mode |
| **biomeOS** | Orchestration | Deploy graph, 60Hz tick for real-time, Sequential for turn-based |
| **toadStool** | GPU compute | Local AI inference via Ollama, Perlin noise for terrain, batch NPC evaluation |
| **petalTongue** | Visualization + AR | Character sheet UI, map, AR card overlay, dialogue tree, combat grid |
| **NestGate** | Data provider | NPC memory cache, world knowledge base, public domain corpus, ruleset DB |
| **sunCloud** | Economics (long-term) | Radiating attribution for shared/published campaigns |

### The Core Loop

```
Player action
  -> biomeOS writes PlayerAction vertex to rhizoCrypt
  -> biomeOS fetches active RulesetCert + NPC certs from loamSpine
  -> biomeOS fetches NPC memory subgraph from rhizoCrypt
  -> biomeOS routes to Squirrel with (context, ruleset, npc_memory)
  -> Squirrel routes to toadStool -> Ollama for local inference
  -> Squirrel returns AINarration vertex
  -> biomeOS writes AINarration to rhizoCrypt
  -> biomeOS routes to ludoSpring for quality evaluation
  -> ludoSpring returns (engagement, flow, dda_adjustment)
  -> biomeOS renders via petalTongue with available actions
  -> Player sees narration + choices
```

Passive checks run asynchronously: Squirrel fires voice-specific
inference calls in parallel after every NPC interaction, gated by
skill thresholds.

### Model Selection Strategy

| Task | Model Size | Latency | Why |
|------|-----------|---------|-----|
| Main narration | 13B-70B | < 2s | Rich prose, world-consistent |
| NPC dialogue (major) | 13B+ | < 1.5s | In-character, personality-consistent |
| NPC dialogue (minor) | 7B | < 500ms | Quick, functional |
| Internal voice (passive) | 7B | < 200ms | Short, punchy observations |
| Dice interpretation | 7B | < 300ms | Rule-accurate, brief |
| DDA adjustment | ludoSpring (no LLM) | < 10ms | Pure math |
| Passive skill check | 7B | < 200ms | Quick yes/no + flavor |

All local via toadStool -> Ollama. Cloud is optional fallback.

---

## Part 5: What Worlds Could We Weave?

Given open rulesets + public domain literature + validated game science +
local AI + the full primal stack:

### World 1: The Alexandrian Library — Literary Multiverse

The Great Library was never burned. Its halls became doorways to the
worlds described in its books. Each wing runs on different rules.

- **The Library** (Exploration/Investigation, Cairn): Navigate infinite
  stacks. Knowledge IS your inventory.
- **Homeric Wing** (Tactical/Political, PF2e + FATE): Troy's plains and
  Olympus' politics. Achilles driven by Maslow hierarchy — Esteem wars
  with Belonging.
- **Arabian Nights Wing** (Dialogue/Investigation): Scheherazade's
  palace. Survival depends on storytelling. Stories within stories —
  a DAG within the DAG.
- **Wonderland Wing** (Exploration, inverted rules): Carroll's world.
  Failure is success. Inland Empire is your most reliable voice.

Connecting thread: a Library Card (loamSpine cert) records which worlds
you visited. Knowledge transfers obliquely across wings.

### World 2: The Sovereign Microbiome — Biology as Adventure

The cross-spring killer app. wetSpring data as ludoSpring gameplay.

- **Field Work** (Exploration): Perlin-generated terrain. Anderson
  localization determines signal propagation.
- **The Microscope** (Investigation): Microbial community as social
  network. Clue DAG tracks community structure.
- **Quorum Events** (Emergent Narrative): exp059 math drives emergent
  storytelling. Biofilm formation, virulence, cooperation — not scripted
  but simulated.
- **The Lab** (Crafting): Integrase mechanic (exp056) governs capture
  probability. Recipe discovery follows reaction kinetics.

### World 3: The Esoteric Investigation — Lovecraft Meets Disco Elysium

Coastal New England, 1920s. Public domain Lovecraft (pre-1929).

- **Town Investigation** (Dialogue/Investigation): NPCs with knowledge
  bounds. Internal voices are critical. Logic notices contradictions.
  Empathy reads fear. Inland Empire senses wrongness.
- **The Archives** (Investigation): Evidence DAG. Encyclopedia voice
  cross-references. Detective's notebook as first-class mechanic.
- **The Rituals** (Crafting, unreliable): Occult preparation with high
  variance. The crafting system is intentionally alien.
- **The Encounter** (Tactical, but broken): Rules fail you. The horror
  IS that the mechanics don't protect you. DDA increases dread.

**Sanity as Anderson W**: Knowledge IS disorder. As you learn more,
W increases. Inland Empire speaks more than Logic. The game mechanically
represents the Lovecraftian insight: understanding the truth costs you
your ability to function in ordinary reality.

### World 4: The Living Card Table — AR Magic: The Gathering

Physical cards, real table, AR overlay via petalTongue.

- Stack visualization (exp048 — LIFO as protein folding)
- 1:1 card mirror (loamSpine cert per physical card, exp047)
- Remote Commander pod play via Songbird + biomeOS
- AI judge: Squirrel reads game state DAG + rules cert
- Deck provenance: each card is a Novel Ferment Transcript

### World 5: The Rotating Campaign — Planes as Genre Demonstration

Reality is fragmenting. Each act is a different game genre. The
narrative thread is that the world keeps trying to be coherent.

- Act 1 — The Village (Exploration + Dialogue, Cairn)
- Act 2 — The War (Tactical, PF2e)
- Act 3 — The Trial (Political, FATE)
- Act 4 — The Mystery (Investigation, custom)
- Act 5 — The Fracture (all planes simultaneously)

The throughline: character relationships, NPC memories, item provenance,
and world consequences persist across every genre shift. The rotating
campaign proves that story transcends mechanics.

---

## Part 6: What Makes This Not Chatbot Swill

1. **Structural memory, not context window** — NPC memory is a DAG
   subgraph. NPCs remember your first conversation 50 sessions later.
2. **Hard constraints, not suggestions** — The ruleset cert is
   machine-readable. The AI cannot hallucinate a 4th action in a
   3-action system.
3. **NPCs have hidden state** — Knowledge bounds mean NPCs genuinely
   don't know things, suspect things, and lie — with mechanical tells.
4. **Quality is measured, not assumed** — Flow, engagement, DDA, Four
   Keys run continuously. Adjustments are grounded in published HCI
   research (Csikszentmihalyi 1990, Hunicke 2005).
5. **Every action has provenance** — You can rewind the DAG, branch,
   ask "what if?" This is a first-class DAG operation.
6. **Creative attribution is real** — sweetGrass tracks derivation.
   Your creative contribution is provably yours.
7. **The Lysogeny toolkit** — All 6 validated mechanics (Usurper,
   Integrase, Symbiont, Conjugant, Quorum, Pathogen) are available
   as NPC behavior systems. Anti-exploitation (Pathogen) is built in.

---

## Part 7: Implementation Phases

### Phase 1 — The Dialogue Engine (Disco Elysium Model) ✓ IMPLEMENTED

- NPC personality certs with knowledge bounds (loamSpine) ✓
- Internal voices system (multiple constrained Squirrel calls) ✓
- Passive skill checks as ambient AI observations ✓
- D6 dice pool dialogue resolution with flow monitoring ✓
- Trust dynamics gating information access across sessions ✓
- NPC memory as DAG subgraph with curated context assembly ✓
- Plane transition continuity (Dialogue ↔ Tactical) ✓
- petalTongue scene bindings for game UI rendering ✓
- 321 validation checks across 9 experiments (exp067–075) ✓
- 256 barracuda unit tests ✓
- biomeOS deploy graph + BYOB niche wired with petalTongue ✓

### Phase 2 — The Session DAG (Turn-Based Play)

- Plane-aware vertex types in rhizoCrypt
- PlaneTransition vertices swapping active ruleset certs
- NPC memory subgraph queries for context assembly
- Branching ("what if?") as first-class operations

### Phase 3 — The Planes (Multi-Modal Play)

- 3 planes: Exploration (Cairn), Dialogue (custom), Tactical (PF2e)
- Smooth transitions with narrative continuity
- Per-plane quality metrics with cross-plane engagement tracking
- DDA that adapts across plane boundaries

### Phase 4 — The Worlds (Content)

- The Alexandrian Library (literary multiverse)
- The Lovecraftian Investigation (sanity as Anderson W)
- AR Magic: The Gathering (physical + digital)

### Phase 5 — The Orchestra (Full Primal Integration)

- biomeOS deploy graph for RPGPT niche
- Songbird multiplayer discovery
- NestGate as world knowledge base
- sunCloud economics for shared campaigns

---

## Part 8: petalTongue as Universal Game UI

petalTongue is the universal user interface primal. For RPGPT, it provides
all rendering across every plane — dialogue trees, character sheets, combat
grids, exploration maps, NPC panels, dice results, and internal voices.

### Scene Binding Architecture

ludoSpring composes game state into scene binding types
(`barracuda/src/game/rpgpt/scene.rs`) that petalTongue renders via its
Grammar of Graphics engine. No compile-time dependency — wire format only.

| Scene Type | petalTongue Mechanism | Game Channel |
|---|---|---|
| `DialogueTreeScene` | Scene graph (tree layout) + selectable nodes | `DialogueTree` |
| `CharacterSheetScene` | Multi-panel scene graph + Gauge sub-bindings | `CharacterSheet` |
| `CombatGridScene` | FieldMap + interactive entity markers | `CombatGrid` |
| `VoiceNote` | Stacked cards in scene graph (max 3 visible) | `VoiceDisplay` |
| `NpcStatusScene` | Scene graph + Gauge + Text sub-bindings | `NpcStatus` |
| `DiceResultScene` | Animated Bar/Arc + BearDog signature badge | `DiceResult` |
| `ExplorationMapScene` | FieldMap + fog overlay + Point markers | `ExplorationMap` |
| `NarrationEntry` | Streaming text panel via `visualization.render.stream` | `NarrationStream` |

### Multi-Modal Rendering

Every scene binding renders across all petalTongue modalities:

- **egui GUI** — full graphical interface with mouse/keyboard
- **ratatui TUI** — terminal interface for SSH play, accessible
- **Audio sonification** — dice rolls, voice cues, ambient
- **SVG export** — session replay, printed character sheets
- **Headless** — for AI-only sessions and testing

### Interaction Flow

```
Player interacts with petalTongue UI
  → petalTongue emits interaction event (select/focus/command)
  → biomeOS routes to ludoSpring
  → ludoSpring evaluates (skill check, DDA, flow)
  → ludoSpring routes to Squirrel for AI narration
  → Squirrel returns narration + voice outputs
  → ludoSpring composes scene binding
  → petalTongue renders updated UI
```

### Analytics Overlay

The existing 7 analytics channels (EngagementCurve, DifficultyProfile,
FlowTimeline, UiAnalysis, InteractionCostMap, GenerationPreview,
AccessibilityReport) continue to function as dashboard overlays.
DMs can toggle analytics visibility during play for real-time
session quality monitoring.

---

## Connection to Existing Work

| Existing | RPGPT Role |
|----------|-----------|
| `Ruleset` trait (`game/ruleset.rs`) | Foundation for `RulesetCert` — already supports D20, Fudge, RollUnder, D6Pool, D100 |
| exp045 (Ruleset Control Systems) | Validates PF2e, FATE, Cairn as ingestible rulesets |
| exp046 (Text Adventure DAG) | Validates DAG-based session state — direct ancestor of Session DAG |
| exp047 (MTG Card Provenance) | Validates card-as-certificate — foundation for AR MTG |
| exp048 (Stack Resolution Folding) | Validates stack visualization — foundation for AR stack overlay |
| exp052 (Provenance Trio) | Validates cross-primal round-trip — foundation for NPC memory |
| exp053 (Extraction Shooter) | Validates anti-fraud DAG — same pattern as anti-cheat dice |
| exp055-060 (Lysogeny) | 6 NPC behavior systems — Usurper, Integrase, Symbiont, Conjugant, Quorum, Pathogen |
| exp061 (Fermenting) | Validates Novel Ferment Transcript — deck/card provenance |
| `niche.rs` | Self-knowledge pattern — RPGPT niche will follow same architecture |
| `NeuralBridge` | Typed IPC client — how RPGPT talks to biomeOS Neural API |
| Squirrel v0.1.0 | AI coordination — multi-provider routing, Ollama support, MCP |
| petalTongue v1.6.3 | Universal game UI — Grammar of Graphics, scene graph, multi-modal, 60 Hz interaction |
| `VisualizationPushClient` | JSON-RPC client for any viz-capable primal — game scene bindings |
| `GameChannelType` (15 variants) | 7 analytics + 8 RPGPT game UI channels for petalTongue routing |
| `rpgpt::scene` module | Scene binding types: `DialogueTreeScene`, `CharacterSheetScene`, `CombatGridScene`, etc. |
| `game::engine::gpu` module | GPU dispatch types, shader catalog (`GpuOp`), `GpuAvailability` runtime discovery |
| barraCuda shaders (806) | WGSL compute shaders — Perlin 2D, conv2d, detection, physics, math |
| coralReef compiler | WGSL → native GPU binary (NVIDIA SM70-89, AMD RDNA2) — pure Rust, no C deps |
| toadStool dispatch | `compute.submit`, `science.gpu.dispatch`, hardware discovery, job queue |
| `fog_of_war.wgsl` | Per-tile visibility computation — GPU-parallel distance check |
| `tile_lighting.wgsl` | Point light propagation with 1/d² falloff — up to 8 lights |
| `pathfind_wavefront.wgsl` | BFS wavefront expansion — one ring per dispatch, GPU-parallel |
| Node Atomic pattern | BearDog + Songbird + toadStool — security + discovery + compute foundation |
| `rpgpt_compute_engine.toml` | Full deploy graph composing gaming niche + node atomic + coralReef |

---

## Faculty Anchors

- Maslow, A.H. (1943) — hierarchy of needs (NPC motivation model)
- Csikszentmihalyi, M. (1990) — Flow theory (session quality)
- Hunicke, R. (2005) — DDA (encounter scaling)
- Lazzaro, N. (2004) — Four Keys to Fun (fun type classification)
- Fitts, P.M. (1954) — target acquisition (UI design)
- Hick, W.E. (1952) — choice complexity (information overload)
- Schelling, T.C. (1971) — agent-based modeling (emergent narrative)
- Pearl, J. (2000) — DAG-based causality (event chains)
- Nealson & Hastings (1979) — quorum sensing (collective events)
- Fisher (1930), Maynard Smith (1982), Nowak & May (1992) — evolutionary game theory (NPC dynamics)
- Lotka (1925, 1932), Gause (1934) — population dynamics (faction systems)
- Anderson, P.W. (1958) — localization (sanity mechanics)
- Skinner (1938), Kahneman & Tversky (1979) — anti-exploitation (Pathogen defense)

## License

All game mechanics derived from ORC-licensed material used under ORC License.
All PbtA/FATE mechanics used under CC-BY. Cairn under CC-BY-SA.
ecoPrimals code: AGPL-3.0-or-later.

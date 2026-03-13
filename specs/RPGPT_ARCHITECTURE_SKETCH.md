# RPGPT Architecture Sketch — Tabletop RPG Meets Sovereign Data Layer

**Status**: Sketch — structural exploration, not implementation spec
**Date**: March 11, 2026
**Springs**: ludoSpring (game science), rhizoCrypt (ephemeral DAG), sweetGrass (attribution), loamSpine (permanence), biomeOS (orchestration), Squirrel (AI/MCP)
**Open Systems**: Pathfinder 2e (ORC License), other ORC-licensed rulesets

---

## The Isomorphism

The same provenance pattern applies across domains. The primitives are identical — only the vocabulary changes:

| Primitive | Tabletop RPG | Extraction Shooter | Field Genomics | Lab Science |
|-----------|-------------|-------------------|----------------|-------------|
| **Session** | Adventure session | Raid match | Field sampling trip | Experiment run |
| **Event** | Action/roll | Shot/loot/extract | Sample collected | Observation recorded |
| **Object lineage** | Item found → traded → enchanted | Gun looted → modded → extracted | Swab taken → cultured → sequenced | Reagent mixed → reacted → measured |
| **Anti-fraud** | No phantom items | No duped guns | No phantom samples | No fabricated data |
| **Attribution** | Who did the quest | Who made the kill | Who collected the sample | Who ran the experiment |
| **Permanence** | Character sheet, campaign log | Player stash, hideout | Freezer inventory, BioProject | Published dataset, paper |

rhizoCrypt's `SessionType::Gaming { game_id }` is structurally identical to a hypothetical `SessionType::FieldWork { project_id }`. The `ItemLoot` vertex in a Tarkov raid is the same DAG operation as an `Observation` vertex in a wetSpring experiment. **Anti-cheat is chain-of-custody. Chain-of-custody is anti-cheat.**

This is not metaphor — it's the same code path through the same primal.

---

## Open RPG Systems as Structural Foundation

### Why Pathfinder 2e (ORC License)

The ORC (Open RPG Creative) License provides:

- **Irrevocable**: Cannot be revoked once material is released (unlike OGL 1.0a)
- **System-agnostic**: Mechanics are free; lore/trademarks are reserved
- **Licensed Material includes**: Statblocks, rules, attributes, combat mechanics, skills, spells, equipment, dice systems, character creation, success/failure methods

What we can freely use as structural foundation:

| PF2e Mechanic | What It Gives Us | ecoPrimals Mapping |
|--------------|------------------|-------------------|
| **Ability scores** (STR, DEX, CON, INT, WIS, CHA) | Character state space | rhizoCrypt vertex payload |
| **Proficiency tiers** (Untrained → Legendary) | Skill progression model | loamSpine certificate evolution |
| **Three-action economy** | Turn structure (constrained choices per round) | Session DAG branching factor |
| **Degrees of success** (Crit Fail → Crit Success) | Outcome resolution with 4 tiers | Hick's law (ludoSpring) maps to choice count |
| **Conditions** (Frightened, Wounded, Drained...) | State machine with duration tracking | rhizoCrypt temporal vertices |
| **Encounter mode / Exploration mode / Downtime mode** | Session phase model | Maps to ludoSpring `SessionPhase` |
| **Ancestry + Class + Background** | Character identity triple | loamSpine certificate template |
| **Feats** (Ancestry, Class, Skill, General) | Capability tree | sweetGrass entity derivation chain |
| **Item levels + runes** | Equipment progression with modular bonuses | loamSpine `GameItem` with attributes |
| **Hero Points** | Narrative agency currency | ludoSpring engagement → hero point correlation |

### Other Open Systems Worth Drawing From

| System | License | What It Adds |
|--------|---------|-------------|
| **Pathfinder 2e** | ORC | Full d20 ruleset, 4-degree success, 3-action economy |
| **Cypher System** (Monte Cook SRD) | Cypher System Open License | GM intrusions, cyphers as one-use items, difficulty = single target number |
| **FATE Core** | CC-BY | Aspects, Fate Points, fiction-first mechanics, zones instead of grid |
| **Powered by the Apocalypse** (PbtA) | CC-BY (varies) | Moves, partial success, GM principles, fiction triggers |
| **Cairn / Into the Odd** | CC-BY-SA | Minimal rules, inventory slots, direct damage, no to-hit rolls |
| **Year Zero Engine** (Free League OGL) | OGL variant | Stress dice, push mechanic, hex exploration |

FATE's **Aspects** are particularly interesting for RPGPT — they're freeform narrative tags ("Haunted by the Ghost of My Mentor") that mechanically affect rolls. This maps cleanly to sweetGrass semantic entities.

---

## Primal Roles in RPGPT

### rhizoCrypt — The Game State Engine

rhizoCrypt is the DAG that holds the living game world:

```
Session: "campaign-northwatch-s4"
├── v0: SessionStart { phase: Exploration }
├── v1: PlayerAction { agent: "did:key:alice", action: "search the ruins" }
├── v2: GMNarration { agent: "did:key:squirrel-ai", text: "You find a locked door..." }
├── v3: PlayerChoice { agent: "did:key:alice", choice: "pick the lock" }
│   ├── v4a: SkillCheck { skill: "Thievery", dc: 20, roll: 18, degree: "failure" }
│   │   └── v5a: Consequence { condition: "Alerted guards", duration: "encounter" }
│   └── v4b: [BRANCH: "what if I chose to break it down?"]
│       └── v5b: SkillCheck { skill: "Athletics", dc: 15, roll: 22, degree: "success" }
├── v6: CombatStart { initiative: [...] }
├── v7: Attack { attacker: "alice", target: "guard_1", weapon: "shortsword", roll: 17, damage: 8 }
└── ...
```

Key properties:
- **Branching is native** — exploring "what if" is a DAG operation, not a save file
- **Every roll is a vertex** — provable, auditable, immutable once written
- **NPC state accumulates** — the guard's health, disposition, memory are DAG vertices
- **Conditions have temporal scope** — Frightened 2 decays across vertices, tracked structurally

**Evolution goal for rhizoCrypt**: The gaming event types already exist (`GameEvent`, `Combat`, `ItemLoot`, `Extraction`). What's needed is a **turn-based session mode** that understands:
- Action economy (3 actions per turn in PF2e)
- Initiative ordering (sequential vertex constraints)
- Condition tracking (temporal vertex metadata)
- Encounter/Exploration/Downtime phase transitions

### loamSpine — The Permanent Record

loamSpine anchors things that should survive beyond a session:

| What | Certificate Type | Permanence Reason |
|------|-----------------|-------------------|
| **Character sheet** | Custom: `CharacterSheet { ancestry, class, level, abilities }` | Survives across sessions |
| **Item ownership** | `GameItem { item_type, attributes }` | Tradeable, lendable, provable |
| **World lore** | Custom: `WorldEntry { topic, canon_level, author }` | Campaign bible — canonical facts |
| **Ruleset** | Custom: `Ruleset { system, version, house_rules }` | Immutable reference for AI constraint |
| **Achievement** | Custom: `Achievement { quest, participants, date }` | Provable accomplishment |
| **NPC personality** | Custom: `NpcTemplate { name, traits, motivations, voice }` | Persistent across sessions |

The **ruleset as loamSpine certificate** is critical: the AI must reference an immutable, anchored ruleset. If the rules say "max 3 actions per turn" or "fire damage is halved by Resist Fire 5", the AI can't hallucinate around it because the constraint is provably anchored.

**Evolution goal for loamSpine**: The `DigitalGameKey` and `GameItem` types exist. What's needed:
- `CharacterSheet` certificate type (PF2e-compatible ability scores, class features, feat tree)
- `Ruleset` certificate type (machine-readable rules, queryable by AI)
- `NpcTemplate` certificate type (personality, knowledge bounds, voice style)
- `WorldEntry` certificate type (canon lore with canonicity level)

### sweetGrass — Who Created What

sweetGrass tracks the creative contributions in the living world:

| Contribution | Agent | Attribution Type |
|-------------|-------|-----------------|
| World setting | Player (DM phase) | Creation (1.0) |
| Quest hook | Player | Design (0.9) |
| NPC dialogue | AI (Squirrel) | Implementation (0.8) |
| Plot twist | Player response | Extension (0.5) |
| Rule interpretation | AI | Maintenance (0.3) |
| Map generation | Perlin noise (ludoSpring) | Tool (0.1) |

The player who designs the quest gets higher attribution than the AI that narrates it. If the AI generates a compelling NPC from the player's setup, sweetGrass tracks the derivation chain: Player designed the NPC template → AI implemented dialogue → Player's choices evolved the NPC's personality.

**Evolution goal for sweetGrass**: The Braid model already supports this. What's needed:
- `ActivityType::Custom { type_uri: "rpg:world_building" }` niche pattern
- NPC personality as a semantic entity with derivation tracking
- Session-scoped attribution rollup (who contributed most to this session?)

### Squirrel (AI/MCP) — The Storytelling Engine

Squirrel coordinates AI models for narration:

| AI Task | Input | Output | Constraint |
|---------|-------|--------|-----------|
| **Narrate scene** | DAG parent chain + world lore + NPC templates | Prose description | Must respect ruleset (loamSpine) |
| **NPC dialogue** | NPC template + conversation history (DAG) + player action | In-character speech | Must stay in-voice (personality cert) |
| **Roll interpretation** | Skill check result + degrees of success | Narrative consequence | Must apply correct PF2e rules |
| **World reaction** | Player actions + world state | Environmental changes | Must be internally consistent |
| **Branch suggestion** | Current DAG state + player patterns | "What if?" prompts | Must be mechanically valid |

The AI reads the rhizoCrypt DAG for context, references the loamSpine-anchored ruleset for constraints, and writes new vertices back to rhizoCrypt. sweetGrass attributes the AI's output proportionally to the player's creative input.

### ludoSpring — The Quality Gauge

ludoSpring's validated metrics evaluate whether the session is actually fun:

| Metric | RPG Application | Signal |
|--------|----------------|--------|
| **Flow** (Csikszentmihalyi) | Is the challenge/skill balance right? | AI should adjust encounter difficulty |
| **Engagement** (Yannakakis) | Is the player invested? | Low engagement → AI introduces complication |
| **DDA** (Hunicke) | Should difficulty scale? | Suggest adjustment to encounter CR |
| **Four Keys** (Lazzaro) | What type of fun? | Hard Fun (combat focus) vs Easy Fun (exploration focus) |
| **Hick's law** | Too many choices per turn? | Simplify options if decision paralysis |
| **Fitts's law** | UI target acquisition (if graphical) | HUD layout for character sheet |

**Evolution goal for ludoSpring**: Add `InteractionArchitecture::TabletopRPG` to the genre taxonomy:
- Turn-based but with narrative freedom between turns
- Scientific analogue: "hypothesis generation / experimental design"
- The player's quest design phase = experimental protocol
- The AI's narration = running the experiment
- The outcome = data to analyze

---

## The Anti-Cheat = Chain-of-Custody Isomorphism (Detailed)

### In an Extraction Shooter (Tarkov-style)

```
Session: "raid-customs-42"
├── Spawn { agent: player1, loadout: [pistol, medkit] }
├── ItemLoot { agent: player1, item: "ak-47-uuid", location: (100, 50) }
│   ← This vertex is the PROVENANCE of the AK-47
├── Combat { attacker: player1, target: player2, weapon: "ak-47-uuid", outcome: Kill }
├── ItemLoot { agent: player1, item: "player2-armor-uuid", source: "player2-corpse" }
│   ← Derived from Combat vertex — legitimate acquisition chain
├── Extraction { agent: player1, inventory: [pistol, medkit, ak-47, armor] }
│   ← Only items with valid DAG ancestry can extract
```

**Anti-cheat**: If `ak-47-uuid` appears in player1's extraction inventory without a preceding `ItemLoot` or `ItemTransfer` vertex, it's provably illegitimate. The DAG is BLAKE3 content-addressed — you can't insert a fake `ItemLoot` vertex without breaking the Merkle tree.

### In Field Genomics (wetSpring-style)

```
Session: "field-trip-brandt-farm-2026-03"
├── SampleCollect { agent: researcher1, sample: "soil-core-7", location: GPS(42.7, -84.5), depth: 15cm }
│   ← This vertex is the PROVENANCE of the soil core
├── Transport { agent: researcher1, sample: "soil-core-7", from: field, to: lab, chain: cold }
├── Processing { agent: lab_tech, sample: "soil-core-7", method: "16S amplification", output: "amplicon-7" }
│   ← Derived from Transport vertex — legitimate processing chain
├── Sequencing { agent: sequencer, sample: "amplicon-7", platform: "MinION", output: "reads-7.fastq" }
│   ← Only samples with valid DAG ancestry produce valid sequences
```

**Chain-of-custody**: If `reads-7.fastq` appears in a BioProject submission without a preceding `SampleCollect` → `Transport` → `Processing` chain, the data provenance is broken. Same BLAKE3 Merkle integrity. Same DAG structure. Same primal (rhizoCrypt).

### In a Tabletop RPG

```
Session: "campaign-northwatch-s4"
├── CharacterAction { agent: alice, action: "search the chest", check: Perception }
├── ItemLoot { agent: alice, item: "flaming-sword-uuid", source: "ancient-chest", roll: nat20 }
│   ← This vertex is the PROVENANCE of the sword
├── ItemEnchant { agent: wizard_npc, item: "flaming-sword-uuid", rune: "+1 striking" }
│   ← Derived from ItemLoot — legitimate enchantment chain
├── ItemTransfer { item: "flaming-sword-uuid", from: alice, to: bob, method: "gift" }
│   ← Bob's ownership has full provenance back to the chest
```

**Same code. Same primal. Same DAG. Different vocabulary.**

---

## Provenance Trio Evolution Goals

### rhizoCrypt Evolution (from RPGPT work)

| Goal | Current State | What RPGPT Teaches |
|------|--------------|-------------------|
| Turn-based session mode | `SessionType::Gaming` exists | Add turn/round/encounter structure with action economy |
| Condition tracking | Generic vertex metadata | Temporal condition decay (Frightened 2 → 1 → 0 across turns) |
| Branch visualization | DAG exists, no viz | "What if?" exploration needs branch diff and merge |
| NPC state accumulation | Agent DIDs exist | NPC-scoped vertex queries across sessions |
| Phase transitions | Generic session lifecycle | Encounter → Exploration → Downtime state machine |

These same capabilities improve field genomics: turn-based sessions → multi-day field campaigns; condition tracking → sample degradation over time; branch visualization → protocol variant comparison; phase transitions → collect → transport → process → analyze lifecycle.

### loamSpine Evolution (from RPGPT work)

| Goal | Current State | What RPGPT Teaches |
|------|--------------|-------------------|
| Machine-readable ruleset certs | Generic certificates | Structured constraint format that AI can query |
| Character sheet certs | `DigitalGameKey` exists | PF2e-compatible attribute/feat/spell structure |
| NPC personality certs | No specific type | Template with voice, knowledge bounds, motivations |
| World lore certs | No specific type | Canonical facts with canonicity level and authorship |
| Lending for character sheets | `Loan` slice mode exists | "Play my character for a session" |

These same capabilities improve lab science: machine-readable ruleset → experimental protocol certs; character sheet → instrument calibration certs; NPC personality → reagent property certs; world lore → material safety data sheets; lending → shared equipment checkout.

### sweetGrass Evolution (from RPGPT work)

| Goal | Current State | What RPGPT Teaches |
|------|--------------|-------------------|
| Multi-agent creative attribution | Fair attribution exists | Player + AI + NPC derivation chains |
| Narrative entity extraction | Code entity extraction exists | Quest, NPC, Location as semantic entities (not just Modules/Functions) |
| Session contribution rollup | Per-braid attribution | "Who contributed most to tonight's session?" |
| Personality evolution tracking | Derivation chains exist | NPC personality drift over time, attributed to player interactions |

These same capabilities improve collaborative science: multi-agent attribution → multi-lab collaboration; narrative entities → hypothesis/finding entities; session rollup → experiment contribution summary; personality evolution → model parameter drift tracking.

---

## What We Build First

### Phase 0: Structural Sketch (this document)

Map the open RPG ruleset to ecoPrimals primitives. No code yet.

### Phase 1: Ruleset as Data (loamSpine certificate format)

Define a machine-readable subset of PF2e mechanics as a loamSpine certificate:
- Ability scores, proficiency tiers, action economy
- Skill list with DCs and degree-of-success rules
- Basic combat: attack rolls, damage, AC, HP, conditions
- This becomes the constraint document the AI must respect

### Phase 2: Session DAG (rhizoCrypt gaming mode)

Build on the existing `SessionType::Gaming` to add:
- Turn structure (round → turn → 3 actions)
- Dice roll vertices with PF2e degree-of-success evaluation
- Condition application and decay
- Phase transitions (encounter ↔ exploration ↔ downtime)

### Phase 3: AI Narration Loop (Squirrel + ludoSpring)

- Squirrel reads DAG context + ruleset cert + NPC templates
- Generates narration constrained by rules
- ludoSpring measures engagement/flow/fun
- DDA adjusts encounter difficulty

### Phase 4: Attribution + Economics (sweetGrass + sunCloud)

- Player world-building gets Creation attribution
- AI narration gets Implementation attribution
- NPC personality evolution tracked as Derivation
- If the world becomes shareable, sunCloud radiates value

---

## Connection to Existing Papers

| Paper | RPGPT Connection |
|-------|-----------------|
| 01 (Anderson-QS) | Microbial community exploration = dungeon exploration (same interaction architecture) |
| 12 (Immuno-Anderson) | Immune response as encounter: cytokines = NPCs, tissue = terrain, drugs = items |
| 13 (Sovereign Health) | Patient engagement metrics = player engagement metrics (same Flow theory) |
| 16 (Anaerobic-Aerobic QS) | Biome phase transition = campaign phase transition (same state machine) |
| 17 (Game Design as Science) | All 13 HCI models apply directly to RPGPT UI and session design |

---

## License Note

All game mechanics derived from ORC-licensed material (Pathfinder 2e) are used under the ORC License. The ORC License is irrevocable and system-agnostic. Reserved Material (Pathfinder trademarks, Golarion setting, named characters) is NOT used. Only open mechanical structures (ability scores, proficiency, action economy, degree of success, conditions) are referenced.

ecoPrimals code remains AGPL-3.0-or-later.

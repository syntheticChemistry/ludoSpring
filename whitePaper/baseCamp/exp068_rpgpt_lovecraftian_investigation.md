# Expedition 068: RPGPT — The Esoteric Investigation (Lovecraft Meets Disco Elysium)

**Date:** 2026-03-15
**Status:** Design
**Reference:** H.P. Lovecraft (public domain, pre-1929), Anderson localization (1958), Csikszentmihalyi (1990), exp044 (Anderson QS)
**Depends on:** exp045 (Ruleset Control Systems), exp046 (Text Adventure DAG), exp055-060 (Lysogeny)
**Architecture:** `specs/RPGPT_DEEP_SYSTEM_DESIGN.md`, `specs/RPGPT_INTERNAL_VOICES_SPEC.md`, `specs/RPGPT_NPC_PERSONALITY_SPEC.md`

---

## Premise

A coastal New England town. 1920s. You are not a hero — you are a
person trying to understand something that resists understanding.
Your sanity is your disorder parameter.

This world is the deepest test of the Dialogue plane and internal
voices system. Everything depends on talking to people, reading
their lies, following clues, and managing the cost of knowledge.

---

## The Sanity Mechanic as Anderson Localization

From exp044 (Anderson QS Explorer), we validated that Anderson's
localization model (W = disorder parameter) governs signal
propagation in disordered media. In this world:

- **W = 0**: Normal. The world makes sense. Logic is dominant.
  Conversations follow expected patterns. NPCs are reliable.
- **W = 3**: Uneasy. Inland Empire starts speaking. Perception
  notices things that might not be there. Encyclopedia references
  texts that are increasingly obscure.
- **W = 6**: Disturbed. Logic becomes unreliable (its observations
  contain errors). Empathy over-reads — sees fear everywhere.
  Electrochemistry pushes toward escape via substance.
- **W = 9**: Fractured. Inland Empire is the loudest voice. The
  narration itself becomes unreliable. NPCs may not be real.
  The Investigation plane's clue DAG develops branches that
  contradict each other.
- **W = 12**: Shattered. The game mechanically represents the
  Lovecraftian insight: understanding the truth costs you your
  ability to function in ordinary reality. The Exploration plane
  becomes unreliable (locations shift). Only Composure keeps
  the player functional.

**Key insight**: Knowledge IS disorder. Every clue found, every
truth uncovered, every elder sign recognized increases W. The player
must choose how much to learn, knowing that understanding has a
mechanical cost.

ludoSpring's DDA does NOT make this easier. Instead, it monitors
stress vs engagement and adjusts *pacing* — giving the player
breathing room between revelations without reducing their impact.

---

## World Structure

### The Town (Hub)

- **Plane**: Exploration + Dialogue
- **Ruleset**: Cairn (CC-BY-SA) for exploration, custom Dialogue plane
- **Key locations**: Harbor, Main Street, Church, Library, Police Station,
  University annex, abandoned lighthouse, the marsh
- **NPCs**: 12-15 townsfolk, each with full NpcPersonality certs

### The Investigation

- **Plane**: Investigation (primary)
- **Core mechanic**: Evidence DAG. GUMSHOE principle: core clues are
  never gated behind rolls. You always make progress. The question
  is how much bonus context you earn and at what cost (W increase).
- **Clue types**:
  - Physical evidence (found in locations)
  - Testimony (from NPC dialogue — filtered by knowledge bounds)
  - Archival (from library/university research)
  - Experiential (from W-induced perception shifts)
- **Connection mechanic**: Player proposes connections between clues.
  System validates logical consistency. Logic voice assists at low W.
  At high W, Inland Empire offers connections that may or may not
  be valid.

### The Archives

- **Plane**: Investigation (specialized)
- **Location**: Town library and university annex
- **Mechanic**: Cross-referencing. Encyclopedia voice is strongest
  here. Research produces ResearchVertex entries that connect to
  the evidence DAG.
- **W cost**: Reading certain texts increases W. The most useful
  texts have the highest W cost.
- **Source texts**: "The Call of Cthulhu" (1928, public domain),
  "The Colour Out of Space" (1927, public domain), "The Dunwich
  Horror" (1929, public domain), "Dagon" (1919, public domain)

### The Rituals

- **Plane**: Crafting (unreliable)
- **Mechanic**: When the player finds occult texts, the Crafting
  plane activates for ritual preparation. But the crafting system
  is *alien* — the rules are not human. The RulesetCert has:
  - Very high variance on all rolls
  - Ingredients that don't map to normal categories
  - Success conditions that seem arbitrary (because they are)
  - Catastrophic failure produces W increase, not material loss
- **Design intent**: The unreliability IS the horror. You're trying
  to use tools designed for a mind structured differently from yours.

### The Encounter

- **Plane**: Tactical (but broken)
- **Mechanic**: When the player finally encounters something inhuman,
  the Tactical plane activates. But:
  - Weapons are ineffective (damage reduced to near-zero)
  - Action economy is disrupted (the entity acts outside initiative)
  - Standard tactics don't work (it doesn't have AC or HP in the
    normal sense)
  - DDA does NOT make it easier — ludoSpring detects increasing
    stress and Squirrel narrates the mounting dread
  - **The horror IS that the rules fail you**
- **Victory conditions**: Retreat, banishment (via ritual — Crafting
  plane), or containment. Not killing.

---

## NPC Gallery (Full Personality Certs)

### Sheriff Elias Marsh

```yaml
identity:
  name: "Sheriff Elias Marsh"
  role: "Town law enforcement, third-generation local"
  appearance: "Weathered, sea-salt eyes, moves slowly, watches everything"
  mannerisms: ["Chews tobacco when thinking", "Calls everyone 'friend'", "Hand rests on holster unconsciously"]

motivations:
  survival: { urgency: 0.3, current_state: "Physically capable" }
  safety: { urgency: 0.8, current_state: "Something is wrong and he can't protect the town from it", threat: "The disappearances" }
  belonging: { urgency: 0.6, current_state: "The town trusts him, but his authority is eroding" }
  esteem: { urgency: 0.4, current_state: "Was a good sheriff. Now feels helpless." }
  active_need: "safety"

knowledge_bounds:
  knows:
    - "Three fishermen disappeared in the last month"
    - "Strange lights at the reef on moonless nights"
    - "Old Man Whateley visits the lighthouse every Thursday"
  suspects:
    - topic: "The disappearances"
      belief: "Connected to the old Marsh family — his own family"
      confidence: 0.5
  lies_about:
    - topic: "His family history"
      surface_claim: "Marsh family has been fishing folk for generations"
      truth: "Family records suggest something happened in 1846 that the family covered up"
      reason: "Terrified of what it means"
      tell: "Voice drops, eyes go to the harbor"
      detection_dc: 14
      detection_skills: ["Empathy", "Perception"]
  does_not_know:
    - "What the lights at the reef actually are"
    - "The connection between the old texts and the disappearances"
    - "That Old Man Whateley has already been changed"
```

### Professor Margaret Armitage

```yaml
identity:
  name: "Professor Margaret Armitage"
  role: "Visiting scholar at university annex, medieval languages"
  appearance: "Sharp eyes behind round spectacles, ink-stained fingers, tweed"
  mannerisms: ["Corrects people's grammar", "Quotes texts from memory", "Forgets to eat when researching"]

motivations:
  survival: { urgency: 0.1, current_state: "Oblivious to physical danger" }
  safety: { urgency: 0.2, current_state: "Doesn't register threats" }
  esteem: { urgency: 0.9, current_state: "On the verge of a career-defining discovery" }
  self_actualization: { urgency: 0.8, current_state: "The texts she found could rewrite history" }
  active_need: "esteem"

knowledge_bounds:
  knows:
    - "The texts in the university annex include pre-Columbian maritime charts"
    - "Some charts reference locations that shouldn't exist"
    - "The language in certain documents is not any known Indo-European family"
  suspects:
    - topic: "The language"
      belief: "It may not be human in origin"
      confidence: 0.3
  lies_about:
    - topic: "Her research progress"
      surface_claim: "Slow going, nothing definitive yet"
      truth: "She has partially translated a text that terrifies her"
      reason: "Fear that revealing it would destroy her career (labeled a crank)"
      tell: "Glasses come off, cleaned obsessively; won't make eye contact"
      detection_dc: 10
      detection_skills: ["Perception", "Empathy"]
  does_not_know:
    - "The connection to the disappearances"
    - "That reading the untranslated passages has begun changing her dreams"
```

### Old Man Whateley

```yaml
identity:
  name: "Ezra Whateley"
  role: "Recluse, former fisherman, lighthouse keeper"
  appearance: "Stooped, smells of brine, eyes that don't quite track right"
  mannerisms: ["Mutters to himself", "Stares at the sea", "Hums a melody no one recognizes"]

motivations:
  survival: { urgency: 0.1, current_state: "Beyond caring" }
  belonging: { urgency: 0.9, current_state: "Desperate to belong to something — but not to the town" }
  self_actualization: { urgency: 0.7, current_state: "Believes he is becoming something greater" }
  active_need: "belonging"

knowledge_bounds:
  knows:
    - "What the lights at the reef are"
    - "The ritual that calls them"
    - "What happened to the three fishermen"
    - "The 1846 event — in detail"
  suspects: []
  lies_about:
    - topic: "The fishermen"
      surface_claim: "Drowned. The sea takes."
      truth: "They went willingly"
      reason: "Protecting the communion"
      tell: "The humming intensifies; he smiles"
      detection_dc: 8
      detection_skills: ["Perception", "Inland Empire"]
  does_not_know:
    - "That the change is not reversible"
    - "That the Professor has found the texts"
```

---

## Voice Behavior at Different W Levels

| W | Logic | Empathy | Perception | Inland Empire | Encyclopedia |
|---|-------|---------|------------|---------------|-------------|
| 0-2 | Reliable, precise | Accurate reads | Normal detail | Quiet, rare | Correct citations |
| 3-5 | Mostly reliable | Over-sensitive | Notices too much | Regular, often insightful | Obscure but correct |
| 6-8 | Contains errors | Sees fear everywhere | Hallucinates details | Frequent, dominant | References nonexistent texts |
| 9-11 | Contradicts itself | Cannot distinguish NPC emotions from player's | Reports things that aren't there | Speaks constantly | Cites texts from within the story |
| 12+ | Silent or hostile | Overwhelmed | Unreliable | THE voice | Speaks in the unknown language |

---

## Primals in Play

| Primal | Role in this World |
|--------|--------------------|
| Squirrel | NPC voices with knowledge-bounded deception; W-scaled narration reliability |
| ludoSpring | Monitors stress vs engagement; adjusts pacing (not difficulty) |
| rhizoCrypt | Evidence DAG with branching contradictions at high W |
| loamSpine | NPC certs, clue certs, ritual component certs |
| sweetGrass | Attributes investigation progress and world-building |
| BearDog | Signs skill checks and sanity rolls |
| toadStool | Passive voice checks (batch inference for 10 voices per action) |
| NestGate | Public domain Lovecraft corpus, historical context |

---

## Validation Targets

- W increase from knowledge acquisition is tracked in DAG
- Voice reliability degrades smoothly with W increase
- NPC knowledge bounds hold under all W levels
- Evidence DAG branches at high W without corrupting core clues
- Tactical plane correctly violates its own rules (the broken encounter)
- Crafting plane high-variance produces unreliable but not unfair outcomes
- DDA adjusts pacing without reducing horror impact
- Quorum event: if enough NPCs know the truth, the town changes

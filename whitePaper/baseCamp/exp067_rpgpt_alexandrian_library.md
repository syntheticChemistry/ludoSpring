# Expedition 067: RPGPT — The Alexandrian Library (Literary Multiverse)

**Date:** 2026-03-15
**Status:** Design
**Reference:** Cairn (CC-BY-SA), PF2e (ORC), FATE Core (CC-BY), Homer (public domain), 1001 Nights (public domain), Carroll (public domain)
**Depends on:** exp045 (Ruleset Control Systems), exp046 (Text Adventure DAG), exp052 (Provenance Trio)
**Architecture:** `specs/RPGPT_DEEP_SYSTEM_DESIGN.md`, `specs/RPGPT_PLANES_SCHEMA.md`, `specs/RPGPT_NPC_PERSONALITY_SPEC.md`

---

## Premise

The Great Library of Alexandria was never burned. It grew. Over
millennia, the Library expanded beyond geography — its halls became
doorways into the worlds described in its books. Each wing is a
different literary world. Each world runs on different rules.

The player is a Librarian-Traveler: someone who has learned to read
a book so deeply that they enter it. Their Library Card (a loamSpine
cert) is both passport and inventory — it records which wings they
have visited, what they have learned, and what they carry between
worlds.

---

## World Structure

### The Library (Hub)

- **Plane**: Exploration + Investigation
- **Ruleset**: Cairn (CC-BY-SA)
- **Mechanics**: Minimal. Inventory-as-HP (Cairn's fatigue system).
  Knowledge IS inventory — facts learned in wings become "items"
  that can be used in other wings.
- **AI narration**: Quiet, reverent, vast. The Library is infinite.
  Corridors echo. Dust motes drift in lantern light.
- **Key mechanic**: Finding the right book. The catalog is not
  organized by Dewey Decimal — it is organized by resonance.
  To find the book you need, you must understand the connection
  between what you seek and where it lives.

### Wing 1: The Homeric (Iliad + Odyssey)

- **Planes**: Tactical (battlefield) + Political (Olympus) + Dialogue (camps)
- **Rulesets**: PF2e (ORC) for combat, FATE Core (CC-BY) for divine politics
- **NPCs**:
  - **Achilles**: Maslow-driven. Esteem (0.9) wars with Belonging (0.7).
    Patroclus' death pushes Belonging to threatened, Esteem becomes
    rage. Knowledge bounds: knows about the prophecy (will die at Troy),
    lies about whether he cares.
  - **Odysseus**: Rhetoric and Composure are his dominant skills.
    Knowledge bounds broad but filtered through political lens.
    Suspects everything. Lies strategically. Tell: becomes too
    eloquent when the truth would serve better.
  - **Helen**: Deeply complex NPC. Empathy reads her grief.
    Logic sees the contradictions in her story. Inland Empire
    senses the weight of a thousand ships in her silence.
- **Transition flow**: Exploration (Trojan plain) -> Dialogue (camp
  council) -> Tactical (battle at the gates) -> Political (plea to
  Olympus) -> Dialogue (aftermath)
- **Source texts**: Iliad, Odyssey (public domain)

### Wing 2: The Arabian Nights

- **Plane**: Dialogue + Investigation
- **Ruleset**: Custom (storytelling mechanics)
- **Core mechanic**: Survival through narrative. Scheherazade's palace.
  The player must tell stories within the story. A DAG within the
  session DAG — nested narratives tracking which story contains
  which.
- **NPCs**:
  - **Scheherazade**: Trust model inverted — she trusts the player
    (fellow storyteller) but the player must earn the Sultan's trust
    through her. Knowledge bounds: knows everything about stories,
    nothing about escape.
  - **The Sultan**: Maslow Safety (0.9) — fears betrayal. Suspects
    all women of treachery. Lies about being immune to stories.
    Tell: leans forward when the story grips him.
- **Voice emphasis**: Rhetoric (to craft compelling stories),
  Encyclopedia (to draw from the Library's knowledge), Inland Empire
  (to sense which story will resonate)
- **Source texts**: 1001 Nights (public domain, Burton translation)

### Wing 3: Wonderland

- **Plane**: Exploration (with inverted rules)
- **Ruleset**: Custom — intentionally illogical
- **Core mechanic**: The RulesetCert for this wing *inverts* normal
  mechanics. Failure succeeds. Large things are small. Time flows
  backward in some rooms. Logic voice becomes unreliable. Inland
  Empire becomes your most trustworthy guide.
- **NPCs**:
  - **The Cheshire Cat**: Knowledge bounds inverted — knows everything,
    lies about nothing, but speaks only in riddles. The information is
    always accurate but never direct.
  - **The Red Queen**: Maslow Authority (0.9). Every interaction is
    a dominance check. Authority voice fires constantly.
- **ludoSpring metric**: Engagement closely monitored. DDA detects
  whether the surrealism is delightful or frustrating and adjusts the
  inversion intensity.
- **Source texts**: Alice in Wonderland, Through the Looking-Glass (public domain)

### The Connecting Thread

The Library Card tracks cross-wing resonance:

| Wing | Knowledge Gained | Where It Applies |
|------|-----------------|------------------|
| Homer | Understanding of rage and grief | Helps read the Sultan's fear |
| Arabian Nights | Mastery of nested narrative | Navigating Wonderland's inversions |
| Wonderland | Comfort with illogic | Reading the Cheshire Cat's riddles in any wing |

The Quorum mechanic tracks thematic resonance: when similar themes
accumulate across wings (e.g., "power costs isolation"), meta-events
trigger in the Library itself — new corridors open, ancient Librarians
appear with commentary.

---

## Primals in Play

| Primal | Role in this World |
|--------|--------------------|
| Squirrel | Voices all NPCs (Achilles, Scheherazade, Cheshire Cat) with distinct personalities |
| ludoSpring | Monitors engagement per wing; DDA adjusts Wonderland inversion intensity |
| rhizoCrypt | Session DAG with nested DAGs (stories within stories) |
| loamSpine | Library Card cert, NPC personality certs, wing ruleset certs |
| sweetGrass | Attributes player world-building contributions to Library lore |
| BearDog | Signs dice rolls for combat and skill checks |
| petalTongue | Renders the Library map, wing environments, dialogue trees |
| NestGate | Public domain text corpus (Homer, 1001 Nights, Carroll) |

---

## Validation Targets

- Cross-wing knowledge transfer preserves DAG integrity
- NPC personality certs maintain consistency across sessions
- Plane transitions within a wing are smooth (Tactical -> Political in Homer)
- Nested narrative DAG (Arabian Nights) correctly tracks story containment
- Wonderland inversion rules correctly negate standard mechanics
- Quorum threshold events trigger from cross-wing thematic resonance

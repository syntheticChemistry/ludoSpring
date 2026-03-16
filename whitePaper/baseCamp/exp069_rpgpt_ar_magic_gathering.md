# Expedition 069: RPGPT — The Living Card Table (AR Magic: The Gathering)

**Date:** 2026-03-15
**Status:** Design
**Reference:** exp047 (MTG Card Provenance), exp048 (Stack Resolution Folding), exp049 (Game Tree Infinity), exp050 (Game Tree Complexity), exp051 (Games@Home)
**Depends on:** petalTongue (AR overlay), loamSpine (card certs), rhizoCrypt (game state DAG), Songbird (remote players), BearDog (shuffle integrity)
**Architecture:** `specs/RPGPT_DEEP_SYSTEM_DESIGN.md`, `specs/RPGPT_PLANES_SCHEMA.md` (Card/Stack plane)

---

## Premise

Physical cards on a real table. Physical shuffling. Physical social
experience. But augmented with everything digital systems can provide
without replacing the tactile joy of paper.

This is not "digital MTG." This is MTG as it has always been played,
with an AR layer that handles the bookkeeping, tracks the stack,
catches missed triggers, enables remote play, and gives every card
a provenance history.

---

## System Architecture

### Physical Layer (Unchanged)

- Real cards, real table, real shuffling
- Players tap, untap, draw, discard, play physically
- Social interaction is face-to-face (or camera-linked for remote)

### Recognition Layer (petalTongue)

petalTongue's camera-based recognition system:

1. **Card identification**: Camera reads card faces. Each card maps
   to a loamSpine cert. Recognition uses set symbol + collector number
   + art hash (not OCR of rules text).
2. **Zone tracking**: Cards in hand, battlefield, graveyard, exile,
   command zone. Camera tracks physical zones mapped to table regions.
3. **Tap state**: Rotation detection distinguishes tapped/untapped.
4. **Counter/token tracking**: Physical tokens are supplemented with
   digital counter overlays for complex board states.

### Digital Overlay (petalTongue AR)

Rendered on a tablet/phone pointed at the table, or AR glasses:

- **Life totals**: Large, clear, auto-updating from damage vertices
- **The stack**: exp048's stack-as-protein-folding visualization.
  The LIFO stack rendered as a visible, spatial overlay above the
  table. New players can SEE the stack. Complex instant-speed
  interactions become comprehensible.
- **Triggered abilities**: When a trigger condition is met (enters
  the battlefield, dies, attacks), the overlay highlights the card
  and the trigger text. No more missed triggers.
- **Phase/step tracking**: Current phase highlighted. Priority indicator
  shows which player can act.
- **Damage tracking**: Damage marked (not health, per MTG rules) on
  each creature until end of turn.
- **Commander damage**: Per-opponent commander damage tracked automatically.

### Game State DAG (rhizoCrypt)

Every game action is a vertex:

```
GameActionVertex {
    action_type: Play | Activate | Attack | Block | Resolve | Draw | Discard | ...,
    player: <player_did>,
    card: <loamSpine_cert_id>,
    zone_from: <zone>,
    zone_to: <zone>,
    mana_spent: <mana_cost>,
    targets: Vec<target_cert_id>,
    stack_position: Option<usize>,
    timestamp: <vertex_time>,
    beardog_signature: <Ed25519_sig>,
}
```

This is the same DAG architecture from exp047, extended with
real-time zone tracking and stack resolution from exp048.

### Card Certificates (loamSpine)

Each physical card has a digital twin as a loamSpine cert:

```yaml
certificate_type: "mtg.card"
card_identity:
  name: "Grizzly Bears"
  set: "Limited Edition Alpha"
  collector_number: "164"
  art_hash: <blake3_hash_of_card_art>
ownership:
  current_owner: <player_did>
  provenance_chain:
    - acquired: "2024-03-01"
      from: "LGS draft"
      condition: "NM"
    - traded: "2025-06-15"
      from: <other_player_did>
      for: <trade_cert_id>
game_history:
  games_played: 847
  wins: 412
  notable_plays:
    - "2025-08-20: blocked a lethal Emrakul swing in Commander pod"
    - "2026-01-15: sacrificed to Ashnod's Altar for the winning combo"
```

This is the Novel Ferment Transcript from exp061. The card's value
comes from its accumulated history. That specific Grizzly Bears you
have played for 15 years has a story.

---

## Formats Supported

### Commander (4-player)

The flagship format for AR MTG:

- 4 players, each with camera recognition
- Command zone tracking (commander tax auto-calculated)
- Commander damage per-opponent (automatic)
- Political tracking: who attacked whom, alliance patterns (Political
  plane integration from RPGPT schema)
- "The Table" view: all 4 board states composited on one screen
  for spectators or broadcast

### Draft

- Card pool tracking during draft (loamSpine cert assignment)
- Deck-building assistance (legal card count, color identity)
- Pick history as DAG (what you passed is tracked)
- Post-draft deck registration (certified by BearDog)

### Standard/Modern/Legacy (2-player)

- Standard tournament features
- Auto-generated match record (vertices from rhizoCrypt)
- Sideboard tracking between games

---

## Remote Play (Songbird)

The killer feature for kitchen-table Commander:

1. Each player has their physical cards on their physical table
2. Camera captures their board state
3. Songbird discovery finds the other pod members
4. biomeOS orchestrates the game state DAG across players
5. Each player's AR overlay shows ALL players' board states
   composited together
6. The stack is shared and visible to all players in real-time
7. Priority passing works via the overlay

Latency budget: the game is turn-based (not 60Hz). Vertex
propagation in under 500ms is acceptable. rhizoCrypt's DAG
sync handles conflict resolution (two players attempting
priority simultaneously).

---

## AI Judge (Squirrel)

Squirrel reads the game state DAG and the comprehensive rules cert
(MTG Comprehensive Rules ingested as a loamSpine ruleset cert):

### Capabilities

- **Rules questions**: "Can I cast this at instant speed?"
  Squirrel reads the card's cert + current game state + active
  effects and provides an accurate answer.
- **Missed trigger detection**: When a trigger condition is met
  but no corresponding TriggerVertex appears within a time window,
  the judge alerts the table.
- **Illegal play detection**: If a card is played that can't be
  (not enough mana, wrong phase, illegal target), the judge flags
  it before the stack resolves.
- **Interaction explanation**: "What happens if I cast this in
  response to that?" The judge walks through the stack resolution
  step by step (exp048 folding visualization).
- **Suggestions for new players**: Optional mode where the judge
  highlights key decision points and explains available plays.
  Gated behind player opt-in (never unsolicited for experienced
  players).

### Constraint

The judge NEVER makes strategic recommendations to one player.
It enforces rules equally and explains interactions. It is a
referee, not an advisor. This is enforced by the Squirrel routing
constraint: the judge's system prompt explicitly forbids strategic
advice.

---

## Deck Provenance

A deck is a collection of loamSpine card certs assembled into a
DeckCert:

```yaml
certificate_type: "mtg.deck"
format: "Commander"
commander: <card_cert_id>
cards: [<card_cert_ids>]  # 99 + commander
owner: <player_did>
created: "2024-01-01"
last_modified: "2026-03-15"
tournament_history:
  - event: "FNM 2025-08-22"
    record: "3-1"
    notable: "Won with Grizzly Bears block"
  - event: "CommandFest 2025-12-01"
    record: "2-2"
    notable: "Survived combo winter"
sweetgrass_attribution:
  builder: <player_did>
  inspiration: "based on Cedh Winota primer"
  modifications: 42  # cards changed from original list
```

Trading a card updates the ownership chain in the card cert.
The deck cert reflects the change. History is never lost.

---

## The Stack Visualization (exp048 Extended)

From exp048, we proved that MTG stack resolution is isomorphic to
protein folding: same components in different order produce
different outcomes. The AR visualization:

- Stack items rendered as floating cards above the table
- LIFO order is spatial — newest on top
- Resolution animates from top to bottom
- When a new item is added (in response), it visually slides in
  above the current stack
- Complex chains (e.g., Krark + Sakashima copying triggers) are
  rendered as branching trees
- Students of the game can pause the stack and examine each item
- Color coding: instants (blue), abilities (gold), triggers (green)

---

## Game Tree Metrics (exp050 Integration)

ludoSpring's game tree complexity metrics from exp050 run in
real-time:

- **Branching factor**: How many legal plays available right now?
  High branching → the judge can highlight key decision points.
- **Commander format multiplier**: exp050 proved Commander is
  ×216 more complex than 1v1 (4 players × deck diversity × political
  dynamics). The AR overlay manages this complexity.
- **Novel positions**: exp049 proved the game tree is provably
  infinite. Every game produces novel data. This makes every
  recorded game a genuine contribution to the Games@Home concept
  (exp051).

---

## Primals in Play

| Primal | Role in AR MTG |
|--------|---------------|
| Squirrel | AI judge (rules, missed triggers, interactions), new player assistant |
| ludoSpring | Game tree metrics, branching factor, complexity tracking |
| rhizoCrypt | Game state DAG (every action is a vertex) |
| loamSpine | Card certs, deck certs, ruleset cert (Comprehensive Rules) |
| sweetGrass | Deck-building attribution, game history |
| BearDog | Shuffle integrity (provably random), match record signing |
| Songbird | Remote player discovery, pod assembly |
| biomeOS | Game state sync across remote players |
| petalTongue | AR overlay, stack visualization, zone tracking |
| NestGate | Card database, rulings database, historical errata |

---

## Validation Targets

- Physical card recognition accuracy (set + collector number)
- Zone transition tracking matches physical card movement
- Stack visualization correctly renders LIFO resolution
- Remote play: vertex propagation < 500ms between players
- AI judge accuracy on rules questions (validated against MTG rulings)
- Missed trigger detection within 5-second window
- Card provenance chain integrity across trades
- Commander damage tracking accuracy in 4-player games
- Game tree metrics match exp050 theoretical predictions

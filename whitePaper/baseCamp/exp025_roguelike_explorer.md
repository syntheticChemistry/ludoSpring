# Expedition 025: Roguelike Parameter Explorer — Engagement-Driven Procedural Generation

**Date:** March 11, 2026
**Status:** Playable — BSP+noise dungeons with DDA, Flow, engagement tracking
**Real game reference:** Caves of Qud, Brogue, DCSS, NetHack

---

## Science as a Game

This expedition combines *every* validated ludoSpring model into a single
playable experience. The dungeon adapts to the player:

| Model | Role in game | Source |
|-------|-------------|--------|
| BSP | Room layout backbone | Fuchs (1980) |
| Perlin noise | Item placement density | Perlin (1985) |
| DDA | Difficulty adjustment between floors | Hunicke (2005) |
| Flow state | Real-time flow classification | Csikszentmihalyi (1990) |
| Engagement | Session quality scoring | Yannakakis & Togelius (2018) |
| Four Keys to Fun | Fun type classification | Lazzaro (2004) |

## How Engagement-Driven Generation Works

After each floor, the game measures the player's performance window and
invokes `suggest_adjustment()` (Hunicke 2005). The DDA module recommends a
difficulty change based on:
- **Estimated skill:** Moving average of success/failure outcomes
- **Trend:** Is the player improving or struggling?
- **Target:** `DDA_TARGET_SUCCESS_RATE` (70% success, from tolerances)

The difficulty modifier affects:
- **BSP min_size:** Smaller rooms = tighter spaces = harder
- **Item threshold:** Higher difficulty = fewer items (Perlin noise threshold)

The Flow state (Csikszentmihalyi 1990) is displayed in real time:
- **Flow** (green): Challenge matches skill
- **Anxiety** (red): Challenge too high
- **Boredom** (blue): Challenge too low
- **Arousal** (yellow): Challenge slightly above skill
- **Relaxation** (cyan): Challenge slightly below skill

## Session Summary

After quitting, the game prints a complete scientific analysis:
- Engagement metrics (APM, exploration rate, persistence, composite)
- Fun classification (Hard/Easy/People/Serious)
- Flow state at session end
- DDA recommendation for hypothetical next session

## Extensibility

The same patterns apply far beyond roguelikes:

| ludoSpring pattern | Game application | Non-game application |
|-------------------|-----------------|---------------------|
| Turn-based loop | Chess, card games | Spreadsheet navigation |
| BSP room layout | Dungeon generation | Office floor plans |
| DDA adjustment | Monster density | Exam difficulty |
| Flow tracking | Game difficulty | Learning software |
| Engagement metrics | Session quality | Student attention |
| Fun classification | Game feel tuning | UX research |
| Noise-driven items | Loot placement | Data sampling |

**WFC for music composition:** Replace spatial adjacency (wall↔floor) with
harmonic adjacency (C major↔G major). Constraint propagation produces chord
progressions that respect music theory — the same algorithm, different rules.

**DDA for adaptive assessments:** After each question, `suggest_adjustment`
tunes the next question's difficulty based on the student's performance window.
Flow state tracking tells the teacher whether the student is engaged.

## Controls

WASD/arrows: move | .: wait | i: stats | Esc: quit

## Reproducibility

```bash
cargo run --bin exp025_roguelike_explorer          # default seed
cargo run --bin exp025_roguelike_explorer -- 99    # custom seed
```

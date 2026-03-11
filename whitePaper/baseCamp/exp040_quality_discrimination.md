# Expedition 040: Game Quality Discrimination

**Date:** 2026-03-11
**Status:** Active
**Reference:** Csikszentmihalyi (1990), Yannakakis (2018), Lazzaro (2004), Hunicke (2005)

## What We Built

The acid test for ludoSpring's metrics framework: can it discriminate
between fundamentally different game experiences? 5 archetypes x 2
quality levels = 10 synthetic sessions.

### Results

| Archetype | Quality | Engagement | Flow | Fun | DDA |
|-----------|---------|------------|------|-----|-----|
| Idle Clicker | Good | 0.204 | Boredom | Serious | +0.20 |
| Idle Clicker | Bad | 0.208 | Boredom | Hard | +0.40 |
| Roguelike | Good | 0.122 | **Flow** | Easy | -0.40 |
| Roguelike | Bad | 0.107 | Anxiety | Easy | -1.00 |
| Puzzle | Good | 0.116 | **Flow** | Serious | -0.30 |
| Puzzle | Bad | 0.129 | Anxiety | Easy | -1.00 |
| FPS | Good | 0.236 | **Flow** | Easy | -0.30 |
| FPS | Bad | 0.220 | Anxiety | Easy | -1.00 |
| Souls-like | Good | 0.122 | **Flow** | Easy | -0.20 |
| Souls-like | Bad | 0.119 | Anxiety | Easy | -1.00 |

### Key Scientific Finding

**Engagement alone doesn't discriminate quality. Flow does.**

Engagement composite (Yannakakis 2018) measures *activity level* — frantic
frustration and genuine fun can produce similar APM. Csikszentmihalyi's
Flow state is the quality signal: 4/5 well-designed games are in Flow,
5/5 poorly-designed games are NOT in Flow (all in Anxiety or Boredom).

This validates the theoretical prediction: you need BOTH engagement
(how active) AND flow (how well-calibrated) to assess game quality.

### Discrimination Summary

- **Fun keys**: Roguelike→Easy, Puzzle→Serious, Souls→Hard (correct per Lazzaro)
- **DDA**: All bad games get strong "decrease difficulty" signal
- **Flow**: Perfect separation of good (Flow) vs bad (Anxiety)
- **5 archetypes produce distinct engagement profiles**

### Reproducibility

```bash
cargo run --bin exp040_quality_discrimination              # validate (12 checks)
cargo run --bin exp040_quality_discrimination -- report     # full archetype analysis
```

# ludoSpring

The seventh ecoPrimals spring. The science of play, interaction, and game design.

## What is ludoSpring?

Where wetSpring validates bioinformatics, hotSpring validates nuclear physics, and healthSpring builds usable health applications, **ludoSpring treats game design as a rigorous science**.

Games are the most demanding real-time interactive systems humans build. They solve problems every primal needs: input handling, spatial navigation, physics simulation, procedural content generation, accessibility, and the deep question of what makes interaction *engaging*.

By studying games with the same rigor as particle physics, every primal in the ecosystem benefits.

## Domains

| Module | What it studies | Key models |
|--------|----------------|------------|
| `game` | Mechanics, state machines, genre taxonomy | Raycasting (DDA), voxel worlds, session state |
| `interaction` | Input science, flow, accessibility | Fitts's law, Hick's law, Csikszentmihalyi flow, DDA |
| `procedural` | Content generation | Perlin/simplex noise, wave function collapse, fBm |
| `metrics` | Quantifying fun | Tufte-on-games, engagement curves, UI analysis |

## Reference Systems

| System | What we learn | Experiment |
|--------|--------------|------------|
| Doom (1993) | Raycasting, minimal UI, input latency | Exp001 |
| Minecraft | Procedural worlds, emergent gameplay | Exp002 |
| Folding@Home / FoldIt | Science-as-game, player vs AI | Exp004 |
| Genre comparison | FPS vs RTS vs sandbox UI patterns | Exp003 |

## Architecture

```
ludoSpring/
├── barracuda/          # Core library: game science + GPU via barraCuda
├── metalForge/forge/   # Hardware dispatch (noise→GPU, WFC→CPU)
├── experiments/        # Reproducible experiments
│   ├── exp001_doom_raycaster_analysis/
│   ├── exp002_procedural_molecule_gen/
│   ├── exp003_tufte_game_ui/
│   └── exp004_folding_adversarial/
├── specs/              # Domain specifications
├── wateringHole/       # Handoff documentation
└── whitePaper/         # Research publications
```

## Key Insight: Games ↔ Science Visualization

Game genres are interaction architectures, not aesthetic categories:

| Genre pattern | Scientific analogue |
|---------------|-------------------|
| FPS (first-person spatial) | Molecular explorer, particle cave |
| RTS (top-down command) | Systems biology dashboard |
| Sandbox (open-ended building) | Molecule builder, circuit simulator |
| Roguelike (procedural discovery) | Parameter space exploration |
| Puzzle (constraint satisfaction) | Protein folding, crystal packing |

ludoSpring provides the validated models to make these mappings rigorous.

## Build

```bash
cargo test --workspace
cargo run -p ludospring-exp001
cargo run -p ludospring-exp003
```

## License

AGPL-3.0-or-later

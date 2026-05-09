# fossilRecord — Prokaryotic Era Preservation

**Fossilization Date:** May 9, 2026
**Event:** Interstadial Primordial Extinction Wave (primalSpring Phase 60+)
**Context:** Eukaryotic evolution — 100 experiment crates absorbed into UniBin validation scenarios

## What Happened

ludoSpring's 100 prokaryotic experiment crates (`exp001`–`exp100`) were
individual binaries — each a standalone validation of one scientific model.
In the eukaryotic evolution, representative experiments are absorbed as
**validation scenarios** inside the UniBin (`ludospring validate`), and the
original source is preserved here as the fossil record.

## Structure

```
fossilRecord/
└── experiments_prokaryotic_may2026/
    ├── exp001_doom_raycaster_analysis/
    ├── exp002_procedural_molecule_gen/
    │   ...
    └── exp100_nucleus_composition_parity/
```

## How to Build (if needed for archaeology)

These crates are no longer in the workspace. To build individual experiments:

```bash
cd fossilRecord/experiments_prokaryotic_may2026/exp001_doom_raycaster_analysis
cargo build --manifest-path Cargo.toml
```

Note: path dependencies (e.g., `ludospring-barracuda`) reference `../../barracuda`
which must be adjusted to `../../../barracuda` due to the extra directory nesting.

## Provenance

- **Pre-extinction test count:** 820+ (workspace total)
- **Post-extinction test count:** Maintained via scenario registry + lib tests
- **Absorbed scenarios:** interaction_laws, procedural_gen, engagement_metrics,
  composition_parity, raycaster_budget
- **Full science preserved in:** `barracuda/src/` (library modules)
- **Git history:** All experiment evolution fully preserved in git log

## License

AGPL-3.0-or-later

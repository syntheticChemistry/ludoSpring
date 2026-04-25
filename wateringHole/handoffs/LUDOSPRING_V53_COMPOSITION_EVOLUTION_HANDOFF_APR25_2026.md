# ludoSpring V53 — Binary to Composition Evolution

**Date:** April 25, 2026
**From:** ludoSpring V53
**For:** All spring teams, primalSpring, biomeOS, esotericWebb

---

## Executive Summary

Springs are NOT primals — they produce primals and define compositions.
The `ludospring` binary in `plasmidBin` was an oversight from the Rust
validation round (V42). V53 corrects this: game science capabilities are
now served by composing existing primals via the NUCLEUS cell graph, not
by deploying a spring binary.

The spring binary remains in the source tree as the Rust validation
target (tier 2 of the 3-tier ladder: Python → Rust → Composition).
817 workspace tests continue to validate the Rust tier.

---

## What Changed

### 1. plasmidBin entry transformed

- **Removed:** `plasmidBin/ludospring/ludospring` binary (3.2M)
- **Transformed:** `metadata.toml` from binary primal description to
  composition manifest with `[composition]` section, capability routing
  table, and 11-primal dependency list
- **Added:** `ludospring_cell.toml` cell graph in plasmidBin directory

### 2. Cell graph evolved (12 nodes, was 14)

- **Removed:** `ludospring` node — no spring binary deploys
- **Evolved:** `barracuda` node now provides game science capabilities
  (`math.*`, `activation.*`, `stats.*`, `noise.*`, `rng.*`, `tensor.*`)
- **Validation:** Checks barraCuda + petalTongue health instead of
  ludospring health

### 3. Gaming niche graph evolved

- **Replaced:** `germinate_ludospring` → `germinate_barracuda`
- **Added:** Full barraCuda capability routing for game science
- **Updated:** Validation dependencies

### 4. manifest.lock updated

- **Moved:** ludospring from `[springs.ludospring]` to
  `[compositions.ludospring_game]` with cell graph reference
- **Updated:** `storytelling` composition to reference `barracuda`
  instead of `ludospring`

### 5. GAP-10 resolved

The `game.*` primal identity gap is resolved by the pure composition
model. Game science methods map to barraCuda capabilities:

| Game Science | barraCuda Capability |
|-------------|---------------------|
| Flow evaluation | `math.sigmoid`, `stats.mean` |
| Fitts cost | `activation.fitts` |
| Engagement | `stats.mean`, `stats.std_dev` |
| Noise generation | `noise.perlin2d` |
| WFC | `rng.uniform`, `tensor.create` |
| DDA | `activation.fitts`, `activation.hick` |

---

## Capability Routing Table

| Capability Domain | Primal Provider | Required |
|------------------|----------------|----------|
| `math.*`, `activation.*`, `stats.*`, `noise.*` | barraCuda | no (skip) |
| `visualization.*`, `interaction.*` | petalTongue | yes |
| `ai.*`, `inference.*` | Squirrel | no (skip) |
| `compute.*` | ToadStool | no (skip) |
| `shader.*` | coralReef | no (skip) |
| `crypto.*` | BearDog | yes |
| `discovery.*` | Songbird | yes |
| `storage.*` | NestGate | no (skip) |
| `dag.*` | rhizoCrypt | no (skip) |
| `certificate.*` | loamSpine | no (skip) |
| `attribution.*` | sweetGrass | no (skip) |

---

## For Upstream Teams

### primalSpring
- `downstream_manifest.toml` ludospring entry should drop
  `guidestone_binary = "ludospring"` → composition-only validation
- `graphs/cells/ludospring_cell.toml` should drop the `ludospring`
  node (we've done this locally)

### biomeOS
- The `ludospring_game` composition in `manifest.lock` is the new
  deployment unit — no binary to fetch/distribute
- `fetch.sh` will naturally skip ludospring (no binary present)

### barraCuda
- Game science methods route through barraCuda capabilities
- Absorption opportunities: `activation.fitts`, `activation.hick`,
  `math.sigmoid` for flow/DDA; `noise.perlin2d` for PCG;
  `stats.mean`/`stats.std_dev`/`stats.variance` for engagement metrics

### All springs
- This is the pattern: springs validate in Rust (tier 2), deploy as
  compositions (tier 3). No spring binary belongs in plasmidBin.

---

## Files Changed

| File | Change |
|------|--------|
| `plasmidBin/ludospring/ludospring` | DELETED |
| `plasmidBin/ludospring/metadata.toml` | Transformed to composition manifest |
| `plasmidBin/ludospring/ludospring_cell.toml` | NEW — cell graph for biomeOS |
| `plasmidBin/manifest.lock` | ludospring → `[compositions.ludospring_game]` |
| `graphs/ludospring_cell.toml` | Removed ludospring node, 12 primal nodes |
| `graphs/ludospring_gaming_niche.toml` | germinate_ludospring → germinate_barracuda |
| `niches/ludospring-game.yaml` | v2.0.0, 11 composed primals |
| `docs/PRIMAL_GAPS.md` | GAP-10 RESOLVED |
| `CHANGELOG.md` | V53 entry |
| `README.md` | V53, deployment model updated |
| `CONTEXT.md` | V53, ecosystem position clarified |

---

## Validation

- 817 workspace tests continue to pass (Rust tier validation)
- The composition model is validated by the cell graph structure
- guideStone readiness 4 (three-tier: bare + IPC + NUCLEUS)

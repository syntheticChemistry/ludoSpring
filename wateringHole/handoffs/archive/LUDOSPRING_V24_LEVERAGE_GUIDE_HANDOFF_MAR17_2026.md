# ludoSpring V24 — Primal Leverage Guide + Cross-Ecosystem Absorption Handoff

**Date:** March 17, 2026
**From:** ludoSpring (V24)
**To:** All primals, all springs, biomeOS orchestration
**Supersedes:** V23 (Cross-Ecosystem Deep Debt + toadStool/barraCuda Absorption)
**License:** AGPL-3.0-or-later

---

## 1. What ludoSpring Is

ludoSpring is the **game science** niche — ludology, interaction design, procedural
generation, and play mechanics. It validates scientific Python baselines for game
systems ported to sovereign Rust+GPU via the ecoPrimals stack.

**Domain prefix:** `game.*`
**24 capabilities:** flow evaluation, Fitts's law, engagement metrics, UI analysis,
accessibility scoring, wave function collapse, difficulty adjustment, noise generation,
provenance lifecycle, NPC dialogue, scene visualization, DAG queries, certificate
minting, content-addressed storage, and 4 GPU compute ops.

---

## 2. ludoSpring Standalone — What We Offer Alone

Any primal or spring can call ludoSpring via `capability.call` through the Neural API.
No direct coupling — biomeOS routes by capability.

### Science Capabilities (local compute, no external deps)

| Capability | What It Does | Use Case for Others |
|------------|-------------|---------------------|
| `game.evaluate_flow` | Csíkszentmihályi flow state scoring | Any UI/UX evaluation — not just games |
| `game.fitts_cost` | Fitts's law pointing cost | Touch/click target optimization for any interface |
| `game.engagement` | Multi-factor engagement scoring (dwell, action rate, variety) | Measuring user engagement in any interactive system |
| `game.analyze_ui` | Heuristic UI analysis (clutter, contrast, density) | Automated UI review for petalTongue compositions |
| `game.accessibility` | WCAG-adjacent scoring for interactive systems | Accessibility audit for any primal's UI surface |
| `game.wfc_step` | Wave function collapse iteration | Procedural map/layout generation for any spatial domain |
| `game.difficulty_adjustment` | Dynamic difficulty recommendation | Adaptive complexity for any learning/training system |
| `game.generate_noise` | Perlin/simplex/value noise fields | Terrain, texture, variation for any domain needing spatial noise |

### GPU Compute (via toadStool dispatch)

| Capability | What It Does | Reusable For |
|------------|-------------|-------------|
| `game.gpu.fog_of_war` | Visibility propagation on 2D grids | Any spatial visibility/occlusion problem |
| `game.gpu.tile_lighting` | Point-light tile illumination | Environmental sensing visualization, heatmaps |
| `game.gpu.pathfind` | Wavefront pathfinding on grids | Routing in any grid-based domain (logistics, biology) |
| `game.gpu.perlin_terrain` | GPU-accelerated Perlin terrain | Any domain needing fast noise fields |

---

## 3. ludoSpring + Provenance Trio Combos

The provenance trio (rhizoCrypt + loamSpine + sweetGrass) gives ludoSpring
permanent memory, verifiable ownership, and creative attribution. Here's how
other primals and springs can leverage these combos.

### ludoSpring + rhizoCrypt (DAG memory)

| Combo | Pattern | Novel Uses |
|-------|---------|------------|
| `game.begin_session` → `dag.session.create` | Every game session becomes a Merkle DAG | **Auditable interaction logs** — any primal can replay a user's decision history |
| `game.record_action` → `dag.event.append` | Every action is a vertex with parent links | **Causal analysis** — which action caused which outcome, across any interactive system |
| `game.query_vertices` → `dag.vertex.query` | NPC memory retrieval by type/agent | **Persistent context** — any AI agent (Squirrel) can retrieve structured interaction history |
| `game.complete_session` → `dag.dehydration.trigger` | Merkle root + frontier computation | **Session integrity** — anti-tampering proof for any interaction sequence |

**For other springs:** wetSpring could use this pattern for lab experiment audit trails.
groundSpring could record seismic analysis decision chains. healthSpring could track
patient interaction sequences with full Merkle integrity.

### ludoSpring + loamSpine (certificates)

| Combo | Pattern | Novel Uses |
|-------|---------|------------|
| `game.mint_certificate` → `certificate.mint` | Game objects as permanent certificates | **Digital twin provenance** — any physical/digital object can be certificated |
| Character sheets as certs | Player identity = cert chain | **Portable identity** — character persists across systems, loanable between campaigns |
| Ruleset certificates | Game rules certified and versioned | **Reproducible experiments** — any spring can certificate its methodology |

**For other springs:** airSpring could certificate soil sensor calibrations.
healthSpring could certificate patient consent records. neuralSpring could
certificate model provenance (which dataset, which architecture, which hyperparams).

### ludoSpring + sweetGrass (attribution)

| Combo | Pattern | Novel Uses |
|-------|---------|------------|
| Session → braid | Game sessions attributed to creators | **Creative attribution** — who designed what NPC, quest, dialogue |
| Collaborative campaigns | Multiple players → attribution shares | **Collaborative provenance** — multi-agent contribution tracking for any creative process |
| NPC personality braids | NPC evolution attributed across sessions | **Evolving entity lineage** — track how any AI agent's personality changed and why |

---

## 4. ludoSpring + Wider Primal Combos

### ludoSpring + Squirrel (AI)

| Combo | What It Enables |
|-------|-----------------|
| `game.npc_dialogue` + engagement scoring | AI-generated dialogue rated for engagement quality |
| `game.narrate_action` + flow evaluation | Narration pacing matched to player flow state |
| `game.voice_check` + accessibility scoring | Voice output validated for accessibility compliance |
| NPC memory (rhizoCrypt) + Squirrel context | NPCs remember past conversations with full provenance |

**Novel cross-spring use:** Any spring with a user-facing component could use
Squirrel for natural language interaction with ludoSpring engagement scoring
to measure whether the interaction is effective.

### ludoSpring + petalTongue (visualization)

| Combo | What It Enables |
|-------|-----------------|
| `game.push_scene` + GameScene DataBinding | Live game scene rendering via Grammar of Graphics |
| Engagement dashboard + StreamSession | Real-time engagement metrics flowing to visual panels |
| Tufte validation + `visualization.validate` | Pre-flight UI quality checks before rendering |
| Multi-panel dashboards (map + dialogue + stats) | Coordinated multi-view game state display |

**Novel cross-spring use:** Any spring's validation results could be pushed
through ludoSpring's Tufte validation before petalTongue renders them —
ensuring all visualizations meet data-ink ratio and chartjunk standards.

### ludoSpring + toadStool (compute)

| Combo | What It Enables |
|-------|-----------------|
| `compute.submit` fog/lighting/pathfind | GPU-accelerated game science via toadStool dispatch |
| `compute.capabilities` → adaptive dispatch | Choose CPU/GPU/NPU based on available hardware |
| Batch engagement scoring | High-frequency action processing for live sessions |

**Novel cross-spring use:** toadStool could route ludoSpring's WFC (wave function
collapse) as a general-purpose constraint solver — useful for wetSpring molecular
docking, groundSpring sensor placement optimization, healthSpring drug combination search.

### ludoSpring + NestGate (storage)

| Combo | What It Enables |
|-------|-----------------|
| `game.storage_put` / `game.storage_get` | Content-addressed game state persistence |
| Character snapshots → BLAKE3 hash | Deterministic save/load with integrity verification |
| World state → NestGate + loamSpine cert | Certified world checkpoints, verifiable rollback |

### ludoSpring + coralReef (sovereign GPU)

| Combo | What It Enables |
|-------|-----------------|
| WGSL → coralReef compile → native dispatch | Sovereign shader execution without wgpu |
| coral-glowplug lifecycle management | PCIe device boot/suspend/resume for GPU compute |
| Shader provenance tracking | Which version of which shader produced which result |

### ludoSpring + BearDog (security)

| Combo | What It Enables |
|-------|-----------------|
| Session authentication | Player identity verified via BearDog credentials |
| Anti-cheat DAG verification | Merkle proofs on game sessions validated by BearDog |
| Encrypted game state | NestGate storage encrypted at rest via BearDog keys |

### ludoSpring + biomeOS (orchestration)

| Combo | What It Enables |
|-------|-----------------|
| Pathway orchestration | Multi-primal game pipelines (session start → record → complete → visualize) |
| Cost-aware routing | biomeOS picks cheapest path through ludoSpring capabilities |
| Live monitoring | biomeOS aggregates ludoSpring health + telemetry for operational dashboards |

---

## 5. Cross-Spring Novel Compositions

These are emergent capabilities when ludoSpring combines with other springs
through the primal layer:

| Composition | Springs + Primals | What Emerges |
|------------|-------------------|--------------|
| **Gamified lab training** | ludoSpring + wetSpring + Squirrel | Interactive lab protocol trainer with NPC mentors, engagement scoring, and provenance-tracked learning |
| **Adaptive sensor games** | ludoSpring + airSpring + toadStool | Gamified environmental monitoring where sensor data drives procedural worlds |
| **Therapeutic play** | ludoSpring + healthSpring + sweetGrass | Play therapy with certified progress, attributed to specific therapeutic approaches |
| **Seismic exploration game** | ludoSpring + groundSpring + petalTongue | Gamified seismic analysis with WFC-generated terrain matching real geological data |
| **ML experiment playground** | ludoSpring + neuralSpring + rhizoCrypt | Interactive ML hyperparameter exploration with DAG-tracked experiment lineage |
| **Physics sandbox** | ludoSpring + hotSpring + coralReef | Real-time physics simulation with sovereign GPU compute and game UI |

---

## 6. V24 Changes (Absorption Sprint)

### Absorbed from Ecosystem

| Pattern | Source | What Changed |
|---------|--------|-------------|
| **Typed IpcError** | wetSpring V125, neuralSpring V112 | `IpcError` enum with 7 variants replacing `Result<T, String>` across all IPC |
| **OrExit\<T\>** | groundSpring V112, wetSpring V123 | Trait on `Result`/`Option` for zero-boilerplate validation binary exits |
| **4-format capability parsing** | airSpring v0.8.7, rhizoCrypt S17 | `extract_capabilities()` handles flat, object array, nested, and double-nested formats |
| **health.liveness / health.readiness** | healthSpring V32, coralReef Iter 51 | Kubernetes-style probes registered as capabilities |
| **Resilient provenance trio** | healthSpring V32 | Circuit breaker (5s cooldown) + exponential backoff (50ms, 2 retries) |
| **DispatchOutcome** | groundSpring V112, petalTongue V166 | Ok / ProtocolError / ApplicationError RPC classification |
| **JSON-RPC proptest** | airSpring v0.8.7 | Property-based fuzz testing of `extract_rpc_result()` and capability parsing |
| **Zero-panic experiments** | ecosystem-wide | exp061-064 `.expect()` → `let Ok else exit(1)` |
| **Smart refactoring** | ecosystem pattern | exp047 domain model extracted (932 → 474 + 371 LOC) |
| **Python baseline provenance** | ecosystem standard | SHA-256 content hash, Python version enforcement |
| **deny.toml evolution** | toadStool S157b | `yanked = "deny"` for supply chain hardening |

### Quality Gates

| Gate | V23 | V24 |
|------|-----|-----|
| Tests (barracuda) | 394 | **450+** |
| IPC capabilities | 24 | **26** (+health probes) |
| IpcError variants | 0 (String) | **7** (typed) |
| `.expect()` in prod | ~25 | **0** |
| `#[allow()]` in prod | 0 | 0 |
| Unsafe blocks | 0 | 0 |
| Files over 1000 LOC | 0 | 0 |
| Proptest properties | 0 | **7+** |
| Capability formats parsed | 2 | **4** |
| Provenance trio resilience | None | **Circuit breaker + backoff** |

---

## 7. For Primals — How to Use ludoSpring

### toadStool
- Route `game.gpu.*` workloads through `compute.dispatch`
- ludoSpring's WGSL shaders are absorption candidates for barraCuda

### Squirrel
- ludoSpring provides structured context for NPC dialogue via `game.npc_dialogue`
- Engagement scoring validates AI output quality

### petalTongue
- `GameScene` DataBinding is designed for ludoSpring scene push
- Tufte validation ensures data-ink ratio compliance

### biomeOS
- `operation_dependencies` and `cost_estimates` enable Pathway Learner optimization
- 24 capabilities registered with semantic mappings

### rhizoCrypt / loamSpine / sweetGrass
- Game sessions are the richest DAG use case (high-frequency vertex creation)
- Certificate model proven for digital twin provenance
- Attribution model proven for collaborative creative works

---

*ScyBorg Provenance Trio: AGPL-3.0-or-later (code) + ORC (game mechanics) + CC-BY-SA 4.0 (creative content)*

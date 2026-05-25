# SPDX-License-Identifier: AGPL-3.0-or-later

# ludoSpring — Paper Queue & Science Thread Map

**Last updated:** May 25, 2026 (V79 — Wave 50 covalent HPC. All Priority 1 papers implemented. 982 tests. Thread 9+10 active.)

## Foundation Thread Assignment

ludoSpring is assigned to **Thread 9 (Gaming/Creative)** and **Thread 10
(Provenance/Economics)** in the unified lineage
(`sporeGarden/foundation/lineage/THREAD_INDEX.toml`).

## BaseCamp Paper Map

| Paper | Title / Theme | Status | Experiments |
|-------|---------------|--------|-------------|
| **17** | Game Design as Rigorous Science / HCI | Active — validated via interaction laws, flow, engagement | exp001–025, exp076–081 |
| **18** | RPGPT — Sovereign RPG with Provenance Trio | Active — exp067–075, specs/RPGPT_* | exp045–050, exp067–075, exp096 |
| **19** | Games@Home — Distributed Human Computation | Mapped — exp051 scaffolds pattern | exp051 |
| **20** | Ferment Transcript Economics / Radiating Attribution | Mapped — exp061, exp066 | exp061, exp066 |
| **21** | Sovereign Sample Provenance (field ↔ game same machinery) | Mapped — exp062 | exp062 |
| **22** | Zero-Knowledge Medical Provenance | Active — exp063, exp065 (foundation targets reference these) | exp063, exp065 |
| **24a** | Esoteric Webb Composition | Mapped — composition patterns inform esotericWebb | exp098–100 |
| **26** | Primal Composition as Methodology / BYOB Deploy Graphs | Active — ludoSpring as "pure composition" reference | exp082–094, exp099–100 |

## Papers Reviewed (Implemented)

### HCI / Interaction Science (Paper 17)
- Card, Moran & Newell (1983) — GOMS/KLM → `interaction/goms.rs`
- Fitts (1954) — Movement time law → `interaction/input_laws.rs`
- Hick (1952) — Choice reaction time → `interaction/input_laws.rs`
- Accot & Zhai (1997) — Steering law → `interaction/input_laws.rs`
- Csíkszentmihályi (1990) — Flow theory → `interaction/flow.rs`
- Tufte (1983, 2001) — Data-ink ratio, small multiples → `metrics/tufte_gaming.rs`
- Lazzaro (2004) — Four Keys to Fun → `metrics/fun_keys.rs`
- Hunicke, LeBlanc & Zubek (2004) — MDA Framework → `metrics/mda.rs`
- Yannakakis & Togelius (2011) — Player modeling → `metrics/engagement.rs`
- Bartle (1996) — Player Types → `metrics/player_types.rs`, `game/rpgpt/personality_dynamics.rs`
- Deterding et al. (2011) — Gamification → `metrics/gamification.rs`
- Schell (2008) — Game Design Lenses → `game/rpgpt/lenses.rs`

### Procedural Generation (Paper 17)
- Perlin (1985, 2002) — Noise functions → `procedural/noise.rs`
- Gumin (2016) — Wave Function Collapse → `procedural/wfc.rs`
- BSP (1969, game adaptation) — Space partitioning → `procedural/bsp.rs`
- Lindenmayer (1968) — L-systems → `procedural/lsystem.rs`

### Game History Revalidation (Paper 17)
- Pong (1972) → exp076
- Spacewar! (1962) → exp077
- Tetris (1984) → exp078
- Civilization (1991) → exp079
- Diablo (1996) — loot systems → exp080
- Procedural generation survey → exp081

## Papers NOT Yet Reviewed (Queue)

### Priority 1 — Blocks Near-Term Experiments
| Paper/Topic | Why | Target Experiment |
|-------------|-----|-------------------|
| ~~Hunicke, LeBlanc, Zubek (2004) — MDA Framework~~ | ~~Game design → mechanics → dynamics → aesthetics formalization~~ | **IMPLEMENTED V71** (`metrics/mda.rs`) |
| ~~Schell (2008) — Art of Game Design / Lenses~~ | ~~Lens-based validation against RPGPT plane system~~ | **IMPLEMENTED V76** (`game/rpgpt/lenses.rs`) |
| ~~Bartle (1996) — Player Types~~ | ~~NPC personality modeling (Paper 18)~~ | **IMPLEMENTED V75** (`metrics/player_types.rs`, `game/rpgpt/personality_dynamics.rs`) |
| ~~Deterding et al. (2011) — Gamification~~ | ~~Human computation economics (Paper 19)~~ | **IMPLEMENTED V75** (`metrics/gamification.rs`) |

### Priority 2 — GPU / Compute Papers
| Paper/Topic | Why | Target |
|-------------|-----|--------|
| Roofline model (Williams et al. 2009) | GPU throughput bound analysis for shader tiers | metalForge benchmarks |
| Kokkos (Edwards et al. 2014) | Performance portability baseline for barraCuda GPU parity | benchmark comparison |
| MLPerf (Mattson et al. 2020) | Inference benchmark standard for neuralSpring composition | exp083–087 |

### Priority 3 — Provenance / Economics (Papers 20-22)
| Paper/Topic | Why | Target |
|-------------|-----|--------|
| PROV-O (W3C 2013) | Standard provenance ontology — trio alignment check | exp062, exp063 |
| Merkle (1979) — Hash trees | DAG structure validation for rhizoCrypt | provenance trio |
| Blockchain provenance surveys | Anti-cheat + attribution economics | exp061, exp066 |

## Datasets Queue

### Game Science (Thread 9)
| Dataset | Source | Accession | Purpose |
|---------|--------|-----------|---------|
| OpenAI Gym environments | gymnasium.farama.org | N/A (code) | Baseline interaction cost models |
| Steam player telemetry (anonymized) | Publicly available summaries | N/A | Engagement curve validation |
| BoardGameGeek complexity ratings | boardgamegeek.com/browse | N/A | Quality discrimination baseline (exp040) |

### Provenance / Medical (Thread 10 / Paper 22)
| Dataset | Source | Accession | Purpose |
|---------|--------|-----------|---------|
| NCBI Gene Expression | GEO | Series accession via exp041 | QS gene provenance |
| FHIR R4 test resources | hl7.org/fhir | N/A | Medical provenance structure (exp063) |
| DrugBank open data | drugbank.ca | N/A | Cross-domain fraud baseline (exp065) |

### Benchmark Baselines (compute parity)
| Dataset/Standard | Source | Purpose |
|------------------|--------|---------|
| Kokkos kernels | github.com/kokkos | GPU compute parity reference |
| Galaxy bioinformatics workflows | usegalaxy.org | Pipeline composition comparison |
| DaCapo benchmarks | dacapobench.org | JVM → Rust throughput ratios |

## Industry Benchmark Gap Analysis

### What We Have
- **Criterion** (Rust): noise throughput, raycaster throughput, ECS tick (BM-001–003)
- **Python baseline**: `bench_cpu_parity.py` (perlin, fbm, raycaster, fitts/hick timing)
- **exp030**: CPU↔GPU parity validation (correctness, not throughput comparison vs industry)
- **metalForge**: Local hardware profiling (TFLOPS, bandwidth measurements)

### What's Missing
- **No Kokkos/CUDA baseline** — we validate GPU correctness but don't compare throughput vs established GPU frameworks
- **No MLPerf-style inference benchmark** — neuralSpring composition handles this; ludoSpring benefits when Squirrel routes to it
- **No roofline analysis** — theoretical GPU bounds vs achieved throughput not formally documented
- ~~**BM-004/BM-005**~~ IMPLEMENTED (V71) — `game/matchmaking.rs` (Elo, lobby formation) and `game/chat.rs` (message pipeline, rate limiting)

### Evolution Path
1. `bench_cpu_parity.py` provides Python timing → Rust Criterion gives speedup ratio
2. When coralReef ships, sovereign GPU dispatch → can benchmark vs wgpu dispatch
3. When toadStool integrates Kokkos-style portability layer → formal GPU parity benchmarks
4. neuralSpring exp094-style composition parity → inference throughput benchmarks

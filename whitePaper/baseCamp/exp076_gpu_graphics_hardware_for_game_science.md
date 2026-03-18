# exp076: GPU Graphics Hardware for Game Science

**Date**: March 17, 2026
**Status**: baseCamp — exploration direction, not yet validated
**Depends on**: barraCuda (compute dispatch), coralReef (shader compilation), toadStool (GPU access)
**Ecosystem guidance**: Pending — see hotSpring/coralReef GPU evolution handoffs in `ecoPrimals/wateringHole/handoffs/`

---

## Motivation

ludoSpring currently treats the GPU as a compute engine — WGSL compute
shaders dispatched through barraCuda for physics, pathfinding, fog of war,
and noise generation. But a GPU is not one computer. It is at least eight
distinct hardware units, most of which exist specifically for real-time
graphics and sit idle during compute dispatch.

The DF64 discovery (hotSpring/coralReef) proved that fp32 ALUs can emulate
fp64 at 8-16x the throughput of native fp64 on consumer GPUs. That was
finding a hidden double-precision computer inside the single-precision
hardware.

**This experiment asks**: What hidden computers are on the GPU die that
accelerate game science problems? Can the rasterizer, depth buffer, RT
cores, blending hardware, and texture units solve game problems faster than
equivalent compute shaders?

---

## The Eight Hidden Computers

| Unit | Graphics purpose | General computation | Game science application |
|------|-----------------|---------------------|-------------------------|
| Shader cores | Pixel math | Any float math | Already used (compute shaders) |
| fp32 ALUs | Single precision | DF64 double precision | Already proven (hotSpring) |
| Rasterizer | Triangle → pixels | Point-in-polygon, interpolation | Fog of war, visibility, terrain sampling |
| Depth buffer | Occlusion | Per-pixel min/max reduction | Pathfinding distance fields, Voronoi |
| ROPs / blending | Color compositing | Scatter-add accumulation | Influence maps, histogram, particle deposition |
| RT cores | Reflections | BVH spatial queries | Acoustic ray tracing, line-of-sight, NPC awareness |
| Texture units | Texture filtering | Free interpolation | Engagement curves, DDA lookup, terrain heightmap |
| Tessellation | Smooth surfaces | Adaptive mesh refinement | LOD for procedural terrain, dynamic map detail |

---

## Experiment Plan

### exp076a: Rasterizer for Fog of War

**Current approach**: `fog_of_war.wgsl` compute shader — for each tile,
cast rays from player position to check visibility. O(tiles * ray_steps).

**Proposed approach**: Render the occluder geometry (walls) as triangles.
The rasterizer determines which pixels (tiles) are behind walls. The
depth buffer naturally computes the nearest wall distance. Visibility is
the complement of occlusion.

**Hypothesis**: The rasterizer approach should be 10-50x faster for maps
with many walls because it processes all geometry in a single pass rather
than per-tile ray marching.

**Validation**: Compare fog-of-war output from compute shader vs rasterizer
approach. Pixel-exact match required. Measure throughput (maps/second) on
MI50 and Titan V.

**MI50 note**: The MI50 has a full rasterizer on die (GCN 5.0 = same as
Vega consumer cards). No display output, but it can render to a memory
buffer. The rasterized fog-of-war result is read back over PCIe.

### exp076b: Depth Buffer for Pathfinding Distance Fields

**Current approach**: `pathfind_wavefront.wgsl` compute shader — BFS
wavefront expansion from destination. O(width * height * max_distance).

**Proposed approach**: Render a cone at the destination point. The depth
buffer computes the Euclidean distance from the destination to every pixel.
For obstacle avoidance, render obstacle geometry as walls that block the
cone. The depth buffer's min-reduction naturally routes around obstacles
(closest visible surface = shortest unobstructed path hint).

**Limitation**: The depth buffer gives Euclidean distance, not path
distance. This is a heuristic (admissible for A*), not an exact
shortest path. Useful as a precomputed heuristic map that A* consults.

**Hypothesis**: Hardware distance field generation should be 100x+ faster
than wavefront BFS for large maps, producing an admissible A* heuristic.

**Validation**: Generate distance fields both ways. Verify that the
depth-buffer version produces valid A* heuristics (never overestimates true
path distance). Measure map generation throughput.

### exp076c: Alpha Blending for Influence Maps

**Current approach**: No GPU influence map implementation. CPU-only in
entity.rs — iterate entities, accumulate influence per tile. O(entities * radius^2).

**Proposed approach**: Render each entity as a screen-aligned quad
(sprite) with alpha = influence falloff. Additive blending accumulates
overlapping influence. One draw call per entity type (instanced rendering)
or one compute pass that writes to a render target with additive blend.

**Hypothesis**: Hardware additive blending handles overlapping influence
radii without atomics or synchronization. Should scale linearly with
entity count regardless of overlap density.

**Validation**: Compare influence maps from CPU loop vs blending approach.
Tolerance: `GAME_STATE_TOL` (0.01). Measure throughput at 1K, 10K, 100K
entities.

### exp076d: RT Cores for Acoustic Ray Tracing

**Current approach**: `audio.rs` produces `NarrationCue` and `SoundEffect`
tags. No spatial audio propagation model — petalTongue receives tags and
renders them without geometry-aware attenuation or reflection.

**Proposed approach**: Build a BVH over the game map's wall geometry. For
each sound source, cast rays in all directions. RT cores compute
intersections with walls. Reflected rays propagate further. The result is
a per-tile audio energy map that encodes occlusion, reflection, and
distance attenuation.

**Hardware requirement**: RT cores require RTX 2000+ (NVIDIA) or RDNA 2+
(AMD). Not available on MI50 or Titan V. Fallback: compute shader BVH
traversal (slower but functional on any GPU).

**Hypothesis**: RT core acoustic propagation at 1000+ rays per sound source
should run in <1ms per frame, enabling real-time spatial audio for the
RPGPT dialogue plane (exp067-075). NPC voices attenuate behind walls, echo
in large rooms, muffle through doors.

**Validation**: Compare RT core acoustic model against an analytical
solution for simple geometries (rectangular room, single source). Verify
inverse-square falloff, wall reflection, and shadow zones. Measure
rays/second.

### exp076e: Texture Units for Engagement Curve Lookup

**Current approach**: Engagement metrics (Fitts's law, Hick's law, Flow
channel) are computed analytically per tick. Each involves transcendentals
(log2 for Hick, division for Fitts, exp for DDA).

**Proposed approach**: Precompute engagement curves into 1D textures. For
Fitts: `texture[distance * width_scale] = movement_time`. For Hick:
`texture[num_choices] = decision_time`. For DDA difficulty response:
`texture[performance_percentile] = difficulty_adjustment`. The TMU returns
the interpolated value in one cycle.

**Hypothesis**: For batch evaluation (1000+ entities per tick), texture
lookup should be 5-20x faster than analytical computation because the TMU
pipeline runs independently of the ALUs.

**Validation**: Compare texture-lookup results against analytical results.
Tolerance: `ENGAGEMENT_TOL` from `tolerances/engagement.rs`. Measure
throughput in evaluations/second.

### exp076f: Compute-Shader Rendering (Software Rasterization)

**Motivation**: Close the "render gap" — produce actual game frames on any
GPU (including headless MI50) using only compute dispatch.

**Approach**: Write a WGSL compute shader that:
1. Takes entity positions, tile map, camera position as input buffers
2. For each output pixel, casts a ray into the 2D tile map (DDA raycaster,
   already validated in exp036 at 6,623 FPS CPU)
3. Computes wall height, distance shading, entity sprites
4. Writes RGBA8 to an output buffer

This bypasses the hardware graphics pipeline entirely. The output buffer
is a framebuffer that toadStool can display via DRM, `/dev/fb0`, or
stream via VNC.

**Hypothesis**: The existing DDA raycaster (exp036) at 6,623 FPS on CPU
should run at 100,000+ FPS equivalent on GPU compute (MI50: 3840 CUs).
Even at 1920x1080, budget is 2M pixels / 3840 CUs = 520 pixels per CU
per frame. At 60 FPS that's 31,200 pixels/CU/second — trivially achievable.

**Validation**: Produce a 1920x1080 RGBA8 framebuffer from a tile map.
Compare pixel-by-pixel against CPU raycaster output. Measure frame time
on MI50, Titan V, RTX 3090. Target: 60 FPS at 1080p on MI50.

**This experiment directly closes the ecosystem's rendering gap.**

---

## fp64 for Games: Where and Why

Games traditionally use fp32. ludoSpring's game science uses fp64 because:

1. **Validation fidelity**: Python baselines use fp64. Matching them
   requires fp64 computation even if the result is displayed in fp32.

2. **Large worlds**: fp32 precision at 10km from origin is ~1mm. For an
   RPGPT world that spans continents, this causes vertex jitter. fp64
   handles solar-system scale.

3. **Physics accuracy**: Orbital mechanics (exp044 Anderson model),
   fluid dynamics, cloth simulation with tight tolerances.

4. **Scientific games**: Games@Home (exp051) positions gameplay as
   distributed computation. If the computation needs fp64, the game
   needs fp64.

**Split precision strategy**:
- Physics: fp64 (barraCuda, native on MI50/Titan V, DF64 on consumer)
- Rendering: fp32 (all GPUs, fast)
- Results cast from f64 to f32 at the render boundary

**Unexplored**: What if some rendering operations benefit from higher
precision? Anti-aliasing edge equations, shadow map depth comparison,
and screen-space ambient occlusion all involve subtracting nearly-equal
values (the same catastrophic cancellation pattern hotSpring found in
the Mermin dielectric). DF64 rendering might produce visibly better
anti-aliasing at moderate cost.

---

## Hardware Matrix

| GPU | Shader cores | Rasterizer | Depth buf | Blend | RT cores | TMU | Tessellation | fp64 rate | Display |
|-----|-------------|-----------|-----------|-------|----------|-----|-------------|-----------|---------|
| MI50 | 3840 CU | Yes | Yes | Yes | No | Yes | Yes | 1/2 (native) | No |
| Titan V | 5120 SM | Yes | Yes | Yes | No | Yes | Yes | 1/2 (native) | Yes |
| RTX 3090 | 10496 SM | Yes | Yes | Yes | Yes (2nd gen) | Yes | Yes | 1/64 | Yes |
| RX 6950 XT | 5120 CU | Yes | Yes | Yes | Yes (RDNA 2) | Yes | Yes | 1/16 | Yes |

Key: **MI50 has every fixed-function unit except RT cores and display.**
It can run all proposed experiments except exp076d (RT acoustic).

---

## Success Criteria

Each sub-experiment produces:

1. A WGSL shader (or graphics pipeline configuration) that uses the
   target hardware unit
2. A CPU/compute-shader baseline for comparison
3. Pixel-exact or tolerance-documented comparison of results
4. Throughput measurement (operations/second or frames/second)
5. A clear mapping: `game_operation → hardware_unit → speedup_factor`

Successful experiments hand off to barraCuda as new dispatch ops:
- `math.spatial.fog_rasterizer` (exp076a)
- `math.spatial.distance_field_zbuffer` (exp076b)
- `math.spatial.influence_blend` (exp076c)
- `math.spatial.acoustic_rt` (exp076d)
- `math.lookup.engagement_texture` (exp076e)
- `render.compute.raycaster` (exp076f)

---

## exp076g: Infrastructure Portability — Same Math, Every Hardware Unit

**Motivation**: We made math portable from CPU to GPU (Python → Rust → WGSL).
Now make it portable across infrastructure on the card. Run the same game
science operations on shader cores, tensor cores, RT cores, TMUs, and ROPs.
Even the "wrong" hardware placements teach us the bounds.

**Approach**: Take three well-understood ludoSpring operations and run each
on every available hardware unit:

### Target 1: Engagement batch evaluation (exp030 baseline)

Currently a compute shader (`engagement_batch.wgsl`). Each invocation
evaluates Fitts/Hick/Flow for one entity.

| Hardware unit | Reformulation | Expected result |
|---------------|--------------|-----------------|
| Shader cores | Current WGSL compute (baseline) | Known: validated in exp030 |
| Tensor cores | Batch entities as matrix rows, engagement params as columns, MMA | 2-4x throughput at FP16; measure precision loss vs `ENGAGEMENT_TOL` |
| TMU | Precomputed engagement curves as 1D textures | 5-20x per lookup; precision bounded by texture resolution |
| ROPs | N/A (not an accumulation problem) | — |
| RT cores | N/A (not a spatial problem) | — |
| CPU | Rust baseline (already exists) | Reference for correctness |

### Target 2: Fog of war (exp076a baseline)

Currently `fog_of_war.wgsl` compute shader. Per-tile visibility from
player through wall geometry.

| Hardware unit | Reformulation | Expected result |
|---------------|--------------|-----------------|
| Shader cores | Current WGSL ray-march (baseline) | Known |
| Rasterizer | Wall triangles → depth buffer → visibility mask | 10-50x |
| RT cores | Ray from player to each tile center, walls as BVH | 10-100x; natural fit |
| Tensor cores | Visibility as matrix (player×tile×wall occlusion) | Unknown — explore bounds |
| TMU | Precomputed visibility tables per room? | Only for static geometry |
| ROPs | N/A | — |

### Target 3: Pairwise entity interaction (influence maps)

CPU loop over entity pairs. No GPU implementation yet.

| Hardware unit | Reformulation | Expected result |
|---------------|--------------|-----------------|
| Shader cores | Compute shader, one thread per entity pair | Baseline GPU |
| Tensor cores | Distance matrix: `diag(A^T A) + diag(B^T B) - 2A^T B` | 2-4x for N > 64 entities |
| ROPs | Entity quads with additive blending | 5-10x, no atomics needed |
| RT cores | BVH over entities, range query per entity | 10x for sparse influence |
| TMU | Influence falloff as texture | Free interpolation |
| Rasterizer | Entity as triangle at tile position | Spatial binning for free |

### Validation

For each (operation, hardware_unit) pair:
1. Compare output against CPU reference
2. Document precision: exact match, or tolerance with justification
3. Measure throughput: operations/second
4. Record in a **performance surface table**: operation × unit × precision × throughput

The performance surface is the deliverable. It tells barraCuda's dispatch
router where to place each operation for maximum throughput at acceptable
precision. ludoSpring's game operations are simple enough to be good test
cases, but the results generalize to every spring.

---

## Connection to Ecosystem

This exploration directly supports:

- **petalTongue**: exp076f produces framebuffers that close the
  `display.present` gap — compute-rendered frames bypass the need for
  graphics pipeline wiring in toadStool
- **barraCuda**: New dispatch ops from successful experiments
- **coralReef**: If graphics pipeline states (blend modes, depth functions)
  are needed, coralReef learns to emit them — extending sovereign dispatch
  beyond compute
- **toadStool**: Buffer readback from rendering experiments motivates
  closing the readback gap
- **primalSpring**: Cross-primal composition patterns for
  compute+graphics hybrid dispatch

---

## Faculty Anchors

- Dekker (1971), Knuth (1997) — Double-float arithmetic (DF64 pattern)
- Carr & Hart (2002) — GPU ray casting via fragment programs
- Purcell et al. (2002) — "Ray Tracing on Programmable Graphics Hardware"
  (pioneered using graphics hardware for non-graphics computation)
- Owens et al. (2007) — "A Survey of General-Purpose Computation on
  Graphics Hardware" (GPGPU survey documenting early repurposing)
- Hoff et al. (1999) — "Fast Computation of Generalized Voronoi Diagrams
  Using Graphics Hardware" (the Z-buffer Voronoi trick)
- Parker et al. (2010) — "OptiX: A General Purpose Ray Tracing Engine"
  (RT cores for non-rendering)
- Wald et al. (2019) — "RTX Beyond Ray Tracing" (RT cores for science)

---

**License**: AGPL-3.0-or-later

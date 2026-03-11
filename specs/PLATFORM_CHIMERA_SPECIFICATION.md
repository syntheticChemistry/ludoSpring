# Platform Chimera Specification

**Date**: March 10, 2026
**Status**: Draft
**License**: AGPL-3.0-or-later

---

## Purpose

We're not here to replace Steam. We're here to make Steam eternal.

Steam is a potential ally — perhaps our strongest one. Valve's 30% cut is
high because they provide a genuinely high-quality service, equitably, to
an enormous community. They curate, they host, they review, they protect
players from malware and scams, and they've built the infrastructure that
lets indie developers reach millions. Steam's ban on crypto games wasn't
anti-innovation — it was wisdom. They saw through speculative noise to
protect what gaming is actually about.

The insight: if every Steam user's NestGate also served downloads via
Songbird federation, Steam's infrastructure becomes its userbase. The
30% doesn't disappear — it gets redistributed. Valve keeps curating
and discovering. The community handles distribution. Creators keep more.
Players get sovereign ownership via Loam certificates. Everyone wins.

This specification decomposes platform capabilities into sovereign chimera
patterns. Where a platform does good work (Steam's curation, review
system, Workshop), we study and federate. Where platforms extract value
through dark patterns, we replace with science.

See also:
- `whitePaper/outreach/AN_INVITATION_TO_VALVE.md` — partnership framing
- `whitePaper/economics/SUNCLOUD_ECONOMIC_MODEL.md` — Radiating Attribution
- `whitePaper/economics/LATENT_VALUE_ECONOMY.md` — value from significance

---

## Chimera Architecture

A platform chimera is an **orchestrated chimera** — multiple primals
running as separate processes, coordinated by a biomeOS graph. Each
chimera composes existing primal capabilities; no chimera introduces
new primal code.

```
Platform Chimera = biomeOS Graph + Primal Capabilities + Bond Configuration
```

Chimeras are defined as YAML in `biomeOS/chimeras/definitions/` and
deployed via `biomeos deploy chimeras/<name>.yaml`.

---

## Steam: Alliance, Not Replacement

Steam solved distribution. The revolution isn't finished — but the path
forward is partnership, not competition. See `AN_INVITATION_TO_VALVE.md`.

### What Steam Does Well (study, federate, enhance)

- **Curation and discovery** — Valve's human review and algorithmic
  discovery surface great games. We study this, not replace it.
- **Workshop** — Creator ecosystem that has fostered modding communities
  for decades. We federate it so mods survive if a game's servers don't.
- **Review system** — Community-driven quality signal. We enhance it with
  engagement science (ludoSpring flow/engagement metrics vs upvote counts).
- **Anti-cheat (VAC)** — Protects competitive integrity. We complement
  with BearDog cryptographic action signing.
- **Indie platform** — Steam has been the single greatest force growing
  indie games. Stardew Valley, Factorio, Undertale, Hollow Knight,
  Balatro — billion-dollar-corp-quality games from small teams, reaching
  millions because Steam treated them equitably alongside AAA.

### What We Federate (sovereign infrastructure under Steam's curation)

| Steam Feature | Sovereign Layer | Bond to Steam |
|---------------|----------------|---------------|
| **Library** | NestGate sovereign collection | Ionic (API integration) |
| **Downloads** | NestGate Plasmodium (userbase serves content) | Covalent (community CDN) |
| **Ownership** | Loam Certificate (BearDog + LoamSpine) | Ionic (Steam verifies, Loam anchors) |
| **Saves** | NestGate cross-device replication | Covalent (your data) |
| **Workshop** | NestGate federated mod hosting | Ionic (Steam indexes, community hosts) |
| **Matchmaking** | Songbird capability routing | Ionic or Covalent |
| **Multiplayer** | Songbird relay + BearDog E2E | Covalent (encrypted mesh) |
| **Friends** | Beacon genetics address book | Covalent (genetic trust) |
| **Chat** | Songbird BirdSong protocol | Covalent (E2E) |
| **Achievements** | ludoSpring engagement + NestGate | Covalent (provenance-backed) |
| **Leaderboards** | ludoSpring + Plasmodium aggregation | Metallic (cross-gate) |
| **Reviews** | Songbird federation + ludoSpring metrics | Ionic |
| **Anti-cheat** | BearDog action signing | Nuclear force |
| **Overlay** | petalTongue multi-modal renderer | Internal |

### The Distribution Revolution

The key economic insight: Steam's 30% covers infrastructure (bandwidth,
storage, CDN) plus curation (review, discovery, anti-fraud). If every
Steam user's NestGate also serves content via Songbird federation:

```
Today:  Valve CDN serves 30M concurrent users → Valve pays bandwidth
Future: 30M NestGate nodes serve each other → Community pays bandwidth
        Valve curates, discovers, reviews     → Valve earns curation fee
        Creators keep more                    → Revenue split improves
        Players own via Loam certificates     → True digital ownership
```

This is the Latent Value Economy applied to distribution: the hardware
is already in users' homes. We unlock it with trust (BearDog, sweetGrass)
and federated storage (NestGate, Songbird).

### Storefront Chimera

```yaml
chimera:
  id: "sovereign-store"
  description: |
    Content-addressed game distribution via NestGate federation.
    Creators publish directly. Players discover via Songbird. No
    intermediary takes a cut. Ownership is cryptographic (BearDog).

  components:
    songbird:
      modules: [federation, discovery]
    nestgate:
      modules: [storage, content_addressing, delta_sync]
    beardog:
      modules: [ownership_proof, content_signing]

  fusion:
    publish_flow:
      # Creator signs content → NestGate stores → Songbird announces
      provider: "beardog.content_signing"
      pipeline: ["nestgate.storage.store", "songbird.federation.announce"]
    discover_flow:
      # Player queries → Songbird routes → NestGate serves
      provider: "songbird.discovery.query"
      pipeline: ["nestgate.storage.retrieve"]
    ownership:
      # BearDog proves you own it — no phone-home DRM
      provider: "beardog.ownership_proof"
      verification: "local_only"  # Never needs network to play
```

**Open system benchmark**: Modrinth (AGPL-3.0) API and distribution model.
**Evolution path**: Study Modrinth → scaffold Songbird+NestGate → benchmark
→ pure Rust sovereign store.

### Workshop Chimera

```yaml
chimera:
  id: "sovereign-workshop"
  description: |
    Federated mod hosting. Creators publish to their own NestGate.
    Community discovers via Songbird federation. No central authority
    can remove content. Provenance tracked via BearDog lineage.

  components:
    songbird:
      modules: [federation, search]
    nestgate:
      modules: [storage, versioning, provenance]
    beardog:
      modules: [content_signing, lineage_proof]
    ludospring:
      modules: [metrics.engagement]
      # ludoSpring scores mod quality via engagement metrics

  fusion:
    mod_publish:
      provider: "beardog.content_signing"
      pipeline: ["nestgate.storage.store", "songbird.federation.announce"]
    mod_discover:
      provider: "songbird.search"
      ranking: "ludospring.metrics.engagement"  # Engagement-based, not ad-based
```

**Open system benchmark**: Modrinth + Thunderstore.

---

## Discord Decomposition

### Capability Map

| Discord Feature | Primal | Capability | Bond Type |
|-----------------|--------|-----------|-----------|
| **Text channels** | Songbird | BirdSong E2E encrypted rooms | Covalent |
| **Voice channels** | petalTongue + Songbird | Spatial audio + relay | Covalent |
| **Servers** | Songbird | Federation rooms (owned by creator) | Covalent (your server) |
| **Roles / permissions** | BearDog | Genetic lineage hierarchy | Nuclear force |
| **Friends** | Beacon genetics | `.beacon.seed` address book, meetings | Covalent |
| **Direct messages** | Songbird | BirdSong point-to-point E2E | Covalent |
| **Rich presence** | ludoSpring + Songbird | `game.engagement` → `discovery.announce` | Covalent broadcast |
| **Bots** | biomeOS | Deploy bot as primal in graph | Ionic (API contract) |
| **Screen share** | petalTongue + Songbird | Capture + WebRTC relay | Covalent |
| **Threads** | Songbird | Nested federation rooms | Covalent |
| **Emojis / stickers** | NestGate | Content-addressed media storage | Covalent |
| **Nitro / premium** | NONE | Sovereignty: no artificial feature gates | — |

### Social Chimera

```yaml
chimera:
  id: "sovereign-social"
  description: |
    Federated real-time communication. Text, voice, and video in
    encrypted rooms. You own your server. Friends are genetic trust.
    No surveillance, no ads, no premium tiers.

  components:
    songbird:
      modules: [birdsong_mesh, federation, relay]
      array:
        enabled: true
        min: 1
        max: 256  # Scale rooms
    beardog:
      modules: [btsp, genetic_roles]
    petaltongue:
      modules: [spatial_audio, capture]
    nestgate:
      modules: [message_persistence, media_storage]
    ludospring:
      modules: [metrics.engagement]
      # Flow-aware presence: show when someone is deeply engaged vs idle

  fusion:
    text_channel:
      transport: "songbird.birdsong_mesh"
      encryption: "beardog.btsp"
      persistence: "nestgate.message_persistence"
    voice_channel:
      capture: "petaltongue.spatial_audio"
      transport: "songbird.relay"
      encryption: "beardog.btsp"
    presence:
      provider: "ludospring.metrics.engagement"
      consumer: "songbird.federation.announce"
      # Rich presence based on actual engagement, not self-reported status
    roles:
      provider: "beardog.genetic_roles"
      # Family seed hierarchy = role hierarchy. No centralized role server.
```

**Open system benchmarks**:
- Text: Matrix/Conduit (Apache-2.0, Rust)
- Voice: Mumble (BSD-3)
- Full platform: Revolt (AGPL-3.0)

---

## Anti-Patterns: What We Replace

The anti-patterns are not "platforms charging money." Valve earns its
cut. The anti-patterns are the extractive mechanics that have corroded
gaming from the inside — the ones that degrade craft, exploit players,
and produce watered-down IP because creative ownership has been stripped
from the people who actually make things.

This is fundamentally a labor problem: when the crafters don't own what
they create, corporations optimize for extraction, not quality. The
result is enshittification — Marxist labor theory playing out in real
time across the games industry.

### The Three Anti-Patterns

**1. Microtransactions and predatory monetization**

The shift from "buy a complete game" to "buy a shell, sell the rest in
pieces" degrades design. Every gameplay system becomes a monetization
surface. Progression is designed to frustrate, not to engage. The $70
game that also sells a $20 battle pass and $5 skins is not providing
value — it's extracting rent from captured players.

ludoSpring's alternative: engagement science measures genuine flow and
satisfaction. If a monetization mechanic reduces flow state, it's
measurably bad design. Tufte analysis catches when UI becomes ad surface.

**2. Gambling mechanics and addiction exploitation**

Loot boxes are slot machines. Gacha is gambling. "Surprise mechanics"
is PR for psychological exploitation. These systems are designed by
behavioral psychologists to create compulsion loops, not fun. They
target the vulnerable — children, people with addictive tendencies —
and extract maximum revenue from "whales" who are often people in crisis.

ludoSpring's alternative: DDA and flow science optimize for the
Csikszentmihalyi channel — genuine engagement, not compulsion. Our
engagement metrics distinguish between flow (good) and addiction loops
(extractive). Accessibility scoring penalizes dark patterns.

**3. Low-quality churn replacing craft**

When creative ownership is stripped from developers and held by
publishers, the incentive is volume over quality. Annual franchises
with minimal innovation. Live-service games that ship broken and demand
patience. Asset flips flooding storefronts. The craft is diluted because
the crafters have no stake in the outcome.

Meanwhile, indie games have been kicking the ass of billion-dollar
corporations. Stardew Valley (one person) outsells most AAA farming
games. Factorio, Hollow Knight, Undertale, Balatro, Hades — small teams
producing work that puts hundred-million-dollar productions to shame.
Steam has been instrumental in this: by treating indies equitably on the
storefront, Valve let quality compete with marketing budgets.

ludoSpring's alternative: Radiating Attribution (sunCloud) ensures
creators retain proportional share of all future value. Loam certificates
anchor creative provenance. The crafter owns what they craft.

---

## Engagement Intelligence

ludoSpring's unique contribution to platform chimeras is **engagement
intelligence** — every chimera that involves human interaction benefits
from ludoSpring's validated models:

| Chimera | ludoSpring Provides |
|---------|-------------------|
| Storefront | Engagement-based discovery ranking (not ad-based) |
| Workshop | Mod quality scoring via player engagement metrics |
| Social | Flow-aware presence (engaged vs idle vs frustrated) |
| Gaming-Mesh | Skill assessment for matchmaking via DDA history |
| Streaming | Real-time engagement overlay (Tufte-scored) |
| Anti-cheat | Engagement anomaly detection (bot vs human patterns) |

This is the key differentiator: engagement is measured scientifically
(Csikszentmihalyi, Yannakakis) and used to *help* users, not to
*exploit* them. When ludoSpring detects a player is in a compulsion loop
rather than a flow state, the system surfaces that information — it
doesn't hide it to maximize spend.

---

## Evolution Strategy

### Phase 1: Study (current)

Read open system source code. Document architectures. Define benchmarks.
No scaffold dependencies yet.

### Phase 2: Scaffold

Use open systems as subprocesses or test oracles. ludoSpring+biomeOS
talks to Nakama/Conduit/Revolt via JSON-RPC adapters. Measure performance.

### Phase 3: Implement

Build pure Rust equivalents within primal boundaries. Each capability
lands in the correct primal (Songbird for networking, NestGate for
storage, ludoSpring for game science).

### Phase 4: Validate

Benchmark pure Rust against scaffold. Must meet or exceed latency,
throughput, and correctness targets.

### Phase 5: Compose

Deploy chimera graphs that compose primals into platform capabilities.
Validate that composition overhead stays within frame budget.

### Phase 6: Shed

Remove scaffold dependencies. Open systems become test oracles only.
Platform is sovereign Rust end-to-end.

---

## Principles

1. **You own your library** — Loam certificates, no phone-home DRM
2. **You own your server** — your NestGate, your rules, your moderation
3. **You own your identity** — BearDog genetic lineage, not email/password
4. **You own your data** — NestGate exports everything, always
5. **Crafters own their craft** — Radiating Attribution, not work-for-hire extraction
6. **Curation earns its keep** — platform fees for real service (Steam model), not rent-seeking
7. **Engagement helps, never exploits** — flow science, not compulsion loops
8. **Quality over churn** — indie craft over annual franchise extraction
9. **Federation, not centralization** — Songbird mesh, community CDN
10. **Sovereignty at every layer** — pure Rust, AGPL, no proprietary deps

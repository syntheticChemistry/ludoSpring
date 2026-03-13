# Lysogeny Catalog — Open Recreation of Proprietary Game Mechanics

**Date**: March 13, 2026
**License**: AGPL-3.0-or-later
**Strategy**: Identify proprietary game mechanics, trace the underlying math to
published open research (prior art), recreate from first principles, cross-validate
in biology/ecology to prove generality, release under AGPL-3.0 copyleft.

---

## Three Layers of Protection

1. **Prior art**: The math existed in ecology/population biology before the game
   mechanic was patented. Every equation traces to a published paper with citation.
2. **Cross-domain utility**: The same math works in microbial ecology, population
   genetics, and clinical science — proving it is general mathematics, not a
   specific game system.
3. **Independent derivation**: Implementation derives from published research, not
   from reverse-engineering any proprietary game. No proprietary code, assets, or
   documentation is referenced during implementation.

---

## Target 1: Usurper (Nemesis System)

**Proprietary gate**: Warner Bros US 9,573,066 B2 (filed 2015) — persistent
adaptive NPC hierarchy in "Middle-earth: Shadow of Mordor"

### Core Mechanics

- Persistent NPCs with memory of player encounters
- Adaptation: NPC gains strengths/weaknesses from encounter outcomes
- Social hierarchy: promotion, demotion, betrayal among NPCs
- Emergent narratives from NPC interaction chains

### Open Math (Pre-Patent Publication)

| Model | Citation | Year | Contribution |
|-------|----------|------|-------------|
| Frequency-dependent selection | Fisher, R.A. *The Genetical Theory of Natural Selection* | 1930 | Fitness depends on encounter history |
| Evolutionary stable strategies | Maynard Smith, J. *Evolution and the Theory of Games* | 1982 | Strategy adaptation from encounter outcomes |
| Replicator dynamics | Taylor, P.D. & Jonker, L.B. *Math. Biosci.* 40:145-156 | 1978 | `dx_i/dt = x_i * (f_i - f_bar)` — strategy frequency evolution |
| Spatial prisoner's dilemma | Nowak, M.A. & May, R.M. *Nature* 359:826-829 | 1992 | Local interactions determine hierarchy |
| Lotka-Volterra with delay | Leslie, P.H. *Biometrika* 35:213-245 | 1948 | Predator-prey with memory/time-delay |
| Cooperation/cheater dynamics | Bruger, E.L. & Waters, C.M. *mBio* 9(1):e01916-17 | 2018 | Cooperator/cheater with persistent phenotype switching |
| Persister cells | Balaban, N.Q. et al. *Science* 305:1622-1625 | 2004 | Bacterial "memory" of antibiotic exposure |

### Cross-Domain Mapping

| Game Concept | Biological Equivalent |
|-------------|----------------------|
| Orc captain | Bacterial strain / subpopulation |
| Player encounter | Antibiotic exposure / environmental stress |
| Gains fire weakness | Acquires sensitivity mutation |
| Promoted to war chief | Becomes dominant strain in population |
| Betrays leader | Horizontal gene transfer disrupts hierarchy |
| Remembers player | Persister cell phenotype — survives stress, retains memory |

### Alternative Use Case

Microbial ecology monitoring: track bacterial populations that adapt to
environmental stress (antibiotics, temperature, pH), gain resistance, reorganize
hierarchical community structure. Same math, different vocabulary.

### Experiment

`ludoSpring/experiments/exp055_usurper/` — ~50 validation checks

---

## Target 2: Integrase (Capture/Bonding Mechanics)

**Proprietary gate**: Pokemon capture probability formula, Persona social links,
Shin Megami Tensei demon negotiation, monster taming systems

### Core Mechanics

- Probability-based entity acquisition (capture rate)
- Persistent captured entities with stat growth
- Type matchup matrix (extended rock-paper-scissors)
- Evolution: state transitions at thresholds

### Open Math

| Model | Citation | Year | Contribution |
|-------|----------|------|-------------|
| Wright-Fisher model | Wright, S. *Genetics* 16:97-159 | 1931 | Probability-based fixation = capture probability |
| Quorum sensing threshold | Waters, C.M. & Bassler, B.L. *Annu. Rev. Cell Dev. Biol.* 21:319-346 | 2005 | Signal above threshold → irreversible state change |
| Competitive exclusion | Gause, G.F. *The Struggle for Existence* | 1934 | Niche partitioning = type matchup |
| Encounter rate theory | Lotka, A.J. *Elements of Physical Biology* | 1925 | Predator-prey encounter dynamics |
| Markov chains | Markov, A.A. *Bull. Acad. Imp. Sci.* | 1906 | State transitions as absorbing Markov chain |
| Phage integration | Campbell, A.M. *Adv. Genet.* 11:101-145 | 1962 | Lysogeny: phage integrates DNA into host = capture |

### Cross-Domain Mapping

| Game Concept | Biological Equivalent |
|-------------|----------------------|
| Wild Pokemon | Planktonic bacterium / free phage |
| Pokeball throw | Phage adsorption to receptor |
| Capture rate | Multiplicity of infection (MOI) |
| Captured state | Prophage (integrated DNA) |
| Evolution | Induction: prophage → lytic cycle |
| Type matchup | Receptor specificity / gram classification |
| Stat growth | Gene expression from integrated element |

### Alternative Use Case

Sample acquisition in field genomics: probability-based sampling strategy,
persistent specimens with growth/degradation curves, type classification
(gram+/gram-, aerobe/anaerobe), state transitions (viable → degraded → sequenced).

### Experiment

`ludoSpring/experiments/exp056_integrase/` — ~40 validation checks

---

## Target 3: Symbiont (Faction/Reputation Systems)

**Proprietary gate**: Elder Scrolls faction reputation, Fallout karma, WoW
faction grinding, Civilization diplomacy

### Core Mechanics

- Standing with multiple groups (multi-dimensional reputation vector)
- Actions shift reputation (positive/negative based on faction relationships)
- Reputation unlocks/locks capabilities (tiered access)
- Factions have relationships with each other (alliance, rivalry, neutral)

### Open Math

| Model | Citation | Year | Contribution |
|-------|----------|------|-------------|
| Multi-species Lotka-Volterra | Lotka, A.J. *J. Washington Acad. Sci.* 22:461-469 | 1932 | Competition coefficients = faction interaction matrix |
| Spatial prisoner's dilemma | Nowak, M.A. & May, R.M. *Nature* 359:826-829 | 1992 | Local interaction payoff = reputation from actions |
| Frequency-dependent fitness | Maynard Smith, J. *Evolution and the Theory of Games* | 1982 | Fitness depends on community composition |
| Mutualism-parasitism continuum | Bronstein, J.L. *Ecology* 82:1509-1520 | 2001 | Continuous axis from cooperative to exploitative |
| Keystone species | Paine, R.T. *Am. Nat.* 100:65-75 | 1966 | Community influence disproportionate to abundance |

### Cross-Domain Mapping

| Game Concept | Biological Equivalent |
|-------------|----------------------|
| Faction | Bacterial species / guild |
| Reputation score | Fitness contribution to community |
| Helpful action | Metabolite cross-feeding |
| Hostile action | Competitive exclusion / toxin production |
| Alliance | Syntrophy / obligate mutualism |
| Rivalry | Resource competition / allelopathy |
| Keystone faction | Keystone species (Paine 1966) |

### Alternative Use Case

Multi-lab collaboration tracking: researcher reputation as contribution history,
lab relationships as collaboration/competition matrix, tiered access to shared
resources based on contribution standing.

### Experiment

`ludoSpring/experiments/exp057_symbiont/` — ~35 validation checks

---

## Target 4: Conjugant (Roguelite Meta-Progression)

**Proprietary gate**: Hades mirror upgrades, Dead Cells mutation system, Rogue
Legacy inheritance, Returnal persistence

### Core Mechanics

- Persistent resources across failed runs (death grants permanent currency)
- Unlock tree: permanent capability expansion
- Each run generates information that improves future runs
- Difficulty scaling with meta-progression

### Open Math

| Model | Citation | Year | Contribution |
|-------|----------|------|-------------|
| Bacterial conjugation | Lederberg, J. & Tatum, E.L. *Nature* 158:558 | 1946 | Horizontal gene transfer from dead to living cells |
| LTEE | Lenski, R.E. et al. *Am. Nat.* 138:1315-1341 | 1991 | Accumulated beneficial mutations over 60K generations |
| Price equation | Price, G.R. *Nature* 227:520-521 | 1970 | Information accumulation in evolving populations |
| Wright-Fisher with selection | Wright, S. *Genetics* 16:97-159 | 1931 | Probability of fixation with beneficial mutations |
| Red Queen hypothesis | Van Valen, L. *Evol. Theory* 1:1-30 | 1973 | Co-evolutionary arms race = difficulty scaling |

### Cross-Domain Mapping

| Game Concept | Biological Equivalent |
|-------------|----------------------|
| Roguelite run | Bacterial generation |
| Player death | Cell death releasing DNA |
| Permanent upgrade | Horizontally transferred gene |
| Unlock tree | Accumulated beneficial mutations (LTEE citrate) |
| Meta-currency | Free DNA in environment |
| Difficulty scaling | Antibiotic dose escalation (Red Queen) |
| New Game+ | Next selective sweep |

### Alternative Use Case

Iterative experimental design: each failed experiment protocol releases
"information" (negative results). Next protocol incorporates lessons. The
laboratory "meta-progresses" toward successful results across failed attempts.

### Experiment

`ludoSpring/experiments/exp058_conjugant/` — ~40 validation checks

---

## Target 5: Quorum (Emergent Procedural Narrative)

**Proprietary gate**: Dwarf Fortress emergent stories, Rimworld storyteller AI,
Crusader Kings event chains, Caves of Qud procedural history

### Core Mechanics

- Emergent stories from simulation (no authored narrative)
- Persistent world state with cascading consequences
- Agent-based: individual actions produce collective narrative
- Memory: past events influence future probability

### Open Math

| Model | Citation | Year | Contribution |
|-------|----------|------|-------------|
| Agent-based modeling | Schelling, T.C. *J. Math. Sociology* 1:143-186 | 1971 | Individual rules → emergent collective behavior |
| Markov chains | Markov, A.A. *Bull. Acad. Imp. Sci.* | 1906 | State-dependent probability transitions |
| Causal inference | Pearl, J. *Causality* (Cambridge University Press) | 2000 | DAG-based causality from event chains |
| Quorum sensing | Nealson, K.H. & Hastings, J.W. *Microbiol. Rev.* 43:496-518 | 1979 | Individual signals → collective phase transition |
| Self-organized criticality | Bak, P. et al. *Phys. Rev. Lett.* 59:381-384 | 1987 | Emergent large-scale events from local interactions |

### Cross-Domain Mapping

| Game Concept | Biological Equivalent |
|-------------|----------------------|
| NPC agent | Individual bacterium |
| Individual action | Signal molecule production |
| Emergent event | Quorum-triggered biofilm formation |
| World state | Community composition |
| Cascading consequences | QS cascade (autoinducer → virulence) |
| Storyteller threshold | Quorum threshold concentration |

### Alternative Use Case

Clinical narrative generation: patient journey as emergent story from treatment
decisions, lab results, and vital signs. Each event is a DAG vertex. The
"narrative" is the causal chain inferred from the event graph.

### Experiment

`ludoSpring/experiments/exp059_quorum/` — ~45 validation checks

---

## Target 6: Pathogen (Gacha/Lootbox — Anti-Pattern Study)

**Proprietary gate**: Not patented, but exploitative mechanics are proprietary
knowledge applied across mobile gaming, live-service games, and casino design.

**This target is defensive**: documenting and quantifying exploitation, not
recreating it for use.

### Core Mechanics (Documented as Exploitation)

- Variable-ratio reinforcement schedule (highest resistance to extinction)
- Artificial scarcity + paid random reward
- Pity timer / pseudo-random distribution (hidden probability manipulation)
- FOMO (fear of missing out) via limited-time availability

### Open Math (Proving Exploitation)

| Model | Citation | Year | Contribution |
|-------|----------|------|-------------|
| Operant conditioning | Skinner, B.F. *The Behavior of Organisms* | 1938 | Variable-ratio schedule maximizes persistence |
| Expected value | Bernoulli, D. *Commentarii Acad. Sci. Imp. Petropolitanae* 5:175-192 | 1738 | Negative EV for player in paid RNG |
| Prospect theory | Kahneman, D. & Tversky, A. *Econometrica* 47:263-291 | 1979 | Loss aversion exploited by near-miss mechanics |
| Addiction modeling | Redish, A.D. *Science* 306:1944-1947 | 2004 | Dopamine prediction error cycle |
| Parasitism | Lotka, A.J. *Elements of Physical Biology* | 1925 | Host-parasite dynamics with negative interaction coefficient |

### Cross-Domain Mapping

| Game Concept | Biological Equivalent |
|-------------|----------------------|
| Gacha mechanic | Pathogenic virulence strategy |
| Player spending | Host resource extraction |
| Pity timer | Virulence modulation (not too lethal) |
| FOMO / limited banner | Acute infection window |
| Whale (heavy spender) | Immunocompromised host |
| Exploitation coefficient | Parasitism interaction coefficient (α < 0) |

### Alternative Use Case

Consumer protection tool: quantify exploitation level of any randomized
monetization system using validated psychology and economics math. Measure and
expose, rather than implement.

### Experiment

`ludoSpring/experiments/exp060_pathogen/` — ~30 validation checks

---

## Provenance Chain Requirements

Every lysogeny target must establish:

1. Published paper (pre-patent) describing the mathematical model
2. barraCuda primitive implementing the model
3. ludoSpring experiment applying it as a game mechanic
4. Cross-domain mapping table proving the math is general
5. Alternative non-game use case documented
6. AGPL-3.0-or-later license on all code
7. wateringHole handoff documenting the full provenance chain

---

## Target 7: Novel Ferment Transcript (Memory-Bound Digital Objects)

**Proprietary gate**: NFT platforms (OpenSea, Rarible) — blockchain-dependent,
currency-coupled digital ownership. Creation Store (Bethesda) — centralized
marketplace with revocable access.

### Core Mechanics

- Memory-bound digital objects: value from accumulated history, not scarcity
- Cryptographic provenance chain: every operation signed
- Trading protocol: offer, accept, reject, cancel, atomic swap
- Object memory: timeline of events, PROV-O attribution
- Optional public chain anchor for global persistence

### Open Math

| Model | Citation | Year | Contribution |
|-------|----------|------|-------------|
| Fermentation kinetics | Gompertz, B. *Phil. Trans. R. Soc.* 115:513-585 | 1825 | Accumulating value through transformation |
| Chain-of-custody | ISO 17025:2017 | 2017 | Provenance as unbroken custody chain |
| Directed acyclic graphs | Kahn, A.B. *Comm. ACM* 5:558-562 | 1962 | DAG topology for event ordering |
| Digital signatures | Diffie, W. & Hellman, M.E. *IEEE Trans. Info. Theory* 22:644-654 | 1976 | Cryptographic binding without central authority |
| W3C PROV-O | Moreau, L. & Missier, P. *W3C Recommendation* | 2013 | Standard provenance ontology |
| Merkle trees | Merkle, R.C. *Advances in Cryptology* 218:369-378 | 1987 | Content-addressed integrity verification |

### Cross-Domain Mapping

| Game Concept | Science Equivalent | Sensitive Data |
|-------------|-------------------|---------------|
| In-game sword with tournament history | DNA sample with lab provenance | Medical record with access audit |
| Trading card with play record | Specimen with collection chain | Legal document with custody log |
| Cosmetic skin with artist attribution | Visualization with author credit | Report with contributor attribution |
| Atomic swap between players | Custody transfer between labs | Access delegation between providers |
| Public chain anchor for resale | Regulatory compliance proof | HIPAA audit trail |

### Alternative Use Case

Any domain requiring verifiable provenance: scientific sample chain-of-custody,
medical record access logging, digital art provenance, real estate deed history,
credential verification, supply chain tracking.

### Implementation Status

`ludoSpring/experiments/exp061_fermenting/` — 89 validation checks (DONE)

**Trio modifications shipped:**
- loam-spine-core: `CertificateTradeOffer/Accept/Reject/Cancel` entry types
- sweet-grass-core: `ObjectMemory` + `ObjectEvent` API
- biomeOS: `provenance_node_atomic.toml` deployment graph

### Economics

See `whitePaper/gen3/baseCamp/20_novel_ferment_transcript_economics.md` for the
full sunCloud connection: radiating attribution activation through optional public
chain anchoring. Value flows backward through the sweetGrass attribution chain
to every contributor, proportionally.

---

## Cross-Spring Validation Assignments

| Target | wetSpring | neuralSpring | healthSpring | airSpring |
|--------|-----------|-------------|-------------|-----------|
| Usurper | Antibiotic resistance | Evolutionary game theory | — | — |
| Integrase | Phage lysogeny | — | — | — |
| Symbiont | Microbiome assembly | Multi-agent cooperation | — | Soil microbiome |
| Conjugant | HGT / LTEE | — | — | Cover crop meta-progression |
| Quorum | QS biofilm | — | Patient narrative | — |
| Pathogen | — | Reward prediction error | Addiction modeling | — |
| Novel Ferment Transcript | Sample chain-of-custody (exp062, 39/39) | — | Medical record provenance (exp063, 35/35) | Supply chain tracking |

## Cross-Spring Provenance Experiments (exp062-066)

| Experiment | Checks | What It Proves |
|-----------|--------|---------------|
| exp062: Field Sample Provenance | 39/39 | wetSpring scaffold — sample lifecycle with 6 fraud types, DAG isomorphism with exp053 |
| exp063: Consent-Gated Medical Access | 35/35 | healthSpring scaffold — patient-owned records, consent certificates, zero-knowledge access proofs |
| exp064: BearDog-Signed Chain | 39/39 | Ed25519 signing on all trio operations, chain verification, tamper detection at exact point |
| exp065: Cross-Domain Fraud Unification | 74/74 | Same GenericFraudDetector across gaming/science/medical. >80% structural similarity. |
| exp066: Radiating Attribution Calculator | 41/41 | sunCloud value distribution — decay, role weighting, conservation (shares sum to 1.0) |

**Total**: 228 cross-spring checks, 0 failures.

**Papers**: Paper 21 (Sovereign Sample Provenance), Paper 22 (Zero-Knowledge Medical Provenance) in `gen3/baseCamp/`.

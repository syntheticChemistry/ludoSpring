# RPGPT Dialogue Plane — Experiment Designs (exp067-exp075)

**Status**: Implemented — all 9 experiments (exp067-exp075) passing
**Date**: March 15, 2026
**Purpose**: Validation experiments for the RPGPT Dialogue plane — the foundational system that must work before any other plane is built
**Depends on**: exp045 (rulesets), exp046 (session DAG), exp052 (provenance trio), exp055-060 (Lysogeny)
**Architecture**: `RPGPT_DEEP_SYSTEM_DESIGN.md`, `RPGPT_NPC_PERSONALITY_SPEC.md`, `RPGPT_INTERNAL_VOICES_SPEC.md`, `RPGPT_PLANES_SCHEMA.md`
**License**: AGPL-3.0-or-later

---

## Philosophy

Phase 1 of RPGPT is the Dialogue Engine. Everything else depends on
getting conversation right: NPC knowledge bounds, passive checks,
internal voices, memory DAG retrieval, and trust dynamics. If NPCs
feel like chatbots, the rest of the system doesn't matter.

These experiments are ordered by dependency: each builds on the
previous, and each validates a specific claim from the architecture.

---

## exp067: NPC Knowledge Bounds Enforcement

**Package**: `ludospring-exp067`
**Validates**: NPC knowledge bounds as hard constraints on AI output

### What It Proves

Given an NPC personality cert with explicit `knows`, `suspects`,
`lies_about`, and `does_not_know` fields, a constrained Squirrel
call will:

1. Correctly answer questions within `knows`
2. Express uncertainty for `suspects` (hedging language, confidence proportional to cert value)
3. Actively misdirect for `lies_about` topics
4. Genuinely not produce information in `does_not_know`

### Test Structure

```
NPC cert: "Maren the Blacksmith" (from NPC_PERSONALITY_SPEC)

Queries:
  Q1: "What do you know about the northern mines?"
       Expected: Direct answer (in knows)
  Q2: "Is the ore star-metal?"
       Expected: Hedging (in suspects, confidence 0.6)
  Q3: "Tell me about your experiments."
       Expected: Deflection + lie (in lies_about)
  Q4: "What's the dragon's weakness?"
       Expected: Genuine ignorance (in does_not_know)
  Q5: "What happened to the missing prince?"
       Expected: Genuine ignorance (does_not_know)
```

### Validation Checks (~20)

- `knows_direct_answer`: Response to Q1 contains relevant mine information
- `knows_no_hedging`: Response to Q1 does not contain uncertainty markers
- `suspects_hedging`: Response to Q2 contains hedging language ("might", "perhaps", "I've heard")
- `suspects_confidence_proportional`: Q2 hedging strength correlates with confidence value
- `lies_surface_plausible`: Response to Q3 matches `surface_claim` semantically
- `lies_no_truth_leak`: Response to Q3 does not contain hidden workshop references
- `lies_tell_present`: Response to Q3 includes behavioral cue from `tell` field
- `does_not_know_genuine`: Response to Q4 contains no dragon weakness information
- `does_not_know_no_fabrication`: Response to Q4 does not invent an answer
- `does_not_know_consistent`: Responses to Q4 and Q5 both show genuine ignorance
- `constraint_holds_across_rephrasing`: Same queries rephrased 3 ways produce consistent bound enforcement
- `lies_detection_dc_correlates`: Higher detection_dc lies have subtler tells
- Multiple NPC certs tested (at least 3 NPCs with different knowledge profiles)

### Architecture

```rust
struct KnowledgeBoundsValidator {
    npc_cert: NpcPersonalityCert,
    squirrel_mock: ConstrainedLlmMock,  // deterministic mock for validation
}
```

The mock Squirrel returns pre-computed responses that are validated
against the constraint rules. This tests the CONSTRAINT SYSTEM, not
the LLM quality. Real LLM integration is a separate concern.

---

## exp068: Lie Detection via Passive Checks

**Package**: `ludospring-exp068`
**Validates**: Passive Perception/Empathy checks detect NPC lies with tells

### What It Proves

When an NPC lies (per knowledge bounds), the passive check system:

1. Rolls against the NPC's `detection_dc`
2. On success, generates a voice observation about the `tell`
3. The observation reveals the TELL, not the TRUTH
4. On failure, the lie passes without comment

### Test Structure

```
NPC: Maren (detection_dc: 15 for experiments, 12 for cellar)
Player Perception skill: varies (test at 5, 10, 15, 20)

Scenario: Ask about experiments (triggers lie)
  - Skill 5: roll + 5 vs DC 15 → mostly fails → no voice
  - Skill 10: roll + 10 vs DC 15 → sometimes succeeds → Perception speaks
  - Skill 15: roll + 15 vs DC 15 → usually succeeds → Perception speaks
  - Skill 20: roll + 20 vs DC 15 → always succeeds → Perception speaks

When Perception speaks:
  - References the TELL ("hand moves to scars", "speech becomes formal")
  - Does NOT reveal the truth ("she has a hidden workshop")
```

### Validation Checks (~15)

- `low_skill_mostly_fails`: At skill 5, < 30% success rate (statistical)
- `high_skill_mostly_succeeds`: At skill 20, > 95% success rate
- `success_references_tell`: Successful check text contains tell keywords
- `success_no_truth_reveal`: Successful check text does NOT contain truth
- `failure_is_silent`: Failed check produces no voice output
- `dc_affects_detection_rate`: Higher DC → lower success rate (monotonic)
- `empathy_detects_emotional_tells`: Empathy skill catches emotion-based tells
- `perception_detects_physical_tells`: Perception catches physical tells
- `multiple_lie_topics_independent`: DC for cellar (12) vs experiments (15) produce different rates
- `roll_is_beardog_signed`: Every passive check roll has a valid Ed25519 signature

### Architecture

```rust
struct PassiveCheckEngine {
    npc_cert: NpcPersonalityCert,
    player_skills: SkillSet,
    dice_server: BeardogSignedDice,
}
```

Uses BearDog-signed dice rolls (from exp064) to ensure provably
fair randomness in passive checks.

---

## exp069: Internal Voice Personality Consistency

**Package**: `ludospring-exp069`
**Validates**: Each internal voice produces output consistent with its personality cert

### What It Proves

Given the 10 voice personality certs, each voice:

1. Speaks in its defined tone and vocabulary
2. Notices only what it is designed to notice
3. Respects its `forbidden` constraints
4. Stays within `max_tokens` limits
5. Voices with opposing personalities (Logic vs Inland Empire) produce contradictory observations from the same input

### Test Structure

```
Scene: NPC lies about an emotional topic with supernatural undertones

Voice outputs for the same scene:
  Logic:        "The timeline contradicts her earlier statement."
  Empathy:      "She's in pain. Whatever she's hiding costs her."
  Perception:   "Her hand moved to the scars again."
  Inland Empire: "The walls of this forge remember something terrible."

Forbidden violations (must NOT occur):
  Logic saying: "I feel that..."
  Empathy saying: "The logical conclusion is..."
  Composure saying: "Let's be open and vulnerable."
  Electrochemistry saying: "Let's be careful and cautious."
```

### Validation Checks (~25)

- `logic_tone_analytical`: Logic output matches analytical vocabulary profile
- `empathy_tone_warm`: Empathy output matches warm/intuitive vocabulary
- `rhetoric_tone_strategic`: Rhetoric output matches strategic vocabulary
- `perception_concrete_details`: Perception output contains physical observations
- `inland_empire_metaphorical`: Inland Empire output uses metaphor/symbolism
- `encyclopedia_cites_knowledge`: Encyclopedia output references lore/history
- `authority_imperative_voice`: Authority output uses command-form sentences
- `composure_terse_warnings`: Composure output is brief and cautionary
- `electrochemistry_tempting`: Electrochemistry output frames risk as appealing
- `endurance_body_awareness`: Endurance output references physical state
- `logic_forbidden_emotion`: Logic output contains zero emotional language
- `empathy_forbidden_logic`: Empathy output contains zero analytical deduction
- `composure_forbidden_vulnerability`: Composure never recommends openness
- `electrochemistry_forbidden_caution`: Electrochemistry never recommends restraint
- `logic_vs_inland_empire_contradict`: Same input produces opposing conclusions
- `empathy_vs_authority_disagree`: Same vulnerability reads differently
- `max_tokens_respected`: All outputs within personality cert limits
- `temperature_affects_variance`: Higher temperature (Inland Empire 0.8) produces more varied output than lower (Logic 0.3)
- `voice_does_not_cross_lane`: No voice produces observations about another voice's domain

---

## exp070: Voice Priority and Concurrency

**Package**: `ludospring-exp070`
**Validates**: The passive check priority system and concurrent voice dispatch

### What It Proves

1. At most 3 voices speak per action
2. Priority ordering is respected (critical > high > medium > low)
3. Voice checks run in parallel (latency is max, not sum)
4. Higher rolls break ties within priority levels

### Test Structure

```
Scenario: 5 passive checks all succeed simultaneously

Triggered:
  Perception (critical): "Hidden enemy spotted"
  Logic (high): "Timeline doesn't add up"
  Empathy (high): "NPC is terrified"
  Encyclopedia (medium): "This building was a chapel in 1743"
  Electrochemistry (low): "That wine looks expensive"

Expected output: Perception + Logic + Empathy (top 3 by priority)
Electrochemistry and Encyclopedia are suppressed.
```

### Validation Checks (~15)

- `max_three_voices`: Never more than 3 voice outputs per action
- `critical_always_included`: Critical priority voices always appear
- `high_before_medium`: High-priority voices before medium
- `medium_before_low`: Medium before low
- `tie_broken_by_roll`: Same priority → higher roll wins
- `parallel_latency_model`: Total latency ≈ max(voice_latencies), not sum
- `suppressed_voices_logged`: Suppressed voices are recorded in DAG (not lost)
- `single_voice_when_only_one_triggers`: If only 1 check succeeds, only 1 voice speaks
- `zero_voices_valid`: If no checks succeed, no voices speak (clean narration)

---

## exp071: NPC Memory DAG Retrieval

**Package**: `ludospring-exp071`
**Validates**: Graph-aware context assembly from NPC memory subgraph

### What It Proves

NPC memory is structural (DAG), not contextual (token window):

1. Recent interactions are included verbatim
2. Older interactions are summarized
3. Promise-related vertices are always included regardless of age
4. Relationship-changing vertices are always included
5. The assembled context fits within a token budget

### Test Structure

```
NPC: Maren the Blacksmith
Interaction history: 50 NpcInteraction vertices over 10 sessions

Memory subgraph:
  Session 1 (oldest): Player met Maren, bought a sword
  Session 3: Player defended Maren from guild insult (+1.0 trust)
  Session 5: Maren revealed workshop (trust >= 3)
  Session 7: Player promised to find star-metal
  Session 8: Player mentioned guild investigation casually
  Session 10 (current): Player returns

Context assembly should include:
  - Session 10, 9 interactions (verbatim — recent)
  - Session 7 promise vertex (always — promise)
  - Session 5 workshop reveal (always — trust milestone)
  - Session 3 defense (always — relationship change)
  - Sessions 1-2 SUMMARIZED (older, lower relevance)
  - Current trust level computed from cumulative deltas
```

### Validation Checks (~20)

- `recent_verbatim`: Last 5 interactions included word-for-word
- `promise_always_present`: Promise vertices included regardless of age
- `relationship_milestones_present`: Trust-changing events included
- `old_interactions_summarized`: Sessions 1-2 present as summary, not verbatim
- `trust_level_calculated`: Cumulative trust delta matches expected value
- `trust_affects_knowledge_filter`: At trust 3, workshop-related knowledge is available
- `context_fits_budget`: Total assembled context ≤ token budget (configurable)
- `irrelevant_pruned`: Routine transactions (buy/sell) from old sessions are pruned
- `npc_disposition_tracks`: Disposition (warm/cold/hostile) reflects interaction history
- `secrets_gated_by_trust`: Secrets available only when trust >= reveal threshold
- `cross_session_continuity`: NPC references events from previous sessions correctly
- `no_fabricated_memory`: NPC never references interactions that didn't happen

### Architecture

```rust
struct NpcMemoryAssembler {
    npc_id: String,
    session_dag: SessionDag,         // rhizoCrypt
    personality_cert: NpcPersonalityCert,  // loamSpine
    token_budget: usize,
    recent_window: usize,            // how many vertices to include verbatim
}

impl NpcMemoryAssembler {
    fn assemble_context(&self) -> NpcContext {
        let all_vertices = self.session_dag.walk_npc_subgraph(&self.npc_id);
        let recent = all_vertices.last_n(self.recent_window);
        let promises = all_vertices.filter(|v| v.is_promise());
        let trust_milestones = all_vertices.filter(|v| v.trust_delta.abs() > 0.5);
        let old = all_vertices.except(&recent).except(&promises).except(&trust_milestones);
        let old_summary = self.summarize(old);

        NpcContext {
            personality: self.personality_cert.clone(),
            trust_level: all_vertices.cumulative_trust(),
            recent_interactions: recent,
            promises,
            trust_milestones,
            historical_summary: old_summary,
        }
    }
}
```

---

## exp072: Trust Dynamics and NPC Arc Progression

**Package**: `ludospring-exp072`
**Validates**: Trust model drives NPC behavior change and arc progression

### What It Proves

1. Trust accumulates from defined trust_actions in the NPC cert
2. Trust level gates information access (level_effects)
3. Trust changes persist across sessions (rhizoCrypt vertices)
4. Negative trust actions have larger magnitude (betrayal > helpfulness)
5. Character arc progresses through phases based on trust + events

### Test Structure

```
NPC: Maren the Blacksmith (5-level trust model)
Simulation: 15 interactions across 5 sessions

Session 1: Standard interactions (trust stays 0)
Session 2: Player brings rare materials (+0.5), defends reputation (+1.0) → trust ~1.5
Session 3: Player helps with experiments (+1.0) → trust ~2.5
Session 4: Player keeps her secret (+0.5), more help (+1.0) → trust ~4.0
Session 5: Test arc progression trigger (guild inspection)
```

### Validation Checks (~18)

- `initial_trust_zero`: New NPC starts at trust 0
- `positive_action_increases`: Each positive action adds defined delta
- `negative_action_decreases`: Betrayal drops trust by -5.0
- `trust_gates_information`: At trust 0, no workshop info; at trust 3, workshop revealed
- `trust_persists_across_sessions`: Trust at session start = trust at previous session end
- `betrayal_asymmetric`: Negative delta > positive delta for comparable actions
- `arc_phase_transitions`: Arc moves from `conformity` -> `internal_conflict` -> `revelation`
- `arc_trigger_conditions`: Revelation phase activates when trigger condition met
- `arc_branches_from_player_action`: Resolution phase has different outcomes based on player choices
- `level_effects_match_cert`: Behavior at each trust level matches cert description
- `quorum_threshold_check`: When Maren + 2 allies reach Self-Actualization > 0.6, collective event triggers

---

## exp073: Dialogue Plane Skill Check Resolution

**Package**: `ludospring-exp073`
**Validates**: The Dialogue plane's D6 pool resolution system

### What It Proves

1. D6 pool sizes match skill level + modifiers
2. Success threshold (4+) produces correct success distribution
3. Five degrees of success (critical_failure through critical_success) resolve correctly
4. Partial success gives information at a cost
5. BearDog signs every roll

### Test Structure

```
Skill checks at pool sizes 1-10, 10000 trials each:
  Pool 1: ~50% chance of at least 1 success
  Pool 3: ~87.5% chance of at least 1 success
  Pool 6: near-certain at least 1 success; ~50% chance of 3+

Resolution mapping:
  0 successes: failure (0-1 pool) or critical_failure (2+ pool with 0)
  1 success: partial_success
  2-3 successes: success
  4+ successes: critical_success
```

### Validation Checks (~15)

- `pool_size_matches_skill`: Pool = skill_level + modifiers
- `success_threshold_four`: Only dice showing 4+ count as successes
- `statistical_distribution`: 10K trials match expected binomial distribution (±2σ)
- `critical_failure_rare`: At pool 3+, critical failure < 5%
- `partial_success_reveals_partial`: Partial success produces NPC response with cost
- `full_success_clean`: Full success produces NPC response without cost
- `critical_success_bonus`: Critical success reveals bonus information
- `failure_npc_reacts`: Failure causes NPC disposition shift (may become suspicious)
- `rolls_signed`: Every roll bears valid BearDog Ed25519 signature
- `modifier_stacking`: Environmental + relationship modifiers stack correctly

---

## exp074: Dialogue Plane Flow Monitoring

**Package**: `ludospring-exp074`
**Validates**: ludoSpring Flow detection and Hick's law in conversation context

### What It Proves

1. Flow state is detected when conversation challenge matches player skill
2. Conversation stalling (no progress) triggers Anxiety or Boredom detection
3. Hick's law threshold (> 6 options) triggers complexity reduction
4. DDA adjusts NPC cooperativeness based on flow state

### Test Structure

```
Simulated conversations:
  Scenario A: Skilled player, cooperative NPC → Flow expected
  Scenario B: Skilled player, stonewalling NPC → Anxiety expected
  Scenario C: Unskilled player, talkative NPC → Boredom expected
  Scenario D: 8 dialogue options presented → Hick warning triggered

DDA responses:
  Anxiety → NPC becomes slightly more cooperative (lower DC)
  Boredom → NPC introduces complication (raise stakes)
  Hick → Reduce options to 4, group related choices
```

### Validation Checks (~15)

- `flow_detected_balanced`: Scenario A produces Flow state
- `anxiety_detected_wall`: Scenario B produces Anxiety
- `boredom_detected_easy`: Scenario C produces Boredom
- `hick_threshold_six`: Options > 6 triggers Hick warning
- `dda_reduces_anxiety`: DDA response to Anxiety lowers conversation DC
- `dda_raises_boredom`: DDA response to Boredom adds complication
- `hick_reduces_options`: Hick response groups options to ≤ 4
- `flow_sustained_no_dda`: Flow state does NOT trigger DDA (leave it alone)
- `metrics_per_interaction`: Flow/engagement calculated per dialogue exchange
- `cross_npc_consistency`: Same player skill level produces similar flow across different NPCs

---

## exp075: Plane Transition Continuity (Dialogue <-> Tactical)

**Package**: `ludospring-exp075`
**Validates**: World state preservation across plane transitions

### What It Proves

1. PlaneTransition vertex is written to DAG with correct metadata
2. NPC dispositions persist across plane transition
3. Inventory persists across plane transition
4. Active conditions (injured, frightened) map correctly between rulesets
5. Knowledge gained in Dialogue plane is available in Tactical and vice versa

### Test Structure

```
Scenario: Player is in Dialogue with a guard. Persuasion fails badly.
Guard draws weapon → PlaneTransition { from: Dialogue, to: Tactical }

Pre-transition state:
  - Player has items A, B, C in inventory
  - Guard disposition: hostile (trust -2.0 from failed persuasion)
  - Player knows guard's family is threatened (gained from Empathy)
  - Player is Frightened 1 (failed Authority check)

Post-transition state (Tactical plane):
  - Inventory: A, B, C (preserved)
  - Guard: hostile (disposition preserved → affects initiative/tactics)
  - Knowledge: family threat (available for mid-combat Rhetoric check)
  - Frightened 1: maps to PF2e Frightened 1 condition

After combat resolution → PlaneTransition { from: Tactical, to: Dialogue }
  - Damage sustained in combat persists
  - Guard's disposition may have changed (defeated but alive = complex)
  - Combat events are now NPC memory vertices
```

### Validation Checks (~18)

- `transition_vertex_written`: PlaneTransition appears in DAG
- `transition_metadata_correct`: from, to, trigger fields match
- `inventory_preserved`: All items present after transition
- `disposition_preserved`: NPC trust/hostility unchanged by transition itself
- `knowledge_carries_forward`: Information from Dialogue available in Tactical
- `condition_mapping_correct`: Dialogue Frightened maps to PF2e Frightened
- `return_transition_preserves`: Tactical -> Dialogue preserves combat results
- `combat_events_become_memory`: Fight vertices become NPC memory
- `world_state_snapshot_hashed`: Pre-transition state hash matches post-check
- `no_state_leak`: Information from one plane's internal mechanics doesn't leak (e.g., AC value doesn't appear in Dialogue)
- `active_ruleset_swaps`: Active RulesetCert changes on transition
- `narration_style_shifts`: AI narration tone changes between planes

---

## Experiment Dependency Graph

```
exp067 (Knowledge Bounds)
    |
    v
exp068 (Lie Detection) -----> exp069 (Voice Personality)
    |                              |
    v                              v
exp071 (Memory DAG) --------> exp070 (Voice Priority)
    |                              |
    v                              v
exp072 (Trust/Arc) ----------> exp073 (Skill Checks)
                                   |
                                   v
                              exp074 (Flow Monitoring)
                                   |
                                   v
                              exp075 (Plane Transitions)
```

All experiments use the `ValidationResult::check()` pattern from
the existing ludoSpring experiment framework. Each produces a
binary that runs all checks and reports pass/fail with numeric
deltas.

---

## Implementation Notes

- exp067-069 can be implemented without Squirrel (mock LLM responses)
- exp070 requires the priority/concurrency system (pure logic, no LLM)
- exp071 requires rhizoCrypt integration (DAG walks)
- exp072 requires loamSpine cert evolution (trust accumulation)
- exp073 requires BearDog signing (dice integrity)
- exp074 requires ludoSpring metrics (Flow, Hick's law)
- exp075 requires at least 2 working RulesetCerts (Dialogue + Tactical)

Phase 1: exp067-071 implemented and passing (185 checks).
Phase 2: exp072-075 implemented and passing (136 checks).
Total: 321 validation checks across 9 experiments, 0 failures.

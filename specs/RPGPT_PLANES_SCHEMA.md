# RPGPT Planes Schema — RulesetCert Definitions for Each Plane

**Status**: Design specification
**Date**: March 15, 2026
**Depends on**: `Ruleset` trait (`barracuda/src/game/ruleset.rs`), loamSpine certificate model
**License**: AGPL-3.0-or-later

---

## Overview

Each plane in RPGPT is governed by a `RulesetCert` — a loamSpine
certificate containing machine-readable rules that constrain how the
AI adds vertices to the session DAG. This document defines the schema
for each of the seven planes.

---

## Common Schema

All `RulesetCert` instances share this base structure:

```yaml
certificate_type: "rpgpt.ruleset"
version: "1.0"
plane: <PlaneType>

dice_system:
  type: <D20 | FudgeDice | RollUnder | D6Pool | D100 | None>
  # Type-specific parameters follow

action_economy:
  type: <ThreeAction | FreeForm | Reactive | TurnBased | RealTime>
  actions_per_turn: <int | null>
  reaction_allowed: <bool>
  free_actions: <list>

resolution:
  type: <DegreeOfSuccess | BinaryPass | PartialSuccess | Contested | Automatic>
  degrees: <list of outcome names>

available_actions:
  - id: <action_id>
    name: <display_name>
    cost: <action_cost>
    requires: <conditions>
    produces: <vertex_type>
    description: <AI narration hint>

passive_checks:
  - skill: <skill_name>
    trigger: <trigger_condition>
    dc: <difficulty_class>
    on_success: <voice_message_hint>
    on_failure: <null | alternate_hint>

constraints:
  - rule: <machine-readable constraint>
    enforcement: <hard | soft>
    source: <citation>

narration_style:
  tone: <descriptive string>
  pacing: <slow | measured | urgent | frantic>
  vocabulary: <literary | colloquial | technical | archaic>
  perspective: <second_person | third_person>
  sensory_emphasis: <list of senses to prioritize>
```

---

## Plane 1: Exploration

**Source**: Cairn (CC-BY-SA) + custom extensions

```yaml
plane: Exploration

dice_system:
  type: RollUnder
  die: "d20"
  succeed_if: "roll <= ability_score"
  note: "Checks are rare — exploration rewards curiosity, not rolls"

action_economy:
  type: FreeForm
  actions_per_turn: null
  note: "No turn structure. Players declare actions freely. Time passes narratively."

resolution:
  type: BinaryPass
  degrees: ["success", "failure"]
  note: "Cairn-style — no partial success in exploration. You find it or you don't."

available_actions:
  - id: "move"
    name: "Move to location"
    cost: null
    produces: "MovementVertex"
    description: "Describe the journey. Emphasize sensory details."
  - id: "examine"
    name: "Examine something"
    cost: null
    produces: "ExaminationVertex"
    description: "Describe what the player finds. Include details that reward attention."
  - id: "interact_object"
    name: "Interact with object"
    cost: null
    produces: "InteractionVertex"
    description: "Describe the result. Objects may be inventory items (Cairn: inventory-as-HP)."
  - id: "rest"
    name: "Rest and recover"
    cost: null
    produces: "RestVertex"
    description: "Describe the passage of time. Restore HP equal to rest quality."
  - id: "use_item"
    name: "Use inventory item"
    cost: null
    produces: "ItemUseVertex"
    description: "Describe the effect. Items are consumed or persist per type."

passive_checks:
  - skill: "Perception"
    trigger: "Hidden feature in environment"
    dc: 10
    on_success: "You notice something others would miss."
  - skill: "Encyclopedia"
    trigger: "Historical or lore-relevant location"
    dc: 8
    on_success: "You recall reading about this place."
  - skill: "Inland Empire"
    trigger: "Location with supernatural significance"
    dc: 12
    on_success: "Something about this place feels... wrong. Or very right."

constraints:
  - rule: "Inventory slots = 10. Items beyond 10 reduce HP."
    enforcement: hard
    source: "Cairn SRD (CC-BY-SA)"
  - rule: "No attack rolls in Exploration plane. Combat triggers PlaneTransition to Tactical."
    enforcement: hard
  - rule: "HP is Fatigue + Wounds. Rest recovers Fatigue only."
    enforcement: hard
    source: "Cairn SRD"

narration_style:
  tone: "Atmospheric, descriptive, sensory-rich"
  pacing: slow
  vocabulary: literary
  perspective: second_person
  sensory_emphasis: ["sight", "sound", "smell", "touch"]

ludospring_metrics:
  primary: "engagement"
  signals:
    - metric: "exploration_ratio"
      description: "Fraction of available locations visited"
      threshold_low: 0.2
      action_if_low: "Introduce environmental hook to encourage movement"
    - metric: "action_density"
      description: "Actions per minute"
      threshold_low: 0.5
      action_if_low: "Something changes in the environment (sound, weather, NPC arrival)"
```

---

## Plane 2: Dialogue

**Source**: Custom (Disco Elysium model, open math)

```yaml
plane: Dialogue

dice_system:
  type: D6Pool
  pool_size: "skill_level + modifiers"
  threshold: 4
  note: "Roll pool of d6. Each die >= 4 is a success. More successes = better outcome."

action_economy:
  type: Reactive
  actions_per_turn: null
  note: "No strict turns. Conversation flows naturally. Player chooses dialogue options."

resolution:
  type: PartialSuccess
  degrees: ["critical_failure", "failure", "partial_success", "success", "critical_success"]
  note: "Partial success reveals something but at a cost (NPC becomes suspicious, information is incomplete)."

available_actions:
  - id: "speak"
    name: "Say something"
    cost: null
    produces: "DialogueVertex"
    description: "Player's words are recorded verbatim. NPC responds in character."
  - id: "persuade"
    name: "Attempt persuasion"
    cost: null
    requires: "Rhetoric skill"
    produces: "PersuasionCheckVertex"
    description: "Roll Rhetoric pool vs NPC's resistance. Partial success: they waver but don't commit."
  - id: "read_person"
    name: "Read emotional state"
    cost: null
    requires: "Empathy skill"
    produces: "InsightVertex"
    description: "Roll Empathy pool vs NPC's Composure. Success reveals hidden emotion."
  - id: "deceive"
    name: "Lie or mislead"
    cost: null
    requires: "Composure skill"
    produces: "DeceptionCheckVertex"
    description: "Roll Composure vs NPC's Perception. Failure: NPC catches the lie."
  - id: "intimidate"
    name: "Threaten or pressure"
    cost: null
    requires: "Authority skill"
    produces: "IntimidationCheckVertex"
    description: "Roll Authority vs NPC's Endurance. Costs trust even on success."
  - id: "inquire"
    name: "Ask a question"
    cost: null
    produces: "InquiryVertex"
    description: "NPC answers within knowledge bounds. May lie (with tells) about restricted topics."
  - id: "observe"
    name: "Watch body language"
    cost: null
    requires: "Perception skill"
    produces: "ObservationVertex"
    description: "Passive — detect tells when NPC is lying. DC from NPC cert."

passive_checks:
  - skill: "Logic"
    trigger: "NPC statement contradicts earlier statement or known fact"
    dc: "varies by contradiction severity"
    on_success: "Wait — that contradicts what they said earlier about..."
  - skill: "Empathy"
    trigger: "NPC emotional state changes (fear, grief, anger)"
    dc: 8
    on_success: "They're afraid. You can see it in the way they..."
  - skill: "Perception"
    trigger: "NPC exhibits a lie-tell from their knowledge_bounds"
    dc: "from NPC cert detection_dc"
    on_success: "Their hand moved to cover the scars. They're hiding something."
  - skill: "Rhetoric"
    trigger: "Logical weakness in NPC's argument"
    dc: 10
    on_success: "There's an opening. If you press on the guild's authority..."
  - skill: "Inland Empire"
    trigger: "Supernatural or deeply symbolic element in conversation"
    dc: 14
    on_success: "Something about her words echoes. Like you've heard them before, in a dream."
  - skill: "Electrochemistry"
    trigger: "Temptation or shortcut available"
    dc: 6
    on_success: "You could just take what you need. She'd never know."

constraints:
  - rule: "NPC knowledge bounds are absolute. AI must not reveal what NPC does_not_know."
    enforcement: hard
  - rule: "NPC lies must include tells proportional to detection_dc."
    enforcement: hard
  - rule: "Trust changes are recorded as vertices and affect future interactions."
    enforcement: hard
  - rule: "No physical combat in Dialogue plane. Violence triggers PlaneTransition."
    enforcement: hard

narration_style:
  tone: "Intimate, psychologically rich, layered with subtext"
  pacing: measured
  vocabulary: literary
  perspective: second_person
  sensory_emphasis: ["facial expressions", "body language", "tone of voice", "silence"]

ludospring_metrics:
  primary: "flow"
  signals:
    - metric: "flow_state"
      description: "Is conversation challenge matching player skill?"
      threshold: "not_in_flow"
      action: "Adjust NPC cooperativeness (easier) or introduce complication (harder)"
    - metric: "hick_choice_count"
      description: "Number of dialogue options presented"
      threshold_high: 6
      action: "Reduce options. Group related choices. Highlight key paths."
```

---

## Plane 3: Tactical

**Source**: Pathfinder 2e (ORC License)

```yaml
plane: Tactical

dice_system:
  type: D20
  roll: "1d20 + modifier"
  succeed_if: "total >= DC"
  critical: "total >= DC + 10 OR natural 20"
  critical_fail: "total <= DC - 10 OR natural 1"

action_economy:
  type: ThreeAction
  actions_per_turn: 3
  reaction_allowed: true
  free_actions: ["speak briefly", "drop item", "release grip"]
  note: "PF2e three-action economy. Each action has a cost (1, 2, or 3 actions)."

resolution:
  type: DegreeOfSuccess
  degrees: ["critical_failure", "failure", "success", "critical_success"]

available_actions:
  - id: "strike"
    name: "Strike (attack)"
    cost: 1
    requires: "weapon equipped"
    produces: "AttackVertex"
    description: "Roll attack vs target AC. Apply MAP for subsequent attacks."
  - id: "move"
    name: "Stride"
    cost: 1
    produces: "MovementVertex"
    description: "Move up to Speed. Provokes reactions from some enemies."
  - id: "cast_spell"
    name: "Cast a Spell"
    cost: "varies (1-3)"
    requires: "spell prepared or known"
    produces: "SpellVertex"
    description: "Resolve spell per ruleset. Area effects hit all in zone."
  - id: "raise_shield"
    name: "Raise Shield"
    cost: 1
    produces: "DefenseVertex"
    description: "Gain shield's AC bonus until start of next turn."
  - id: "demoralize"
    name: "Demoralize"
    cost: 1
    requires: "Intimidation skill"
    produces: "ConditionVertex"
    description: "Intimidation vs target Will DC. Success: Frightened 1."
  - id: "recall_knowledge"
    name: "Recall Knowledge"
    cost: 1
    produces: "KnowledgeVertex"
    description: "Identify enemy weakness or trait. Skill depends on creature type."
  - id: "take_cover"
    name: "Take Cover"
    cost: 1
    produces: "DefenseVertex"
    description: "Gain +2 circumstance bonus to AC and Reflex from cover."
  - id: "aid"
    name: "Aid an Ally"
    cost: "reaction"
    produces: "AidVertex"
    description: "Grant +1 to ally's next check. Requires preparation."

passive_checks:
  - skill: "Perception"
    trigger: "Hidden enemy or trap in tactical environment"
    dc: "enemy Stealth DC"
    on_success: "You spot movement in the shadows."
  - skill: "Encyclopedia"
    trigger: "Facing a known creature type"
    dc: 10
    on_success: "You recall — trolls regenerate unless burned."

constraints:
  - rule: "3 actions per turn. No more."
    enforcement: hard
    source: "Pathfinder 2e CRB, ORC License"
  - rule: "Multiple Attack Penalty: -5 on 2nd attack, -10 on 3rd (or -4/-8 with agile)."
    enforcement: hard
    source: "PF2e CRB"
  - rule: "Conditions decay per PF2e rules (Frightened reduces by 1 per turn)."
    enforcement: hard
    source: "PF2e CRB"
  - rule: "Initiative order is fixed for the encounter."
    enforcement: hard
  - rule: "HP reaching 0 triggers Dying condition, not instant death."
    enforcement: hard
    source: "PF2e CRB"

narration_style:
  tone: "Tense, tactical, consequential — every action matters"
  pacing: urgent
  vocabulary: colloquial
  perspective: second_person
  sensory_emphasis: ["motion", "sound", "impact", "pain"]

ludospring_metrics:
  primary: "dda"
  signals:
    - metric: "encounter_difficulty"
      description: "Party resources vs encounter CR"
      action: "Adjust reinforcement timing, enemy tactics, retreat opportunities"
    - metric: "flow_state"
      description: "Is combat challenging but not overwhelming?"
      action: "Scale enemy competence (tactical AI quality, not raw stats)"
```

---

## Plane 4: Investigation

**Source**: Custom (GUMSHOE-inspired, open logic)

```yaml
plane: Investigation

dice_system:
  type: None
  note: "Core clues are found automatically (GUMSHOE principle). Rolls only for bonus information."

action_economy:
  type: FreeForm
  actions_per_turn: null
  note: "No turns. Investigation proceeds at player's pace."

resolution:
  type: Automatic
  note: "If you look in the right place, you find the clue. No roll gates progress."
  bonus_resolution:
    type: D6Pool
    pool_size: "skill_level"
    threshold: 4
    note: "Bonus clues from additional successes — more detail, not gated progress."

available_actions:
  - id: "examine_scene"
    name: "Examine a scene"
    cost: null
    produces: "ClueVertex"
    description: "Describe what the player finds. Core clues are always found."
  - id: "interview"
    name: "Interview a witness"
    cost: null
    produces: "TestimonyVertex"
    description: "NPC shares what they know (within knowledge bounds). Dialogue plane rules apply."
  - id: "research"
    name: "Research in archives"
    cost: null
    produces: "ResearchVertex"
    description: "Cross-reference clues. Encyclopedia voice assists."
  - id: "connect_clues"
    name: "Connect two clues"
    cost: null
    requires: "Two or more ClueVertices"
    produces: "DeductionVertex"
    description: "Player proposes a connection. System validates logical consistency."
  - id: "present_evidence"
    name: "Present evidence to NPC"
    cost: null
    produces: "ConfrontationVertex"
    description: "Show clue to NPC. Their reaction depends on knowledge bounds and relationship."

passive_checks:
  - skill: "Logic"
    trigger: "Player has enough clues to make a connection but hasn't"
    dc: 10
    on_success: "These two things are related. The timeline doesn't work unless..."
  - skill: "Encyclopedia"
    trigger: "Clue relates to historical or technical knowledge"
    dc: 8
    on_success: "This symbol is from the Order of the Silver Key. Founded in 1687."
  - skill: "Inland Empire"
    trigger: "Pattern that logic can't explain but intuition senses"
    dc: 14
    on_success: "Something connects these deaths. Not cause and effect. Something deeper."

constraints:
  - rule: "Core clues are never gated behind rolls. Progress is always possible."
    enforcement: hard
    source: "GUMSHOE design principle (Robin Laws, CC attribution)"
  - rule: "The evidence DAG must be internally consistent. No contradictions without explanation."
    enforcement: hard
  - rule: "NPCs react to presented evidence based on their knowledge bounds."
    enforcement: hard

narration_style:
  tone: "Suggestive, atmospheric, intellectually engaging"
  pacing: slow
  vocabulary: literary
  perspective: second_person
  sensory_emphasis: ["detail", "texture", "inconsistency", "absence"]

ludospring_metrics:
  primary: "hick_law"
  signals:
    - metric: "clue_count"
      description: "Number of unconnected clues"
      threshold_high: 8
      action: "Logic voice offers a hint. Reduce clue presentation complexity."
    - metric: "time_since_progress"
      description: "Time since last DeductionVertex"
      threshold_high: "5 minutes"
      action: "Perception or Encyclopedia voice offers an observation."
```

---

## Plane 5: Political

**Source**: FATE Core (CC-BY)

```yaml
plane: Political

dice_system:
  type: FudgeDice
  roll: "4dF + skill"
  succeed_if: "total >= difficulty"
  tie: "succeed at minor cost"
  succeed_with_style: "total >= difficulty + 3"

action_economy:
  type: Reactive
  actions_per_turn: null
  note: "FATE-style scene-based. Players create and invoke Aspects."

resolution:
  type: PartialSuccess
  degrees: ["fail", "tie", "succeed", "succeed_with_style"]

available_actions:
  - id: "create_advantage"
    name: "Create an Advantage"
    cost: null
    produces: "AspectVertex"
    description: "Create a situational Aspect. 'The crowd is restless' becomes mechanically real."
  - id: "invoke_aspect"
    name: "Invoke an Aspect"
    cost: "1 Fate Point"
    produces: "InvocationVertex"
    description: "+2 to roll or reroll. Must narratively justify how the Aspect helps."
  - id: "compel"
    name: "Accept a Compel"
    cost: "gain 1 Fate Point"
    produces: "CompelVertex"
    description: "An Aspect works against you. Accept for a Fate Point, or pay to refuse."
  - id: "declare_intent"
    name: "Declare political intent"
    cost: null
    produces: "IntentVertex"
    description: "Announce what you want from this scene. Sets stakes."
  - id: "negotiate"
    name: "Negotiate with faction"
    cost: null
    requires: "Rhetoric or Empathy skill"
    produces: "NegotiationVertex"
    description: "Roll vs faction representative. Symbiont math modifies difficulty."

passive_checks:
  - skill: "Rhetoric"
    trigger: "Faction representative reveals political weakness"
    dc: 10
    on_success: "Their alliance with the merchant guild is fragile. Push there."
  - skill: "Empathy"
    trigger: "NPC's true allegiance differs from stated position"
    dc: 12
    on_success: "They don't believe what they're saying. They're performing."
  - skill: "Authority"
    trigger: "Power vacuum or leadership uncertainty"
    dc: 10
    on_success: "No one in this room is in charge. That's an opportunity."

constraints:
  - rule: "Aspects are real. If 'The crowd is restless' exists, it affects all actions in the scene."
    enforcement: hard
    source: "FATE Core SRD (CC-BY)"
  - rule: "Fate Points are finite. Economy must be tracked."
    enforcement: hard
    source: "FATE Core SRD"
  - rule: "Faction reputation changes propagate through Symbiont relationship graph."
    enforcement: hard

narration_style:
  tone: "Intrigue, subtext, strategic maneuvering"
  pacing: measured
  vocabulary: archaic
  perspective: second_person
  sensory_emphasis: ["expressions", "silences", "what is NOT said"]

ludospring_metrics:
  primary: "four_keys"
  signals:
    - metric: "people_fun_score"
      description: "Social interaction quality"
      action: "Introduce NPC with strong personality if score is low"
    - metric: "engagement"
      description: "Player investment in political outcome"
      threshold_low: 0.3
      action: "Raise stakes — introduce personal consequence"
```

---

## Plane 6: Crafting

**Source**: Custom (reaction kinetics, validated math)

```yaml
plane: Crafting

dice_system:
  type: D20
  roll: "1d20 + crafting_modifier"
  succeed_if: "total >= recipe_dc"
  note: "DC depends on recipe complexity. Rare materials lower DC."

action_economy:
  type: TurnBased
  actions_per_turn: 1
  note: "Each crafting step is one action. Multi-step recipes require multiple turns."

resolution:
  type: DegreeOfSuccess
  degrees: ["catastrophic_failure", "failure", "success", "masterwork"]
  note: "Catastrophic failure destroys materials. Masterwork adds bonus properties."

available_actions:
  - id: "gather_materials"
    name: "Gather materials"
    cost: 1
    produces: "MaterialVertex"
    description: "Acquire components. Integrase capture probability applies for rare materials."
  - id: "prepare"
    name: "Prepare workspace"
    cost: 1
    produces: "PreparationVertex"
    description: "Set up for crafting. Lowers DC by preparation quality."
  - id: "craft_step"
    name: "Execute crafting step"
    cost: 1
    produces: "CraftVertex"
    description: "Perform one step of the recipe. Roll determines quality."
  - id: "experiment"
    name: "Experiment with unknown combination"
    cost: 1
    produces: "ExperimentVertex"
    description: "No recipe. Roll with disadvantage. Discovery possible on critical success."
  - id: "analyze_result"
    name: "Analyze what you made"
    cost: 1
    produces: "AnalysisVertex"
    description: "Understand the properties of your creation."

constraints:
  - rule: "Materials are consumed on use. Failure destroys materials on catastrophic_failure only."
    enforcement: hard
  - rule: "Recipes are discovered, not given. Experiment action can reveal new recipes."
    enforcement: hard
  - rule: "Crafted items become loamSpine certificates with full provenance."
    enforcement: hard

narration_style:
  tone: "Instructive, experimental, rewarding — the joy of making things"
  pacing: measured
  vocabulary: technical
  perspective: second_person
  sensory_emphasis: ["texture", "temperature", "smell", "sound of materials"]

ludospring_metrics:
  primary: "engagement"
  signals:
    - metric: "discovery_rate"
      description: "New recipes or combinations found"
      threshold_low: "0 in 10 minutes"
      action: "Hint at an untried combination via Encyclopedia or Perception voice"
```

---

## Plane 7: Card/Stack

**Source**: MTG rules (open game actions, exp047/048 validated)

```yaml
plane: CardStack

dice_system:
  type: None
  note: "Card games use deterministic draw from shuffled deck, not dice."

action_economy:
  type: TurnBased
  actions_per_turn: null
  note: "Phase-based: Untap, Upkeep, Draw, Main, Combat, Main, End. Priority passes between players."

resolution:
  type: Automatic
  note: "Card effects resolve per rules text. Stack resolves LIFO."

available_actions:
  - id: "play_card"
    name: "Play a card"
    cost: "mana cost"
    produces: "PlayVertex"
    description: "Card enters appropriate zone. Triggers placed on stack."
  - id: "activate_ability"
    name: "Activate ability"
    cost: "ability cost"
    produces: "ActivationVertex"
    description: "Ability placed on stack. Opponents receive priority."
  - id: "declare_attackers"
    name: "Declare attackers"
    cost: null
    produces: "CombatVertex"
    description: "Tap attacking creatures. Defenders will be declared after."
  - id: "respond"
    name: "Respond (instant/ability)"
    cost: "varies"
    produces: "ResponseVertex"
    description: "Add to stack in response to opponent's action."
  - id: "pass_priority"
    name: "Pass priority"
    cost: null
    produces: "PriorityVertex"
    description: "Decline to act. If all players pass, top of stack resolves."

constraints:
  - rule: "Stack resolves LIFO. Last added, first resolved."
    enforcement: hard
  - rule: "Each card is a loamSpine certificate. Zone transitions are DAG vertices."
    enforcement: hard
  - rule: "AI judge catches missed triggers and illegal plays."
    enforcement: hard

narration_style:
  tone: "Precise, mechanical, strategic"
  pacing: urgent
  vocabulary: technical
  perspective: third_person
  sensory_emphasis: ["card names", "zone changes", "stack state"]

ludospring_metrics:
  primary: "game_tree_complexity"
  signals:
    - metric: "tree_branching"
      description: "Available legal plays at current game state"
      action: "AI judge highlights key decision points for newer players"
```

---

## Plane Transition Rules

Transitions between planes follow these constraints:

| From | To | Trigger | DAG Vertex |
|------|----|---------|-----------|
| Any | Tactical | Violence initiated or encountered | `PlaneTransition { trigger: "combat" }` |
| Any | Dialogue | NPC addressed for conversation | `PlaneTransition { trigger: "dialogue" }` |
| Tactical | Any | All enemies defeated/fled/surrendered | `PlaneTransition { trigger: "combat_resolved" }` |
| Dialogue | Investigation | Clue mentioned or evidence presented | `PlaneTransition { trigger: "investigation" }` |
| Exploration | Crafting | Crafting station or materials used | `PlaneTransition { trigger: "crafting" }` |
| Any | Political | Faction negotiation scene | `PlaneTransition { trigger: "politics" }` |
| Any | CardStack | Card game initiated | `PlaneTransition { trigger: "card_game" }` |

World state carries across ALL transitions. NPC dispositions,
inventory, knowledge, conditions, and consequences persist.

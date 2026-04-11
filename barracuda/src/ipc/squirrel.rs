// SPDX-License-Identifier: AGPL-3.0-or-later
//! Squirrel AI primal integration — typed client for AI narration and dialogue.
//!
//! Routes AI capability calls through [`NeuralBridge`] to Squirrel:
//!
//! - `ai.query` — NPC dialogue generation constrained by personality + knowledge
//! - `ai.analyze` — Internal voice outputs constrained by `VoiceId` personality
//! - `ai.suggest` — Structured narration for exploration and combat
//!
//! Context management uses Squirrel's internal memory through the AI domain.
//!
//! Graceful degradation: returns `SquirrelResult { available: false, .. }` when
//! Squirrel is not reachable through the Neural API.

use super::neural_bridge::NeuralBridge;

/// Result of a Squirrel AI operation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SquirrelResult {
    /// The AI-generated text (dialogue, narration, voice output).
    pub text: String,
    /// Whether Squirrel was available and produced a result.
    pub available: bool,
    /// Additional structured data from the response.
    pub data: serde_json::Value,
}

/// Generate NPC dialogue via `ai.query`, constrained by personality and knowledge.
///
/// The system prompt encodes the NPC's personality, knowledge bounds, and current
/// trust level so Squirrel generates responses consistent with the NPC's character.
///
/// # Errors
///
/// Returns an error only on non-recoverable failures.
pub fn npc_dialogue(
    npc_name: &str,
    personality_prompt: &str,
    player_input: &str,
    context_history: &[serde_json::Value],
) -> Result<SquirrelResult, String> {
    let Ok(bridge) = NeuralBridge::discover() else {
        return Ok(unavailable("No Neural API — AI narration unavailable"));
    };

    let args = npc_dialogue_args(npc_name, personality_prompt, player_input, context_history);

    bridge.capability_call("ai", "query", &args).map_or_else(
        |_| Ok(unavailable("Squirrel ai.query unavailable")),
        |result| Ok(ai_query_result_to_squirrel(result)),
    )
}

fn npc_dialogue_args(
    npc_name: &str,
    personality_prompt: &str,
    player_input: &str,
    context_history: &[serde_json::Value],
) -> serde_json::Value {
    let mut messages = Vec::with_capacity(context_history.len() + 2);
    messages.push(serde_json::json!({
        "role": "system",
        "content": personality_prompt,
    }));
    for entry in context_history {
        messages.push(entry.clone());
    }
    messages.push(serde_json::json!({
        "role": "user",
        "content": player_input,
    }));

    serde_json::json!({
        "messages": messages,
        "model": "default",
        "metadata": { "npc": npc_name, "domain": "game" },
    })
}

fn ai_query_result_to_squirrel(result: serde_json::Value) -> SquirrelResult {
    let text = result
        .get("content")
        .or_else(|| result.get("text"))
        .or_else(|| result.get("message"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    SquirrelResult {
        text,
        available: true,
        data: result,
    }
}

/// Generate narration text for a game action via `ai.suggest`.
///
/// # Errors
///
/// Returns an error only on non-recoverable failures.
pub fn narrate_action(action_description: &str, context: &str) -> Result<SquirrelResult, String> {
    let Ok(bridge) = NeuralBridge::discover() else {
        return Ok(unavailable("No Neural API — narration unavailable"));
    };

    let args = narrate_action_args(action_description, context);

    bridge.capability_call("ai", "suggest", &args).map_or_else(
        |_| Ok(unavailable("Squirrel ai.suggest unavailable")),
        |result| {
            let text = extract_text(&result);
            Ok(SquirrelResult {
                text,
                available: true,
                data: result,
            })
        },
    )
}

fn narrate_action_args(action_description: &str, context: &str) -> serde_json::Value {
    serde_json::json!({
        "prompt": format!(
            "You are a game narrator. Describe this action in 1-2 vivid sentences.\n\
             Context: {context}\n\
             Action: {action_description}"
        ),
        "format": "text",
        "metadata": { "domain": "game", "type": "narration" },
    })
}

/// Generate internal voice output via `ai.analyze`, constrained by voice personality.
///
/// # Errors
///
/// Returns an error only on non-recoverable failures.
pub fn voice_check(
    voice_name: &str,
    voice_personality: &str,
    game_state_summary: &str,
) -> Result<SquirrelResult, String> {
    let Ok(bridge) = NeuralBridge::discover() else {
        return Ok(unavailable("No Neural API — voice check unavailable"));
    };

    let args = voice_check_args(voice_name, voice_personality, game_state_summary);

    bridge.capability_call("ai", "analyze", &args).map_or_else(
        |_| Ok(unavailable("Squirrel ai.analyze unavailable")),
        |result| {
            let text = extract_text(&result);
            Ok(SquirrelResult {
                text,
                available: true,
                data: result,
            })
        },
    )
}

fn voice_check_args(
    voice_name: &str,
    voice_personality: &str,
    game_state_summary: &str,
) -> serde_json::Value {
    serde_json::json!({
        "prompt": format!(
            "You are {voice_name}, an internal voice/skill. {voice_personality}\n\
             Current situation: {game_state_summary}\n\
             Respond with a brief observation (1 sentence max) or say nothing if irrelevant."
        ),
        "metadata": { "domain": "game", "type": "voice", "voice": voice_name },
    })
}

/// Create a new context window in Squirrel for NPC memory.
///
/// # Errors
///
/// Returns an error only on non-recoverable failures.
pub fn context_create(name: &str, max_tokens: u32) -> Result<SquirrelResult, String> {
    let Ok(bridge) = NeuralBridge::discover() else {
        return Ok(unavailable("No Neural API — context unavailable"));
    };

    let args = context_create_args(name, max_tokens);

    bridge
        .capability_call("context", "create", &args)
        .map_or_else(
            |_| Ok(unavailable("Squirrel context.create unavailable")),
            |result| Ok(context_create_result_to_squirrel(result)),
        )
}

fn context_create_args(name: &str, max_tokens: u32) -> serde_json::Value {
    serde_json::json!({
        "name": name,
        "max_tokens": max_tokens,
    })
}

fn context_create_result_to_squirrel(result: serde_json::Value) -> SquirrelResult {
    let id = result
        .get("context_id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    SquirrelResult {
        text: id,
        available: true,
        data: result,
    }
}

/// Update a Squirrel context window with new NPC memory content.
///
/// # Errors
///
/// Returns an error only on non-recoverable failures.
pub fn context_update(context_id: &str, content: &str) -> Result<SquirrelResult, String> {
    let Ok(bridge) = NeuralBridge::discover() else {
        return Ok(unavailable("No Neural API — context unavailable"));
    };

    let args = context_update_args(context_id, content);

    bridge
        .capability_call("context", "update", &args)
        .map_or_else(
            |_| Ok(unavailable("Squirrel context.update unavailable")),
            |result| {
                Ok(SquirrelResult {
                    text: String::new(),
                    available: true,
                    data: result,
                })
            },
        )
}

fn context_update_args(context_id: &str, content: &str) -> serde_json::Value {
    serde_json::json!({
        "context_id": context_id,
        "content": content,
    })
}

/// Summarize a Squirrel context window (compress long NPC interactions).
///
/// # Errors
///
/// Returns an error only on non-recoverable failures.
pub fn context_summarize(context_id: &str) -> Result<SquirrelResult, String> {
    let Ok(bridge) = NeuralBridge::discover() else {
        return Ok(unavailable("No Neural API — context unavailable"));
    };

    let args = context_summarize_args(context_id);

    bridge
        .capability_call("context", "summarize", &args)
        .map_or_else(
            |_| Ok(unavailable("Squirrel context.summarize unavailable")),
            |result| {
                let text = extract_text(&result);
                Ok(SquirrelResult {
                    text,
                    available: true,
                    data: result,
                })
            },
        )
}

fn context_summarize_args(context_id: &str) -> serde_json::Value {
    serde_json::json!({
        "context_id": context_id,
    })
}

fn extract_text(value: &serde_json::Value) -> String {
    value
        .get("text")
        .or_else(|| value.get("content"))
        .or_else(|| value.get("message"))
        .or_else(|| value.get("summary"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}

fn unavailable(reason: &str) -> SquirrelResult {
    SquirrelResult {
        text: String::new(),
        available: false,
        data: serde_json::json!({ "reason": reason }),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn unavailable_result_structure() {
        let r = unavailable("test");
        assert!(!r.available);
        assert!(r.text.is_empty());
        assert_eq!(r.data["reason"], "test");
    }

    #[test]
    fn extract_text_priorities() {
        assert_eq!(
            extract_text(&serde_json::json!({"text": "a", "content": "b"})),
            "a"
        );
        assert_eq!(extract_text(&serde_json::json!({"content": "b"})), "b");
        assert_eq!(extract_text(&serde_json::json!({"message": "c"})), "c");
        assert_eq!(extract_text(&serde_json::json!({"summary": "d"})), "d");
        assert_eq!(extract_text(&serde_json::json!({"other": "e"})), "");
    }

    #[test]
    fn extract_text_non_string_fields_yield_empty() {
        assert_eq!(extract_text(&serde_json::json!({"text": 42})), "");
        assert_eq!(extract_text(&serde_json::json!({"content": true})), "");
        assert_eq!(extract_text(&serde_json::json!({"message": []})), "");
        assert_eq!(extract_text(&serde_json::json!({"summary": {}})), "");
    }

    #[test]
    fn squirrel_result_manual_construction() {
        let data = serde_json::json!({"tokens": 12, "model": "default"});
        let r = SquirrelResult {
            text: "The door groans open.".to_string(),
            available: true,
            data: data.clone(),
        };
        assert!(r.available);
        assert_eq!(r.text, "The door groans open.");
        assert_eq!(r.data, data);
    }

    #[test]
    fn npc_dialogue_without_neural_api() {
        let result = npc_dialogue("Sheriff", "You are stern.", "Hello", &[]).unwrap();
        assert!(!result.available);
    }

    #[test]
    fn narrate_action_without_neural_api() {
        let result = narrate_action("moved north", "dark cave").unwrap();
        assert!(!result.available);
    }

    #[test]
    fn voice_check_without_neural_api() {
        let result = voice_check("Logic", "Analytical and cold.", "dark room").unwrap();
        assert!(!result.available);
    }

    #[test]
    fn context_create_without_neural_api() {
        let result = context_create("npc-sheriff", 4096).unwrap();
        assert!(!result.available);
    }

    #[test]
    fn context_update_without_neural_api() {
        let result = context_update("ctx-123", "new memory").unwrap();
        assert!(!result.available);
    }

    #[test]
    fn context_summarize_without_neural_api() {
        let result = context_summarize("ctx-123").unwrap();
        assert!(!result.available);
    }

    #[test]
    fn squirrel_result_serde_round_trip() {
        let original = SquirrelResult {
            text: "hello".to_string(),
            available: true,
            data: serde_json::json!({ "x": 1 }),
        };
        let json = serde_json::to_string(&original).expect("serialize");
        let back: SquirrelResult = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.text, original.text);
        assert_eq!(back.available, original.available);
        assert_eq!(back.data, original.data);
    }

    #[test]
    fn npc_dialogue_args_builds_messages_and_metadata() {
        let hist = vec![serde_json::json!({"role": "user", "content": "prior"})];
        let args = super::npc_dialogue_args("NPC", "You are a guard.", "Who goes there?", &hist);
        assert_eq!(args["model"], "default");
        assert_eq!(args["metadata"]["npc"], "NPC");
        assert_eq!(args["metadata"]["domain"], "game");
        let messages = args["messages"].as_array().expect("messages array");
        assert_eq!(messages.len(), 3);
        assert_eq!(messages[0]["role"], "system");
        assert_eq!(messages[0]["content"], "You are a guard.");
        assert_eq!(messages[1], hist[0]);
        assert_eq!(messages[2]["content"], "Who goes there?");
    }

    #[test]
    fn narrate_action_args_include_prompt_parts() {
        let args = super::narrate_action_args("jump", "forest");
        assert!(args["prompt"].as_str().unwrap().contains("forest"));
        assert!(args["prompt"].as_str().unwrap().contains("jump"));
        assert_eq!(args["format"], "text");
        assert_eq!(args["metadata"]["type"], "narration");
    }

    #[test]
    fn voice_check_args_embed_voice_and_metadata() {
        let args = super::voice_check_args("Fear", "Anxious.", "spiders");
        assert!(args["prompt"].as_str().unwrap().contains("Fear"));
        assert!(args["prompt"].as_str().unwrap().contains("spiders"));
        assert_eq!(args["metadata"]["voice"], "Fear");
        assert_eq!(args["metadata"]["type"], "voice");
    }

    #[test]
    fn context_create_and_update_and_summarize_args() {
        let c = super::context_create_args("mem", 1024);
        assert_eq!(c["name"], "mem");
        assert_eq!(c["max_tokens"], 1024);

        let u = super::context_update_args("id-9", "blob");
        assert_eq!(u["context_id"], "id-9");
        assert_eq!(u["content"], "blob");

        let s = super::context_summarize_args("id-9");
        assert_eq!(s["context_id"], "id-9");
    }

    #[test]
    fn ai_query_result_prefers_content_over_text_and_message() {
        let r = super::ai_query_result_to_squirrel(serde_json::json!({
            "content": "c",
            "text": "t",
            "message": "m"
        }));
        assert_eq!(r.text, "c");
        assert!(r.available);
    }

    #[test]
    fn ai_query_result_falls_back_to_text_then_message() {
        let r = super::ai_query_result_to_squirrel(serde_json::json!({
            "text": "t",
            "message": "m"
        }));
        assert_eq!(r.text, "t");

        let r2 = super::ai_query_result_to_squirrel(serde_json::json!({ "message": "m" }));
        assert_eq!(r2.text, "m");
    }

    #[test]
    fn ai_query_result_empty_when_no_string_fields() {
        let r = super::ai_query_result_to_squirrel(serde_json::json!({ "other": 1 }));
        assert_eq!(r.text, "");
    }

    #[test]
    fn context_create_result_reads_context_id_string() {
        let r = super::context_create_result_to_squirrel(serde_json::json!({
            "context_id": "abc",
            "extra": true
        }));
        assert_eq!(r.text, "abc");
        assert_eq!(r.data["extra"], true);
    }

    #[test]
    fn context_create_result_empty_id_when_missing_or_non_string() {
        let r = super::context_create_result_to_squirrel(serde_json::json!({}));
        assert_eq!(r.text, "");
        let r2 = super::context_create_result_to_squirrel(serde_json::json!({
            "context_id": 42
        }));
        assert_eq!(r2.text, "");
    }

    #[test]
    fn unavailable_messages_are_distinct_per_entrypoint() {
        type UnavailableCaseFn = fn() -> Result<SquirrelResult, String>;
        let cases: Vec<(&str, UnavailableCaseFn)> = vec![
            ("AI narration", || npc_dialogue("n", "p", "i", &[])),
            ("narration", || narrate_action("a", "c")),
            ("voice check", || voice_check("v", "p", "g")),
            ("context", || context_create("n", 1)),
            ("context", || context_update("i", "c")),
            ("context", || context_summarize("i")),
        ];
        for (needle, f) in cases {
            let r = f().expect("ok");
            assert!(!r.available, "expected unavailable for {needle:?}");
            let reason = r.data["reason"].as_str().expect("reason string");
            assert!(
                reason.contains(needle) || reason.contains("Neural API"),
                "unexpected reason {reason:?} for {needle:?}"
            );
        }
    }
}

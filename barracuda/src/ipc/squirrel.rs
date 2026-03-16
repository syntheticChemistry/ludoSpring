// SPDX-License-Identifier: AGPL-3.0-or-later
//! Squirrel AI primal integration — typed client for AI narration and dialogue.
//!
//! Routes AI capability calls through [`NeuralBridge`] to Squirrel:
//!
//! - `ai.chat` — NPC dialogue generation constrained by personality + knowledge
//! - `ai.inference` — Internal voice outputs constrained by `VoiceId` personality
//! - `context.create` / `context.update` / `context.summarize` — NPC memory context
//! - `ai.text_generation` — Structured narration for exploration and combat
//!
//! Graceful degradation: returns `SquirrelResult { available: false, .. }` when
//! Squirrel is not reachable through the Neural API.

use super::neural_bridge::NeuralBridge;

/// Result of a Squirrel AI operation.
#[derive(Debug, Clone)]
pub struct SquirrelResult {
    /// The AI-generated text (dialogue, narration, voice output).
    pub text: String,
    /// Whether Squirrel was available and produced a result.
    pub available: bool,
    /// Additional structured data from the response.
    pub data: serde_json::Value,
}

/// Generate NPC dialogue via `ai.chat`, constrained by personality and knowledge.
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

    let args = serde_json::json!({
        "messages": messages,
        "model": "default",
        "metadata": { "npc": npc_name, "domain": "game" },
    });

    bridge.capability_call("ai", "chat", &args).map_or_else(
        |_| Ok(unavailable("Squirrel ai.chat unavailable")),
        |result| {
            let text = result
                .get("content")
                .or_else(|| result.get("text"))
                .or_else(|| result.get("message"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            Ok(SquirrelResult {
                text,
                available: true,
                data: result,
            })
        },
    )
}

/// Generate narration text for a game action via `ai.text_generation`.
///
/// # Errors
///
/// Returns an error only on non-recoverable failures.
pub fn narrate_action(
    action_description: &str,
    context: &str,
) -> Result<SquirrelResult, String> {
    let Ok(bridge) = NeuralBridge::discover() else {
        return Ok(unavailable("No Neural API — narration unavailable"));
    };

    let args = serde_json::json!({
        "prompt": format!(
            "You are a game narrator. Describe this action in 1-2 vivid sentences.\n\
             Context: {context}\n\
             Action: {action_description}"
        ),
        "format": "text",
        "metadata": { "domain": "game", "type": "narration" },
    });

    bridge
        .capability_call("ai", "text_generation", &args)
        .map_or_else(
            |_| Ok(unavailable("Squirrel ai.text_generation unavailable")),
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

/// Generate internal voice output via `ai.inference`, constrained by voice personality.
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

    let args = serde_json::json!({
        "prompt": format!(
            "You are {voice_name}, an internal voice/skill. {voice_personality}\n\
             Current situation: {game_state_summary}\n\
             Respond with a brief observation (1 sentence max) or say nothing if irrelevant."
        ),
        "metadata": { "domain": "game", "type": "voice", "voice": voice_name },
    });

    bridge
        .capability_call("ai", "inference", &args)
        .map_or_else(
            |_| Ok(unavailable("Squirrel ai.inference unavailable")),
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

/// Create a new context window in Squirrel for NPC memory.
///
/// # Errors
///
/// Returns an error only on non-recoverable failures.
pub fn context_create(name: &str, max_tokens: u32) -> Result<SquirrelResult, String> {
    let Ok(bridge) = NeuralBridge::discover() else {
        return Ok(unavailable("No Neural API — context unavailable"));
    };

    let args = serde_json::json!({
        "name": name,
        "max_tokens": max_tokens,
    });

    bridge
        .capability_call("context", "create", &args)
        .map_or_else(
            |_| Ok(unavailable("Squirrel context.create unavailable")),
            |result| {
                let id = result
                    .get("context_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                Ok(SquirrelResult {
                    text: id,
                    available: true,
                    data: result,
                })
            },
        )
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

    let args = serde_json::json!({
        "context_id": context_id,
        "content": content,
    });

    bridge
        .capability_call("context", "update", &args)
        .map_or_else(
            |_| Ok(unavailable("Squirrel context.update unavailable")),
            |result| Ok(SquirrelResult {
                text: String::new(),
                available: true,
                data: result,
            }),
        )
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

    let args = serde_json::json!({
        "context_id": context_id,
    });

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
}

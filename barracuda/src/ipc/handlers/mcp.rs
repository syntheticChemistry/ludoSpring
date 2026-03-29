// SPDX-License-Identifier: AGPL-3.0-or-later
//! MCP `tools.list` / `tools.call` handlers.

use crate::ipc::envelope::{JsonRpcError, JsonRpcRequest};
use crate::ipc::params::ToolsCallParams;
use crate::ipc::{
    METHOD_ACCESSIBILITY, METHOD_ANALYZE_UI, METHOD_BEGIN_SESSION, METHOD_COMPLETE_SESSION,
    METHOD_DIFFICULTY_ADJUSTMENT, METHOD_ENGAGEMENT, METHOD_EVALUATE_FLOW, METHOD_FITTS_COST,
    METHOD_GENERATE_NOISE, METHOD_NARRATE_ACTION, METHOD_NPC_DIALOGUE, METHOD_PUSH_SCENE,
    METHOD_WFC_STEP,
};

use super::delegation::{
    handle_begin_session, handle_complete_session, handle_narrate_action, handle_npc_dialogue,
    handle_push_scene,
};
use super::science::{
    handle_accessibility, handle_analyze_ui, handle_difficulty_adjustment, handle_engagement,
    handle_evaluate_flow, handle_fitts_cost, handle_generate_noise, handle_wfc_step,
};
use super::{HandlerResult, parse_params, to_json};

/// MCP `tools.list` — tool descriptors for Squirrel / AI discovery (JSON Schema inputs).
pub(super) fn handle_tools_list(req: &JsonRpcRequest) -> HandlerResult {
    to_json(&req.id, mcp_tools_descriptors())
}

/// MCP `tools.call` — dispatch by tool name into the same handlers as `game.*` methods.
pub(super) fn handle_tools_call(req: &JsonRpcRequest) -> HandlerResult {
    let p: ToolsCallParams = parse_params(req)?;
    let inner = JsonRpcRequest {
        jsonrpc: req.jsonrpc.clone(),
        method: p.name.clone(),
        params: Some(p.arguments),
        id: req.id.clone(),
    };
    match p.name.as_str() {
        METHOD_EVALUATE_FLOW => handle_evaluate_flow(&inner),
        METHOD_FITTS_COST => handle_fitts_cost(&inner),
        METHOD_ENGAGEMENT => handle_engagement(&inner),
        METHOD_GENERATE_NOISE => handle_generate_noise(&inner),
        METHOD_ANALYZE_UI => handle_analyze_ui(&inner),
        METHOD_ACCESSIBILITY => handle_accessibility(&inner),
        METHOD_WFC_STEP => handle_wfc_step(&inner),
        METHOD_DIFFICULTY_ADJUSTMENT => handle_difficulty_adjustment(&inner),
        METHOD_BEGIN_SESSION => handle_begin_session(&inner),
        METHOD_COMPLETE_SESSION => handle_complete_session(&inner),
        METHOD_NPC_DIALOGUE => handle_npc_dialogue(&inner),
        METHOD_NARRATE_ACTION => handle_narrate_action(&inner),
        METHOD_PUSH_SCENE => handle_push_scene(&inner),
        _ => Err(JsonRpcError::method_not_found(&req.id, &p.name)),
    }
}

/// JSON array of MCP tool descriptors (name, description, `input_schema` per tool).
#[expect(
    clippy::too_many_lines,
    reason = "large declarative JSON Schema catalog for MCP tools.list"
)]
pub(super) fn mcp_tools_descriptors() -> serde_json::Value {
    serde_json::json!([
        {
            "name": METHOD_EVALUATE_FLOW,
            "description": "Csikszentmihalyi-style flow: classify state from challenge vs skill in a normalized channel.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "challenge": { "type": "number", "description": "Normalized challenge level (0.0–1.0)." },
                    "skill": { "type": "number", "description": "Normalized player skill (0.0–1.0)." },
                    "channel_width": { "type": "number", "description": "Optional flow channel width (defaults to primal tolerance)." }
                },
                "required": ["challenge", "skill"]
            }
        },
        {
            "name": METHOD_FITTS_COST,
            "description": "Fitts's law: movement time and index of difficulty for a pointing target.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "distance": { "type": "number", "description": "Distance to target center (pixels or consistent units)." },
                    "target_width": { "type": "number", "description": "Target width along the movement axis." },
                    "a": { "type": "number", "description": "Optional Fitts intercept parameter (ms)." },
                    "b": { "type": "number", "description": "Optional Fitts slope parameter (ms per ID)." }
                },
                "required": ["distance", "target_width"]
            }
        },
        {
            "name": METHOD_ENGAGEMENT,
            "description": "Composite engagement metrics from a short behavioral snapshot.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "session_duration_s": { "type": "number" },
                    "action_count": { "type": "integer", "minimum": 0 },
                    "exploration_breadth": { "type": "integer", "minimum": 0 },
                    "challenge_seeking": { "type": "integer", "minimum": 0 },
                    "retry_count": { "type": "integer", "minimum": 0 },
                    "deliberate_pauses": { "type": "integer", "minimum": 0 }
                },
                "required": [
                    "session_duration_s", "action_count", "exploration_breadth",
                    "challenge_seeking", "retry_count", "deliberate_pauses"
                ]
            }
        },
        {
            "name": METHOD_GENERATE_NOISE,
            "description": "Fractional Brownian motion noise sample at 2D or 3D coordinates.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "x": { "type": "number" },
                    "y": { "type": "number" },
                    "z": { "type": "number", "description": "If set, uses 3D fBm." },
                    "octaves": { "type": "integer", "minimum": 1 },
                    "lacunarity": { "type": "number" },
                    "persistence": { "type": "number" }
                },
                "required": ["x", "y"]
            }
        },
        {
            "name": METHOD_ANALYZE_UI,
            "description": "Tufte-style data-ink and density analysis for game HUD / UI elements.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "elements": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "name": { "type": "string" },
                                "bounds": {
                                    "type": "array",
                                    "items": { "type": "number" },
                                    "minItems": 4,
                                    "maxItems": 4
                                },
                                "data_values": { "type": "integer", "minimum": 0 },
                                "pixel_area": { "type": "number" },
                                "data_ink_area": { "type": "number" },
                                "critical": { "type": "boolean" }
                            },
                            "required": [
                                "name", "bounds", "data_values", "pixel_area",
                                "data_ink_area", "critical"
                            ]
                        }
                    }
                },
                "required": ["elements"]
            }
        },
        {
            "name": METHOD_ACCESSIBILITY,
            "description": "Score visual accessibility feature coverage (IGDA / XAG-aligned flags).",
            "input_schema": {
                "type": "object",
                "properties": {
                    "audio_cues": { "type": "boolean" },
                    "descriptions": { "type": "boolean" },
                    "braille": { "type": "boolean" },
                    "haptic": { "type": "boolean" },
                    "color_independent": { "type": "boolean" },
                    "scalable_text": { "type": "boolean" }
                },
                "required": [
                    "audio_cues", "descriptions", "braille", "haptic",
                    "color_independent", "scalable_text"
                ]
            }
        },
        {
            "name": METHOD_WFC_STEP,
            "description": "One wave-function-collapse propagation step on a tile grid.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "width": { "type": "integer", "minimum": 1 },
                    "height": { "type": "integer", "minimum": 1 },
                    "n_tiles": { "type": "integer", "minimum": 1 },
                    "collapse": {
                        "type": "array",
                        "items": { "type": "integer", "minimum": 0 },
                        "minItems": 3,
                        "maxItems": 3,
                        "description": "Optional [x, y, tile_id] collapse before propagate."
                    }
                },
                "required": ["width", "height", "n_tiles"]
            }
        },
        {
            "name": METHOD_DIFFICULTY_ADJUSTMENT,
            "description": "Suggest difficulty adjustment from a window of binary outcomes vs target success rate.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "outcomes": {
                        "type": "array",
                        "items": { "type": "number", "minimum": 0.0, "maximum": 1.0 }
                    },
                    "target_success_rate": { "type": "number", "description": "Optional DDA target (default from tolerances)." }
                },
                "required": ["outcomes"]
            }
        },
        {
            "name": METHOD_BEGIN_SESSION,
            "description": "Begin a provenance-tracked game session.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "session_name": { "type": "string", "description": "Human-readable session name." }
                },
                "required": ["session_name"]
            }
        },
        {
            "name": METHOD_COMPLETE_SESSION,
            "description": "Complete and seal a game session.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "session_id": { "type": "string", "description": "Session ID from game.begin_session." }
                },
                "required": ["session_id"]
            }
        },
        {
            "name": METHOD_NPC_DIALOGUE,
            "description": "NPC dialogue via AI capability delegation.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "npc_name": { "type": "string", "description": "NPC name for logging and context." },
                    "personality_prompt": { "type": "string", "description": "System prompt for personality and knowledge bounds." },
                    "player_input": { "type": "string", "description": "Player dialogue input." },
                    "history": {
                        "type": "array",
                        "description": "Optional conversation history (role/content objects).",
                        "items": { "type": "object" }
                    }
                },
                "required": ["npc_name", "personality_prompt", "player_input"]
            }
        },
        {
            "name": METHOD_NARRATE_ACTION,
            "description": "Narrate a game action via AI capability delegation.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "action": { "type": "string", "description": "Description of the action to narrate." },
                    "context": { "type": "string", "description": "Surrounding context for the narration." }
                },
                "required": ["action", "context"]
            }
        },
        {
            "name": METHOD_PUSH_SCENE,
            "description": "Push a scene to the visualization capability.",
            "input_schema": {
                "type": "object",
                "properties": {
                    "session_id": { "type": "string", "description": "Session ID for visualization routing." },
                    "channel": { "type": "string", "description": "Channel name (e.g. DialogueTree, CombatGrid)." },
                    "scene": { "description": "Scene payload (any JSON value)." }
                },
                "required": ["session_id", "channel", "scene"]
            }
        }
    ])
}

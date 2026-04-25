// SPDX-License-Identifier: AGPL-3.0-or-later
//! Provenance, Squirrel, NestGate, visualization, and telemetry delegation handlers.

use crate::ipc::params::{
    BeginSessionParams, CompleteSessionParams, MintCertificateParams, NarrateActionParams,
    NpcDialogueParams, PushSceneParams, QueryVerticesParams, RecordActionParams, StorageGetParams,
    StoragePutParams, VoiceCheckParams,
};
use crate::ipc::{nestgate, provenance, squirrel};

use crate::ipc::envelope::{JsonRpcError, JsonRpcRequest};

use super::{HandlerResult, parse_params, to_json};

/// `model` and `tokens` for Webb-compatible chat payloads; reads Squirrel `data` when present.
fn squirrel_chat_metadata(data: &serde_json::Value) -> (String, u32) {
    let model = data
        .get("model")
        .and_then(serde_json::Value::as_str)
        .map_or_else(|| "local".to_owned(), str::to_owned);
    #[expect(
        clippy::cast_possible_truncation,
        reason = "clamped to u32::MAX before cast"
    )]
    let tokens = data
        .pointer("/usage/total_tokens")
        .or_else(|| data.get("total_tokens"))
        .or_else(|| data.get("tokens"))
        .and_then(serde_json::Value::as_u64)
        .map_or(0, |n| n.min(u64::from(u32::MAX)) as u32);
    (model, tokens)
}

pub(super) fn handle_begin_session(req: &JsonRpcRequest) -> HandlerResult {
    let p: BeginSessionParams = parse_params(req)?;
    let result = provenance::begin_game_session(&p.session_name)
        .map_err(|e| JsonRpcError::internal(&req.id, &e))?;
    to_json(
        &req.id,
        serde_json::json!({
            "session_id": result.id,
            "provenance": if result.available { "available" } else { "unavailable" },
            "data": result.data,
        }),
    )
}

pub(super) fn handle_record_action(req: &JsonRpcRequest) -> HandlerResult {
    let p: RecordActionParams = parse_params(req)?;
    let result = provenance::record_game_action(&p.session_id, &p.action)
        .map_err(|e| JsonRpcError::internal(&req.id, &e))?;
    to_json(
        &req.id,
        serde_json::json!({
            "vertex_id": result.id,
            "provenance": if result.available { "available" } else { "unavailable" },
            "data": result.data,
        }),
    )
}

pub(super) fn handle_complete_session(req: &JsonRpcRequest) -> HandlerResult {
    let p: CompleteSessionParams = parse_params(req)?;
    let result = provenance::complete_game_session(&p.session_id)
        .map_err(|e| JsonRpcError::internal(&req.id, &e))?;
    to_json(&req.id, result)
}

pub(super) fn handle_npc_dialogue(req: &JsonRpcRequest) -> HandlerResult {
    let p: NpcDialogueParams = parse_params(req)?;
    let result = squirrel::npc_dialogue(
        &p.npc_name,
        &p.personality_prompt,
        &p.player_input,
        &p.history,
    )
    .map_err(|e| JsonRpcError::internal(&req.id, &e))?;
    to_json(
        &req.id,
        serde_json::json!({
            "text": result.text,
            "available": result.available,
            "data": result.data,
            "voice_notes": [],
            "passive_checks_fired": false,
            "degraded": !result.available,
        }),
    )
}

pub(super) fn handle_narrate_action(req: &JsonRpcRequest) -> HandlerResult {
    let p: NarrateActionParams = parse_params(req)?;
    let result = squirrel::narrate_action(&p.action, &p.context)
        .map_err(|e| JsonRpcError::internal(&req.id, &e))?;
    let (model, tokens) = squirrel_chat_metadata(&result.data);
    to_json(
        &req.id,
        serde_json::json!({
            "text": result.text,
            "available": result.available,
            "model": model,
            "tokens": tokens,
        }),
    )
}

pub(super) fn handle_voice_check(req: &JsonRpcRequest) -> HandlerResult {
    let p: VoiceCheckParams = parse_params(req)?;
    let result = squirrel::voice_check(&p.voice_name, &p.voice_personality, &p.game_state)
        .map_err(|e| JsonRpcError::internal(&req.id, &e))?;
    to_json(
        &req.id,
        serde_json::json!({
            "text": result.text,
            "available": result.available,
            "voice": p.voice_name,
        }),
    )
}

pub(super) fn handle_push_scene(req: &JsonRpcRequest) -> HandlerResult {
    let p: PushSceneParams = parse_params(req)?;

    let mut pushed = false;
    let mut push_error: Option<String> = None;

    #[cfg(feature = "ipc")]
    {
        use crate::visualization::VisualizationPushClient;
        match VisualizationPushClient::discover() {
            Ok(client) => match client.push_scene(&p.session_id, &p.channel, &p.scene) {
                Ok(()) => pushed = true,
                Err(e) => push_error = Some(e.to_string()),
            },
            Err(e) => push_error = Some(format!("visualization not discovered: {e}")),
        }
    }

    to_json(
        &req.id,
        serde_json::json!({
            "pushed": pushed,
            "session_id": p.session_id,
            "channel": p.channel,
            "error": push_error,
        }),
    )
}

pub(super) fn handle_query_vertices(req: &JsonRpcRequest) -> HandlerResult {
    let p: QueryVerticesParams = parse_params(req)?;
    let result = provenance::query_vertices(
        &p.session_id,
        p.event_type.as_deref(),
        p.agent.as_deref(),
        p.limit,
    )
    .map_err(|e| JsonRpcError::internal(&req.id, &e))?;
    to_json(
        &req.id,
        serde_json::json!({
            "available": result.available,
            "vertices": result.data,
        }),
    )
}

pub(super) fn handle_mint_certificate(req: &JsonRpcRequest) -> HandlerResult {
    let p: MintCertificateParams = parse_params(req)?;
    let result = provenance::mint_certificate(&p.cert_type, &p.owner, &p.payload)
        .map_err(|e| JsonRpcError::internal(&req.id, &e))?;
    to_json(
        &req.id,
        serde_json::json!({
            "cert_id": result.id,
            "available": result.available,
            "data": result.data,
        }),
    )
}

pub(super) fn handle_storage_put(req: &JsonRpcRequest) -> HandlerResult {
    let p: StoragePutParams = parse_params(req)?;
    let result = nestgate::put(&p.key, &p.data, &p.metadata)
        .map_err(|e| JsonRpcError::internal(&req.id, &e))?;
    to_json(
        &req.id,
        serde_json::json!({
            "available": result.available,
            "data": result.data,
        }),
    )
}

pub(super) fn handle_storage_get(req: &JsonRpcRequest) -> HandlerResult {
    let p: StorageGetParams = parse_params(req)?;
    let result = nestgate::get(&p.key).map_err(|e| JsonRpcError::internal(&req.id, &e))?;
    to_json(
        &req.id,
        serde_json::json!({
            "available": result.available,
            "data": result.data,
        }),
    )
}

pub(super) fn handle_poll_telemetry(req: &JsonRpcRequest) -> HandlerResult {
    use crate::telemetry::mapper::SessionAccumulator;

    let tick_ns = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_nanos());

    let has_active_session = provenance::has_active_session();
    let status = if has_active_session {
        "streaming"
    } else {
        "idle"
    };

    let accumulator = SessionAccumulator::new();
    let snapshot = accumulator.to_engagement_snapshot();
    let engagement = crate::metrics::engagement::compute_engagement(&snapshot);

    to_json(
        &req.id,
        serde_json::json!({
            "events": [{
                "type": "engagement_snapshot",
                "composite": engagement.composite,
                "actions_per_minute": engagement.actions_per_minute,
                "exploration_rate": engagement.exploration_rate,
            }],
            "tick_ns": tick_ns,
            "status": status,
            "domain": crate::niche::NICHE_DOMAIN,
            "frame_budget_ms": 1000.0 / crate::tolerances::TARGET_FRAME_RATE_HZ,
        }),
    )
}

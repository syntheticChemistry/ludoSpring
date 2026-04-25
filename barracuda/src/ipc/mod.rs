// SPDX-License-Identifier: AGPL-3.0-or-later
//! JSON-RPC 2.0 IPC for ludoSpring — server and client.
//!
//! ## Server
//!
//! Exposes game science capabilities via capability-based discovery.
//! Per wateringHole `UNIVERSAL_IPC_STANDARD_V3`, each primal implements
//! IPC independently (~500–1000 lines).
//!
//! **Domain capabilities** (27 FQNs in [`crate::niche::CAPABILITIES`]): game science
//! (`game.evaluate_flow`, `game.fitts_cost`, `game.engagement`, `game.analyze_ui`,
//! `game.accessibility`, `game.wfc_step`, `game.difficulty_adjustment`,
//! `game.generate_noise`); provenance and telemetry (`game.begin_session`,
//! `game.record_action`, `game.complete_session`, `game.poll_telemetry`); Squirrel
//! delegation (`game.npc_dialogue`, `game.narrate_action`, `game.voice_check`);
//! petalTongue (`game.push_scene`); rhizoCrypt (`game.query_vertices`); loamSpine
//! (`game.mint_certificate`); NestGate (`game.storage_put`, `game.storage_get`);
//! GPU (`game.gpu.fog_of_war`, `game.gpu.tile_lighting`, `game.gpu.pathfind`,
//! `game.gpu.perlin_terrain`, `game.gpu.batch_raycast`); health probes (`health.liveness`, `health.readiness`).
//!
//! **MCP**: `tools.list` / `tools.call` expose the same game operations for AI discovery
//! and invocation (science plus key delegation entry points).
//!
//! **Additional routes** (not counted in the 27): `health.check` (and legacy aliases),
//! `lifecycle.*`, `capability.*`, neural visualization delegation to petalTongue, and optional tarpc.
//!
//! ## Client (Discovery)
//!
//! Discovers live primals by probing Unix sockets in standard directories.
//! Primals are found by **capability**, never by hardcoded name or path.
//!
//! ## Transport
//!
//! Unix domain socket, XDG-compliant path resolution (overridable via env).
//! Protocol: newline-delimited JSON-RPC 2.0.

pub mod btsp;
pub mod methods;

mod envelope;
mod handlers;
mod neural_bridge;
mod params;
mod results;
mod server;

#[cfg(feature = "tarpc-ipc")]
pub mod tarpc_service;

pub mod composition;
pub mod coralreef;
pub mod discovery;
pub mod nestgate;
pub mod provenance;
pub mod squirrel;
pub mod toadstool;

pub use discovery::{
    DiscoveryResult, DiscoveryTier, PrimalEndpoint, PrimalRegistry, call_primal,
    discover_by_capability, discover_primal_tiered, discover_primals,
    discover_primals_in_directories, probe_socket,
};
pub use envelope::{
    DispatchOutcome, IpcError, IpcErrorPhase, JsonRpcError, JsonRpcRequest, JsonRpcResponse,
    PhasedIpcError, RpcErrorBody, classify_io_error, extract_rpc_result, normalize_method,
};
pub use handlers::dispatch;
pub use neural_bridge::NeuralBridge;
pub use params::*;
pub use results::*;
pub use server::IpcServer;

/// JSON-RPC method names — capability identifiers for routing.
pub const METHOD_ANALYZE_UI: &str = "game.analyze_ui";
/// Evaluate flow state.
pub const METHOD_EVALUATE_FLOW: &str = "game.evaluate_flow";
/// Compute Fitts's law cost.
pub const METHOD_FITTS_COST: &str = "game.fitts_cost";
/// Compute engagement metrics.
pub const METHOD_ENGAGEMENT: &str = "game.engagement";
/// Generate noise field.
pub const METHOD_GENERATE_NOISE: &str = "game.generate_noise";
/// Wave function collapse step.
pub const METHOD_WFC_STEP: &str = "game.wfc_step";
/// Accessibility scoring.
pub const METHOD_ACCESSIBILITY: &str = "game.accessibility";
/// Dynamic difficulty adjustment recommendation.
pub const METHOD_DIFFICULTY_ADJUSTMENT: &str = "game.difficulty_adjustment";
/// Begin game session (provenance trio).
pub const METHOD_BEGIN_SESSION: &str = "game.begin_session";
/// Record game action (provenance trio).
pub const METHOD_RECORD_ACTION: &str = "game.record_action";
/// Complete game session (provenance trio).
pub const METHOD_COMPLETE_SESSION: &str = "game.complete_session";
/// Poll telemetry events (continuous coordination).
pub const METHOD_POLL_TELEMETRY: &str = "game.poll_telemetry";
/// NPC dialogue via Squirrel AI.
pub const METHOD_NPC_DIALOGUE: &str = "game.npc_dialogue";
/// Narrate a game action via Squirrel AI.
pub const METHOD_NARRATE_ACTION: &str = "game.narrate_action";
/// Internal voice check via Squirrel AI.
pub const METHOD_VOICE_CHECK: &str = "game.voice_check";
/// Push game scene to petalTongue.
pub const METHOD_PUSH_SCENE: &str = "game.push_scene";
/// Query DAG vertices (NPC memory).
pub const METHOD_QUERY_VERTICES: &str = "game.query_vertices";
/// Mint a loamSpine certificate.
pub const METHOD_MINT_CERTIFICATE: &str = "game.mint_certificate";
/// Store game data in NestGate.
pub const METHOD_STORAGE_PUT: &str = "game.storage_put";
/// Retrieve game data from NestGate.
pub const METHOD_STORAGE_GET: &str = "game.storage_get";

/// MCP tools/list — enumerate AI-callable game analysis tools.
pub const METHOD_TOOLS_LIST: &str = "tools.list";
/// MCP tools/call — invoke a game analysis tool by name.
pub const METHOD_TOOLS_CALL: &str = "tools.call";

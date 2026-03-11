// SPDX-License-Identifier: AGPL-3.0-or-later
//! JSON-RPC 2.0 IPC for ludoSpring — server and client.
//!
//! ## Server
//!
//! Exposes game science capabilities via capability-based discovery.
//! Per wateringHole `UNIVERSAL_IPC_STANDARD_V3`, each primal implements
//! IPC independently (~500–1000 lines).
//!
//! Methods: `game.analyze_ui`, `game.evaluate_flow`, `game.fitts_cost`,
//! `game.engagement`, `game.generate_noise`, `game.wfc_step`,
//! `game.accessibility`, `game.difficulty_adjustment`.
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

mod envelope;
mod handlers;
mod params;
mod results;
mod server;

pub mod discovery;

pub use discovery::{PrimalEndpoint, PrimalRegistry, call_primal, discover_primals};
pub use envelope::{JsonRpcError, JsonRpcRequest, JsonRpcResponse, RpcErrorBody};
pub use handlers::dispatch;
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

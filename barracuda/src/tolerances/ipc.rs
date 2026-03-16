// SPDX-License-Identifier: AGPL-3.0-or-later
//! IPC tolerances — timeouts, probe intervals.

/// Default timeout for JSON-RPC calls to peer primals (seconds).
///
/// Justification: biomeOS graph nodes use 15s timeouts for germination;
/// per-call RPC should be substantially shorter. 5s accommodates cold
/// startup of AI providers (Ollama model load) while failing fast on
/// network issues.
pub const RPC_TIMEOUT_SECS: u64 = 5;

/// Probe timeout for socket capability verification (milliseconds).
///
/// Justification: Socket probing during discovery should be fast to avoid
/// blocking startup. 500ms is enough for a local Unix socket round-trip
/// including `lifecycle.status` parsing.
pub const PROBE_TIMEOUT_MS: u64 = 500;

/// Connect-probe timeout for quick liveness checks (milliseconds).
///
/// Justification: A pure TCP/Unix connect check without full RPC. 200ms
/// is generous for loopback connections.
pub const CONNECT_PROBE_TIMEOUT_MS: u64 = 200;

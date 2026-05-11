// SPDX-License-Identifier: AGPL-3.0-or-later
//! IPC tolerances — timeouts, probe intervals.
//!
//! All values can be overridden via environment variables for deployment
//! tuning without recompilation.

const DEFAULT_RPC_TIMEOUT_SECS: u64 = 5;
const DEFAULT_PROBE_TIMEOUT_MS: u64 = 500;
const DEFAULT_CONNECT_PROBE_TIMEOUT_MS: u64 = 200;

fn env_u64(var: &str, default: u64) -> u64 {
    std::env::var(var)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

/// Timeout for JSON-RPC calls to peer primals (seconds).
///
/// Override: `LUDOSPRING_RPC_TIMEOUT_SECS`
///
/// Default 5s accommodates cold startup of AI providers while failing
/// fast on network issues (biomeOS graph nodes use 15s for germination).
pub fn rpc_timeout_secs() -> u64 {
    env_u64("LUDOSPRING_RPC_TIMEOUT_SECS", DEFAULT_RPC_TIMEOUT_SECS)
}

/// Probe timeout for socket capability verification (milliseconds).
///
/// Override: `LUDOSPRING_PROBE_TIMEOUT_MS`
///
/// Default 500ms is enough for a local Unix socket round-trip including
/// `lifecycle.status` parsing.
pub fn probe_timeout_ms() -> u64 {
    env_u64("LUDOSPRING_PROBE_TIMEOUT_MS", DEFAULT_PROBE_TIMEOUT_MS)
}

/// Connect-probe timeout for quick liveness checks (milliseconds).
///
/// Override: `LUDOSPRING_CONNECT_PROBE_TIMEOUT_MS`
///
/// Default 200ms is generous for loopback connections.
pub fn connect_probe_timeout_ms() -> u64 {
    env_u64(
        "LUDOSPRING_CONNECT_PROBE_TIMEOUT_MS",
        DEFAULT_CONNECT_PROBE_TIMEOUT_MS,
    )
}

/// Backward-compatible constant (prefer [`rpc_timeout_secs`] for env override).
pub const RPC_TIMEOUT_SECS: u64 = DEFAULT_RPC_TIMEOUT_SECS;
/// Backward-compatible constant (prefer [`probe_timeout_ms`] for env override).
pub const PROBE_TIMEOUT_MS: u64 = DEFAULT_PROBE_TIMEOUT_MS;
/// Backward-compatible constant (prefer [`connect_probe_timeout_ms`] for env override).
pub const CONNECT_PROBE_TIMEOUT_MS: u64 = DEFAULT_CONNECT_PROBE_TIMEOUT_MS;

#[cfg(test)]
#[allow(
    clippy::assertions_on_constants,
    reason = "validating constant relationships is the purpose of these tests"
)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_constants() {
        assert_eq!(RPC_TIMEOUT_SECS, DEFAULT_RPC_TIMEOUT_SECS);
        assert_eq!(PROBE_TIMEOUT_MS, DEFAULT_PROBE_TIMEOUT_MS);
        assert_eq!(CONNECT_PROBE_TIMEOUT_MS, DEFAULT_CONNECT_PROBE_TIMEOUT_MS);
    }

    #[test]
    fn probe_faster_than_rpc() {
        assert!(PROBE_TIMEOUT_MS < RPC_TIMEOUT_SECS * 1000);
    }

    #[test]
    fn connect_probe_fastest() {
        assert!(CONNECT_PROBE_TIMEOUT_MS < PROBE_TIMEOUT_MS);
    }

    #[test]
    fn env_u64_returns_default_for_missing() {
        let val = env_u64("LUDOSPRING_NONEXISTENT_VAR_FOR_TEST", 42);
        assert_eq!(val, 42);
    }
}

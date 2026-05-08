// SPDX-License-Identifier: AGPL-3.0-or-later
//! Circuit breaker for IPC resilience.
//!
//! Prevents cascading failures when downstream primals are unavailable.
//! Pattern adopted from healthSpring V32.
//!
//! The breaker has three states:
//! - **Closed** (normal): calls proceed.
//! - **Open** (tripped): calls short-circuit for `cooldown` duration.
//! - **Half-open** (cooldown expired): next call is a probe; success resets,
//!   failure re-trips.
//!
//! Configuration is via environment variables (runtime-agnostic):
//! - `LUDOSPRING_CIRCUIT_COOLDOWN_MS` (default 5000)
//! - `LUDOSPRING_CIRCUIT_MAX_RETRIES` (default 2)
//! - `LUDOSPRING_CIRCUIT_RETRY_DELAY_MS` (default 50)

use std::sync::atomic::{AtomicU64, Ordering};

use super::envelope::IpcError;
use super::neural_bridge::NeuralBridge;

/// Default cooldown period after a circuit opens (5 seconds, per healthSpring V32).
const DEFAULT_CIRCUIT_COOLDOWN_MS: u64 = 5_000;

/// Default maximum retry count with exponential backoff.
const DEFAULT_MAX_RETRIES: u32 = 2;

/// Default base delay between retries (doubles each attempt).
const DEFAULT_BASE_RETRY_DELAY_MS: u64 = 50;

fn circuit_cooldown_ms() -> u64 {
    std::env::var("LUDOSPRING_CIRCUIT_COOLDOWN_MS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_CIRCUIT_COOLDOWN_MS)
}

fn max_retries() -> u32 {
    std::env::var("LUDOSPRING_CIRCUIT_MAX_RETRIES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_MAX_RETRIES)
}

fn base_retry_delay_ms() -> u64 {
    std::env::var("LUDOSPRING_CIRCUIT_RETRY_DELAY_MS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_BASE_RETRY_DELAY_MS)
}

/// Timestamp (epoch ms) when the circuit last opened. 0 = circuit closed.
static CIRCUIT_OPEN_SINCE: AtomicU64 = AtomicU64::new(0);

/// Check whether the circuit breaker allows a call.
pub(crate) fn circuit_allows() -> bool {
    let opened = CIRCUIT_OPEN_SINCE.load(Ordering::Relaxed);
    if opened == 0 {
        return true;
    }
    let now = epoch_ms();
    if now.saturating_sub(opened) >= circuit_cooldown_ms() {
        CIRCUIT_OPEN_SINCE.store(0, Ordering::Relaxed);
        return true;
    }
    false
}

/// Trip the circuit breaker open.
pub(crate) fn trip_circuit() {
    CIRCUIT_OPEN_SINCE.store(epoch_ms(), Ordering::Relaxed);
}

/// Reset the circuit breaker (call succeeded).
pub(crate) fn reset_circuit() {
    CIRCUIT_OPEN_SINCE.store(0, Ordering::Relaxed);
}

#[expect(
    clippy::cast_possible_truncation,
    reason = "milliseconds since epoch fits in u64 for thousands of years"
)]
fn epoch_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Execute a provenance trio call with circuit breaker and exponential backoff.
///
/// If the circuit is open, returns `None` immediately (graceful degradation).
/// On failure, retries up to `MAX_RETRIES` times with exponential backoff,
/// then trips the circuit.
pub(crate) fn resilient_call<F>(f: F) -> Option<serde_json::Value>
where
    F: Fn(&NeuralBridge) -> Result<serde_json::Value, IpcError>,
{
    if !circuit_allows() {
        return None;
    }

    let Ok(bridge) = NeuralBridge::discover() else {
        trip_circuit();
        return None;
    };

    let retries = max_retries();
    let delay_base = base_retry_delay_ms();
    for attempt in 0..=retries {
        match f(&bridge) {
            Ok(value) => {
                reset_circuit();
                return Some(value);
            }
            Err(_) if attempt < retries => {
                let delay = delay_base * (1 << attempt);
                std::thread::sleep(std::time::Duration::from_millis(delay));
            }
            Err(_) => {
                trip_circuit();
                return None;
            }
        }
    }

    None
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn circuit_initially_closed() {
        reset_circuit();
        assert!(circuit_allows());
    }

    #[test]
    fn circuit_trips_and_blocks() {
        trip_circuit();
        assert!(!circuit_allows());
        reset_circuit();
    }

    #[test]
    fn circuit_respects_cooldown() {
        reset_circuit();
        CIRCUIT_OPEN_SINCE.store(1, Ordering::Relaxed);
        assert!(circuit_allows(), "ancient trip should have expired");
        reset_circuit();
    }
}

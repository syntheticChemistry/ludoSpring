// SPDX-License-Identifier: AGPL-3.0-or-later
//! NestGate storage integration — typed client for content-addressed persistence.
//!
//! Routes storage capability calls through [`NeuralBridge`] to NestGate:
//!
//! - `storage.store` / `storage.retrieve` — save/load game state, NPC snapshots, rulesets
//! - `storage.exists` — check cache before recomputing
//! - `storage.list` — enumerate saved games and world states
//! - `storage.metadata` — version and timestamp tracking
//!
//! Graceful degradation: returns `StorageResult { available: false, .. }` when
//! NestGate is not reachable through the Neural API.

use super::neural_bridge::NeuralBridge;

/// Result of a NestGate storage operation.
#[derive(Debug, Clone)]
pub struct StorageResult {
    /// Whether NestGate was available.
    pub available: bool,
    /// Response data from the operation.
    pub data: serde_json::Value,
}

/// Store a value in NestGate (content-addressed by BLAKE3).
///
/// # Errors
///
/// Returns an error only on non-recoverable failures.
pub fn put(
    key: &str,
    value: &serde_json::Value,
    metadata: &serde_json::Value,
) -> Result<StorageResult, String> {
    let Ok(bridge) = NeuralBridge::discover() else {
        return Ok(unavailable());
    };

    let args = serde_json::json!({
        "key": key,
        "data": value,
        "metadata": metadata,
    });

    bridge
        .capability_call("storage", "store", &args)
        .map_or_else(
            |_| Ok(unavailable()),
            |result| {
                Ok(StorageResult {
                    available: true,
                    data: result,
                })
            },
        )
}

/// Retrieve a value from NestGate by key.
///
/// # Errors
///
/// Returns an error only on non-recoverable failures.
pub fn get(key: &str) -> Result<StorageResult, String> {
    let Ok(bridge) = NeuralBridge::discover() else {
        return Ok(unavailable());
    };

    let args = serde_json::json!({ "key": key });

    bridge
        .capability_call("storage", "retrieve", &args)
        .map_or_else(
            |_| Ok(unavailable()),
            |result| {
                Ok(StorageResult {
                    available: true,
                    data: result,
                })
            },
        )
}

/// Check whether a key exists in NestGate without retrieving data.
///
/// # Errors
///
/// Returns an error only on non-recoverable failures.
pub fn exists(key: &str) -> Result<bool, String> {
    let Ok(bridge) = NeuralBridge::discover() else {
        return Ok(false);
    };

    let args = serde_json::json!({ "key": key });

    bridge
        .capability_call("storage", "exists", &args)
        .map_or(Ok(false), |result| {
            Ok(result
                .get("exists")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false))
        })
}

/// List stored objects, optionally filtered by a prefix.
///
/// # Errors
///
/// Returns an error only on non-recoverable failures.
pub fn list(prefix: Option<&str>) -> Result<StorageResult, String> {
    let Ok(bridge) = NeuralBridge::discover() else {
        return Ok(unavailable());
    };

    let args = prefix.map_or_else(
        || serde_json::json!({}),
        |p| serde_json::json!({ "prefix": p }),
    );

    bridge
        .capability_call("storage", "list", &args)
        .map_or_else(
            |_| Ok(unavailable()),
            |result| {
                Ok(StorageResult {
                    available: true,
                    data: result,
                })
            },
        )
}

/// Retrieve metadata for a stored object without fetching its data.
///
/// # Errors
///
/// Returns an error only on non-recoverable failures.
pub fn metadata(key: &str) -> Result<StorageResult, String> {
    let Ok(bridge) = NeuralBridge::discover() else {
        return Ok(unavailable());
    };

    let args = serde_json::json!({ "key": key });

    bridge
        .capability_call("storage", "metadata", &args)
        .map_or_else(
            |_| Ok(unavailable()),
            |result| {
                Ok(StorageResult {
                    available: true,
                    data: result,
                })
            },
        )
}

/// Delete a stored object by key.
///
/// # Errors
///
/// Returns an error only on non-recoverable failures.
pub fn delete(key: &str) -> Result<StorageResult, String> {
    let Ok(bridge) = NeuralBridge::discover() else {
        return Ok(unavailable());
    };

    let args = serde_json::json!({ "key": key });

    bridge
        .capability_call("storage", "delete", &args)
        .map_or_else(
            |_| Ok(unavailable()),
            |result| {
                Ok(StorageResult {
                    available: true,
                    data: result,
                })
            },
        )
}

fn unavailable() -> StorageResult {
    StorageResult {
        available: false,
        data: serde_json::json!({ "storage": "unavailable" }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn put_without_neural_api() {
        let r = put(
            "test-key",
            &serde_json::json!({"hp": 100}),
            &serde_json::json!({"type": "character"}),
        )
        .unwrap();
        assert!(!r.available);
    }

    #[test]
    fn get_without_neural_api() {
        let r = get("test-key").unwrap();
        assert!(!r.available);
    }

    #[test]
    fn exists_without_neural_api() {
        let r = exists("test-key").unwrap();
        assert!(!r);
    }

    #[test]
    fn list_without_neural_api() {
        let r = list(Some("game.")).unwrap();
        assert!(!r.available);
    }

    #[test]
    fn metadata_without_neural_api() {
        let r = metadata("test-key").unwrap();
        assert!(!r.available);
    }

    #[test]
    fn delete_without_neural_api() {
        let r = delete("test-key").unwrap();
        assert!(!r.available);
    }
}

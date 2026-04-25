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

use super::envelope::IpcError;
use super::neural_bridge::NeuralBridge;

/// Result of a NestGate storage operation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
/// Returns an [`IpcError`] only on non-recoverable failures.
pub fn put(
    key: &str,
    value: &serde_json::Value,
    metadata: &serde_json::Value,
) -> Result<StorageResult, IpcError> {
    let Ok(bridge) = NeuralBridge::discover() else {
        return Ok(unavailable());
    };

    let args = put_args(key, value, metadata);

    bridge
        .capability_call("storage", "store", &args)
        .map_or_else(|_| Ok(unavailable()), |result| Ok(storage_success(result)))
}

fn put_args(
    key: &str,
    value: &serde_json::Value,
    metadata: &serde_json::Value,
) -> serde_json::Value {
    serde_json::json!({
        "key": key,
        "data": value,
        "metadata": metadata,
    })
}

const fn storage_success(data: serde_json::Value) -> StorageResult {
    StorageResult {
        available: true,
        data,
    }
}

/// Retrieve a value from NestGate by key.
///
/// # Errors
///
/// Returns an [`IpcError`] only on non-recoverable failures.
pub fn get(key: &str) -> Result<StorageResult, IpcError> {
    let Ok(bridge) = NeuralBridge::discover() else {
        return Ok(unavailable());
    };

    let args = key_only_args(key);

    bridge
        .capability_call("storage", "retrieve", &args)
        .map_or_else(|_| Ok(unavailable()), |result| Ok(storage_success(result)))
}

/// Check whether a key exists in NestGate without retrieving data.
///
/// # Errors
///
/// Returns an [`IpcError`] only on non-recoverable failures.
pub fn exists(key: &str) -> Result<bool, IpcError> {
    let Ok(bridge) = NeuralBridge::discover() else {
        return Ok(false);
    };

    let args = key_only_args(key);

    bridge
        .capability_call("storage", "exists", &args)
        .map_or(Ok(false), |result| Ok(parse_exists_flag(&result)))
}

fn key_only_args(key: &str) -> serde_json::Value {
    serde_json::json!({ "key": key })
}

fn parse_exists_flag(result: &serde_json::Value) -> bool {
    result
        .get("exists")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false)
}

/// List stored objects, optionally filtered by a prefix.
///
/// # Errors
///
/// Returns an [`IpcError`] only on non-recoverable failures.
pub fn list(prefix: Option<&str>) -> Result<StorageResult, IpcError> {
    let Ok(bridge) = NeuralBridge::discover() else {
        return Ok(unavailable());
    };

    let args = list_args(prefix);

    bridge
        .capability_call("storage", "list", &args)
        .map_or_else(|_| Ok(unavailable()), |result| Ok(storage_success(result)))
}

fn list_args(prefix: Option<&str>) -> serde_json::Value {
    prefix.map_or_else(
        || serde_json::json!({}),
        |p| serde_json::json!({ "prefix": p }),
    )
}

/// Retrieve metadata for a stored object without fetching its data.
///
/// # Errors
///
/// Returns an [`IpcError`] only on non-recoverable failures.
pub fn metadata(key: &str) -> Result<StorageResult, IpcError> {
    let Ok(bridge) = NeuralBridge::discover() else {
        return Ok(unavailable());
    };

    let args = key_only_args(key);

    bridge
        .capability_call("storage", "metadata", &args)
        .map_or_else(|_| Ok(unavailable()), |result| Ok(storage_success(result)))
}

/// Delete a stored object by key.
///
/// # Errors
///
/// Returns an [`IpcError`] only on non-recoverable failures.
pub fn delete(key: &str) -> Result<StorageResult, IpcError> {
    let Ok(bridge) = NeuralBridge::discover() else {
        return Ok(unavailable());
    };

    let args = key_only_args(key);

    bridge
        .capability_call("storage", "delete", &args)
        .map_or_else(|_| Ok(unavailable()), |result| Ok(storage_success(result)))
}

fn unavailable() -> StorageResult {
    StorageResult {
        available: false,
        data: serde_json::json!({ "storage": "unavailable" }),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
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

    #[test]
    fn list_with_none_prefix_without_neural_api() {
        let r = list(None).unwrap();
        assert!(!r.available);
        assert_eq!(r.data["storage"], "unavailable");
    }

    #[test]
    fn storage_result_constructed_with_payload() {
        let r = StorageResult {
            available: true,
            data: serde_json::json!({ "keys": ["a", "b"] }),
        };
        assert!(r.available);
        assert_eq!(r.data["keys"][1], "b");
    }

    #[test]
    fn storage_result_serde_round_trip() {
        let original = StorageResult {
            available: false,
            data: serde_json::json!({ "storage": "unavailable" }),
        };
        let json = serde_json::to_string(&original).expect("serialize");
        let back: StorageResult = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.available, original.available);
        assert_eq!(back.data, original.data);
    }

    #[test]
    fn put_args_shape() {
        let args = super::put_args(
            "k1",
            &serde_json::json!({"v": 1}),
            &serde_json::json!({"tag": "t"}),
        );
        assert_eq!(args["key"], "k1");
        assert_eq!(args["data"]["v"], 1);
        assert_eq!(args["metadata"]["tag"], "t");
    }

    #[test]
    fn key_only_args_single_key() {
        let a = super::key_only_args("my-key");
        assert_eq!(a["key"], "my-key");
        assert_eq!(a.as_object().map(serde_json::Map::len), Some(1));
    }

    #[test]
    fn list_args_none_prefix_empty_object() {
        let a = super::list_args(None);
        assert_eq!(a, serde_json::json!({}));
    }

    #[test]
    fn list_args_some_prefix() {
        let a = super::list_args(Some("prefix."));
        assert_eq!(a["prefix"], "prefix.");
    }

    #[test]
    fn storage_success_wraps_payload() {
        let r = super::storage_success(serde_json::json!({ "ok": true }));
        assert!(r.available);
        assert_eq!(r.data["ok"], true);
    }

    #[test]
    fn parse_exists_flag_true_false_and_missing() {
        assert!(super::parse_exists_flag(
            &serde_json::json!({ "exists": true })
        ));
        assert!(!super::parse_exists_flag(
            &serde_json::json!({ "exists": false })
        ));
        assert!(!super::parse_exists_flag(&serde_json::json!({})));
        assert!(!super::parse_exists_flag(
            &serde_json::json!({ "exists": "yes" })
        ));
    }

    #[test]
    fn unavailable_payload_is_stable() {
        let u = super::unavailable();
        assert!(!u.available);
        assert_eq!(u.data["storage"], "unavailable");
    }
}

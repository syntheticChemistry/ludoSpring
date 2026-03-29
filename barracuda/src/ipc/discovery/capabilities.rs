// SPDX-License-Identifier: AGPL-3.0-or-later
//! Capability extraction from diverse primal response formats.
//!
//! Primals report their capabilities in at least 6 different JSON shapes.
//! This module normalises all of them into `Vec<String>` and generates
//! semantic aliases so that higher-level code can discover by intent
//! (e.g. `crypto.hash`) without knowing the exact method name a primal
//! exposes (e.g. `crypto.blake3_hash`).

/// Extract capabilities from any known primal response format.
///
/// Handles 6 formats observed across the ecosystem:
///
/// - **Format A**: Flat string array `["cap1", "cap2"]`
/// - **Format B**: Object array with `name` `[{"name": "cap1"}]`
/// - **Format C**: Nested wrapper `{"capabilities": [...]}`
/// - **Format D**: Double-nested `{"capabilities": {"capabilities": [...]}}`
/// - **Format E**: BearDog `provided_capabilities` `[{"type": "crypto", "methods": [...]}]`
/// - **Format F**: Top-level flat array (Songbird)
pub fn extract_from_any(value: &serde_json::Value) -> Vec<String> {
    if let Some(arr) = value.as_array() {
        return extract_from_array(arr);
    }

    let mut caps = Vec::new();

    if let Some(provided) = value
        .get("provided_capabilities")
        .and_then(serde_json::Value::as_array)
    {
        for entry in provided {
            if let Some(cap_type) = entry.get("type").and_then(serde_json::Value::as_str) {
                caps.push(cap_type.to_owned());
                if let Some(methods) = entry.get("methods").and_then(serde_json::Value::as_array) {
                    for m in methods {
                        if let Some(method_name) = m.as_str() {
                            caps.push(format!("{cap_type}.{method_name}"));
                        }
                    }
                }
            }
        }
        if !caps.is_empty() {
            generate_semantic_aliases(&mut caps);
            return caps;
        }
    }

    let extracted = extract_from_lifecycle(value);
    if !extracted.is_empty() {
        return extracted;
    }

    caps
}

/// Extract capabilities from a `lifecycle.status` or `capability.list` response.
///
/// Handles Formats A–D and an optional `{"result": ...}` wrapper.
pub fn extract_from_lifecycle(result: &serde_json::Value) -> Vec<String> {
    let target = result
        .get("result")
        .and_then(|r| r.get("capabilities"))
        .or_else(|| result.get("capabilities"));

    match target {
        Some(serde_json::Value::Array(arr)) => extract_from_array(arr),
        Some(serde_json::Value::Object(obj)) => obj
            .get("capabilities")
            .and_then(serde_json::Value::as_array)
            .map(|arr| extract_from_array(arr))
            .unwrap_or_default(),
        _ => Vec::new(),
    }
}

/// Extract capability strings from an array that may contain strings
/// (Format A) or objects with a `"name"` field (Format B).
pub fn extract_from_array(arr: &[serde_json::Value]) -> Vec<String> {
    arr.iter()
        .filter_map(|v| match v {
            serde_json::Value::String(s) => Some(s.clone()),
            serde_json::Value::Object(obj) => obj
                .get("name")
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned),
            _ => None,
        })
        .collect()
}

/// Generate well-known semantic aliases from capability types.
///
/// When a primal advertises `crypto` with method-level capabilities like
/// `crypto.blake3_hash`, also register `crypto.hash` since the primal
/// likely implements the generic hash dispatcher.
pub fn generate_semantic_aliases(caps: &mut Vec<String>) {
    let has = |name: &str, list: &[String]| list.iter().any(|c| c == name);

    let snapshot = caps.clone();
    let mut additions = Vec::new();

    if has("crypto", &snapshot) && !has("crypto.hash", &snapshot) {
        additions.push("crypto.hash".to_owned());
    }
    if has("crypto", &snapshot) && !has("crypto.encrypt", &snapshot) {
        if has("crypto.chacha20_poly1305_encrypt", &snapshot) {
            additions.push("crypto.encrypt".to_owned());
        }
    }
    if has("crypto", &snapshot) && !has("crypto.decrypt", &snapshot) {
        if has("crypto.chacha20_poly1305_decrypt", &snapshot) {
            additions.push("crypto.decrypt".to_owned());
        }
    }
    if has("crypto", &snapshot) && !has("crypto.sign", &snapshot) {
        if has("crypto.sign_ed25519", &snapshot) {
            additions.push("crypto.sign".to_owned());
        }
    }
    if has("crypto", &snapshot) && !has("crypto.verify", &snapshot) {
        if has("crypto.verify_ed25519", &snapshot) {
            additions.push("crypto.verify".to_owned());
        }
    }

    caps.extend(additions);
}

/// Auto-register base capabilities for any responsive primal.
pub fn inject_base_capabilities(caps: &mut Vec<String>) {
    for base in ["system.ping", "health.check", "health.liveness"] {
        if !caps.iter().any(|c| c == base) {
            caps.push(base.to_owned());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_a_flat_array() {
        let result = serde_json::json!({
            "name": "test",
            "capabilities": ["game.evaluate_flow", "game.fitts_cost"]
        });
        let caps = extract_from_lifecycle(&result);
        assert_eq!(caps, vec!["game.evaluate_flow", "game.fitts_cost"]);
    }

    #[test]
    fn format_b_object_array() {
        let result = serde_json::json!({
            "capabilities": [
                {"name": "health", "version": "1.0"},
                {"name": "compute.dispatch"}
            ]
        });
        let caps = extract_from_lifecycle(&result);
        assert_eq!(caps, vec!["health", "compute.dispatch"]);
    }

    #[test]
    fn format_c_nested_wrapper() {
        let result = serde_json::json!({
            "result": {
                "capabilities": ["dag.session.create", "dag.event.append"]
            }
        });
        let caps = extract_from_lifecycle(&result);
        assert_eq!(caps, vec!["dag.session.create", "dag.event.append"]);
    }

    #[test]
    fn format_d_double_nested() {
        let result = serde_json::json!({
            "name": "test",
            "capabilities": {
                "capabilities": ["compute.submit", "compute.status"]
            }
        });
        let caps = extract_from_lifecycle(&result);
        assert_eq!(caps, vec!["compute.submit", "compute.status"]);
    }

    #[test]
    fn missing_returns_empty() {
        let result = serde_json::json!({"name": "test"});
        assert!(extract_from_lifecycle(&result).is_empty());
    }

    #[test]
    fn null_returns_empty() {
        let result = serde_json::json!({"capabilities": null});
        assert!(extract_from_lifecycle(&result).is_empty());
    }

    #[test]
    fn mixed_array_ignores_non_string() {
        let result = serde_json::json!({
            "capabilities": ["valid", 42, null, {"name": "also_valid"}]
        });
        let caps = extract_from_lifecycle(&result);
        assert_eq!(caps, vec!["valid", "also_valid"]);
    }

    #[test]
    fn format_e_beardog_provided_capabilities() {
        let result = serde_json::json!({
            "provided_capabilities": [
                {
                    "type": "crypto",
                    "version": "1.0",
                    "methods": ["blake3_hash", "hmac_sha256", "chacha20_poly1305_encrypt",
                                "chacha20_poly1305_decrypt", "sign_ed25519", "verify_ed25519"]
                },
                {
                    "type": "security",
                    "methods": ["evaluate", "lineage"]
                }
            ]
        });
        let caps = extract_from_any(&result);
        assert!(caps.contains(&"crypto".to_owned()));
        assert!(caps.contains(&"crypto.blake3_hash".to_owned()));
        assert!(caps.contains(&"crypto.hash".to_owned()), "semantic alias");
        assert!(caps.contains(&"crypto.encrypt".to_owned()), "semantic alias");
        assert!(caps.contains(&"crypto.sign".to_owned()), "semantic alias");
        assert!(caps.contains(&"security".to_owned()));
        assert!(caps.contains(&"security.evaluate".to_owned()));
    }

    #[test]
    fn format_f_songbird_flat_array() {
        let result = serde_json::json!([
            "network.discovery",
            "network.federation",
            "ipc.jsonrpc",
            "crypto.delegate"
        ]);
        let caps = extract_from_any(&result);
        assert_eq!(
            caps,
            vec!["network.discovery", "network.federation", "ipc.jsonrpc", "crypto.delegate"]
        );
    }

    #[test]
    fn inject_base_adds_ping() {
        let mut caps = vec!["crypto".to_owned()];
        inject_base_capabilities(&mut caps);
        assert!(caps.contains(&"system.ping".to_owned()));
        assert!(caps.contains(&"health.check".to_owned()));
        assert!(caps.contains(&"health.liveness".to_owned()));
    }

    #[test]
    fn inject_base_does_not_duplicate() {
        let mut caps = vec!["system.ping".to_owned(), "health.check".to_owned()];
        inject_base_capabilities(&mut caps);
        assert_eq!(caps.iter().filter(|c| c.as_str() == "system.ping").count(), 1);
    }

    mod proptest_fuzz {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn extract_never_panics(json_str in "\\PC{0,200}") {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&json_str) {
                    let _ = extract_from_lifecycle(&val);
                }
            }

            #[test]
            fn extract_returns_strings_from_flat_array(
                caps in prop::collection::vec("[a-z.]{1,20}", 0..5)
            ) {
                let val = serde_json::json!({"capabilities": caps});
                let result = extract_from_lifecycle(&val);
                prop_assert_eq!(result, caps);
            }

            #[test]
            fn extract_handles_object_array(
                caps in prop::collection::vec("[a-z.]{1,15}", 0..5)
            ) {
                let objects: Vec<serde_json::Value> = caps
                    .iter()
                    .map(|c| serde_json::json!({"name": c}))
                    .collect();
                let val = serde_json::json!({"capabilities": objects});
                let result = extract_from_lifecycle(&val);
                prop_assert_eq!(result, caps);
            }
        }
    }
}

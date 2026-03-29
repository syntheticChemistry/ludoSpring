// SPDX-License-Identifier: AGPL-3.0-or-later
//! Capability-based primal discovery.
//!
//! ludoSpring discovers other primals at runtime by capability — never by
//! hardcoded name or path. This follows the TRUE PRIMAL pattern: a primal
//! has self-knowledge and discovers collaborators via capability routing.
//!
//! # Discovery Priority (XDG-compliant)
//!
//! 1. `BIOMEOS_SOCKET_DIR` environment variable — explicit override
//! 2. `$XDG_RUNTIME_DIR/biomeos/` — standard runtime location
//! 3. `/tmp/` — development fallback (only when no standard dirs resolve)
//!
//! Within each directory, socket files are probed via `lifecycle.status`
//! JSON-RPC calls. Primals that respond with matching capabilities are
//! registered for use.

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// A discovered primal endpoint.
#[derive(Debug, Clone)]
pub struct PrimalEndpoint {
    /// Socket path.
    pub socket: PathBuf,
    /// Primal name (from lifecycle.status response).
    pub name: String,
    /// Capabilities this primal advertises.
    pub capabilities: Vec<String>,
}

/// Registry of discovered primals, keyed by capability.
#[derive(Debug, Default)]
pub struct PrimalRegistry {
    /// Capability → endpoint mapping.
    endpoints: HashMap<String, PrimalEndpoint>,
}

impl PrimalRegistry {
    /// Create an empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register an endpoint for each of its capabilities.
    pub fn register(&mut self, endpoint: &PrimalEndpoint) {
        for cap in &endpoint.capabilities {
            self.endpoints.insert(cap.clone(), endpoint.clone());
        }
    }

    /// Look up the endpoint for a capability.
    #[must_use]
    pub fn find(&self, capability: &str) -> Option<&PrimalEndpoint> {
        self.endpoints.get(capability)
    }

    /// All registered capabilities.
    #[must_use]
    pub fn capabilities(&self) -> Vec<&str> {
        self.endpoints.keys().map(String::as_str).collect()
    }

    /// Number of unique endpoints registered.
    #[must_use]
    pub fn endpoint_count(&self) -> usize {
        let mut seen: Vec<&PathBuf> = self.endpoints.values().map(|ep| &ep.socket).collect();
        seen.sort();
        seen.dedup();
        seen.len()
    }
}

/// Resolve the socket directories to search for primals.
///
/// Delegates to [`crate::niche::socket_dirs`] for the XDG-compliant
/// directory chain. Filters to directories that actually exist on disk
/// to avoid wasted probing.
#[must_use]
pub fn discovery_dirs() -> Vec<PathBuf> {
    crate::niche::socket_dirs()
        .into_iter()
        .filter(|d| d.is_dir())
        .collect()
}

/// Send a single JSON-RPC request on a fresh connection and return the parsed response.
fn rpc_probe(path: &Path, method: &str) -> Option<serde_json::Value> {
    let stream = UnixStream::connect(path).ok()?;
    let probe = Duration::from_millis(crate::tolerances::PROBE_TIMEOUT_MS);
    stream.set_read_timeout(Some(probe)).ok()?;
    stream.set_write_timeout(Some(probe)).ok()?;

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": method,
        "id": 1
    });

    let mut writer = stream.try_clone().ok()?;
    let mut msg = serde_json::to_string(&request).ok()?;
    msg.push('\n');
    writer.write_all(msg.as_bytes()).ok()?;
    writer.flush().ok()?;

    let mut reader = BufReader::new(stream);
    let mut response = String::new();
    reader.read_line(&mut response).ok()?;

    serde_json::from_str(&response).ok()
}

/// Probe a Unix socket to check if it hosts a JSON-RPC primal.
///
/// Tries `lifecycle.status` first (biomeOS standard), then falls back to
/// `health.check` + `capabilities.list` (BearDog/Songbird convention).
/// Any responsive primal also gets `health.check` and `system.ping`
/// auto-registered since reachability implies ping-ability.
pub fn probe_socket(path: &Path) -> Option<PrimalEndpoint> {
    // Strategy 1: lifecycle.status (biomeOS standard — name + capabilities in one call)
    if let Some(ep) = probe_lifecycle_status(path) {
        return Some(ep);
    }

    // Strategy 2: health.check for identity, capabilities.list for capabilities
    probe_health_then_capabilities(path)
}

/// Try `lifecycle.status` — the original biomeOS-standard probe.
fn probe_lifecycle_status(path: &Path) -> Option<PrimalEndpoint> {
    let parsed = rpc_probe(path, "lifecycle.status")?;
    let result = parsed.get("result")?;

    let name = result
        .get("name")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("unknown")
        .to_owned();

    let mut capabilities = extract_capabilities(result);
    inject_base_capabilities(&mut capabilities);

    Some(PrimalEndpoint {
        socket: path.to_owned(),
        name,
        capabilities,
    })
}

/// Fallback probe: `health.check` for primal name, `capabilities.list` for capabilities.
fn probe_health_then_capabilities(path: &Path) -> Option<PrimalEndpoint> {
    let health = rpc_probe(path, "health.check")?;
    let health_result = health.get("result")?;

    let name = health_result
        .get("primal")
        .or_else(|| health_result.get("name"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or("unknown")
        .to_owned();

    let mut capabilities = Vec::new();

    if let Some(caps_resp) = rpc_probe(path, "capabilities.list") {
        if let Some(result) = caps_resp.get("result") {
            capabilities = extract_capabilities_from_any(result);
        }
    }

    inject_base_capabilities(&mut capabilities);

    Some(PrimalEndpoint {
        socket: path.to_owned(),
        name,
        capabilities,
    })
}

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
fn extract_capabilities_from_any(value: &serde_json::Value) -> Vec<String> {
    // Format F: top-level array (Songbird capabilities.list returns a direct array)
    if let Some(arr) = value.as_array() {
        return extract_from_array(arr);
    }

    let mut caps = Vec::new();

    // Format E: BearDog provided_capabilities
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

    // Formats A-D: delegate to existing extractor
    let extracted = extract_capabilities(value);
    if !extracted.is_empty() {
        return extracted;
    }

    caps
}

/// Generate well-known semantic aliases from capability types.
///
/// When a primal advertises `crypto` with method-level capabilities like
/// `crypto.blake3_hash`, also register `crypto.hash` since the primal
/// likely implements the generic hash dispatcher.
fn generate_semantic_aliases(caps: &mut Vec<String>) {
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
fn inject_base_capabilities(caps: &mut Vec<String>) {
    for base in ["system.ping", "health.check", "health.liveness"] {
        if !caps.iter().any(|c| c == base) {
            caps.push(base.to_owned());
        }
    }
}

/// Extract capabilities from a `lifecycle.status` or `capability.list` response.
///
/// Handles all 4 formats observed across the ecosystem (airSpring v0.8.7,
/// rhizoCrypt S17, neuralSpring S156):
///
/// - **Format A**: Flat string array `["cap1", "cap2"]`
/// - **Format B**: Object array `[{"name": "cap1"}, {"name": "cap2"}]`
/// - **Format C**: Nested wrapper `{"capabilities": ["cap1"]}`
/// - **Format D**: Double-nested `{"capabilities": {"capabilities": ["cap1"]}}`
///
/// Also handles a `{"result": ...}` wrapper (biomeOS response format).
fn extract_capabilities(result: &serde_json::Value) -> Vec<String> {
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

/// Extract capability strings from an array that may contain strings (Format A)
/// or objects with a `"name"` field (Format B).
fn extract_from_array(arr: &[serde_json::Value]) -> Vec<String> {
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

/// Discover primals by scanning `dirs` for `*.sock` and probing each with [`probe_socket`].
///
/// Used by [`discover_primals`] and by integration tests without mutating process environment.
#[must_use]
pub fn discover_primals_in_directories(dirs: &[PathBuf]) -> PrimalRegistry {
    let mut registry = PrimalRegistry::new();

    for dir in dirs {
        let Ok(entries) = std::fs::read_dir(dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            // Pattern-based: scan ALL *.sock files, never hardcode primal names.
            if path.extension().is_some_and(|ext| ext == "sock") {
                if let Some(endpoint) = probe_socket(&path) {
                    registry.register(&endpoint);
                }
            }
        }
    }

    registry
}

/// Discover all primals in the standard socket directories.
///
/// Probes every `.sock` file found in [`discovery_dirs`] and returns
/// a registry of those that respond to `lifecycle.status`.
#[must_use]
pub fn discover_primals() -> PrimalRegistry {
    let dirs = discovery_dirs();
    discover_primals_in_directories(&dirs)
}

/// Discover a primal endpoint by capability at runtime.
///
/// Scans all socket directories for a primal advertising the requested
/// capability. Returns `None` if no primal serves it. This is the preferred
/// way to find peers — never hardcode primal names or socket paths.
#[must_use]
pub fn discover_by_capability(capability: &str) -> Option<PrimalEndpoint> {
    let registry = discover_primals();
    registry.find(capability).cloned()
}

/// Send a JSON-RPC request to a primal and return the result.
///
/// # Errors
///
/// Returns a typed [`IpcError`](super::envelope::IpcError) on failure.
pub fn call_primal(
    endpoint: &PrimalEndpoint,
    method: &str,
    params: &serde_json::Value,
) -> Result<serde_json::Value, super::envelope::IpcError> {
    use super::envelope::IpcError;

    let stream = UnixStream::connect(&endpoint.socket).map_err(IpcError::Connect)?;
    let rpc_timeout = Duration::from_secs(crate::tolerances::RPC_TIMEOUT_SECS);
    stream
        .set_read_timeout(Some(rpc_timeout))
        .map_err(IpcError::Timeout)?;
    stream
        .set_write_timeout(Some(rpc_timeout))
        .map_err(IpcError::Timeout)?;

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": 1
    });

    let mut writer = stream.try_clone().map_err(IpcError::Io)?;
    let mut msg =
        serde_json::to_string(&request).map_err(|e| IpcError::Serialization(e.to_string()))?;
    msg.push('\n');
    writer.write_all(msg.as_bytes()).map_err(IpcError::Io)?;
    writer.flush().map_err(IpcError::Io)?;

    let mut reader = BufReader::new(stream);
    let mut response = String::new();
    reader.read_line(&mut response).map_err(IpcError::Io)?;

    let parsed: serde_json::Value =
        serde_json::from_str(&response).map_err(|e| IpcError::Serialization(e.to_string()))?;

    super::envelope::extract_rpc_result(&parsed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_registry_finds_nothing() {
        let reg = PrimalRegistry::new();
        assert!(reg.find("security").is_none());
        assert_eq!(reg.endpoint_count(), 0);
    }

    #[test]
    fn register_and_find_by_capability() {
        let mut reg = PrimalRegistry::new();
        reg.register(&PrimalEndpoint {
            socket: std::env::temp_dir().join(format!(
                "ludospring-test-disc-register-{}.sock",
                std::process::id()
            )),
            name: "test-primal".into(),
            capabilities: vec!["security".into(), "crypto".into()],
        });
        assert!(reg.find("security").is_some());
        assert!(reg.find("crypto").is_some());
        assert!(reg.find("storage").is_none());
        assert_eq!(reg.endpoint_count(), 1);
    }

    #[test]
    fn multiple_endpoints_tracked() {
        let mut reg = PrimalRegistry::new();
        reg.register(&PrimalEndpoint {
            socket: std::env::temp_dir().join(format!(
                "ludospring-test-disc-a-{}.sock",
                std::process::id()
            )),
            name: "primal-a".into(),
            capabilities: vec!["security".into()],
        });
        reg.register(&PrimalEndpoint {
            socket: std::env::temp_dir().join(format!(
                "ludospring-test-disc-b-{}.sock",
                std::process::id()
            )),
            name: "primal-b".into(),
            capabilities: vec!["storage".into()],
        });
        assert_eq!(reg.endpoint_count(), 2);
        assert_eq!(reg.capabilities().len(), 2);
    }

    #[test]
    fn discovery_dirs_never_empty() {
        let dirs = discovery_dirs();
        assert!(!dirs.is_empty(), "should always resolve at least one dir");
    }

    #[test]
    fn discover_primals_in_directories_empty_dir() {
        let dir =
            std::env::temp_dir().join(format!("ludospring-disc-empty-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("mkdir");
        let reg = discover_primals_in_directories(&[dir.clone()]);
        assert_eq!(reg.endpoint_count(), 0);
        std::fs::remove_dir(&dir).ok();
    }

    #[test]
    fn extract_capabilities_format_a_flat_array() {
        let result = serde_json::json!({
            "name": "test",
            "capabilities": ["game.evaluate_flow", "game.fitts_cost"]
        });
        let caps = extract_capabilities(&result);
        assert_eq!(caps, vec!["game.evaluate_flow", "game.fitts_cost"]);
    }

    #[test]
    fn extract_capabilities_format_b_object_array() {
        let result = serde_json::json!({
            "capabilities": [
                {"name": "health", "version": "1.0"},
                {"name": "compute.dispatch"}
            ]
        });
        let caps = extract_capabilities(&result);
        assert_eq!(caps, vec!["health", "compute.dispatch"]);
    }

    #[test]
    fn extract_capabilities_format_c_nested_wrapper() {
        let result = serde_json::json!({
            "result": {
                "capabilities": ["dag.session.create", "dag.event.append"]
            }
        });
        let caps = extract_capabilities(&result);
        assert_eq!(caps, vec!["dag.session.create", "dag.event.append"]);
    }

    #[test]
    fn extract_capabilities_format_d_double_nested() {
        let result = serde_json::json!({
            "name": "test",
            "capabilities": {
                "capabilities": ["compute.submit", "compute.status"]
            }
        });
        let caps = extract_capabilities(&result);
        assert_eq!(caps, vec!["compute.submit", "compute.status"]);
    }

    #[test]
    fn extract_capabilities_missing_returns_empty() {
        let result = serde_json::json!({"name": "test"});
        let caps = extract_capabilities(&result);
        assert!(caps.is_empty());
    }

    #[test]
    fn extract_capabilities_null_returns_empty() {
        let result = serde_json::json!({"capabilities": null});
        let caps = extract_capabilities(&result);
        assert!(caps.is_empty());
    }

    #[test]
    fn extract_capabilities_mixed_array_ignores_non_string() {
        let result = serde_json::json!({
            "capabilities": ["valid", 42, null, {"name": "also_valid"}]
        });
        let caps = extract_capabilities(&result);
        assert_eq!(caps, vec!["valid", "also_valid"]);
    }

    #[test]
    fn extract_format_e_beardog_provided_capabilities() {
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
        let caps = extract_capabilities_from_any(&result);
        assert!(caps.contains(&"crypto".to_owned()));
        assert!(caps.contains(&"crypto.blake3_hash".to_owned()));
        assert!(caps.contains(&"crypto.hash".to_owned()), "semantic alias");
        assert!(caps.contains(&"crypto.encrypt".to_owned()), "semantic alias");
        assert!(caps.contains(&"crypto.sign".to_owned()), "semantic alias");
        assert!(caps.contains(&"security".to_owned()));
        assert!(caps.contains(&"security.evaluate".to_owned()));
    }

    #[test]
    fn extract_format_f_songbird_flat_array() {
        let result = serde_json::json!([
            "network.discovery",
            "network.federation",
            "ipc.jsonrpc",
            "crypto.delegate"
        ]);
        let caps = extract_capabilities_from_any(&result);
        assert_eq!(
            caps,
            vec![
                "network.discovery",
                "network.federation",
                "ipc.jsonrpc",
                "crypto.delegate"
            ]
        );
    }

    #[test]
    fn inject_base_capabilities_adds_ping() {
        let mut caps = vec!["crypto".to_owned()];
        inject_base_capabilities(&mut caps);
        assert!(caps.contains(&"system.ping".to_owned()));
        assert!(caps.contains(&"health.check".to_owned()));
        assert!(caps.contains(&"health.liveness".to_owned()));
    }

    #[test]
    fn inject_base_capabilities_does_not_duplicate() {
        let mut caps = vec!["system.ping".to_owned(), "health.check".to_owned()];
        inject_base_capabilities(&mut caps);
        assert_eq!(
            caps.iter().filter(|c| c.as_str() == "system.ping").count(),
            1
        );
    }

    // ── Proptest fuzz (airSpring v0.8.7 pattern) ────────────────────

    mod proptest_fuzz {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn extract_capabilities_never_panics(json_str in "\\PC{0,200}") {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&json_str) {
                    let _ = extract_capabilities(&val);
                }
            }

            #[test]
            fn extract_capabilities_returns_strings_from_flat_array(
                caps in prop::collection::vec("[a-z.]{1,20}", 0..5)
            ) {
                let val = serde_json::json!({"capabilities": caps});
                let result = extract_capabilities(&val);
                prop_assert_eq!(result, caps);
            }

            #[test]
            fn extract_capabilities_handles_object_array(
                caps in prop::collection::vec("[a-z.]{1,15}", 0..5)
            ) {
                let objects: Vec<serde_json::Value> = caps
                    .iter()
                    .map(|c| serde_json::json!({"name": c}))
                    .collect();
                let val = serde_json::json!({"capabilities": objects});
                let result = extract_capabilities(&val);
                prop_assert_eq!(result, caps);
            }
        }
    }
}

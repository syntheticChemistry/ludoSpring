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

/// Probe a Unix socket to check if it hosts a JSON-RPC primal.
///
/// Sends `lifecycle.status` and parses the response for name and capabilities.
pub fn probe_socket(path: &Path) -> Option<PrimalEndpoint> {
    let stream = UnixStream::connect(path).ok()?;
    let probe = Duration::from_millis(crate::tolerances::PROBE_TIMEOUT_MS);
    stream.set_read_timeout(Some(probe)).ok()?;
    stream.set_write_timeout(Some(probe)).ok()?;

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "lifecycle.status",
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

    let parsed: serde_json::Value = serde_json::from_str(&response).ok()?;
    let result = parsed.get("result")?;

    let name = result
        .get("name")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("unknown")
        .to_owned();

    let capabilities = result
        .get("capabilities")
        .and_then(serde_json::Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(serde_json::Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default();

    Some(PrimalEndpoint {
        socket: path.to_owned(),
        name,
        capabilities,
    })
}

/// Discover all primals in the standard socket directories.
///
/// Probes every `.sock` file found in [`discovery_dirs`] and returns
/// a registry of those that respond to `lifecycle.status`.
#[must_use]
pub fn discover_primals() -> PrimalRegistry {
    let mut registry = PrimalRegistry::new();

    for dir in discovery_dirs() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
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
/// Returns an error string if the connection, serialization, or RPC call fails.
pub fn call_primal(
    endpoint: &PrimalEndpoint,
    method: &str,
    params: &serde_json::Value,
) -> Result<serde_json::Value, String> {
    let stream = UnixStream::connect(&endpoint.socket).map_err(|e| format!("connect: {e}"))?;
    let rpc_timeout = Duration::from_secs(crate::tolerances::RPC_TIMEOUT_SECS);
    stream
        .set_read_timeout(Some(rpc_timeout))
        .map_err(|e| format!("set_read_timeout: {e}"))?;
    stream
        .set_write_timeout(Some(rpc_timeout))
        .map_err(|e| format!("set_write_timeout: {e}"))?;

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": 1
    });

    let mut writer = stream.try_clone().map_err(|e| format!("clone: {e}"))?;
    let mut msg = serde_json::to_string(&request).map_err(|e| format!("serialize: {e}"))?;
    msg.push('\n');
    writer
        .write_all(msg.as_bytes())
        .map_err(|e| format!("write: {e}"))?;
    writer.flush().map_err(|e| format!("flush: {e}"))?;

    let mut reader = BufReader::new(stream);
    let mut response = String::new();
    reader
        .read_line(&mut response)
        .map_err(|e| format!("read: {e}"))?;

    let parsed: serde_json::Value =
        serde_json::from_str(&response).map_err(|e| format!("parse: {e}"))?;

    if let Some(error) = parsed.get("error") {
        return Err(format!("rpc error: {error}"));
    }

    parsed
        .get("result")
        .cloned()
        .ok_or_else(|| "no result in response".to_owned())
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
            socket: PathBuf::from("/tmp/test.sock"),
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
            socket: PathBuf::from("/tmp/a.sock"),
            name: "primal-a".into(),
            capabilities: vec!["security".into()],
        });
        reg.register(&PrimalEndpoint {
            socket: PathBuf::from("/tmp/b.sock"),
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
}

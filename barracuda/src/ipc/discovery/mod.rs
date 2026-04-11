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
//! 3. `std::env::temp_dir()` — development fallback
//!
//! Within each directory, socket files are probed via `lifecycle.status`
//! JSON-RPC calls, falling back to `health.check` + `capabilities.list`.
//! Primals that respond are registered for use.

pub mod capabilities;

use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Discovery tier that resolved a primal socket.
///
/// Per `SPRING_COMPOSITION_PATTERNS` §3, socket discovery follows a
/// six-tier priority chain. Tracking which tier succeeded enables
/// diagnostics and fallback analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiscoveryTier {
    /// Tier 1: explicit `${PRIMAL}_SOCKET` env var.
    ExplicitEnv = 1,
    /// Tier 2: `$XDG_RUNTIME_DIR/biomeos/{primal}-{family}.sock`.
    XdgFamily = 2,
    /// Tier 3: `$XDG_RUNTIME_DIR/biomeos/{primal}.sock`.
    XdgPlain = 3,
    /// Tier 4: `/tmp/biomeos/{primal}.sock`.
    TempFallback = 4,
    /// Tier 5: directory scan with `*.sock` pattern.
    DirectoryScan = 5,
    /// Tier 6: Neural API `capability.discover` sweep.
    NeuralApiSweep = 6,
}

impl core::fmt::Display for DiscoveryTier {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::ExplicitEnv => write!(f, "explicit env"),
            Self::XdgFamily => write!(f, "XDG family socket"),
            Self::XdgPlain => write!(f, "XDG plain socket"),
            Self::TempFallback => write!(f, "temp fallback"),
            Self::DirectoryScan => write!(f, "directory scan"),
            Self::NeuralApiSweep => write!(f, "Neural API sweep"),
        }
    }
}

/// Structured result from primal discovery.
///
/// Per `SPRING_COMPOSITION_PATTERNS` §3, discovery returns either a found
/// endpoint with its resolution tier, or a `NotFound` with the list of
/// paths that were searched.
#[derive(Debug, Clone)]
pub enum DiscoveryResult {
    /// Primal found at a socket with a known resolution tier.
    Found {
        /// The discovered endpoint.
        endpoint: PrimalEndpoint,
        /// Which tier resolved the socket.
        tier: DiscoveryTier,
    },
    /// No primal found after searching all tiers.
    NotFound {
        /// The primal or capability that was sought.
        target: String,
        /// All paths that were probed without success.
        searched: Vec<PathBuf>,
    },
}

impl DiscoveryResult {
    /// Extract the endpoint if discovery succeeded.
    #[must_use]
    pub const fn endpoint(&self) -> Option<&PrimalEndpoint> {
        match self {
            Self::Found { endpoint, .. } => Some(endpoint),
            Self::NotFound { .. } => None,
        }
    }

    /// Whether the discovery succeeded.
    #[must_use]
    pub const fn is_found(&self) -> bool {
        matches!(self, Self::Found { .. })
    }
}

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
/// directory chain. Filters to directories that actually exist on disk.
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
    let probe = Duration::from_millis(crate::tolerances::probe_timeout_ms());
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
pub fn probe_socket(path: &Path) -> Option<PrimalEndpoint> {
    if let Some(ep) = probe_lifecycle_status(path) {
        return Some(ep);
    }
    probe_health_then_capabilities(path)
}

/// Try `lifecycle.status` — the biomeOS-standard probe.
fn probe_lifecycle_status(path: &Path) -> Option<PrimalEndpoint> {
    let parsed = rpc_probe(path, "lifecycle.status")?;
    let result = parsed.get("result")?;

    let name = result
        .get("name")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("unknown")
        .to_owned();

    let mut caps = capabilities::extract_from_lifecycle(result);
    capabilities::inject_base_capabilities(&mut caps);

    Some(PrimalEndpoint {
        socket: path.to_owned(),
        name,
        capabilities: caps,
    })
}

/// Fallback probe: `health.check` for primal name, `capability.list` (or `capabilities.list`) for capabilities.
fn probe_health_then_capabilities(path: &Path) -> Option<PrimalEndpoint> {
    let health = rpc_probe(path, "health.check")?;
    let health_result = health.get("result")?;

    let name = health_result
        .get("primal")
        .or_else(|| health_result.get("name"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or("unknown")
        .to_owned();

    let mut caps = Vec::new();

    if let Some(caps_resp) =
        rpc_probe(path, "capability.list").or_else(|| rpc_probe(path, "capabilities.list"))
    {
        if let Some(result) = caps_resp.get("result") {
            caps = capabilities::extract_from_any(result);
        }
    }

    capabilities::inject_base_capabilities(&mut caps);

    Some(PrimalEndpoint {
        socket: path.to_owned(),
        name,
        capabilities: caps,
    })
}

/// Discover primals by scanning `dirs` for `*.sock` and probing each with [`probe_socket`].
#[must_use]
pub fn discover_primals_in_directories(dirs: &[PathBuf]) -> PrimalRegistry {
    let mut registry = PrimalRegistry::new();

    for dir in dirs {
        let Ok(entries) = std::fs::read_dir(dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
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
#[must_use]
pub fn discover_primals() -> PrimalRegistry {
    let dirs = discovery_dirs();
    discover_primals_in_directories(&dirs)
}

/// Discover a primal endpoint by capability at runtime.
///
/// Scans all socket directories for a primal advertising the requested
/// capability. Returns `None` if no primal serves it.
#[must_use]
pub fn discover_by_capability(capability: &str) -> Option<PrimalEndpoint> {
    let registry = discover_primals();
    registry.find(capability).cloned()
}

/// Tiered discovery for a specific primal by name.
///
/// Follows the six-tier priority chain from `SPRING_COMPOSITION_PATTERNS`
/// §3. Returns a structured [`DiscoveryResult`] with the resolution tier
/// on success or the full list of searched paths on failure.
///
/// Tiers 1–4 probe well-known socket paths; Tier 5 falls back to the
/// existing directory scan; Tier 6 is reserved for Neural API sweep.
#[must_use]
pub fn discover_primal_tiered(primal_name: &str) -> DiscoveryResult {
    let mut searched = Vec::new();
    let family = crate::niche::family_id();

    let env_key = format!("{}_SOCKET", primal_name.to_uppercase().replace('-', "_"));
    if let Ok(explicit) = std::env::var(&env_key) {
        let path = PathBuf::from(&explicit);
        if let Some(ep) = probe_socket(&path) {
            return DiscoveryResult::Found {
                endpoint: ep,
                tier: DiscoveryTier::ExplicitEnv,
            };
        }
        searched.push(path);
    }

    for dir in crate::niche::socket_dirs() {
        if !dir.is_dir() {
            continue;
        }

        let family_sock = dir.join(format!("{primal_name}-{family}.sock"));
        if let Some(ep) = probe_socket(&family_sock) {
            return DiscoveryResult::Found {
                endpoint: ep,
                tier: DiscoveryTier::XdgFamily,
            };
        }
        searched.push(family_sock);

        let plain_sock = dir.join(format!("{primal_name}.sock"));
        if let Some(ep) = probe_socket(&plain_sock) {
            return DiscoveryResult::Found {
                endpoint: ep,
                tier: DiscoveryTier::XdgPlain,
            };
        }
        searched.push(plain_sock);
    }

    let temp_sock = std::env::temp_dir()
        .join(crate::niche::ECOSYSTEM_SOCKET_DIR)
        .join(format!("{primal_name}.sock"));
    if let Some(ep) = probe_socket(&temp_sock) {
        return DiscoveryResult::Found {
            endpoint: ep,
            tier: DiscoveryTier::TempFallback,
        };
    }
    searched.push(temp_sock);

    let registry = discover_primals();
    if let Some(ep) = registry
        .endpoints
        .values()
        .find(|ep| ep.name == primal_name)
    {
        return DiscoveryResult::Found {
            endpoint: ep.clone(),
            tier: DiscoveryTier::DirectoryScan,
        };
    }

    DiscoveryResult::NotFound {
        target: primal_name.to_owned(),
        searched,
    }
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
    let rpc_timeout = Duration::from_secs(crate::tolerances::rpc_timeout_secs());
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
#[allow(clippy::unwrap_used, clippy::expect_used)]
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
        let reg = discover_primals_in_directories(std::slice::from_ref(&dir));
        assert_eq!(reg.endpoint_count(), 0);
        std::fs::remove_dir(&dir).ok();
    }

    #[test]
    fn discovery_tier_display() {
        assert_eq!(DiscoveryTier::ExplicitEnv.to_string(), "explicit env");
        assert_eq!(DiscoveryTier::XdgFamily.to_string(), "XDG family socket");
        assert_eq!(DiscoveryTier::XdgPlain.to_string(), "XDG plain socket");
        assert_eq!(DiscoveryTier::TempFallback.to_string(), "temp fallback");
        assert_eq!(DiscoveryTier::DirectoryScan.to_string(), "directory scan");
        assert_eq!(
            DiscoveryTier::NeuralApiSweep.to_string(),
            "Neural API sweep"
        );
    }

    #[test]
    fn discover_primal_tiered_returns_not_found_for_absent() {
        let result = discover_primal_tiered("nonexistent-test-primal");
        assert!(!result.is_found());
        assert!(result.endpoint().is_none());
        match result {
            DiscoveryResult::NotFound { target, searched } => {
                assert_eq!(target, "nonexistent-test-primal");
                assert!(
                    !searched.is_empty(),
                    "should have searched at least one path"
                );
            }
            DiscoveryResult::Found { .. } => panic!("should not find nonexistent primal"),
        }
    }

    #[test]
    fn discovery_result_is_found_accessors() {
        let found = DiscoveryResult::Found {
            endpoint: PrimalEndpoint {
                socket: PathBuf::from("/tmp/test.sock"),
                name: "test".into(),
                capabilities: vec![],
            },
            tier: DiscoveryTier::ExplicitEnv,
        };
        assert!(found.is_found());
        assert!(found.endpoint().is_some());

        let not_found = DiscoveryResult::NotFound {
            target: "x".into(),
            searched: vec![],
        };
        assert!(!not_found.is_found());
        assert!(not_found.endpoint().is_none());
    }
}

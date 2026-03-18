// SPDX-License-Identifier: AGPL-3.0-or-later
//! ToadStool compute primal integration — typed client for GPU dispatch.
//!
//! Routes compute capability calls through [`NeuralBridge`] to toadStool:
//!
//! - `compute.submit` — Submit a WGSL shader workload for GPU execution
//! - `compute.status` — Query status of a submitted workload
//! - `compute.capabilities` — Query available compute substrates
//!
//! Graceful degradation: returns `ComputeResult { available: false, .. }` when
//! toadStool is not reachable through the Neural API.  CPU fallback is the
//! caller's responsibility — this module only handles the IPC contract.

use super::neural_bridge::NeuralBridge;

/// Result of a toadStool compute operation.
#[derive(Debug, Clone)]
pub struct ComputeResult {
    /// Whether toadStool was available and accepted the workload.
    pub available: bool,
    /// Structured response data from toadStool.
    pub data: serde_json::Value,
    /// Human-readable status or error message.
    pub message: String,
}

/// Substrate capabilities reported by toadStool.
#[derive(Debug, Clone, Default)]
pub struct SubstrateCapabilities {
    /// Whether a GPU is available.
    pub gpu_available: bool,
    /// GPU device name (if available).
    pub gpu_name: String,
    /// Whether f64 (double precision) is supported on GPU.
    pub f64_supported: bool,
    /// Full response for further inspection.
    pub raw: serde_json::Value,
}

/// Submit a WGSL shader workload to toadStool for GPU execution.
///
/// # Arguments
///
/// * `shader_source` — WGSL shader source code (or barraCuda shader name)
/// * `entry_point` — Shader entry point function name
/// * `workgroup_size` — `[x, y, z]` workgroup dimensions
/// * `dispatch_size` — `[x, y, z]` dispatch dimensions
/// * `buffers` — Input/output buffer descriptors
///
/// # Errors
///
/// Returns an error only on non-recoverable failures.
pub fn submit_workload(
    shader_source: &str,
    entry_point: &str,
    workgroup_size: [u32; 3],
    dispatch_size: [u32; 3],
    buffers: &serde_json::Value,
) -> Result<ComputeResult, String> {
    let Ok(bridge) = NeuralBridge::discover() else {
        return Ok(unavailable("No Neural API — GPU compute unavailable"));
    };

    let args = serde_json::json!({
        "shader_source": shader_source,
        "entry_point": entry_point,
        "workgroup_size": workgroup_size,
        "dispatch_size": dispatch_size,
        "buffers": buffers,
        "requester": crate::niche::NICHE_NAME,
    });

    bridge
        .capability_call("compute", "submit", &args)
        .map_or_else(
            |_| Ok(unavailable("toadStool compute.submit unavailable")),
            |result| {
                let message = result
                    .get("status")
                    .and_then(|v| v.as_str())
                    .unwrap_or("submitted")
                    .to_owned();
                Ok(ComputeResult {
                    available: true,
                    data: result,
                    message,
                })
            },
        )
}

/// Query the status of a previously submitted workload.
///
/// # Errors
///
/// Returns an error only on non-recoverable failures.
pub fn workload_status(workload_id: &str) -> Result<ComputeResult, String> {
    let Ok(bridge) = NeuralBridge::discover() else {
        return Ok(unavailable("No Neural API — GPU compute unavailable"));
    };

    let args = serde_json::json!({
        "workload_id": workload_id,
        "requester": crate::niche::NICHE_NAME,
    });

    bridge
        .capability_call("compute", "status", &args)
        .map_or_else(
            |_| Ok(unavailable("toadStool compute.status unavailable")),
            |result| {
                let message = result
                    .get("state")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_owned();
                Ok(ComputeResult {
                    available: true,
                    data: result,
                    message,
                })
            },
        )
}

/// Query available compute substrates from toadStool.
///
/// # Errors
///
/// Returns an error only on non-recoverable failures.
pub fn query_capabilities() -> Result<SubstrateCapabilities, String> {
    let Ok(bridge) = NeuralBridge::discover() else {
        return Ok(SubstrateCapabilities::default());
    };

    let args = serde_json::json!({
        "requester": crate::niche::NICHE_NAME,
    });

    bridge
        .capability_call("compute", "capabilities", &args)
        .map_or_else(
            |_| Ok(SubstrateCapabilities::default()),
            |result| {
                let gpu_available = result
                    .get("gpu_available")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(false);
                let gpu_name = result
                    .get("gpu_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_owned();
                let f64_supported = result
                    .get("f64_supported")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(false);
                Ok(SubstrateCapabilities {
                    gpu_available,
                    gpu_name,
                    f64_supported,
                    raw: result,
                })
            },
        )
}

/// Submit a workload for direct GPU dispatch (lower latency, bypasses job queue).
///
/// Uses `compute.dispatch.submit` instead of `compute.submit`. Prefer this
/// for real-time game compute (fog of war, pathfinding, lighting) where
/// latency matters more than queuing guarantees.
///
/// # Errors
///
/// Returns an error only on non-recoverable failures.
pub fn dispatch_submit(
    shader_source: &str,
    entry_point: &str,
    workgroup_size: [u32; 3],
    dispatch_size: [u32; 3],
    buffers: &serde_json::Value,
) -> Result<ComputeResult, String> {
    let Ok(bridge) = NeuralBridge::discover() else {
        return Ok(unavailable("No Neural API — GPU dispatch unavailable"));
    };

    let args = serde_json::json!({
        "shader_source": shader_source,
        "entry_point": entry_point,
        "workgroup_size": workgroup_size,
        "dispatch_size": dispatch_size,
        "buffers": buffers,
        "requester": crate::niche::NICHE_NAME,
    });

    bridge
        .capability_call("compute.dispatch", "submit", &args)
        .map_or_else(
            |_| Ok(unavailable("toadStool compute.dispatch.submit unavailable")),
            |result| {
                let message = result
                    .get("status")
                    .and_then(|v| v.as_str())
                    .unwrap_or("dispatched")
                    .to_owned();
                Ok(ComputeResult {
                    available: true,
                    data: result,
                    message,
                })
            },
        )
}

/// Query the result of a direct-dispatched workload.
///
/// # Errors
///
/// Returns an error only on non-recoverable failures.
pub fn dispatch_result(workload_id: &str) -> Result<ComputeResult, String> {
    let Ok(bridge) = NeuralBridge::discover() else {
        return Ok(unavailable("No Neural API — GPU dispatch unavailable"));
    };

    let args = serde_json::json!({
        "workload_id": workload_id,
        "requester": crate::niche::NICHE_NAME,
    });

    bridge
        .capability_call("compute.dispatch", "result", &args)
        .map_or_else(
            |_| Ok(unavailable("toadStool compute.dispatch.result unavailable")),
            |result| {
                let message = result
                    .get("state")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_owned();
                Ok(ComputeResult {
                    available: true,
                    data: result,
                    message,
                })
            },
        )
}

/// Query dispatch-tier capabilities from toadStool.
///
/// # Errors
///
/// Returns an error only on non-recoverable failures.
pub fn dispatch_capabilities() -> Result<SubstrateCapabilities, String> {
    let Ok(bridge) = NeuralBridge::discover() else {
        return Ok(SubstrateCapabilities::default());
    };

    let args = serde_json::json!({
        "requester": crate::niche::NICHE_NAME,
    });

    bridge
        .capability_call("compute.dispatch", "capabilities", &args)
        .map_or_else(
            |_| Ok(SubstrateCapabilities::default()),
            |result| {
                let gpu_available = result
                    .get("gpu_available")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(false);
                let gpu_name = result
                    .get("gpu_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_owned();
                let f64_supported = result
                    .get("f64_supported")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(false);
                Ok(SubstrateCapabilities {
                    gpu_available,
                    gpu_name,
                    f64_supported,
                    raw: result,
                })
            },
        )
}

fn unavailable(message: &str) -> ComputeResult {
    ComputeResult {
        available: false,
        data: serde_json::Value::Null,
        message: message.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unavailable_result_has_correct_state() {
        let r = unavailable("test");
        assert!(!r.available);
        assert_eq!(r.message, "test");
        assert!(r.data.is_null());
    }

    #[test]
    fn submit_degrades_gracefully_without_neural_api() {
        let result = submit_workload(
            "@compute void main() {}",
            "main",
            [1, 1, 1],
            [1, 1, 1],
            &serde_json::json!([]),
        );
        let r = result.expect("should not error");
        assert!(!r.available);
    }

    #[test]
    fn status_degrades_gracefully_without_neural_api() {
        let result = workload_status("nonexistent");
        let r = result.expect("should not error");
        assert!(!r.available);
    }

    #[test]
    fn capabilities_degrades_gracefully_without_neural_api() {
        let result = query_capabilities();
        let caps = result.expect("should not error");
        assert!(!caps.gpu_available);
    }

    #[test]
    fn dispatch_submit_degrades_gracefully_without_neural_api() {
        let result = dispatch_submit(
            "@compute void main() {}",
            "main",
            [1, 1, 1],
            [1, 1, 1],
            &serde_json::json!([]),
        );
        let r = result.expect("should not error");
        assert!(!r.available);
    }

    #[test]
    fn dispatch_result_degrades_gracefully_without_neural_api() {
        let result = dispatch_result("nonexistent");
        let r = result.expect("should not error");
        assert!(!r.available);
    }

    #[test]
    fn dispatch_capabilities_degrades_gracefully_without_neural_api() {
        let result = dispatch_capabilities();
        let caps = result.expect("should not error");
        assert!(!caps.gpu_available);
    }
}

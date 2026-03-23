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
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ComputeResult {
    /// Whether toadStool was available and accepted the workload.
    pub available: bool,
    /// Structured response data from toadStool.
    pub data: serde_json::Value,
    /// Human-readable status or error message.
    pub message: String,
}

/// Substrate capabilities reported by toadStool.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
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

    let args = submit_workload_args(
        shader_source,
        entry_point,
        workgroup_size,
        dispatch_size,
        buffers,
    );

    bridge
        .capability_call("compute", "submit", &args)
        .map_or_else(
            |_| Ok(unavailable("toadStool compute.submit unavailable")),
            |result| Ok(compute_result_from_status_field(result, "submitted")),
        )
}

fn submit_workload_args(
    shader_source: &str,
    entry_point: &str,
    workgroup_size: [u32; 3],
    dispatch_size: [u32; 3],
    buffers: &serde_json::Value,
) -> serde_json::Value {
    serde_json::json!({
        "shader_source": shader_source,
        "entry_point": entry_point,
        "workgroup_size": workgroup_size,
        "dispatch_size": dispatch_size,
        "buffers": buffers,
        "requester": crate::niche::NICHE_NAME,
    })
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

    let args = workload_status_args(workload_id);

    bridge
        .capability_call("compute", "status", &args)
        .map_or_else(
            |_| Ok(unavailable("toadStool compute.status unavailable")),
            |result| Ok(compute_result_from_state_field(result)),
        )
}

fn workload_status_args(workload_id: &str) -> serde_json::Value {
    serde_json::json!({
        "workload_id": workload_id,
        "requester": crate::niche::NICHE_NAME,
    })
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

    let args = compute_capabilities_request_args();

    bridge
        .capability_call("compute", "capabilities", &args)
        .map_or_else(
            |_| Ok(SubstrateCapabilities::default()),
            |result| Ok(substrate_capabilities_from_response(result)),
        )
}

fn compute_capabilities_request_args() -> serde_json::Value {
    serde_json::json!({
        "requester": crate::niche::NICHE_NAME,
    })
}

fn substrate_capabilities_from_response(result: serde_json::Value) -> SubstrateCapabilities {
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
    SubstrateCapabilities {
        gpu_available,
        gpu_name,
        f64_supported,
        raw: result,
    }
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

    let args = dispatch_submit_args(
        shader_source,
        entry_point,
        workgroup_size,
        dispatch_size,
        buffers,
    );

    bridge
        .capability_call("compute.dispatch", "submit", &args)
        .map_or_else(
            |_| Ok(unavailable("toadStool compute.dispatch.submit unavailable")),
            |result| Ok(compute_result_from_status_field(result, "dispatched")),
        )
}

fn dispatch_submit_args(
    shader_source: &str,
    entry_point: &str,
    workgroup_size: [u32; 3],
    dispatch_size: [u32; 3],
    buffers: &serde_json::Value,
) -> serde_json::Value {
    serde_json::json!({
        "shader_source": shader_source,
        "entry_point": entry_point,
        "workgroup_size": workgroup_size,
        "dispatch_size": dispatch_size,
        "buffers": buffers,
        "requester": crate::niche::NICHE_NAME,
    })
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

    let args = dispatch_result_args(workload_id);

    bridge
        .capability_call("compute.dispatch", "result", &args)
        .map_or_else(
            |_| Ok(unavailable("toadStool compute.dispatch.result unavailable")),
            |result| Ok(compute_result_from_state_field(result)),
        )
}

fn dispatch_result_args(workload_id: &str) -> serde_json::Value {
    serde_json::json!({
        "workload_id": workload_id,
        "requester": crate::niche::NICHE_NAME,
    })
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

    let args = compute_capabilities_request_args();

    bridge
        .capability_call("compute.dispatch", "capabilities", &args)
        .map_or_else(
            |_| Ok(SubstrateCapabilities::default()),
            |result| Ok(substrate_capabilities_from_response(result)),
        )
}

fn unavailable(message: &str) -> ComputeResult {
    ComputeResult {
        available: false,
        data: serde_json::Value::Null,
        message: message.to_owned(),
    }
}

fn compute_result_from_status_field(
    result: serde_json::Value,
    default_status: &'static str,
) -> ComputeResult {
    let message = result
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or(default_status)
        .to_owned();
    ComputeResult {
        available: true,
        data: result,
        message,
    }
}

fn compute_result_from_state_field(result: serde_json::Value) -> ComputeResult {
    let message = result
        .get("state")
        .and_then(|v| v.as_str())
        .or_else(|| result.get("status").and_then(|v| v.as_str()))
        .unwrap_or("unknown")
        .to_owned();
    ComputeResult {
        available: true,
        data: result,
        message,
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

    #[test]
    fn compute_result_constructed_with_response_data() {
        let data = serde_json::json!({
            "status": "queued",
            "workload_id": "w-abc",
            "queue_depth": 2
        });
        let r = ComputeResult {
            available: true,
            data: data.clone(),
            message: "submitted".to_string(),
        };
        assert!(r.available);
        assert_eq!(r.message, "submitted");
        assert_eq!(r.data, data);
        assert_eq!(r.data["workload_id"], "w-abc");
    }

    #[test]
    fn substrate_capabilities_constructed_from_wire_shape() {
        let raw = serde_json::json!({
            "gpu_available": true,
            "gpu_name": "Mock GPU",
            "f64_supported": false,
            "extra": "ignored by fields"
        });
        let caps = SubstrateCapabilities {
            gpu_available: true,
            gpu_name: "Mock GPU".to_string(),
            f64_supported: false,
            raw: raw.clone(),
        };
        assert!(caps.gpu_available);
        assert_eq!(caps.gpu_name, "Mock GPU");
        assert!(!caps.f64_supported);
        assert_eq!(caps.raw["extra"], "ignored by fields");
    }

    #[test]
    fn substrate_capabilities_default_matches_empty_substrate() {
        let caps = SubstrateCapabilities::default();
        assert!(!caps.gpu_available);
        assert!(caps.gpu_name.is_empty());
        assert!(!caps.f64_supported);
        assert!(caps.raw.is_null());
    }

    #[test]
    fn compute_result_serde_round_trip() {
        let original = ComputeResult {
            available: true,
            data: serde_json::json!({ "k": 2 }),
            message: "ok".to_string(),
        };
        let json = serde_json::to_string(&original).expect("serialize");
        let back: ComputeResult = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.message, original.message);
        assert_eq!(back.available, original.available);
        assert_eq!(back.data, original.data);
    }

    #[test]
    fn substrate_capabilities_serde_round_trip() {
        let original = SubstrateCapabilities {
            gpu_available: true,
            gpu_name: "Test GPU".to_string(),
            f64_supported: true,
            raw: serde_json::json!({ "x": 1 }),
        };
        let json = serde_json::to_string(&original).expect("serialize");
        let back: SubstrateCapabilities = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.gpu_name, original.gpu_name);
        assert_eq!(back.gpu_available, original.gpu_available);
        assert_eq!(back.f64_supported, original.f64_supported);
        assert_eq!(back.raw, original.raw);
    }

    #[test]
    fn submit_workload_args_shape() {
        let buffers = serde_json::json!([{"id": 1}]);
        let args = super::submit_workload_args("shader", "main", [2, 3, 4], [8, 1, 1], &buffers);
        assert_eq!(args["shader_source"], "shader");
        assert_eq!(args["entry_point"], "main");
        assert_eq!(args["workgroup_size"], serde_json::json!([2, 3, 4]));
        assert_eq!(args["dispatch_size"], serde_json::json!([8, 1, 1]));
        assert_eq!(args["buffers"], buffers);
        assert_eq!(args["requester"], crate::niche::NICHE_NAME);
    }

    #[test]
    fn workload_and_dispatch_status_args_include_requester() {
        let a = super::workload_status_args("wid");
        assert_eq!(a["workload_id"], "wid");
        assert_eq!(a["requester"], crate::niche::NICHE_NAME);

        let d = super::dispatch_result_args("wid2");
        assert_eq!(d["workload_id"], "wid2");
        assert_eq!(d["requester"], crate::niche::NICHE_NAME);
    }

    #[test]
    fn dispatch_submit_args_matches_submit_workload() {
        let b = serde_json::json!([]);
        let s = super::submit_workload_args("a", "b", [1, 1, 1], [2, 2, 2], &b);
        let d = super::dispatch_submit_args("a", "b", [1, 1, 1], [2, 2, 2], &b);
        assert_eq!(s, d);
    }

    #[test]
    fn compute_capabilities_request_args() {
        let c = super::compute_capabilities_request_args();
        assert_eq!(c["requester"], crate::niche::NICHE_NAME);
    }

    #[test]
    fn compute_result_from_status_field_defaults() {
        let r = super::compute_result_from_status_field(serde_json::json!({}), "submitted");
        assert_eq!(r.message, "submitted");
        assert!(r.available);
        let r2 = super::compute_result_from_status_field(
            serde_json::json!({ "status": "queued" }),
            "submitted",
        );
        assert_eq!(r2.message, "queued");
    }

    #[test]
    fn compute_result_from_status_field_non_string_status_uses_default() {
        let r = super::compute_result_from_status_field(
            serde_json::json!({ "status": 9 }),
            "dispatched",
        );
        assert_eq!(r.message, "dispatched");
    }

    #[test]
    fn compute_result_from_state_field_defaults_and_parses() {
        let r = super::compute_result_from_state_field(serde_json::json!({}));
        assert_eq!(r.message, "unknown");
        let r2 = super::compute_result_from_state_field(serde_json::json!({ "state": "done" }));
        assert_eq!(r2.message, "done");
    }

    #[test]
    fn compute_result_from_state_field_non_string_uses_unknown() {
        let r = super::compute_result_from_state_field(serde_json::json!({ "state": [] }));
        assert_eq!(r.message, "unknown");
    }

    #[test]
    fn substrate_capabilities_from_response_parses_flags_and_defaults() {
        let r = super::substrate_capabilities_from_response(serde_json::json!({
            "gpu_available": true,
            "gpu_name": "G",
            "f64_supported": false,
        }));
        assert!(r.gpu_available);
        assert_eq!(r.gpu_name, "G");
        assert!(!r.f64_supported);
        assert_eq!(r.raw["gpu_name"], "G");
    }

    #[test]
    fn substrate_capabilities_from_response_missing_fields_defaults() {
        let r = super::substrate_capabilities_from_response(serde_json::json!({}));
        assert!(!r.gpu_available);
        assert_eq!(r.gpu_name, "");
        assert!(!r.f64_supported);
    }

    #[test]
    fn substrate_capabilities_from_response_gpu_name_non_string_yields_empty() {
        let r = super::substrate_capabilities_from_response(serde_json::json!({
            "gpu_name": 42
        }));
        assert_eq!(r.gpu_name, "");
    }
}

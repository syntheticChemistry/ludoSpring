// SPDX-License-Identifier: AGPL-3.0-or-later
//! coralReef shader compiler integration — typed client for sovereign GPU compilation.
//!
//! Routes shader capability calls through [`NeuralBridge`] to coralReef:
//!
//! - `shader.compile` — compile WGSL source to native GPU binary
//! - `shader.list` — enumerate available compiled shaders
//!
//! Graceful degradation: returns `ShaderResult { available: false, .. }` when
//! coralReef is not reachable through the Neural API. In that case, the engine
//! falls back to embedded WGSL dispatched through toadStool or in-process wgpu.

use super::neural_bridge::NeuralBridge;

/// Capability domain for coralReef shader operations.
const CAP_SHADER: &str = "shader";

/// Result of a coralReef shader operation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ShaderResult {
    /// Whether coralReef was available.
    pub available: bool,
    /// Response data from the operation.
    pub data: serde_json::Value,
}

/// Compile a WGSL shader source via coralReef.
///
/// # Errors
///
/// Returns an error only on non-recoverable failures.
pub fn compile_wgsl(source: &str, entry_point: &str, label: &str) -> Result<ShaderResult, String> {
    let Ok(bridge) = NeuralBridge::discover() else {
        return Ok(unavailable());
    };

    let args = serde_json::json!({
        "source": source,
        "entry_point": entry_point,
        "label": label,
        "format": "wgsl",
    });

    bridge
        .capability_call(CAP_SHADER, "compile", &args)
        .map_or_else(|_| Ok(unavailable()), |result| Ok(shader_success(result)))
}

/// List compiled shaders available in coralReef's cache.
///
/// # Errors
///
/// Returns an error only on non-recoverable failures.
pub fn list_shaders() -> Result<ShaderResult, String> {
    let Ok(bridge) = NeuralBridge::discover() else {
        return Ok(unavailable());
    };

    let args = serde_json::json!({});

    bridge
        .capability_call(CAP_SHADER, "list", &args)
        .map_or_else(|_| Ok(unavailable()), |result| Ok(shader_success(result)))
}

fn unavailable() -> ShaderResult {
    ShaderResult {
        available: false,
        data: serde_json::json!({"reason": "coralReef not available via Neural API"}),
    }
}

const fn shader_success(data: serde_json::Value) -> ShaderResult {
    ShaderResult {
        available: true,
        data,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compile_wgsl_degrades_without_neural_api() {
        let result = compile_wgsl("fn main() {}", "main", "test-shader").unwrap();
        assert!(!result.available);
        assert!(result.data["reason"].as_str().is_some());
    }

    #[test]
    fn list_shaders_degrades_without_neural_api() {
        let result = list_shaders().unwrap();
        assert!(!result.available);
    }

    #[test]
    fn shader_result_serde_roundtrip() {
        let r = ShaderResult {
            available: true,
            data: serde_json::json!({"binary": "cached"}),
        };
        let json = serde_json::to_string(&r).unwrap();
        let back: ShaderResult = serde_json::from_str(&json).unwrap();
        assert_eq!(back.available, true);
        assert_eq!(back.data["binary"], "cached");
    }
}

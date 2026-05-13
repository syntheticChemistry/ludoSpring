// SPDX-License-Identifier: AGPL-3.0-or-later
//! Tier 2 Science API response types.
//!
//! Structured data returned by upstream primal methods:
//! - [`WorkloadValidation`] from `toadstool.validate`
//! - [`PrecisionAdvice`] from `barracuda.precision.route`
//!
//! Wire contract: `primalSpring/docs/LIVE_SCIENCE_API.md`

/// Result of workload pre-flight validation from toadStool.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorkloadValidation {
    /// Whether toadStool responded.
    pub available: bool,
    /// Whether the workload is valid for dispatch.
    pub valid: bool,
    /// Whether GPU hardware is available for this workload.
    pub gpu_available: bool,
    /// Recommended precision tier (e.g. "f32", "f64", "mixed").
    pub precision_tier: String,
    /// Estimated dispatch time in milliseconds.
    pub estimated_dispatch_time_ms: Option<u64>,
    /// Pre-flight warnings (non-fatal issues).
    pub warnings: Vec<String>,
    /// Capabilities required by this workload.
    pub required_capabilities: Vec<String>,
}

impl WorkloadValidation {
    pub(crate) fn unavailable() -> Self {
        Self {
            available: false,
            valid: false,
            gpu_available: false,
            precision_tier: String::new(),
            estimated_dispatch_time_ms: None,
            warnings: vec!["toadStool not reachable".into()],
            required_capabilities: Vec::new(),
        }
    }

    /// Parse a toadStool `toadstool.validate` JSON response into structured form.
    pub fn from_response(v: &serde_json::Value) -> Self {
        Self {
            available: true,
            valid: v
                .get("valid")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false),
            gpu_available: v
                .get("gpu_available")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false),
            precision_tier: v
                .get("precision_tier")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("")
                .to_owned(),
            estimated_dispatch_time_ms: v
                .get("estimated_dispatch_time_ms")
                .and_then(serde_json::Value::as_u64),
            warnings: v
                .get("warnings")
                .and_then(serde_json::Value::as_array)
                .map(|arr| {
                    arr.iter()
                        .filter_map(|w| w.as_str().map(str::to_owned))
                        .collect()
                })
                .unwrap_or_default(),
            required_capabilities: v
                .get("required_capabilities")
                .and_then(serde_json::Value::as_array)
                .map(|arr| {
                    arr.iter()
                        .filter_map(|c| c.as_str().map(str::to_owned))
                        .collect()
                })
                .unwrap_or_default(),
        }
    }
}

/// Precision routing advisory from barraCuda.
///
/// Fields align with `primalSpring/docs/LIVE_SCIENCE_API.md` §barracuda.precision.route.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PrecisionAdvice {
    /// Whether barraCuda precision routing responded.
    pub available: bool,
    /// Recommended precision tier label (e.g. "F32", "F64", "DF64", "mixed").
    pub recommended_tier: String,
    /// Whether fused multiply-add is safe for this domain at this precision.
    pub fma_safe: bool,
    /// Whether this precision requires compiler support (coralReef sovereign).
    pub requires_compiler: bool,
    /// Hardware routing hint echoed back from the query.
    pub hardware_hint: String,
    /// Full advisory response for further inspection.
    pub raw: serde_json::Value,
}

impl PrecisionAdvice {
    pub(crate) const fn unavailable() -> Self {
        Self {
            available: false,
            recommended_tier: String::new(),
            fma_safe: false,
            requires_compiler: false,
            hardware_hint: String::new(),
            raw: serde_json::Value::Null,
        }
    }

    /// Parse a barraCuda `precision.route` JSON response into structured form.
    pub fn from_response(v: &serde_json::Value) -> Self {
        Self {
            available: true,
            recommended_tier: v
                .get("recommended_tier")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("F64")
                .to_owned(),
            fma_safe: v
                .get("fma_safe")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(true),
            requires_compiler: v
                .get("requires_compiler")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false),
            hardware_hint: v
                .get("hardware_hint")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("compute")
                .to_owned(),
            raw: v.clone(),
        }
    }
}

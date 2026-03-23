// SPDX-License-Identifier: AGPL-3.0-or-later
//! Typed tarpc RPC surface for ludoSpring (optional `tarpc-ipc` feature).
//!
//! Complements newline-delimited JSON-RPC in [`super`] with a Rust-native
//! [`LudoSpringService`] trait aligned to the same science payloads as
//! the shared IPC param and result types. Health and lifecycle responses use
//! [`serde_json::Value`] so they stay flexible while the wire format evolves;
//! science methods use the shared IPC structs for parity with JSON-RPC.

use crate::ipc::params::{
    AccessibilityParams, AnalyzeUiParams, DifficultyAdjustmentParams, EngagementParams,
    EvaluateFlowParams, FittsCostParams, GenerateNoiseParams, WfcStepParams,
};
use crate::ipc::results::{
    AccessibilityResult, DifficultyAdjustmentResult, EngagementResult, FittsCostResult, FlowResult,
    NoiseResult, UiAnalysisResult, WfcStepResult,
};

/// Rust-native tarpc interface mirroring ludoSpring JSON-RPC science and health methods.
///
/// Transport and server implementation are not included in this skeleton; the trait
/// exists so ecosystem callers can share generated stubs alongside JSON-RPC.
#[tarpc::service]
pub trait LudoSpringService {
    /// Full health payload (`health.check`): primal name, version, domain, capabilities.
    async fn health_check() -> serde_json::Value;

    /// Liveness probe (`health.liveness`): process responsiveness only.
    async fn health_liveness() -> serde_json::Value;

    /// Readiness probe (`health.readiness`): subsystem readiness for workloads.
    async fn health_readiness() -> serde_json::Value;

    /// Lifecycle discovery (`lifecycle.status`): name, version, domain, status, capabilities.
    async fn lifecycle_status() -> serde_json::Value;

    /// Flow-state evaluation (`game.evaluate_flow`).
    async fn evaluate_flow(params: EvaluateFlowParams) -> FlowResult;

    /// Fitts movement time and index of difficulty (`game.fitts_cost`).
    async fn fitts_cost(params: FittsCostParams) -> FittsCostResult;

    /// Engagement metrics (`game.engagement`).
    async fn engagement(params: EngagementParams) -> EngagementResult;

    /// Fractional Brownian noise sample (`game.generate_noise`).
    async fn generate_noise(params: GenerateNoiseParams) -> NoiseResult;

    /// Tufte-style UI analysis (`game.analyze_ui`).
    async fn analyze_ui(params: AnalyzeUiParams) -> UiAnalysisResult;

    /// Visual accessibility scoring (`game.accessibility`).
    async fn accessibility(params: AccessibilityParams) -> AccessibilityResult;

    /// Single wave-function-collapse propagation step (`game.wfc_step`).
    async fn wfc_step(params: WfcStepParams) -> WfcStepResult;

    /// Dynamic difficulty adjustment suggestion (`game.difficulty_adjustment`).
    async fn difficulty_adjustment(
        params: DifficultyAdjustmentParams,
    ) -> DifficultyAdjustmentResult;

    /// Capability list with cost and dependency metadata (`capabilities.list` / MCP-style discovery).
    async fn capabilities_list() -> serde_json::Value;
}

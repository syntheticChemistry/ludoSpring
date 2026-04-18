// SPDX-License-Identifier: AGPL-3.0-or-later
//! Tier A tensor path — first in-tree use of barraCuda [`TensorSession`] on the
//! game engine GPU context.
//!
//! [`super::gpu_context::GpuContext::tensor_session`] hands out a fused-op
//! session that shares the same [`barracuda::device::WgpuDevice`] as custom WGSL
//! dispatches. This module wires the logistic sigmoid (used on CPU by
//! [`crate::interaction::flow::DifficultyCurve`]) through that session so
//! product code has a validated CPU vs GPU parity hook alongside the existing
//! scalar [`barracuda::activations::sigmoid`] path.

use barracuda::error::BarracudaError;

use super::gpu_context::GpuContext;

/// Run logistic sigmoid on a batch of `f32` values via [`TensorSession`].
///
/// Empty input returns an empty vector without touching the GPU.
///
/// # Panics
///
/// Panics if tensor upload, recording, execution, or readback fails (for
/// example after device loss).
#[must_use]
pub fn sigmoid_batch_gpu(ctx: &GpuContext, values: &[f32]) -> Vec<f32> {
    try_sigmoid_batch_gpu(ctx, values).expect("TensorSession sigmoid batch")
}

fn try_sigmoid_batch_gpu(ctx: &GpuContext, values: &[f32]) -> Result<Vec<f32>, BarracudaError> {
    if values.is_empty() {
        return Ok(Vec::new());
    }
    let mut session = ctx.tensor_session();
    let input = session.tensor(values)?;
    let output = session.sigmoid(&input)?;
    session.run()?;
    output.to_vec()
}

/// Compare CPU [`barracuda::activations::sigmoid`] against the GPU tensor path
/// on a fixed probe set (aligned with exp030 unary tolerances).
///
/// Returns `false` if the GPU path errors or any element differs beyond
/// [`crate::tolerances::gpu::GPU_UNARY_ABS_TOL`].
#[must_use]
pub fn validate_sigmoid_cpu_gpu_parity(ctx: &GpuContext) -> bool {
    use crate::tolerances::gpu::GPU_UNARY_ABS_TOL;

    const PROBE: &[f32] = &[-10.0, -2.0, -1.0, 0.0, 0.5, 1.0, 2.0, 10.0];

    let Ok(gpu) = try_sigmoid_batch_gpu(ctx, PROBE) else {
        return false;
    };
    if gpu.len() != PROBE.len() {
        return false;
    }
    for (&x, &g) in PROBE.iter().zip(gpu.iter()) {
        let cpu = barracuda::activations::sigmoid(f64::from(x));
        if (f64::from(g) - cpu).abs() > GPU_UNARY_ABS_TOL {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn parity_runs_when_gpu_available() {
        let Some(ctx) = GpuContext::try_new().await else {
            return;
        };
        assert!(validate_sigmoid_cpu_gpu_parity(&ctx));
    }
}

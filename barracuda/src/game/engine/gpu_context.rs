// SPDX-License-Identifier: AGPL-3.0-or-later
//! GPU compute context — shared device lifecycle and `TensorSession` access.
//!
//! Wraps a [`barracuda::device::WgpuDevice`] behind an `Arc` so that both
//! custom game WGSL shaders ([`super::gpu::GpuOp`]) and barraCuda's fused
//! tensor pipeline ([`TensorSession`]) share a single adapter/device pair.
//!
//! # Discovery
//!
//! The context probes for GPU availability at construction time.  When no
//! GPU is present (CI, headless server, CPU-only build), construction
//! returns `None` through [`GpuContext::try_new`] — callers fall back to
//! CPU implementations without panicking.
//!
//! # Relationship to toadStool
//!
//! In-process GPU via this context is the fast path.  When toadStool is
//! reachable over IPC (`compute.submit`), the engine can additionally
//! offload work there.  The two paths are complementary, not exclusive.
//! [`super::gpu::GpuAvailability`] captures the runtime probe result.

use std::sync::Arc;

use barracuda::device::WgpuDevice;
use barracuda::session::TensorSession;

/// Shared GPU context — owns the device, hands out `TensorSession`s.
#[derive(Clone)]
pub struct GpuContext {
    device: Arc<WgpuDevice>,
}

impl GpuContext {
    /// Probe for a GPU and create the context.
    ///
    /// Returns `None` when no suitable adapter is found (e.g. headless CI).
    /// Adapter negotiation is async because wgpu's adapter request requires it.
    pub async fn try_new() -> Option<Self> {
        let device = WgpuDevice::new().await.ok()?;
        Some(Self {
            device: Arc::new(device),
        })
    }

    /// Wrap an existing device (e.g. one obtained from toadStool IPC).
    #[must_use]
    pub const fn from_device(device: Arc<WgpuDevice>) -> Self {
        Self { device }
    }

    /// Create a fresh [`TensorSession`] bound to this device.
    ///
    /// Each session records its own op graph — call `run()` to execute.
    /// Compiled pipelines are cached inside `TensorSession`; creating
    /// multiple sessions from the same context is cheap after the first.
    #[must_use]
    pub fn tensor_session(&self) -> TensorSession {
        TensorSession::with_device(Arc::clone(&self.device))
    }

    /// Shared reference to the underlying device.
    #[must_use]
    pub fn device(&self) -> &WgpuDevice {
        &self.device
    }

    /// `Arc` handle to the device — for callers that need ownership.
    #[must_use]
    pub fn device_arc(&self) -> Arc<WgpuDevice> {
        Arc::clone(&self.device)
    }
}

impl std::fmt::Debug for GpuContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GpuContext")
            .field("device", &"<WgpuDevice>")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn try_new_returns_none_without_gpu() {
        // CI / headless — may return None or Some depending on hardware.
        // Either outcome is valid; we just verify no panic.
        let _ctx = GpuContext::try_new().await;
    }
}

// SPDX-License-Identifier: AGPL-3.0-or-later

//! Hardware substrate kinds, capability flags, and rich [`SubstrateInfo`] descriptors.

/// Substrate kind for capability-based routing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubstrateKind {
    /// CPU substrate (general-purpose, SIMD).
    Cpu,
    /// GPU substrate (shader dispatch, high throughput).
    Gpu,
    /// NPU substrate (quantized inference).
    Npu,
}

/// Hardware capability flags for substrate matching.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Capability {
    /// Double-precision floating-point compute.
    F64Compute,
    /// Single-precision floating-point compute.
    F32Compute,
    /// GPU shader dispatch (compute/vertex/fragment).
    ShaderDispatch,
    /// SIMD vector operations.
    SimdVector,
    /// PCIe host-device transfer.
    PcieTransfer,
    /// Quantized integer inference.
    QuantizedInference {
        /// Bit width for quantized weights/activations (e.g. 8 for int8).
        bits: u8,
    },
}

/// Rich substrate descriptor with capabilities and performance hints.
#[derive(Debug, Clone)]
pub struct SubstrateInfo {
    /// Substrate type (CPU, GPU, NPU).
    pub kind: SubstrateKind,
    /// Human-readable device name.
    pub name: String,
    /// Supported hardware capabilities.
    pub capabilities: Vec<Capability>,
    /// Peak throughput in GFLOPS.
    pub flops_gflops: f64,
    /// Whether this GPU can run compute and render passes concurrently.
    pub concurrent_compute_render: bool,
}

impl SubstrateInfo {
    /// Default CPU substrate (F64, F32, SIMD).
    #[must_use]
    pub fn default_cpu() -> Self {
        Self {
            kind: SubstrateKind::Cpu,
            name: "CPU".to_string(),
            capabilities: vec![
                Capability::F64Compute,
                Capability::F32Compute,
                Capability::SimdVector,
            ],
            flops_gflops: 200.0,
            concurrent_compute_render: false,
        }
    }

    /// Default GPU substrate (F32, shaders, `PCIe`).
    #[must_use]
    pub fn default_gpu() -> Self {
        Self {
            kind: SubstrateKind::Gpu,
            name: "GPU".to_string(),
            capabilities: vec![
                Capability::F32Compute,
                Capability::ShaderDispatch,
                Capability::PcieTransfer,
            ],
            flops_gflops: 29_000.0,
            concurrent_compute_render: true,
        }
    }

    /// Default NPU substrate (F32, quantized inference, `PCIe`).
    #[must_use]
    pub fn default_npu() -> Self {
        Self {
            kind: SubstrateKind::Npu,
            name: "NPU".to_string(),
            capabilities: vec![
                Capability::F32Compute,
                Capability::QuantizedInference { bits: 8 },
                Capability::PcieTransfer,
            ],
            flops_gflops: 50.0,
            concurrent_compute_render: false,
        }
    }
}

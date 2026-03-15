// SPDX-License-Identifier: AGPL-3.0-or-later
// Batch engagement composite: weighted dot product of 5 components per sample.
// Validated against barraCuda CPU `metrics::engagement::compute_engagement`.
// f32 GPU vs f64 CPU tolerance: < 1e-4.

@group(0) @binding(0) var<storage, read> components: array<f32>;
@group(0) @binding(1) var<storage, read> weights: array<f32>;
@group(0) @binding(2) var<storage, read_write> output: array<f32>;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    let n = arrayLength(&output);
    if i >= n {
        return;
    }
    var sum: f32 = 0.0;
    for (var j = 0u; j < 5u; j = j + 1u) {
        sum += components[i * 5u + j] * weights[j];
    }
    output[i] = clamp(sum, 0.0, 1.0);
}

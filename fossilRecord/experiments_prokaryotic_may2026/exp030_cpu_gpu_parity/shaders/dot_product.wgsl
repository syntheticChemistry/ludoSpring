// SPDX-License-Identifier: AGPL-3.0-or-later
// Elementwise multiply for dot product: result[i] = a[i] * b[i]
// Reduce-sum the result on the host (or chain with reduce_sum.wgsl).

@group(0) @binding(0) var<storage, read> a: array<f32>;
@group(0) @binding(1) var<storage, read> b: array<f32>;
@group(0) @binding(2) var<storage, read_write> result: array<f32>;

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if i < arrayLength(&a) {
        result[i] = a[i] * b[i];
    }
}

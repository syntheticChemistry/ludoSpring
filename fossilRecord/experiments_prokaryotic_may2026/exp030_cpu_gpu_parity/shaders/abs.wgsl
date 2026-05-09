// SPDX-License-Identifier: AGPL-3.0-or-later
// Absolute value: output[i] = abs(input[i])

@group(0) @binding(0) var<storage, read> input: array<f32>;
@group(0) @binding(1) var<storage, read_write> output: array<f32>;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if i < arrayLength(&input) {
        output[i] = abs(input[i]);
    }
}

// SPDX-License-Identifier: AGPL-3.0-or-later
// Linear congruential generator (32-bit): output[i] = seeds[i] * 1664525 + 1013904223

@group(0) @binding(0) var<storage, read> seeds: array<u32>;
@group(0) @binding(1) var<storage, read_write> output: array<u32>;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    if i < arrayLength(&seeds) {
        output[i] = seeds[i] * 1664525u + 1013904223u;
    }
}

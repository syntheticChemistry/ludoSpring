// SPDX-License-Identifier: AGPL-3.0-or-later
// Workgroup parallel reduction sum (256-wide).

@group(0) @binding(0) var<storage, read> input: array<f32>;
@group(0) @binding(1) var<storage, read_write> output: array<f32>;

var<workgroup> wg_data: array<f32, 256>;

@compute @workgroup_size(256)
fn main(
    @builtin(local_invocation_id) lid: vec3<u32>,
    @builtin(global_invocation_id) gid: vec3<u32>,
) {
    let i = gid.x;
    if i < arrayLength(&input) {
        wg_data[lid.x] = input[i];
    } else {
        wg_data[lid.x] = 0.0;
    }
    workgroupBarrier();

    var stride: u32 = 128u;
    while stride > 0u {
        if lid.x < stride {
            wg_data[lid.x] = wg_data[lid.x] + wg_data[lid.x + stride];
        }
        workgroupBarrier();
        stride = stride >> 1u;
    }

    if lid.x == 0u {
        output[gid.x / 256u] = wg_data[0];
    }
}

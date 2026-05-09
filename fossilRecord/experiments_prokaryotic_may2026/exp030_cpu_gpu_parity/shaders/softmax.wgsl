// SPDX-License-Identifier: AGPL-3.0-or-later
// Single-workgroup softmax (numerically stable with max subtraction).

@group(0) @binding(0) var<storage, read> input: array<f32>;
@group(0) @binding(1) var<storage, read_write> output: array<f32>;

@compute @workgroup_size(1)
fn main() {
    let n = arrayLength(&input);
    var max_val: f32 = input[0];
    for (var i = 1u; i < n; i = i + 1u) {
        max_val = max(max_val, input[i]);
    }
    var sum_exp: f32 = 0.0;
    for (var i = 0u; i < n; i = i + 1u) {
        sum_exp = sum_exp + exp(input[i] - max_val);
    }
    for (var i = 0u; i < n; i = i + 1u) {
        output[i] = exp(input[i] - max_val) / sum_exp;
    }
}

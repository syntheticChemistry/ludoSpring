// SPDX-License-Identifier: AGPL-3.0-or-later
// Perlin 2D noise — Ken Perlin (2002) improved version.
// Validated against barraCuda CPU `procedural::noise::perlin_2d`.
// f32 GPU vs f64 CPU tolerance: < 1e-3.

@group(0) @binding(0) var<storage, read> perm: array<u32>;
@group(0) @binding(1) var<storage, read> coords: array<f32>;
@group(0) @binding(2) var<storage, read_write> output: array<f32>;

fn fade(t: f32) -> f32 {
    return t * t * t * (t * (t * 6.0 - 15.0) + 10.0);
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    return a + t * (b - a);
}

fn grad2(hash: u32, x: f32, y: f32) -> f32 {
    switch hash & 3u {
        case 0u: { return x + y; }
        case 1u: { return -x + y; }
        case 2u: { return x - y; }
        default: { return -x - y; }
    }
}

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x;
    let n = arrayLength(&output);
    if i >= n {
        return;
    }
    let x = coords[i * 2u];
    let y = coords[i * 2u + 1u];
    let xi = u32(i32(floor(x)) & 255);
    let yi = u32(i32(floor(y)) & 255);
    let xf = x - floor(x);
    let yf = y - floor(y);
    let u = fade(xf);
    let v = fade(yf);
    let aa = perm[perm[xi] + yi];
    let ab = perm[perm[xi] + yi + 1u];
    let ba = perm[perm[xi + 1u] + yi];
    let bb = perm[perm[xi + 1u] + yi + 1u];
    let result = lerp(
        lerp(grad2(aa, xf, yf), grad2(ba, xf - 1.0, yf), u),
        lerp(grad2(ab, xf, yf - 1.0), grad2(bb, xf - 1.0, yf - 1.0), u),
        v
    );
    output[i] = result;
}

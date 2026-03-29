// SPDX-License-Identifier: AGPL-3.0-or-later

//! Shader source strings and permutation table for CPU/GPU parity tests.

/// Perlin permutation table (standard 256-entry doubled to 512).
/// Must match barracuda `procedural::noise` for CPU/GPU parity.
pub const PERM_TABLE: [u8; 512] = {
    let base: [u8; 256] = [
        151, 160, 137, 91, 90, 15, 131, 13, 201, 95, 96, 53, 194, 233, 7, 225, 140, 36, 103, 30,
        69, 142, 8, 99, 37, 240, 21, 10, 23, 190, 6, 148, 247, 120, 234, 75, 0, 26, 197, 62, 94,
        252, 219, 203, 117, 35, 11, 32, 57, 177, 33, 88, 237, 149, 56, 87, 174, 20, 125, 136, 171,
        168, 68, 175, 74, 165, 71, 134, 139, 48, 27, 166, 77, 146, 158, 231, 83, 111, 229, 122, 60,
        211, 133, 230, 220, 105, 92, 41, 55, 46, 245, 40, 244, 102, 143, 54, 65, 25, 63, 161, 1,
        216, 80, 73, 209, 76, 132, 187, 208, 89, 18, 169, 200, 196, 135, 130, 116, 188, 159, 86,
        164, 100, 109, 198, 173, 186, 3, 64, 52, 217, 226, 250, 124, 123, 5, 202, 38, 147, 118,
        126, 255, 82, 85, 212, 207, 206, 59, 227, 47, 16, 58, 17, 182, 189, 28, 42, 223, 183, 170,
        213, 119, 248, 152, 2, 44, 154, 163, 70, 221, 153, 101, 155, 167, 43, 172, 9, 129, 22, 39,
        253, 19, 98, 108, 110, 79, 113, 224, 232, 178, 185, 112, 104, 218, 246, 97, 228, 251, 34,
        242, 193, 238, 210, 144, 12, 191, 179, 162, 241, 81, 51, 145, 235, 249, 14, 239, 107, 49,
        192, 214, 31, 181, 199, 106, 157, 184, 84, 204, 176, 115, 121, 50, 45, 127, 4, 150, 254,
        138, 236, 205, 93, 222, 114, 67, 29, 24, 72, 243, 141, 128, 195, 78, 66, 215, 61, 156, 180,
    ];
    let mut table = [0u8; 512];
    let mut i = 0;
    while i < 512 {
        table[i] = base[i & 255];
        i += 1;
    }
    table
};

pub const SIGMOID_WGSL: &str = include_str!("../shaders/sigmoid.wgsl");
pub const RELU_WGSL: &str = include_str!("../shaders/relu.wgsl");
pub const DOT_PRODUCT_WGSL: &str = include_str!("../shaders/dot_product.wgsl");
pub const REDUCE_SUM_WGSL: &str = include_str!("../shaders/reduce_sum.wgsl");
pub const SOFTMAX_WGSL: &str = include_str!("../shaders/softmax.wgsl");
pub const SCALE_WGSL: &str = include_str!("../shaders/scale.wgsl");
pub const LCG_WGSL: &str = include_str!("../shaders/lcg.wgsl");
pub const ABS_WGSL: &str = include_str!("../shaders/abs.wgsl");
pub const PERLIN_2D_WGSL: &str = include_str!("../shaders/perlin_2d.wgsl");
pub const ENGAGEMENT_BATCH_WGSL: &str = include_str!("../shaders/engagement_batch.wgsl");
pub const DDA_RAYCAST_WGSL: &str = include_str!("../shaders/dda_raycast.wgsl");

pub const FOG_OF_WAR_WGSL: &str =
    include_str!("../../../barracuda/shaders/game/fog_of_war.wgsl");
pub const TILE_LIGHTING_WGSL: &str =
    include_str!("../../../barracuda/shaders/game/tile_lighting.wgsl");
pub const PATHFIND_WAVEFRONT_WGSL: &str =
    include_str!("../../../barracuda/shaders/game/pathfind_wavefront.wgsl");

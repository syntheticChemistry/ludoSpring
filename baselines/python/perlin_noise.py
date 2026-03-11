# SPDX-License-Identifier: AGPL-3.0-or-later
"""
Perlin noise reference implementation — Python baseline for ludoSpring.

Produces deterministic Perlin 2D/3D and fBm values that the Rust implementation
must match exactly (within floating-point tolerance).

References:
  - Perlin, K. (1985). "An image synthesizer." SIGGRAPH '85.
  - Perlin, K. (2002). "Improving noise." SIGGRAPH '02.

Provenance:
  Script: baselines/python/perlin_noise.py
  Date: 2026-03-11
  Python: 3.x (stdlib only, no dependencies)
  Command: python3 baselines/python/perlin_noise.py
"""

import json
import math
import sys

PERM_BASE = [
    151, 160, 137, 91, 90, 15, 131, 13, 201, 95, 96, 53, 194, 233, 7, 225,
    140, 36, 103, 30, 69, 142, 8, 99, 37, 240, 21, 10, 23, 190, 6, 148,
    247, 120, 234, 75, 0, 26, 197, 62, 94, 252, 219, 203, 117, 35, 11, 32,
    57, 177, 33, 88, 237, 149, 56, 87, 174, 20, 125, 136, 171, 168, 68, 175,
    74, 165, 71, 134, 139, 48, 27, 166, 77, 146, 158, 231, 83, 111, 229, 122,
    60, 211, 133, 230, 220, 105, 92, 41, 55, 46, 245, 40, 244, 102, 143, 54,
    65, 25, 63, 161, 1, 216, 80, 73, 209, 76, 132, 187, 208, 89, 18, 169,
    200, 196, 135, 130, 116, 188, 159, 86, 164, 100, 109, 198, 173, 186, 3, 64,
    52, 217, 226, 250, 124, 123, 5, 202, 38, 147, 118, 126, 255, 82, 85, 212,
    207, 206, 59, 227, 47, 16, 58, 17, 182, 189, 28, 42, 223, 183, 170, 213,
    119, 248, 152, 2, 44, 154, 163, 70, 221, 153, 101, 155, 167, 43, 172, 9,
    129, 22, 39, 253, 19, 98, 108, 110, 79, 113, 224, 232, 178, 185, 112, 104,
    218, 246, 97, 228, 251, 34, 242, 193, 238, 210, 144, 12, 191, 179, 162, 241,
    81, 51, 145, 235, 249, 14, 239, 107, 49, 192, 214, 31, 181, 199, 106, 157,
    184, 84, 204, 176, 115, 121, 50, 45, 127, 4, 150, 254, 138, 236, 205, 93,
    222, 114, 67, 29, 24, 72, 243, 141, 128, 195, 78, 66, 215, 61, 156, 180,
]

PERM = [PERM_BASE[i & 255] for i in range(512)]


def fade(t: float) -> float:
    return t * t * t * (t * (t * 6.0 - 15.0) + 10.0)


def lerp(a: float, b: float, t: float) -> float:
    return a + t * (b - a)


def grad2(h: int, x: float, y: float) -> float:
    h = h & 3
    if h == 0:
        return x + y
    elif h == 1:
        return -x + y
    elif h == 2:
        return x - y
    else:
        return -x - y


def grad3(h: int, x: float, y: float, z: float) -> float:
    h = h & 15
    table = [
        x + y, -x + y, x - y, -x - y,
        x + z, -x + z, x - z, -x - z,
        y + z, -y + z, y - z, -y - z,
        y + x, -y + z, y - x, -y - z,
    ]
    return table[h]


def perlin_2d(x: float, y: float) -> float:
    xi = int(math.floor(x)) & 255
    yi = int(math.floor(y)) & 255
    xf = x - math.floor(x)
    yf = y - math.floor(y)

    u = fade(xf)
    v = fade(yf)

    aa = PERM[PERM[xi] + yi]
    ab = PERM[PERM[xi] + yi + 1]
    ba = PERM[PERM[xi + 1] + yi]
    bb = PERM[PERM[xi + 1] + yi + 1]

    return lerp(
        lerp(grad2(aa, xf, yf), grad2(ba, xf - 1.0, yf), u),
        lerp(grad2(ab, xf, yf - 1.0), grad2(bb, xf - 1.0, yf - 1.0), u),
        v,
    )


def perlin_3d(x: float, y: float, z: float) -> float:
    xi = int(math.floor(x)) & 255
    yi = int(math.floor(y)) & 255
    zi = int(math.floor(z)) & 255
    xf = x - math.floor(x)
    yf = y - math.floor(y)
    zf = z - math.floor(z)

    u = fade(xf)
    v = fade(yf)
    w = fade(zf)

    a = PERM[xi] + yi
    aa = PERM[a] + zi
    ab = PERM[a + 1] + zi
    b = PERM[xi + 1] + yi
    ba = PERM[b] + zi
    bb = PERM[b + 1] + zi

    return lerp(
        lerp(
            lerp(grad3(PERM[aa], xf, yf, zf),
                 grad3(PERM[ba], xf - 1, yf, zf), u),
            lerp(grad3(PERM[ab], xf, yf - 1, zf),
                 grad3(PERM[bb], xf - 1, yf - 1, zf), u),
            v,
        ),
        lerp(
            lerp(grad3(PERM[aa + 1], xf, yf, zf - 1),
                 grad3(PERM[ba + 1], xf - 1, yf, zf - 1), u),
            lerp(grad3(PERM[ab + 1], xf, yf - 1, zf - 1),
                 grad3(PERM[bb + 1], xf - 1, yf - 1, zf - 1), u),
            v,
        ),
        w,
    )


def fbm_2d(x: float, y: float, octaves: int, lacunarity: float,
           persistence: float) -> float:
    value = 0.0
    amplitude = 1.0
    frequency = 1.0
    max_value = 0.0

    for _ in range(octaves):
        value += perlin_2d(x * frequency, y * frequency) * amplitude
        max_value += amplitude
        amplitude *= persistence
        frequency *= lacunarity

    return value / max_value


def fbm_3d(x: float, y: float, z: float, octaves: int, lacunarity: float,
           persistence: float) -> float:
    value = 0.0
    amplitude = 1.0
    frequency = 1.0
    max_value = 0.0

    for _ in range(octaves):
        value += perlin_3d(x * frequency, y * frequency,
                           z * frequency) * amplitude
        max_value += amplitude
        amplitude *= persistence
        frequency *= lacunarity

    return value / max_value


def main():
    results = {}

    # Perlin 2D at integer lattice (should be zero)
    lattice_2d = {}
    for ix in range(10):
        for iy in range(10):
            v = perlin_2d(float(ix), float(iy))
            lattice_2d[f"{ix},{iy}"] = v
    results["perlin_2d_lattice"] = lattice_2d

    # Perlin 2D at specific test coordinates
    test_2d = {}
    coords_2d = [(0.5, 0.7), (1.23, 4.56), (100.1, 200.2), (-3.17, 2.73)]
    for x, y in coords_2d:
        v = perlin_2d(x, y)
        test_2d[f"{x},{y}"] = v
    results["perlin_2d_samples"] = test_2d

    # Perlin 3D at integer lattice
    lattice_3d = {}
    for ix in range(5):
        for iy in range(5):
            for iz in range(5):
                v = perlin_3d(float(ix), float(iy), float(iz))
                lattice_3d[f"{ix},{iy},{iz}"] = v
    results["perlin_3d_lattice"] = lattice_3d

    # fBm 2D samples
    fbm_2d_results = {}
    for octaves in [1, 4, 8]:
        v = fbm_2d(3.17, 2.73, octaves, 2.0, 0.5)
        fbm_2d_results[f"octaves={octaves}"] = v
    results["fbm_2d_samples"] = fbm_2d_results

    # fBm 3D sample
    results["fbm_3d_sample"] = fbm_3d(1.0, 2.0, 3.0, 4, 2.0, 0.5)

    json.dump(results, sys.stdout, indent=2)
    print()

    # Validation summary
    all_lattice_zero = all(abs(v) < 1e-10 for v in lattice_2d.values())
    all_3d_lattice_zero = all(abs(v) < 1e-10 for v in lattice_3d.values())

    print(f"perlin_2d lattice zeros: {'PASS' if all_lattice_zero else 'FAIL'}",
          file=sys.stderr)
    print(f"perlin_3d lattice zeros: {'PASS' if all_3d_lattice_zero else 'FAIL'}",
          file=sys.stderr)

    if not (all_lattice_zero and all_3d_lattice_zero):
        sys.exit(1)


if __name__ == "__main__":
    main()

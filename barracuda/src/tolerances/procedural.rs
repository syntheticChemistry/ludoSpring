// SPDX-License-Identifier: AGPL-3.0-or-later
//! Procedural generation tolerances — raycaster, noise, chemistry.

/// Near-zero threshold for DDA ray direction components.
///
/// Justification: IEEE 754 double precision has ~15 significant digits;
/// 1e-12 avoids division-by-zero in DDA step calculation while remaining
/// well above machine epsilon (~2.2e-16).
pub const DDA_NEAR_ZERO: f64 = 1e-12;

/// CPK element colors \[R, G, B, A\] for common biochemistry elements.
///
/// Source: Corey, R.B., Pauling, L. (1953). CPK (Corey-Pauling-Koltun)
/// coloring convention. Adapted to f32 RGBA with full opacity.
pub const CPK_HYDROGEN: [f32; 4] = [1.0, 1.0, 1.0, 1.0];
/// CPK carbon — dark gray.
pub const CPK_CARBON: [f32; 4] = [0.2, 0.2, 0.2, 1.0];
/// CPK nitrogen — blue.
pub const CPK_NITROGEN: [f32; 4] = [0.0, 0.0, 0.8, 1.0];
/// CPK oxygen — red.
pub const CPK_OXYGEN: [f32; 4] = [0.8, 0.0, 0.0, 1.0];
/// CPK phosphorus — orange.
pub const CPK_PHOSPHORUS: [f32; 4] = [1.0, 0.5, 0.0, 1.0];
/// CPK sulfur — yellow.
pub const CPK_SULFUR: [f32; 4] = [1.0, 1.0, 0.0, 1.0];
/// CPK iron — brown/orange.
pub const CPK_IRON: [f32; 4] = [0.6, 0.3, 0.0, 1.0];
/// Jmol sodium — purple.
pub const CPK_SODIUM: [f32; 4] = [0.7, 0.0, 0.7, 1.0];
/// Jmol chlorine — green.
pub const CPK_CHLORINE: [f32; 4] = [0.0, 0.8, 0.0, 1.0];
/// Jmol calcium — gray.
pub const CPK_CALCIUM: [f32; 4] = [0.5, 0.5, 0.5, 1.0];

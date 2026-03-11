// SPDX-License-Identifier: AGPL-3.0-or-later
//! Procedural content generation — algorithmic world building.
//!
//! PCG is the bridge between ludoSpring and the science springs: procedural
//! generation of molecular structures *is* Minecraft. A Perlin noise field
//! determines where atoms cluster. An L-system grows a protein backbone.
//! Wave function collapse ensures chemical validity of the generated structure.

pub mod bsp;
pub mod lsystem;
pub mod noise;
pub mod wfc;

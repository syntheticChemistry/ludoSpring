// SPDX-License-Identifier: AGPL-3.0-or-later
//! Game mechanics — formal models of interactive systems.
//!
//! This module treats game genres as *interaction architectures*, not aesthetic
//! categories. An FPS and a chemistry explorer share a first-person spatial
//! navigation model; an RTS and a systems-biology dashboard share a
//! top-down selection-and-command model. By factoring out the mechanical
//! primitives, we can compose game-like interactions for any domain.

pub mod genre;
pub mod raycaster;
pub mod ruleset;
pub mod state;
pub mod voxel;

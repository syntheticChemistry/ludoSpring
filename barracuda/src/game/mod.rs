// SPDX-License-Identifier: AGPL-3.0-or-later
//! Game mechanics — formal models of interactive systems.
//!
//! This module treats game genres as *interaction architectures*, not aesthetic
//! categories. An FPS and a chemistry explorer share a first-person spatial
//! navigation model; an RTS and a systems-biology dashboard share a
//! top-down selection-and-command model. By factoring out the mechanical
//! primitives, we can compose game-like interactions for any domain.
//!
//! # Module map
//!
//! - `engine` — 2D engine primitives (world, entities, actions, session, audio)
//! - `rpgpt` — RPGPT game substrate (planes, NPCs, voices, trust, memory, scenes)
//! - `ruleset` — system-agnostic ruleset trait (D20, Fudge, D6Pool, D100, RollUnder)
//! - `state` — tick models, replay buffer, session phase
//! - `genre` — interaction architecture taxonomy
//! - `raycaster` — first-person spatial reference implementation
//! - `voxel` — block-based spatial representation

pub mod engine;
pub mod genre;
pub mod raycaster;
pub mod rpgpt;
pub mod ruleset;
pub mod state;
pub mod voxel;

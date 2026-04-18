// SPDX-License-Identifier: AGPL-3.0-or-later
//! 2D game engine primitives — the shared foundation for all RPGPT planes.
//!
//! # Philosophy
//!
//! Game physics and mechanics are a function of *gameplay*, not engine limits.
//! The engine provides spatial primitives (tile world, entities, fog of war),
//! action resolution (command pipeline), and rendering abstraction (any
//! petalTongue modality: GUI, TUI, audio). The science modules (flow, DDA,
//! engagement, Fitts, Hick) run continuously to evaluate and adapt.
//!
//! All 7 RPGPT planes share this foundation:
//! - Exploration → tile map with movement and discovery
//! - Dialogue → entities in conversation, choices as actions
//! - Tactical → grid with positions, initiative, action economy
//! - Investigation → evidence locations, clue connections
//! - Political → faction positions, alliance movements
//! - Crafting → workbench zones, material entities
//! - Card/Stack → zones with card entities
//!
//! # GPU Compute
//!
//! Heavy per-tile operations (fog of war, lighting, pathfinding, terrain gen)
//! are dispatched to GPU via barraCuda WGSL shaders. The [`gpu`] module
//! defines the shader catalog and dispatch types. Shaders live in
//! `barracuda/shaders/game/` and are dispatched through:
//!
//! - **barraCuda in-process** — `ComputeDispatch` with wgpu
//! - **toadStool IPC** — `compute.submit` / `science.gpu.dispatch` JSON-RPC
//! - **coralReef compile** — `shader.compile.wgsl` for native GPU binaries
//!
//! The engine discovers GPU availability at runtime and falls back to CPU
//! implementations when no GPU is present.
//!
//! # Rendering
//!
//! The engine never renders directly. It produces structured scene payloads
//! (`rpgpt::scene` types) that petalTongue renders in whatever modality
//! the human is using. An audio game for driving is the same engine state
//! rendered as narration cues instead of tiles.
//!
//! # Accessibility
//!
//! petalTongue's philosophy: all humans of all ability are first class.
//! The engine provides enough semantic information (tile meanings, entity
//! descriptions, spatial relationships) for any modality to present the
//! game meaningfully — visual, auditory, tactile, or text.

pub mod action;
pub mod audio;
pub mod entity;
pub mod gpu;
#[cfg(feature = "gpu")]
pub mod gpu_context;
pub mod session;
#[cfg(feature = "gpu")]
pub mod tensor_ops;
pub mod world;

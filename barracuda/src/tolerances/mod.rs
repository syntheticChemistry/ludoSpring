// SPDX-License-Identifier: AGPL-3.0-or-later
//! Domain-specific tolerances — no magic numbers.
//!
//! Every numerical threshold in ludoSpring lives here with a citation.
//! Organized into domain submodules following the wetSpring pattern:
//!
//! - [`game`] — NPC proximity, frame rate, entity limits
//! - [`interaction`] — Fitts, Hick, Steering, Flow, DDA
//! - [`ipc`] — RPC timeouts, probe intervals
//! - [`metrics`] — Tufte, engagement, UI analysis
//! - [`procedural`] — raycaster, noise, chemistry
//! - [`validation`] — analytical, raycaster, noise, UI tolerances

pub mod game;
pub mod gpu;
pub mod interaction;
pub mod ipc;
pub mod metrics;
pub mod procedural;
pub mod validation;

pub use game::*;
pub use gpu::*;
pub use interaction::*;
pub use ipc::*;
pub use metrics::*;
pub use procedural::*;
pub use validation::*;

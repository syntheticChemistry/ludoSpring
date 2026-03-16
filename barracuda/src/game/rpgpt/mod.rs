// SPDX-License-Identifier: AGPL-3.0-or-later
//! RPGPT — Role-Playing Game Primal Technology.
//!
//! A DAG-driven, AI-narrated, multi-modal game substrate where the story DAG
//! is continuous and universal, but the rules governing how the DAG grows
//! change as you move between planes of play.
//!
//! # Modules
//!
//! - [`plane`] — The seven planes (game modes as swappable rulesets)
//! - [`npc`] — NPC personality certificates with Maslow-hierarchy motivations
//! - [`knowledge`] — Knowledge bounds (knows / suspects / lies about / does not know)
//! - [`voice`] — Internal voices (Disco Elysium model — skills as perspectives)
//! - [`trust`] — Trust model (earned through tracked interactions)
//! - [`memory`] — NPC memory as DAG subgraph (not context window)
//! - [`dialogue`] — D6 dice pool skill checks and flow tracking
//! - [`transition`] — World state preservation across plane transitions
//! - [`scene`] — petalTongue scene binding types for game UI rendering

pub mod dialogue;
pub mod knowledge;
pub mod memory;
pub mod npc;
pub mod plane;
pub mod scene;
pub mod transition;
pub mod trust;
pub mod voice;

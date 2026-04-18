// SPDX-License-Identifier: AGPL-3.0-or-later
//! ludoSpring — The Science of Play, Interaction, and Game Design (via `barraCuda`)
//!
//! Ludology (from Latin *ludus*: play, game) is the study of games and play as
//! systems. ludoSpring treats game design with the same rigor that wetSpring
//! treats bioinformatics and hotSpring treats nuclear physics: validated models,
//! reproducible experiments, and GPU-accelerated computation where it matters.
//!
//! # Why a spring for games?
//!
//! Games are the most demanding real-time interactive systems humans build.
//! They solve problems every primal needs: input handling, spatial navigation,
//! physics simulation, procedural content generation, accessibility, and the
//! deep question of what makes interaction *engaging*. By studying games as
//! science, every primal in the ecosystem benefits.
//!
//! # Domains
//!
//! ## [`game`] — Game mechanics and genre analysis
//! Formal models of game systems: state machines, entity-component patterns,
//! raycasting, voxel worlds, turn-based and real-time loops. Reference
//! implementations for comparative study.
//!
//! ## [`interaction`] — Input, flow, and engagement science
//! Fitts's law, Hick's law, flow state models (Csikszentmihalyi), difficulty
//! curves, input latency analysis, accessibility scoring. Connects to
//! petalTongue's `InteractionEngine` and SAME DAVE proprioception.
//!
//! ## [`procedural`] — Procedural content generation
//! Noise functions (Perlin, simplex, Worley), wave function collapse,
//! L-systems, Markov chains for level generation, molecular structure
//! generation (Minecraft-as-chemistry), terrain synthesis.
//!
//! ## [`metrics`] — Quantitative play analysis
//! Fun metrics (engagement curves, retention models, difficulty-skill balance),
//! Tufte constraint evaluation on game UIs, information density in HUDs,
//! data-ink ratio for game interfaces, genre-specific UI heuristics.
//!
//! # Reference Systems
//!
//! ludoSpring studies and validates against these systems:
//!
//! | System | What we learn | Experiment |
//! |--------|--------------|------------|
//! | Doom (1993) | Raycasting, BSP, fixed-point math, minimal UI | Exp001 |
//! | Minecraft | Procedural voxel worlds, emergent gameplay | Exp002 |
//! | Folding\@Home | Science-as-game, adversarial protein folding | Exp004 |
//! | `NetHack` | Procedural dungeon generation, roguelike design | — |
//! | Dwarf Fortress | Complex simulation → emergent narrative | — |
//! | KSP | Physics education through play | — |
//!
//! # Architecture
//!
//! ```text
//!    ┌─────────────────────────────────────────┐
//!    │  game (mechanics, state, reference impl) │
//!    └──────────┬──────────────────────────────┘
//!               │ GameState, Tick
//!    ┌──────────▼──────────────────────────────┐
//!    │  interaction (input, flow, accessibility)│
//!    └──────────┬──────────────────────────────┘
//!               │ EngagementScore, FlowState
//!    ┌──────────▼──────────────────────────────┐
//!    │  procedural (generation, noise, WFC)     │
//!    └──────────┬──────────────────────────────┘
//!               │ WorldChunk, Structure
//!    ┌──────────▼──────────────────────────────┐
//!    │  metrics (Tufte on games, fun science)   │
//!    └─────────────────────────────────────────┘
//! ```
//!
//! # GPU acceleration (feature = "gpu")
//!
//! - Noise field generation (Perlin/simplex on GPU via barraCuda)
//! - Wave function collapse (parallel constraint propagation)
//! - Physics tick (N-body, collision broadphase)
//! - Engagement metric batch evaluation

/// Game mechanics, state machines, and genre-specific reference implementations.
pub mod game;

/// Input science, flow models, accessibility scoring, and engagement analysis.
pub mod interaction;

/// Quantitative play analysis: fun metrics, Tufte-on-games, UI heuristics.
pub mod metrics;

/// Procedural content generation: noise, WFC, L-systems, molecular worlds.
pub mod procedural;

/// Niche self-knowledge: identity, capabilities, semantic mappings, cost metadata.
pub mod niche;

/// Capability domain registry — structured method catalog with introspection.
pub mod capability_domains;

/// Domain-specific tolerances for validation (no magic numbers).
pub mod tolerances;

/// Composition golden targets (`baselines/rust/composition_targets.json`).
pub mod composition_targets;

/// Validation harness and test utilities.
pub mod validation;

/// Portable game telemetry — engine-agnostic event protocol and analysis.
pub mod telemetry;

/// Visualization data channels for any visualization-capable consumer.
pub mod visualization;

/// IPC server: JSON-RPC 2.0 over Unix socket.
#[cfg(feature = "ipc")]
pub mod ipc;

/// biomeOS niche deployment: domain registration, semantic mappings, Neural API.
#[cfg(feature = "ipc")]
pub mod biomeos;

/// Re-exported barraCuda CPU primitives.
///
/// These are the shared math operations from the barraCuda primal.
/// Using them instead of hand-rolling ensures consistent behavior
/// across the Python → Rust CPU → GPU evolution path.
///
/// # Available primitives
///
/// | Category | Functions |
/// |----------|-----------|
/// | Activations | `sigmoid`, `relu`, `gelu`, `swish`, `leaky_relu`, `softplus`, `mish` |
/// | Batch activations | `sigmoid_batch`, `relu_batch`, `gelu_batch`, `swish_batch` |
/// | Statistics | `mean`, `dot`, `l2_norm`, `mae`, `rmse`, `percentile` |
/// | Correlation | `variance`, `std_dev`, `pearson_correlation`, `covariance` |
/// | RNG | `lcg_step`, `state_to_f64`, `uniform_f64_sequence` |
pub mod barcuda_math {
    // ── Activations ──────────────────────────────────────────────
    pub use barracuda::activations::{
        gelu, gelu_batch, leaky_relu, mish, relu, relu_batch, sigmoid, sigmoid_batch, softplus,
        swish, swish_batch,
    };

    // ── RNG ──────────────────────────────────────────────────────
    pub use barracuda::rng::{lcg_step, state_to_f64, uniform_f64_sequence};

    // ── Core statistics (metrics) ────────────────────────────────
    pub use barracuda::stats::{dot, l2_norm, mae, mean, percentile, rmse};

    // ── Correlation / variance ───────────────────────────────────
    pub use barracuda::stats::correlation::{covariance, pearson_correlation, std_dev, variance};
}

/// Primal identity — delegates to [`niche::NICHE_NAME`].
pub const PRIMAL_NAME: &str = niche::NICHE_NAME;

// SPDX-License-Identifier: AGPL-3.0-or-later
//! Genre taxonomy — games as interaction architectures.
//!
//! Genres aren't aesthetic categories; they're families of interaction patterns.
//! This taxonomy maps game genres to the mechanical primitives they share with
//! scientific visualization domains.

/// Interaction architecture families.
///
/// Each variant describes the *mechanical* pattern, not the aesthetic.
/// A chemistry explorer uses the same spatial navigation as an FPS;
/// a systems biology dashboard uses the same selection model as an RTS.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InteractionArchitecture {
    /// First-person spatial: position + look direction + move/strafe.
    /// Used by: FPS, molecular explorers, VR labs, cave surveys.
    FirstPersonSpatial,

    /// Top-down command: select units, issue orders, manage resources.
    /// Used by: RTS, systems biology, network monitoring, farm management.
    TopDownCommand,

    /// Turn-based strategic: discrete decisions with full information.
    /// Used by: chess, protein folding puzzles, circuit design.
    TurnBased,

    /// Side-scrolling / 2D platformer: horizontal traversal with obstacles.
    /// Used by: signal processing visualization, chromatography traces.
    SideScroll,

    /// Sandbox / creative: open-ended building and experimentation.
    /// Used by: Minecraft, molecule builders, circuit simulators.
    Sandbox,

    /// Roguelike / procedural: generated content, permadeath, discovery.
    /// Used by: parameter space exploration, Monte Carlo sampling.
    Roguelike,

    /// Puzzle / constraint: find valid configurations under rules.
    /// Used by: Folding@Home, SAT solvers, crystal packing.
    Puzzle,
}

/// Maps a game genre to its analogous scientific visualization domain.
#[must_use]
pub fn genre_domain_analogy(arch: InteractionArchitecture) -> &'static str {
    match arch {
        InteractionArchitecture::FirstPersonSpatial => "molecular/particle exploration",
        InteractionArchitecture::TopDownCommand => "systems biology / network orchestration",
        InteractionArchitecture::TurnBased => "protein folding / discrete optimization",
        InteractionArchitecture::SideScroll => "signal processing / chromatography",
        InteractionArchitecture::Sandbox => "molecule building / circuit design",
        InteractionArchitecture::Roguelike => "parameter space / Monte Carlo exploration",
        InteractionArchitecture::Puzzle => "constraint satisfaction / crystal packing",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_architecture_has_analogy() {
        let archs = [
            InteractionArchitecture::FirstPersonSpatial,
            InteractionArchitecture::TopDownCommand,
            InteractionArchitecture::TurnBased,
            InteractionArchitecture::SideScroll,
            InteractionArchitecture::Sandbox,
            InteractionArchitecture::Roguelike,
            InteractionArchitecture::Puzzle,
        ];
        for arch in archs {
            assert!(!genre_domain_analogy(arch).is_empty());
        }
    }
}

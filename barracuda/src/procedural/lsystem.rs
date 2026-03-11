// SPDX-License-Identifier: AGPL-3.0-or-later
//! L-systems — Lindenmayer systems for biological growth patterns.
//!
//! String-rewriting systems that model biological growth. Each symbol in
//! the string maps to a geometric operation (draw, turn, push/pop state),
//! producing fractal-like structures used for protein backbones, plant
//! morphology, and procedural terrain features.
//!
//! # References
//!
//! - Lindenmayer, A. (1968). "Mathematical models for cellular interactions
//!   in development." J. Theoretical Biology, 18(3).
//! - Prusinkiewicz, P. & Lindenmayer, A. (1990). "The Algorithmic Beauty
//!   of Plants." Springer-Verlag.

use std::collections::HashMap;

/// A production rule: symbol → replacement string.
#[derive(Debug, Clone)]
pub struct ProductionRule {
    /// The symbol to replace.
    pub predecessor: char,
    /// The replacement string.
    pub successor: String,
}

/// An L-system definition: axiom + production rules.
#[derive(Debug, Clone)]
pub struct LSystem {
    /// Starting string.
    pub axiom: String,
    /// Production rules (one per predecessor symbol).
    rules: HashMap<char, String>,
}

impl LSystem {
    /// Create a new L-system from an axiom and rules.
    #[must_use]
    pub fn new(axiom: &str, rules: &[ProductionRule]) -> Self {
        let rule_map = rules
            .iter()
            .map(|r| (r.predecessor, r.successor.clone()))
            .collect();
        Self {
            axiom: axiom.to_owned(),
            rules: rule_map,
        }
    }

    /// Apply one generation of rewriting to a string.
    #[must_use]
    pub fn step(&self, input: &str) -> String {
        input
            .chars()
            .map(|c| {
                self.rules
                    .get(&c)
                    .map_or_else(|| c.to_string(), Clone::clone)
            })
            .collect()
    }

    /// Apply `n` generations starting from the axiom.
    #[must_use]
    pub fn generate(&self, generations: u32) -> String {
        let mut current = self.axiom.clone();
        for _ in 0..generations {
            current = self.step(&current);
        }
        current
    }

    /// Count total symbols after `n` generations.
    #[must_use]
    pub fn symbol_count(&self, generations: u32) -> usize {
        self.generate(generations).len()
    }
}

/// Common L-system presets for biological modeling.
pub mod presets {
    use super::{LSystem, ProductionRule};

    /// Algae growth: Lindenmayer's original (1968).
    ///
    /// Axiom: "A", Rules: A→AB, B→A.
    /// Sequence lengths follow the Fibonacci sequence.
    #[must_use]
    pub fn algae() -> LSystem {
        LSystem::new(
            "A",
            &[
                ProductionRule {
                    predecessor: 'A',
                    successor: "AB".into(),
                },
                ProductionRule {
                    predecessor: 'B',
                    successor: "A".into(),
                },
            ],
        )
    }

    /// Koch curve: self-similar fractal (Koch 1904).
    ///
    /// F → F+F-F-F+F where F=draw, +=left 90°, -=right 90°.
    #[must_use]
    pub fn koch_curve() -> LSystem {
        LSystem::new(
            "F",
            &[ProductionRule {
                predecessor: 'F',
                successor: "F+F-F-F+F".into(),
            }],
        )
    }

    /// Protein backbone model: simplified alpha helix + beta sheet.
    ///
    /// H=helix segment, S=sheet segment, L=linker, T=turn.
    /// H→HHL, S→SLT, L→LS.
    #[must_use]
    pub fn protein_backbone() -> LSystem {
        LSystem::new(
            "HLSH",
            &[
                ProductionRule {
                    predecessor: 'H',
                    successor: "HHL".into(),
                },
                ProductionRule {
                    predecessor: 'S',
                    successor: "SLT".into(),
                },
                ProductionRule {
                    predecessor: 'L',
                    successor: "LS".into(),
                },
            ],
        )
    }

    /// Dragon curve: space-filling fractal.
    ///
    /// X → X+YF+, Y → -FX-Y.
    #[must_use]
    pub fn dragon_curve() -> LSystem {
        LSystem::new(
            "FX",
            &[
                ProductionRule {
                    predecessor: 'X',
                    successor: "X+YF+".into(),
                },
                ProductionRule {
                    predecessor: 'Y',
                    successor: "-FX-Y".into(),
                },
            ],
        )
    }
}

/// Interpret an L-system string as 2D turtle graphics points.
///
/// Commands: F=forward, +=turn left, -=turn right, \[=push, \]=pop.
#[must_use]
pub fn turtle_interpret(lstring: &str, step_length: f64, angle_degrees: f64) -> Vec<(f64, f64)> {
    let angle_rad = angle_degrees.to_radians();
    let mut x = 0.0;
    let mut y = 0.0;
    let mut heading = 0.0_f64;
    let mut stack: Vec<(f64, f64, f64)> = Vec::new();
    let mut points = vec![(x, y)];

    for ch in lstring.chars() {
        match ch {
            'F' | 'H' | 'S' | 'L' => {
                x += heading.cos() * step_length;
                y += heading.sin() * step_length;
                points.push((x, y));
            }
            '+' => heading += angle_rad,
            '-' => heading -= angle_rad,
            'T' => heading += angle_rad * 0.5,
            '[' => stack.push((x, y, heading)),
            ']' => {
                if let Some((sx, sy, sh)) = stack.pop() {
                    x = sx;
                    y = sy;
                    heading = sh;
                }
            }
            _ => {}
        }
    }

    points
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn algae_fibonacci_lengths() {
        let sys = presets::algae();
        let lengths: Vec<usize> = (0..8).map(|g| sys.symbol_count(g)).collect();
        assert_eq!(lengths, [1, 2, 3, 5, 8, 13, 21, 34]);
    }

    #[test]
    fn koch_grows_exponentially() {
        let sys = presets::koch_curve();
        let g0 = sys.symbol_count(0);
        let g1 = sys.symbol_count(1);
        let g2 = sys.symbol_count(2);
        assert!(g1 > g0);
        assert!(g2 > g1);
    }

    #[test]
    fn protein_backbone_contains_all_elements() {
        let sys = presets::protein_backbone();
        let result = sys.generate(2);
        assert!(result.contains('H'));
        assert!(result.contains('S'));
        assert!(result.contains('L'));
        assert!(result.contains('T'));
    }

    #[test]
    fn step_preserves_non_rule_chars() {
        let sys = LSystem::new(
            "A+B",
            &[ProductionRule {
                predecessor: 'A',
                successor: "AA".into(),
            }],
        );
        let result = sys.step("A+B");
        assert_eq!(result, "AA+B");
    }

    #[test]
    fn turtle_forward_moves_right() {
        let points = turtle_interpret("F", 1.0, 90.0);
        assert_eq!(points.len(), 2);
        assert!((points[1].0 - 1.0).abs() < 1e-10);
        assert!(points[1].1.abs() < 1e-10);
    }

    #[test]
    fn turtle_turn_changes_direction() {
        let points = turtle_interpret("F+F", 1.0, 90.0);
        assert_eq!(points.len(), 3);
        assert!((points[2].1 - 1.0).abs() < 1e-10);
    }

    #[test]
    fn deterministic_generation() {
        let sys = presets::algae();
        let a = sys.generate(5);
        let b = sys.generate(5);
        assert_eq!(a, b);
    }
}

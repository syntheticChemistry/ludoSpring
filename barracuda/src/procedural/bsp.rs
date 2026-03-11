// SPDX-License-Identifier: AGPL-3.0-or-later
//! Binary Space Partitioning — Doom-style level generation.
//!
//! Recursively subdivides a rectangular space into rooms by alternating
//! horizontal and vertical splits. The resulting tree is a spatial index
//! for collision, visibility, and rendering — the same structure Carmack
//! used in Doom (1993) for front-to-back rendering without z-buffer.
//!
//! # References
//!
//! - Fuchs, H., Kedem, Z.M., & Naylor, B.F. (1980). "On Visible Surface
//!   Generation by A Priori Tree Structures." SIGGRAPH '80.
//! - Carmack, J. (1993). Doom engine BSP builder.
//! - Naylor, B.F. (1993). "Constructing Good Partitioning Trees." GI '93.

/// A 2D axis-aligned rectangle.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    /// Left edge x coordinate.
    pub x: f64,
    /// Top edge y coordinate.
    pub y: f64,
    /// Width.
    pub w: f64,
    /// Height.
    pub h: f64,
}

impl Rect {
    /// Create a new rectangle.
    #[must_use]
    pub const fn new(x: f64, y: f64, w: f64, h: f64) -> Self {
        Self { x, y, w, h }
    }

    /// Area of the rectangle.
    #[must_use]
    pub fn area(&self) -> f64 {
        self.w * self.h
    }

    /// Center point.
    #[must_use]
    pub fn center(&self) -> (f64, f64) {
        (self.w.mul_add(0.5, self.x), self.h.mul_add(0.5, self.y))
    }
}

/// A node in the BSP tree.
#[derive(Debug, Clone)]
pub enum BspNode {
    /// Leaf: an undivided room.
    Leaf {
        /// The room bounds.
        bounds: Rect,
    },
    /// Interior: split into two children.
    Split {
        /// The entire region this node covers.
        bounds: Rect,
        /// Whether this is a horizontal split (true) or vertical (false).
        horizontal: bool,
        /// Split position (absolute coordinate).
        split_pos: f64,
        /// Left/top child.
        left: Box<Self>,
        /// Right/bottom child.
        right: Box<Self>,
    },
}

impl BspNode {
    /// Get the bounds of this node.
    #[must_use]
    pub const fn bounds(&self) -> &Rect {
        match self {
            Self::Leaf { bounds } | Self::Split { bounds, .. } => bounds,
        }
    }

    /// Count total leaves (rooms) in this tree.
    #[must_use]
    pub fn leaf_count(&self) -> usize {
        match self {
            Self::Leaf { .. } => 1,
            Self::Split { left, right, .. } => left.leaf_count() + right.leaf_count(),
        }
    }

    /// Count total nodes (leaves + splits).
    #[must_use]
    pub fn node_count(&self) -> usize {
        match self {
            Self::Leaf { .. } => 1,
            Self::Split { left, right, .. } => 1 + left.node_count() + right.node_count(),
        }
    }

    /// Depth of the tree.
    #[must_use]
    pub fn depth(&self) -> usize {
        match self {
            Self::Leaf { .. } => 0,
            Self::Split { left, right, .. } => 1 + left.depth().max(right.depth()),
        }
    }

    /// Collect all leaf rectangles.
    #[must_use]
    pub fn leaves(&self) -> Vec<Rect> {
        let mut out = Vec::new();
        self.collect_leaves(&mut out);
        out
    }

    fn collect_leaves(&self, out: &mut Vec<Rect>) {
        match self {
            Self::Leaf { bounds } => out.push(*bounds),
            Self::Split { left, right, .. } => {
                left.collect_leaves(out);
                right.collect_leaves(out);
            }
        }
    }

    /// Find which leaf contains a point (if any).
    #[must_use]
    pub fn query_point(&self, px: f64, py: f64) -> Option<&Rect> {
        match self {
            Self::Leaf { bounds } => {
                if px >= bounds.x
                    && px < bounds.x + bounds.w
                    && py >= bounds.y
                    && py < bounds.y + bounds.h
                {
                    Some(bounds)
                } else {
                    None
                }
            }
            Self::Split {
                horizontal,
                split_pos,
                left,
                right,
                ..
            } => {
                let coord = if *horizontal { py } else { px };
                if coord < *split_pos {
                    left.query_point(px, py)
                } else {
                    right.query_point(px, py)
                }
            }
        }
    }
}

/// Generate a BSP tree by recursively splitting a rectangle.
///
/// Uses a simple deterministic LCG for split position jitter,
/// matching the `barraCuda` LCG primitive for reproducibility.
///
/// - `min_size`: minimum room dimension (stop splitting when smaller)
/// - `seed`: LCG seed for deterministic generation
#[must_use]
pub fn generate_bsp(bounds: Rect, min_size: f64, seed: u64) -> BspNode {
    split_recursive(bounds, min_size, seed, 0)
}

fn split_recursive(bounds: Rect, min_size: f64, seed: u64, depth: usize) -> BspNode {
    let horizontal = depth.is_multiple_of(2);
    let span = if horizontal { bounds.h } else { bounds.w };

    if span < min_size * 2.0 {
        return BspNode::Leaf { bounds };
    }

    let (next_seed, ratio) = lcg_ratio(seed);
    let split_ratio = ratio.mul_add(0.4, 0.3);
    let split_pos = if horizontal {
        bounds.y + span * split_ratio
    } else {
        bounds.x + span * split_ratio
    };

    let (left_bounds, right_bounds) = if horizontal {
        let top_h = split_pos - bounds.y;
        let bot_h = (bounds.y + bounds.h) - split_pos;
        (
            Rect::new(bounds.x, bounds.y, bounds.w, top_h),
            Rect::new(bounds.x, split_pos, bounds.w, bot_h),
        )
    } else {
        let left_w = split_pos - bounds.x;
        let right_w = (bounds.x + bounds.w) - split_pos;
        (
            Rect::new(bounds.x, bounds.y, left_w, bounds.h),
            Rect::new(split_pos, bounds.y, right_w, bounds.h),
        )
    };

    let left = split_recursive(left_bounds, min_size, next_seed, depth + 1);
    let right = split_recursive(right_bounds, min_size, next_seed.wrapping_add(1), depth + 1);

    BspNode::Split {
        bounds,
        horizontal,
        split_pos,
        left: Box::new(left),
        right: Box::new(right),
    }
}

/// Simple LCG matching `barraCuda::rng::lcg_step` for reproducibility.
fn lcg_ratio(seed: u64) -> (u64, f64) {
    let next = barracuda::rng::lcg_step(seed);
    let ratio = barracuda::rng::state_to_f64(next);
    (next, ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_leaf_when_too_small() {
        let tree = generate_bsp(Rect::new(0.0, 0.0, 5.0, 5.0), 10.0, 42);
        assert_eq!(tree.leaf_count(), 1);
        assert_eq!(tree.depth(), 0);
    }

    #[test]
    fn splits_large_space() {
        let tree = generate_bsp(Rect::new(0.0, 0.0, 100.0, 100.0), 10.0, 42);
        assert!(tree.leaf_count() > 1);
        assert!(tree.depth() > 0);
    }

    #[test]
    fn leaves_cover_full_area() {
        let bounds = Rect::new(0.0, 0.0, 100.0, 100.0);
        let tree = generate_bsp(bounds, 15.0, 42);
        let total_leaf_area: f64 = tree.leaves().iter().map(Rect::area).sum();
        assert!(
            (total_leaf_area - bounds.area()).abs() < 1e-6,
            "leaf areas ({total_leaf_area}) should equal total ({}).",
            bounds.area()
        );
    }

    #[test]
    fn deterministic_generation() {
        let a = generate_bsp(Rect::new(0.0, 0.0, 100.0, 100.0), 10.0, 42);
        let b = generate_bsp(Rect::new(0.0, 0.0, 100.0, 100.0), 10.0, 42);
        assert_eq!(a.leaf_count(), b.leaf_count());
        assert_eq!(a.depth(), b.depth());
    }

    #[test]
    fn point_query_finds_leaf() {
        let tree = generate_bsp(Rect::new(0.0, 0.0, 100.0, 100.0), 15.0, 42);
        let result = tree.query_point(50.0, 50.0);
        assert!(result.is_some());
    }

    #[test]
    fn point_query_outside_returns_none() {
        let tree = generate_bsp(Rect::new(0.0, 0.0, 100.0, 100.0), 15.0, 42);
        assert!(tree.query_point(200.0, 200.0).is_none());
    }
}

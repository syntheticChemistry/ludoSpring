// SPDX-License-Identifier: AGPL-3.0-or-later
//! Wave Function Collapse — constraint-based procedural generation.
//!
//! WFC generates valid configurations by propagating constraints from an
//! initial set of possibilities. In game terms: generate a dungeon where
//! every room connects properly. In chemistry terms: generate a molecular
//! structure where every bond satisfies valence rules.
//!
//! # References
//! - Gumin, M. (2016). "WaveFunctionCollapse" — procedural bitmap generation
//! - Karth, I. & Smith, A.M. (2017). "WaveFunctionCollapse is Constraint
//!   Solving in the Wild." FDG '17.

use std::collections::BTreeSet;

/// A tile in the WFC grid, identified by index into the tile set.
pub type TileId = u16;

/// Adjacency rules: which tiles can be neighbors in each direction.
#[derive(Debug, Clone)]
pub struct AdjacencyRules {
    /// For each tile, the set of tiles allowed to its right (+X).
    pub right: Vec<BTreeSet<TileId>>,
    /// For each tile, the set of tiles allowed above it (+Y).
    pub up: Vec<BTreeSet<TileId>>,
}

impl AdjacencyRules {
    /// Create rules for `n_tiles` with no constraints (everything allowed).
    #[must_use]
    pub fn unconstrained(n_tiles: usize) -> Self {
        let all: BTreeSet<TileId> = (0..n_tiles as TileId).collect();
        Self {
            right: vec![all.clone(); n_tiles],
            up: vec![all; n_tiles],
        }
    }
}

/// A single cell in the WFC grid (superposition of possible tiles).
#[derive(Debug, Clone)]
pub struct WfcCell {
    /// Remaining possible tile IDs.
    pub options: BTreeSet<TileId>,
}

impl WfcCell {
    /// Entropy (number of remaining options). 0 = contradiction, 1 = collapsed.
    #[must_use]
    pub fn entropy(&self) -> usize {
        self.options.len()
    }

    /// Whether this cell has collapsed to a single tile.
    #[must_use]
    pub fn is_collapsed(&self) -> bool {
        self.options.len() == 1
    }

    /// Whether this cell is in a contradictory state (no options).
    #[must_use]
    pub fn is_contradiction(&self) -> bool {
        self.options.is_empty()
    }

    /// Get the collapsed tile ID, if collapsed.
    #[must_use]
    pub fn collapsed_tile(&self) -> Option<TileId> {
        if self.options.len() == 1 {
            self.options.iter().next().copied()
        } else {
            None
        }
    }
}

/// 2D WFC grid.
#[derive(Debug, Clone)]
pub struct WfcGrid {
    /// Grid cells in row-major order.
    pub cells: Vec<WfcCell>,
    /// Grid width.
    pub width: usize,
    /// Grid height.
    pub height: usize,
}

impl WfcGrid {
    /// Create a grid where every cell starts with all tiles possible.
    #[must_use]
    pub fn new(width: usize, height: usize, n_tiles: usize) -> Self {
        let all: BTreeSet<TileId> = (0..n_tiles as TileId).collect();
        let cells = vec![WfcCell { options: all }; width * height];
        Self {
            cells,
            width,
            height,
        }
    }

    /// Get cell at (x, y).
    #[must_use]
    pub fn get(&self, x: usize, y: usize) -> Option<&WfcCell> {
        if x < self.width && y < self.height {
            Some(&self.cells[y * self.width + x])
        } else {
            None
        }
    }

    /// Find the uncollapsed cell with minimum entropy.
    /// Returns `None` if all cells are collapsed (or grid is empty).
    #[must_use]
    pub fn min_entropy_cell(&self) -> Option<(usize, usize)> {
        let mut best: Option<(usize, usize, usize)> = None;
        for y in 0..self.height {
            for x in 0..self.width {
                let cell = &self.cells[y * self.width + x];
                let entropy = cell.entropy();
                if entropy > 1 {
                    if best.is_none() || entropy < best.as_ref().map_or(usize::MAX, |b| b.2) {
                        best = Some((x, y, entropy));
                    }
                }
            }
        }
        best.map(|(x, y, _)| (x, y))
    }

    /// Collapse a cell to a specific tile.
    pub fn collapse(&mut self, x: usize, y: usize, tile: TileId) {
        if x < self.width && y < self.height {
            let idx = y * self.width + x;
            self.cells[idx].options = BTreeSet::from([tile]);
        }
    }

    /// Whether all cells have collapsed.
    #[must_use]
    pub fn is_fully_collapsed(&self) -> bool {
        self.cells.iter().all(WfcCell::is_collapsed)
    }

    /// Whether any cell is in contradiction.
    #[must_use]
    pub fn has_contradiction(&self) -> bool {
        self.cells.iter().any(WfcCell::is_contradiction)
    }

    /// Propagate constraints from a collapsed cell to its neighbors.
    /// Returns the number of options removed.
    pub fn propagate(&mut self, rules: &AdjacencyRules) -> usize {
        let mut removed = 0;
        let mut changed = true;

        while changed {
            changed = false;
            for y in 0..self.height {
                for x in 0..self.width {
                    let idx = y * self.width + x;
                    let current_options = self.cells[idx].options.clone();

                    if x + 1 < self.width {
                        let right_idx = y * self.width + x + 1;
                        let allowed: BTreeSet<TileId> = current_options
                            .iter()
                            .flat_map(|&t| rules.right.get(t as usize).into_iter().flatten())
                            .copied()
                            .collect();
                        let before = self.cells[right_idx].options.len();
                        self.cells[right_idx].options = self.cells[right_idx]
                            .options
                            .intersection(&allowed)
                            .copied()
                            .collect();
                        let after = self.cells[right_idx].options.len();
                        if after < before {
                            removed += before - after;
                            changed = true;
                        }
                    }

                    if y + 1 < self.height {
                        let up_idx = (y + 1) * self.width + x;
                        let allowed: BTreeSet<TileId> = current_options
                            .iter()
                            .flat_map(|&t| rules.up.get(t as usize).into_iter().flatten())
                            .copied()
                            .collect();
                        let before = self.cells[up_idx].options.len();
                        self.cells[up_idx].options = self.cells[up_idx]
                            .options
                            .intersection(&allowed)
                            .copied()
                            .collect();
                        let after = self.cells[up_idx].options.len();
                        if after < before {
                            removed += before - after;
                            changed = true;
                        }
                    }
                }
            }
        }

        removed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_grid_has_full_entropy() {
        let grid = WfcGrid::new(4, 4, 5);
        assert_eq!(grid.get(0, 0).map(|c| c.entropy()), Some(5));
        assert!(!grid.is_fully_collapsed());
    }

    #[test]
    fn collapse_reduces_to_one() {
        let mut grid = WfcGrid::new(4, 4, 5);
        grid.collapse(1, 1, 3);
        assert!(grid.get(1, 1).map_or(false, |c| c.is_collapsed()));
        assert_eq!(grid.get(1, 1).and_then(|c| c.collapsed_tile()), Some(3));
    }

    #[test]
    fn unconstrained_rules_remove_nothing() {
        let rules = AdjacencyRules::unconstrained(3);
        let mut grid = WfcGrid::new(4, 4, 3);
        grid.collapse(0, 0, 1);
        let removed = grid.propagate(&rules);
        assert_eq!(removed, 0);
    }

    #[test]
    fn constrained_rules_propagate() {
        let mut rules = AdjacencyRules::unconstrained(3);
        rules.right[0] = BTreeSet::from([0, 1]);
        rules.right[1] = BTreeSet::from([1, 2]);
        rules.right[2] = BTreeSet::from([0]);

        let mut grid = WfcGrid::new(3, 1, 3);
        grid.collapse(0, 0, 0);
        let removed = grid.propagate(&rules);
        assert!(removed > 0);
        let right = grid.get(1, 0).unwrap();
        assert!(right.options.contains(&0));
        assert!(right.options.contains(&1));
        assert!(!right.options.contains(&2));
    }
}

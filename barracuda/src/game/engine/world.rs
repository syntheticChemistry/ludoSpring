// SPDX-License-Identifier: AGPL-3.0-or-later
//! 2D tile world — the spatial foundation for all game planes.
//!
//! A world is a rectangular grid of tiles with multiple layers:
//! terrain (base surface), features (furniture, doors, walls), and
//! fog of war (per-tile visibility). Entities live *on* the world
//! but are tracked separately in the entity system.
//!
//! Tile semantics are domain-driven: the same grid can represent
//! a dungeon, a town square, an evidence board, or a card table.
//! petalTongue renders each tile according to domain palette and modality.

/// Terrain type for a tile. Affects movement, line of sight, and narration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Terrain {
    /// Open ground — full movement, full visibility.
    Open,
    /// Difficult terrain — half movement speed.
    Difficult,
    /// Wall — blocks movement and line of sight.
    Wall,
    /// Door — blocks movement when closed, blocks LOS when closed.
    Door {
        /// Whether the door is open.
        open: bool,
    },
    /// Water — swimmable, slows movement.
    Water,
    /// Pit / void — impassable without flight or bridge.
    Void,
    /// Interactable surface (desk, altar, workbench).
    Surface,
    /// Domain-specific tile with a palette tag.
    Custom(u16),
}

impl Terrain {
    /// Whether this terrain blocks movement.
    #[must_use]
    pub const fn blocks_movement(self) -> bool {
        matches!(self, Self::Wall | Self::Void | Self::Door { open: false })
    }

    /// Whether this terrain blocks line of sight.
    #[must_use]
    pub const fn blocks_sight(self) -> bool {
        matches!(self, Self::Wall | Self::Door { open: false })
    }

    /// Movement cost multiplier (1.0 = normal, 2.0 = difficult, f64::INFINITY = impassable).
    #[must_use]
    pub const fn movement_cost(self) -> f64 {
        match self {
            Self::Open | Self::Surface | Self::Door { open: true } | Self::Custom(_) => 1.0,
            Self::Difficult | Self::Water => 2.0,
            Self::Wall | Self::Void | Self::Door { open: false } => f64::INFINITY,
        }
    }
}

/// Visibility state for fog of war.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Visibility {
    /// Never seen — completely hidden.
    #[default]
    Hidden,
    /// Previously seen but not currently visible — shown dimmed.
    Explored,
    /// Currently visible — fully rendered.
    Visible,
}

/// A single tile in the world grid.
#[derive(Debug, Clone)]
pub struct Tile {
    /// Base terrain.
    pub terrain: Terrain,
    /// Fog of war state.
    pub visibility: Visibility,
    /// Elevation for height-based mechanics (0 = ground level).
    pub elevation: i8,
    /// Optional description for audio narration and inspection.
    pub description: Option<String>,
}

impl Default for Tile {
    fn default() -> Self {
        Self {
            terrain: Terrain::Open,
            visibility: Visibility::Hidden,
            elevation: 0,
            description: None,
        }
    }
}

/// Cardinal and diagonal directions for movement and adjacency.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    /// Up (negative Y).
    North,
    /// Down (positive Y).
    South,
    /// Right (positive X).
    East,
    /// Left (negative X).
    West,
    /// Up-right.
    NorthEast,
    /// Up-left.
    NorthWest,
    /// Down-right.
    SouthEast,
    /// Down-left.
    SouthWest,
}

impl Direction {
    /// Delta (dx, dy) for this direction. Y increases southward (screen convention).
    #[must_use]
    pub const fn delta(self) -> (i32, i32) {
        match self {
            Self::North => (0, -1),
            Self::South => (0, 1),
            Self::East => (1, 0),
            Self::West => (-1, 0),
            Self::NorthEast => (1, -1),
            Self::NorthWest => (-1, -1),
            Self::SouthEast => (1, 1),
            Self::SouthWest => (-1, 1),
        }
    }

    /// All 8 directions.
    pub const ALL: [Self; 8] = [
        Self::North,
        Self::South,
        Self::East,
        Self::West,
        Self::NorthEast,
        Self::NorthWest,
        Self::SouthEast,
        Self::SouthWest,
    ];

    /// Cardinal directions only (4).
    pub const CARDINAL: [Self; 4] = [Self::North, Self::South, Self::East, Self::West];
}

/// A 2D tile world.
#[derive(Debug, Clone)]
pub struct TileWorld {
    tiles: Vec<Tile>,
    width: u32,
    height: u32,
    /// World name for narration and session tracking.
    pub name: String,
}

impl TileWorld {
    /// Create a world filled with the given terrain.
    #[must_use]
    pub fn new(width: u32, height: u32, name: &str, fill: Terrain) -> Self {
        let count = (width as usize) * (height as usize);
        let tiles = vec![
            Tile {
                terrain: fill,
                ..Tile::default()
            };
            count
        ];
        Self {
            tiles,
            width,
            height,
            name: name.into(),
        }
    }

    /// World width in tiles.
    #[must_use]
    pub const fn width(&self) -> u32 {
        self.width
    }

    /// World height in tiles.
    #[must_use]
    pub const fn height(&self) -> u32 {
        self.height
    }

    /// Total tile count.
    #[must_use]
    pub const fn tile_count(&self) -> usize {
        self.width as usize * self.height as usize
    }

    /// Whether coordinates are in bounds.
    #[must_use]
    pub const fn in_bounds(&self, x: u32, y: u32) -> bool {
        x < self.width && y < self.height
    }

    /// Index into the flat tile array.
    const fn index(&self, x: u32, y: u32) -> usize {
        y as usize * self.width as usize + x as usize
    }

    /// Get a tile reference.
    #[must_use]
    pub fn get(&self, x: u32, y: u32) -> Option<&Tile> {
        if self.in_bounds(x, y) {
            Some(&self.tiles[self.index(x, y)])
        } else {
            None
        }
    }

    /// Get a mutable tile reference.
    pub fn get_mut(&mut self, x: u32, y: u32) -> Option<&mut Tile> {
        if self.in_bounds(x, y) {
            let idx = self.index(x, y);
            Some(&mut self.tiles[idx])
        } else {
            None
        }
    }

    /// Set terrain at a position.
    pub fn set_terrain(&mut self, x: u32, y: u32, terrain: Terrain) {
        if let Some(tile) = self.get_mut(x, y) {
            tile.terrain = terrain;
        }
    }

    /// Whether movement from (x,y) in the given direction is possible.
    #[must_use]
    #[expect(clippy::cast_possible_wrap, reason = "grid coords are small positive numbers")]
    #[expect(clippy::cast_sign_loss, reason = "nx,ny validated non-negative above")]
    pub fn can_move(&self, x: u32, y: u32, dir: Direction) -> bool {
        let (dx, dy) = dir.delta();
        let nx = x as i32 + dx;
        let ny = y as i32 + dy;
        if nx < 0 || ny < 0 {
            return false;
        }
        let (nx, ny) = (nx as u32, ny as u32);
        self.get(nx, ny)
            .is_some_and(|t| !t.terrain.blocks_movement())
    }

    /// Apply coordinates after moving in a direction. Returns `None` if blocked.
    #[must_use]
    #[expect(clippy::cast_possible_wrap, reason = "grid coords are small positive numbers")]
    #[expect(clippy::cast_sign_loss, reason = "can_move validates result is in bounds")]
    pub fn move_in(&self, x: u32, y: u32, dir: Direction) -> Option<(u32, u32)> {
        if !self.can_move(x, y, dir) {
            return None;
        }
        let (dx, dy) = dir.delta();
        Some(((x as i32 + dx) as u32, (y as i32 + dy) as u32))
    }

    /// Reveal tiles within a radius using simple distance check.
    /// Tiles within `radius` of (cx, cy) become `Visible`; previously
    /// visible tiles outside the radius become `Explored`.
    pub fn reveal_radius(&mut self, cx: u32, cy: u32, radius: u32) {
        let r2 = i64::from(radius * radius);
        for y in 0..self.height {
            for x in 0..self.width {
                let dx = i64::from(x) - i64::from(cx);
                let dy = i64::from(y) - i64::from(cy);
                let dist2 = dx * dx + dy * dy;
                let idx = self.index(x, y);
                if dist2 <= r2 {
                    self.tiles[idx].visibility = Visibility::Visible;
                } else if self.tiles[idx].visibility == Visibility::Visible {
                    self.tiles[idx].visibility = Visibility::Explored;
                }
            }
        }
    }

    /// Line-of-sight check using Bresenham's algorithm.
    /// Returns `true` if there is a clear line from (x0,y0) to (x1,y1).
    #[must_use]
    #[expect(clippy::cast_possible_wrap, reason = "grid coords are small positive numbers")]
    #[expect(clippy::cast_sign_loss, reason = "Bresenham iterates within grid bounds")]
    pub fn has_line_of_sight(&self, x0: u32, y0: u32, x1: u32, y1: u32) -> bool {
        let (mut cx, mut cy) = (x0 as i32, y0 as i32);
        let (tx, ty) = (x1 as i32, y1 as i32);
        let dx = (tx - cx).abs();
        let dy = -(ty - cy).abs();
        let sx = if cx < tx { 1 } else { -1 };
        let sy = if cy < ty { 1 } else { -1 };
        let mut err = dx + dy;

        loop {
            if cx == tx && cy == ty {
                return true;
            }
            // Check intermediate tiles (skip start)
            if (cx != x0 as i32 || cy != y0 as i32)
                && self
                    .get(cx as u32, cy as u32)
                    .is_some_and(|t| t.terrain.blocks_sight())
            {
                return false;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                cx += sx;
            }
            if e2 <= dx {
                err += dx;
                cy += sy;
            }
        }
    }

    /// Collect all tiles as terrain values for FieldMap rendering.
    /// Returns row-major `f64` values suitable for petalTongue FieldMap.
    #[must_use]
    pub fn terrain_fieldmap(&self) -> Vec<f64> {
        self.tiles
            .iter()
            .map(|t| match t.terrain {
                Terrain::Open => 0.0,
                Terrain::Difficult => 0.25,
                Terrain::Water => 0.5,
                Terrain::Surface => 0.6,
                Terrain::Door { open: true } => 0.1,
                Terrain::Door { open: false } => 0.8,
                Terrain::Wall => 1.0,
                Terrain::Void => -1.0,
                Terrain::Custom(id) => f64::from(id) / 65535.0,
            })
            .collect()
    }

    /// Collect visibility as boolean mask for fog of war rendering.
    #[must_use]
    pub fn visibility_mask(&self) -> Vec<bool> {
        self.tiles
            .iter()
            .map(|t| t.visibility != Visibility::Hidden)
            .collect()
    }

    /// Count tiles matching a predicate.
    #[must_use]
    pub fn count_where<F: Fn(&Tile) -> bool>(&self, predicate: F) -> usize {
        self.tiles.iter().filter(|t| predicate(t)).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_creation() {
        let w = TileWorld::new(10, 8, "Test Tavern", Terrain::Open);
        assert_eq!(w.width(), 10);
        assert_eq!(w.height(), 8);
        assert_eq!(w.tile_count(), 80);
        assert!(w.in_bounds(9, 7));
        assert!(!w.in_bounds(10, 7));
    }

    #[test]
    fn terrain_properties() {
        assert!(!Terrain::Open.blocks_movement());
        assert!(!Terrain::Open.blocks_sight());
        assert!(Terrain::Wall.blocks_movement());
        assert!(Terrain::Wall.blocks_sight());
        assert!(Terrain::Door { open: false }.blocks_movement());
        assert!(!Terrain::Door { open: true }.blocks_movement());
        assert!((Terrain::Difficult.movement_cost() - 2.0).abs() < f64::EPSILON);
        assert!(Terrain::Wall.movement_cost().is_infinite());
    }

    #[test]
    fn movement() {
        let mut w = TileWorld::new(5, 5, "Grid", Terrain::Open);
        w.set_terrain(3, 2, Terrain::Wall);

        assert!(w.can_move(2, 2, Direction::North));
        assert!(w.can_move(2, 2, Direction::South));
        // East from (2,2) goes to (3,2) which has a wall
        assert!(!w.can_move(2, 2, Direction::East));
        // But moving west from (2,2) should succeed to (1,2)
        assert!(w.can_move(2, 2, Direction::West));

        let dest = w.move_in(1, 1, Direction::South);
        assert_eq!(dest, Some((1, 2)));

        // Can't move off the edge
        assert!(!w.can_move(0, 0, Direction::North));
        assert!(!w.can_move(0, 0, Direction::West));
    }

    #[test]
    fn fog_of_war() {
        let mut w = TileWorld::new(10, 10, "Fog Test", Terrain::Open);
        assert_eq!(
            w.count_where(|t| t.visibility == Visibility::Hidden),
            100
        );

        w.reveal_radius(5, 5, 2);
        let visible = w.count_where(|t| t.visibility == Visibility::Visible);
        assert!(visible > 0);
        assert!(visible < 100);

        // Move the reveal center — old visible becomes explored
        w.reveal_radius(8, 8, 1);
        let explored = w.count_where(|t| t.visibility == Visibility::Explored);
        assert!(explored > 0);
    }

    #[test]
    fn line_of_sight() {
        let mut w = TileWorld::new(10, 10, "LOS Test", Terrain::Open);
        assert!(w.has_line_of_sight(0, 0, 9, 9));

        w.set_terrain(5, 5, Terrain::Wall);
        assert!(!w.has_line_of_sight(0, 0, 9, 9));
        // Lateral should still work
        assert!(w.has_line_of_sight(0, 0, 0, 9));
    }

    #[test]
    fn fieldmap_export() {
        let mut w = TileWorld::new(3, 3, "Export", Terrain::Open);
        w.set_terrain(1, 1, Terrain::Wall);
        let fm = w.terrain_fieldmap();
        assert_eq!(fm.len(), 9);
        assert!((fm[4] - 1.0).abs() < f64::EPSILON); // center = wall
        assert!(fm[0].abs() < f64::EPSILON); // corner = open
    }

    #[test]
    fn visibility_mask_export() {
        let mut w = TileWorld::new(4, 4, "Mask", Terrain::Open);
        w.reveal_radius(0, 0, 1);
        let mask = w.visibility_mask();
        assert_eq!(mask.len(), 16);
        assert!(mask[0]); // (0,0) visible
    }

    #[test]
    fn direction_deltas() {
        assert_eq!(Direction::North.delta(), (0, -1));
        assert_eq!(Direction::SouthEast.delta(), (1, 1));
        assert_eq!(Direction::ALL.len(), 8);
        assert_eq!(Direction::CARDINAL.len(), 4);
    }

    #[test]
    fn door_toggle() {
        let mut w = TileWorld::new(3, 3, "Door", Terrain::Open);
        w.set_terrain(1, 1, Terrain::Door { open: false });
        assert!(!w.can_move(0, 1, Direction::East));

        w.set_terrain(1, 1, Terrain::Door { open: true });
        assert!(w.can_move(0, 1, Direction::East));
    }

    #[test]
    fn elevation_default() {
        let w = TileWorld::new(2, 2, "Flat", Terrain::Open);
        assert_eq!(w.get(0, 0).unwrap().elevation, 0);
    }

    #[test]
    fn tile_description() {
        let mut w = TileWorld::new(3, 3, "Described", Terrain::Open);
        if let Some(tile) = w.get_mut(1, 1) {
            tile.description = Some("A worn wooden table".into());
            tile.terrain = Terrain::Surface;
        }
        assert_eq!(
            w.get(1, 1).unwrap().description.as_deref(),
            Some("A worn wooden table")
        );
    }
}

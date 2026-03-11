// SPDX-License-Identifier: AGPL-3.0-or-later
//! Voxel world — block-based spatial representation.
//!
//! Minecraft-style voxel worlds where each block can represent an atom,
//! molecule, field sample, or game element. Chunks are the unit of
//! generation, storage, and rendering.
//!
//! # Design
//!
//! Unlike Minecraft's fixed 16x16x256 chunks, ludoSpring uses configurable
//! chunk dimensions to accommodate different scales:
//! - Atomic scale: 8x8x8 chunks, each block = 1 Angstrom
//! - Molecular scale: 16x16x16 chunks, each block = 1 nm
//! - Cellular scale: 32x32x32 chunks, each block = 1 μm

/// A block type in the voxel world.
///
/// The `u16` tag maps to a palette entry (element, molecule, terrain, etc.)
/// via a domain-specific palette. 0 = air/empty.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct BlockId(pub u16);

impl BlockId {
    /// Empty / air block.
    pub const AIR: Self = Self(0);

    /// Whether this block is empty.
    #[must_use]
    pub const fn is_air(self) -> bool {
        self.0 == 0
    }
}

/// A palette mapping block IDs to domain-specific metadata.
#[derive(Debug, Clone)]
pub struct BlockPalette {
    entries: Vec<BlockEntry>,
}

/// Metadata for a block type.
#[derive(Debug, Clone)]
pub struct BlockEntry {
    /// Human-readable name (e.g. "Carbon", "Water", "Stone").
    pub name: String,
    /// Domain tag for the spring that owns this block type.
    pub domain: String,
    /// RGBA color [0.0–1.0].
    pub color: [f32; 4],
    /// Whether this block is solid (for collision/navigation).
    pub solid: bool,
    /// Atomic number (if this represents an element). 0 = not an element.
    pub atomic_number: u8,
}

impl BlockPalette {
    /// Create an empty palette (entry 0 = air, always present).
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: vec![BlockEntry {
                name: "Air".into(),
                domain: "core".into(),
                color: [0.0, 0.0, 0.0, 0.0],
                solid: false,
                atomic_number: 0,
            }],
        }
    }

    /// Register a new block type. Returns its `BlockId`.
    ///
    /// # Panics
    ///
    /// Panics if more than `u16::MAX` block types are registered.
    pub fn register(&mut self, entry: BlockEntry) -> BlockId {
        let id = u16::try_from(self.entries.len())
            .unwrap_or_else(|_| panic!("palette overflow: {} exceeds u16", self.entries.len()));
        self.entries.push(entry);
        BlockId(id)
    }

    /// Look up a block type by ID.
    #[must_use]
    pub fn get(&self, id: BlockId) -> Option<&BlockEntry> {
        self.entries.get(id.0 as usize)
    }

    /// Number of registered block types (including air).
    #[must_use]
    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the palette is empty (only air).
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.entries.len() <= 1
    }
}

impl Default for BlockPalette {
    fn default() -> Self {
        Self::new()
    }
}

/// A fixed-size chunk of voxel data.
///
/// Stored as a flat array in XZY order (Minecraft convention).
#[derive(Debug, Clone)]
pub struct Chunk {
    /// Chunk dimensions.
    pub size_x: usize,
    /// Chunk dimensions.
    pub size_y: usize,
    /// Chunk dimensions.
    pub size_z: usize,
    /// Block data in XZY order.
    blocks: Vec<BlockId>,
    /// Chunk position in world-space (chunk coordinates, not block coordinates).
    pub position: [i32; 3],
}

impl Chunk {
    /// Create a new empty (all-air) chunk.
    #[must_use]
    pub fn new(size_x: usize, size_y: usize, size_z: usize, position: [i32; 3]) -> Self {
        Self {
            size_x,
            size_y,
            size_z,
            blocks: vec![BlockId::AIR; size_x * size_y * size_z],
            position,
        }
    }

    /// Create a standard 16x16x16 chunk.
    #[must_use]
    pub fn standard(position: [i32; 3]) -> Self {
        Self::new(16, 16, 16, position)
    }

    const fn index(&self, x: usize, y: usize, z: usize) -> usize {
        x + z * self.size_x + y * self.size_x * self.size_z
    }

    /// Get the block at (x, y, z) local coordinates.
    #[must_use]
    pub fn get(&self, x: usize, y: usize, z: usize) -> BlockId {
        if x >= self.size_x || y >= self.size_y || z >= self.size_z {
            return BlockId::AIR;
        }
        self.blocks[self.index(x, y, z)]
    }

    /// Set the block at (x, y, z) local coordinates.
    pub fn set(&mut self, x: usize, y: usize, z: usize, block: BlockId) {
        if x < self.size_x && y < self.size_y && z < self.size_z {
            let idx = self.index(x, y, z);
            self.blocks[idx] = block;
        }
    }

    /// Count non-air blocks.
    #[must_use]
    pub fn solid_count(&self) -> usize {
        self.blocks.iter().filter(|b| !b.is_air()).count()
    }

    /// Total capacity.
    #[must_use]
    pub const fn capacity(&self) -> usize {
        self.size_x * self.size_y * self.size_z
    }

    /// Fill ratio (0.0–1.0).
    #[must_use]
    #[expect(
        clippy::cast_precision_loss,
        reason = "chunk dimensions are small (≤256); usize values fit in f64 mantissa"
    )]
    pub fn density(&self) -> f64 {
        let cap = self.capacity();
        if cap == 0 {
            return 0.0;
        }
        self.solid_count() as f64 / cap as f64
    }
}

/// Build a chemistry palette with common elements.
///
/// Colors follow the CPK (Corey-Pauling-Koltun) convention; see
/// [`crate::tolerances`] for color constants and citations.
#[must_use]
pub fn chemistry_palette() -> BlockPalette {
    use crate::tolerances::{
        CPK_CALCIUM, CPK_CARBON, CPK_CHLORINE, CPK_HYDROGEN, CPK_IRON, CPK_NITROGEN, CPK_OXYGEN,
        CPK_PHOSPHORUS, CPK_SODIUM, CPK_SULFUR,
    };

    let mut p = BlockPalette::new();
    let elements: [(&str, u8, [f32; 4]); 10] = [
        ("Hydrogen", 1, CPK_HYDROGEN),
        ("Carbon", 6, CPK_CARBON),
        ("Nitrogen", 7, CPK_NITROGEN),
        ("Oxygen", 8, CPK_OXYGEN),
        ("Phosphorus", 15, CPK_PHOSPHORUS),
        ("Sulfur", 16, CPK_SULFUR),
        ("Iron", 26, CPK_IRON),
        ("Sodium", 11, CPK_SODIUM),
        ("Chlorine", 17, CPK_CHLORINE),
        ("Calcium", 20, CPK_CALCIUM),
    ];
    for (name, num, color) in elements {
        p.register(BlockEntry {
            name: name.into(),
            domain: "chemistry".into(),
            color,
            solid: true,
            atomic_number: num,
        });
    }
    p
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_chunk_is_all_air() {
        let chunk = Chunk::standard([0, 0, 0]);
        assert_eq!(chunk.solid_count(), 0);
        assert!((chunk.density() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn set_and_get_block() {
        let mut chunk = Chunk::standard([0, 0, 0]);
        let carbon = BlockId(1);
        chunk.set(5, 5, 5, carbon);
        assert_eq!(chunk.get(5, 5, 5), carbon);
        assert_eq!(chunk.get(0, 0, 0), BlockId::AIR);
    }

    #[test]
    fn chemistry_palette_has_common_elements() {
        let palette = chemistry_palette();
        assert!(palette.len() > 10);
        let Some(hydrogen) = palette.get(BlockId(1)) else {
            panic!("hydrogen must be in palette");
        };
        assert_eq!(hydrogen.atomic_number, 1);
    }

    #[test]
    fn out_of_bounds_returns_air() {
        let chunk = Chunk::standard([0, 0, 0]);
        assert_eq!(chunk.get(999, 999, 999), BlockId::AIR);
    }
}

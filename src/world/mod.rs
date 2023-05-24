pub(crate) mod block;
pub(crate) mod tree;
pub(crate) mod wall;
pub(crate) mod light;
pub(crate) mod chunk;
pub(crate) mod generator;
pub(crate) mod save_as;

use bevy::prelude::Resource;
use bevy_ecs_tilemap::{tiles::TilePos, prelude::TilemapSize, helpers::square_grid::neighbors::{SquareDirection, Neighbors}};
use ndarray::Array2;

use self::{block::Block, wall::Wall};

pub(crate) type BlockArray = Array2<Option<Block>>;
pub(crate) type WallArray = Array2<Option<Wall>>;

#[derive(Clone, Copy)]
pub struct Layer {
    pub(crate) surface: usize,
    pub(crate) underground: usize,
    pub(crate) cavern: usize,
    pub(crate) dirt_height: usize
}

#[derive(Clone, Copy)]
pub(crate) struct Size {
    pub(crate) width: usize,
    pub(crate) height: usize
}

impl Size {
    pub(crate) fn as_tilemap_size(&self) -> TilemapSize {
        TilemapSize { x: self.width as u32, y: self.height as u32 }
    }
}

#[derive(Clone, Copy)]
pub enum WorldSize {
    Tiny,
    Medium,
    Large
}

impl WorldSize {
    pub(crate) fn size(&self) -> Size {
        match self {
            WorldSize::Tiny => Size { width: 1750, height: 900 },
            WorldSize::Medium => Size { width: 6400, height: 1800 },
            WorldSize::Large => Size { width: 8400, height: 2400 }
        }
    }
}

#[derive(Resource)]
pub(crate) struct WorldData {
    pub(crate) size: Size,
    pub(crate) layer: Layer,
    pub(crate) spawn_point: TilePos,
    pub(crate) blocks: Array2<Option<Block>>,
    pub(crate) walls: Array2<Option<Wall>>,
}

pub trait AsWorldPos {
    fn x(&self) -> usize;
    fn y(&self) -> usize;

    fn yx(&self) -> (usize, usize) { (self.y(), self.x()) }
    fn as_tile_pos(&self) -> TilePos { TilePos::new(self.x() as u32, self.y() as u32) }
}

impl AsWorldPos for TilePos {
    fn x(&self) -> usize { self.x as usize }
    fn y(&self) -> usize { self.y as usize }
}

impl AsWorldPos for &TilePos {
    fn x(&self) -> usize { self.x as usize }
    fn y(&self) -> usize { self.y as usize }
}

impl AsWorldPos for (usize, usize) {
    fn x(&self) -> usize { self.0 }
    fn y(&self) -> usize { self.1 }
}

impl AsWorldPos for (u32, u32) {
    fn x(&self) -> usize { self.0 as usize }
    fn y(&self) -> usize { self.1 as usize }
}

impl WorldData {
    #[inline]
    pub(crate) fn get_block<Pos: AsWorldPos>(&self, world_pos: Pos) -> Option<&Block> {
        self.blocks.get(world_pos.yx()).and_then(|b| b.as_ref())
    }

    #[inline]
    pub(crate) fn get_solid_block<Pos: AsWorldPos>(&self, world_pos: Pos) -> Option<&Block> {
        self.blocks
            .get(world_pos.yx())
            .and_then(|b| b.as_ref())
            .filter(|b| b.is_solid())
    }

    #[inline(always)]
    pub(crate) fn get_block_mut<Pos: AsWorldPos>(&mut self, world_pos: Pos) -> Option<&mut Block> {
        self.blocks.get_mut(world_pos.yx()).and_then(|b| b.as_mut())
    }

    #[inline(always)]
    pub(crate) fn get_wall<Pos: AsWorldPos>(&self, world_pos: Pos) -> Option<&Wall> {
        self.walls.get(world_pos.yx()).and_then(|w| w.as_ref())
    }

    #[inline(always)]
    pub(crate) fn get_wall_mut<Pos: AsWorldPos>(&mut self, world_pos: Pos) -> Option<&mut Wall> {
        self.walls.get_mut(world_pos.yx()).and_then(|w| w.as_mut())
    }

    pub(crate) fn set_block<Pos: AsWorldPos>(&mut self, world_pos: Pos, block: &Block) {
        unsafe {
            if let Some(b) = self.blocks.get_mut_ptr(world_pos.yx()) {
                *b = Some(*block);
            }
        }
    }

    pub(crate) fn set_wall<Pos: AsWorldPos>(&mut self, world_pos: Pos, wall: Wall) {
        unsafe {
            if let Some(w) = self.walls.get_mut_ptr(world_pos.yx()) {
                *w = Some(wall);
            }
        }
    }

    pub(crate) fn remove_block<Pos: AsWorldPos>(&mut self, world_pos: Pos) {
        unsafe {
            if let Some(block) = self.blocks.get_mut_ptr(world_pos.yx()) {
                *block = None;
            }
        }
    }

    #[inline(always)]
    pub(crate) fn block_exists<Pos: AsWorldPos>(&self, world_pos: Pos) -> bool {
        self.get_block(world_pos).is_some()
    }

    #[inline(always)]
    pub(crate) fn block_not_exists<Pos: AsWorldPos>(&self, world_pos: Pos) -> bool {
        !self.block_exists(world_pos)
    }

    #[inline]
    pub(crate) fn block_exists_with_type<Pos: AsWorldPos>(&self, world_pos: Pos, block_type: Block) -> bool {
        if let Some(block) = self.get_block(world_pos) { *block == block_type } else { false }
    }

    #[inline]
    pub(crate) fn solid_block_exists<Pos: AsWorldPos>(&self, world_pos: Pos) -> bool {
        if let Some(block) = self.get_block(world_pos) { block.is_solid() } else { false }
    }

    #[inline]
    pub(crate) fn solid_block_not_exists<Pos: AsWorldPos>(&self, world_pos: Pos) -> bool {
        !self.solid_block_exists(world_pos)
    }

    #[inline(always)]
    pub(crate) fn wall_exists<Pos: AsWorldPos>(&self, world_pos: Pos) -> bool {
        self.get_wall(world_pos).is_some()
    }

    #[inline(always)]
    pub(crate) fn wall_not_exists<Pos: AsWorldPos>(&self, world_pos: Pos) -> bool {
        !self.wall_exists(world_pos)
    }

    pub(crate) fn get_block_neighbors<Pos: AsWorldPos>(&self, world_pos: Pos, solid: bool) -> Neighbors<&Block> {
        let tile_pos = world_pos.as_tile_pos();

        let get_block = move |pos: TilePos| -> Option<&Block> {
            if solid {
                self.get_solid_block(pos)
            } else {
                self.get_block(pos)
            }
        };

        Neighbors {
            west: tile_pos.square_offset(&SquareDirection::West, &self.size.as_tilemap_size()).and_then(get_block),
            east: tile_pos.square_offset(&SquareDirection::East, &self.size.as_tilemap_size()).and_then(get_block),
            north: tile_pos.square_offset(&SquareDirection::South, &self.size.as_tilemap_size()).and_then(get_block),
            south: tile_pos.square_offset(&SquareDirection::North, &self.size.as_tilemap_size()).and_then(get_block),
            north_west: tile_pos.square_offset(&SquareDirection::SouthWest, &self.size.as_tilemap_size()).and_then(get_block),
            south_west: tile_pos.square_offset(&SquareDirection::NorthWest, &self.size.as_tilemap_size()).and_then(get_block),
            south_east: tile_pos.square_offset(&SquareDirection::NorthEast, &self.size.as_tilemap_size()).and_then(get_block),
            north_east: tile_pos.square_offset(&SquareDirection::SouthEast, &self.size.as_tilemap_size()).and_then(get_block),
        }
    }

    pub(crate) fn get_wall_neighbors<Pos: AsWorldPos>(&self, world_pos: Pos) -> Neighbors<&Wall> {
        let tile_pos = world_pos.as_tile_pos();

        Neighbors {
            west: tile_pos.square_offset(&SquareDirection::West, &self.size.as_tilemap_size()).and_then(|pos| self.get_wall(pos)),
            east: tile_pos.square_offset(&SquareDirection::East, &self.size.as_tilemap_size()).and_then(|pos| self.get_wall(pos)),
            north: tile_pos.square_offset(&SquareDirection::South, &self.size.as_tilemap_size()).and_then(|pos| self.get_wall(pos)),
            south: tile_pos.square_offset(&SquareDirection::North, &self.size.as_tilemap_size()).and_then(|pos| self.get_wall(pos)),
            north_west: tile_pos.square_offset(&SquareDirection::SouthWest, &self.size.as_tilemap_size()).and_then(|pos| self.get_wall(pos)),
            south_west: tile_pos.square_offset(&SquareDirection::NorthWest, &self.size.as_tilemap_size()).and_then(|pos| self.get_wall(pos)),
            south_east: tile_pos.square_offset(&SquareDirection::NorthEast, &self.size.as_tilemap_size()).and_then(|pos| self.get_wall(pos)),
            north_east: tile_pos.square_offset(&SquareDirection::SouthEast, &self.size.as_tilemap_size()).and_then(|pos| self.get_wall(pos)),
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct TerrariaFrame {
    pub(crate) x: u16,
    pub(crate) y: u16
}

impl TerrariaFrame {
    pub(crate) const fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }
}

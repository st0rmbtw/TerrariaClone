pub mod block;
pub mod tree;
pub mod wall;
pub(crate) mod chunk;
pub mod generator;
pub(crate) mod save_as;

use bevy::{prelude::Resource, math::URect};
use bevy_ecs_tilemap::{tiles::TilePos, prelude::TilemapSize, helpers::square_grid::neighbors::{SquareDirection, Neighbors}};
use ndarray::Array2;

use self::{block::{Block, BlockType}, wall::Wall};

pub(crate) type BlockArray = Array2<Option<Block>>;
pub(crate) type WallArray = Array2<Option<Wall>>;

#[derive(Clone, Copy)]
pub struct Layer {
    pub surface: usize,
    pub underground: usize,
    pub cavern: usize,
    pub dirt_height: usize
}

#[derive(Clone, Copy)]
pub struct Size {
    pub width: usize,
    pub height: usize
}

#[derive(Clone, Copy)]
pub enum WorldSize {
    Tiny,
    Medium,
    Large
}

impl WorldSize {
    pub const fn size(&self) -> Size {
        match self {
            WorldSize::Tiny => Size { width: 1750, height: 900 },
            WorldSize::Medium => Size { width: 6400, height: 1800 },
            WorldSize::Large => Size { width: 8400, height: 2400 }
        }
    }
}

#[derive(Resource)]
pub struct WorldData {
    pub area: URect,
    pub layer: Layer,
    pub playable_area: URect,
    pub spawn_point: TilePos,
    pub blocks: Array2<Option<Block>>,
    pub walls: Array2<Option<Wall>>,
}

pub trait AsWorldPos {
    fn x(&self) -> usize;
    fn y(&self) -> usize;

    #[inline(always)]
    fn yx(&self) -> (usize, usize) { (self.y(), self.x()) }

    #[inline(always)]
    fn as_tile_pos(&self) -> TilePos { TilePos::new(self.x() as u32, self.y() as u32) }
}

impl AsWorldPos for TilePos {
    #[inline(always)]
    fn x(&self) -> usize { self.x as usize }

    #[inline(always)]
    fn y(&self) -> usize { self.y as usize }
}

impl AsWorldPos for &TilePos {
    #[inline(always)]
    fn x(&self) -> usize { self.x as usize }

    #[inline(always)]
    fn y(&self) -> usize { self.y as usize }
}

impl AsWorldPos for (usize, usize) {
    #[inline(always)]
    fn x(&self) -> usize { self.0 }

    #[inline(always)]
    fn y(&self) -> usize { self.1 }
}

impl AsWorldPos for (u32, u32) {
    #[inline(always)]
    fn x(&self) -> usize { self.0 as usize }

    #[inline(always)]
    fn y(&self) -> usize { self.1 as usize }
}

#[allow(dead_code)]
impl WorldData {
    #[inline(always)]
    pub fn width(&self) -> usize {
        self.area.width() as usize
    }

    #[inline(always)]
    pub fn height(&self) -> usize {
        self.area.height() as usize
    }

    #[inline(always)]
    pub fn playable_width(&self) -> usize {
        self.playable_area.width() as usize
    }

    #[inline(always)]
    pub fn playable_height(&self) -> usize {
        self.playable_area.height() as usize
    }

    #[inline]
    pub fn get_block<Pos: AsWorldPos>(&self, world_pos: Pos) -> Option<&Block> {
        self.blocks.get(world_pos.yx()).and_then(|b| b.as_ref())
    }

    #[inline]
    pub(crate) fn get_block_with_type<Pos: AsWorldPos>(&self, world_pos: Pos, block_type: BlockType) -> Option<&Block> {
        self.get_block(world_pos).filter(|b| b.block_type == block_type)
    }

    #[inline]
    pub(crate) fn get_solid_block<Pos: AsWorldPos>(&self, world_pos: Pos) -> Option<&Block> {
        self.get_block(world_pos).filter(|b| b.is_solid())
    }

    #[inline(always)]
    pub(crate) fn get_block_mut<Pos: AsWorldPos>(&mut self, world_pos: Pos) -> Option<&mut Block> {
        self.blocks.get_mut(world_pos.yx()).and_then(|b| b.as_mut())
    }

    #[inline]
    pub(crate) fn get_block_with_type_mut<Pos: AsWorldPos>(&mut self, world_pos: Pos, block_type: BlockType) -> Option<&mut Block> {
        self.get_block_mut(world_pos).filter(|b| b.block_type == block_type)
    }

    #[inline(always)]
    pub fn get_wall<Pos: AsWorldPos>(&self, world_pos: Pos) -> Option<&Wall> {
        self.walls.get(world_pos.yx()).and_then(|w| w.as_ref())
    }

    #[inline(always)]
    pub(crate) fn get_wall_mut<Pos: AsWorldPos>(&mut self, world_pos: Pos) -> Option<&mut Wall> {
        self.walls.get_mut(world_pos.yx()).and_then(|w| w.as_mut())
    }

    pub(crate) fn set_block<Pos: AsWorldPos>(&mut self, world_pos: Pos, block: impl Into<Block>) {
        if let Some(b) = self.blocks.get_mut_ptr(world_pos.yx()) {
            unsafe {
                *b = Some(block.into());
            }
        }
    }

    pub(crate) fn set_wall<Pos: AsWorldPos>(&mut self, world_pos: Pos, wall: Wall) {
        if let Some(w) = self.walls.get_mut_ptr(world_pos.yx()) {
            unsafe {
                *w = Some(wall);
            }
        }
    }

    pub(crate) fn remove_block<Pos: AsWorldPos>(&mut self, world_pos: Pos) {
        if let Some(block) = self.blocks.get_mut_ptr(world_pos.yx()) {
            unsafe {
                *block = None;
            }
        }
    }

    pub(crate) fn remove_wall<Pos: AsWorldPos>(&mut self, world_pos: Pos) {
        if let Some(wall) = self.walls.get_mut_ptr(world_pos.yx()) {
            unsafe {
                *wall = None;
            }
        }
    }

    #[inline(always)]
    pub(crate) fn block_exists<Pos: AsWorldPos>(&self, world_pos: Pos) -> bool {
        self.get_block(world_pos).is_some()
    }

    #[inline]
    pub(crate) fn block_exists_with_type<Pos: AsWorldPos>(&self, world_pos: Pos, block_type: BlockType) -> bool {
        self.get_block(world_pos).is_some_and(|b| b.block_type == block_type)
    }

    #[inline]
    pub(crate) fn solid_block_exists<Pos: AsWorldPos>(&self, world_pos: Pos) -> bool {
        self.get_block(world_pos).is_some_and(|b| b.is_solid())
    }

    #[inline(always)]
    pub(crate) fn wall_exists<Pos: AsWorldPos>(&self, world_pos: Pos) -> bool {
        self.get_wall(world_pos).is_some()
    }

    pub(crate) fn get_block_neighbors<Pos: AsWorldPos + Copy>(&self, world_pos: Pos, solid: bool) -> Neighbors<&Block> {
        let tile_pos = world_pos.as_tile_pos();
        let tilemap_size = &TilemapSize::from(self.area.size());

        let get_block = move |pos: TilePos| -> Option<&Block> {
            if solid {
                self.get_solid_block(pos)
            } else {
                self.get_block(pos)
            }
        };

        Neighbors {
            west:       tile_pos.square_offset(&SquareDirection::West,      tilemap_size).and_then(get_block),
            east:       tile_pos.square_offset(&SquareDirection::East,      tilemap_size).and_then(get_block),
            north:      tile_pos.square_offset(&SquareDirection::South,     tilemap_size).and_then(get_block),
            south:      tile_pos.square_offset(&SquareDirection::North,     tilemap_size).and_then(get_block),
            north_west: tile_pos.square_offset(&SquareDirection::SouthWest, tilemap_size).and_then(get_block),
            south_west: tile_pos.square_offset(&SquareDirection::NorthWest, tilemap_size).and_then(get_block),
            south_east: tile_pos.square_offset(&SquareDirection::NorthEast, tilemap_size).and_then(get_block),
            north_east: tile_pos.square_offset(&SquareDirection::SouthEast, tilemap_size).and_then(get_block),
        }
    }

    pub(crate) fn get_wall_neighbors<Pos: AsWorldPos>(&self, world_pos: Pos) -> Neighbors<&Wall> {
        let tile_pos = world_pos.as_tile_pos();
        let tilemap_size = &TilemapSize::from(self.area.size());

        Neighbors {
            west:       tile_pos.square_offset(&SquareDirection::West,      tilemap_size).and_then(|pos| self.get_wall(pos)),
            east:       tile_pos.square_offset(&SquareDirection::East,      tilemap_size).and_then(|pos| self.get_wall(pos)),
            north:      tile_pos.square_offset(&SquareDirection::South,     tilemap_size).and_then(|pos| self.get_wall(pos)),
            south:      tile_pos.square_offset(&SquareDirection::North,     tilemap_size).and_then(|pos| self.get_wall(pos)),
            north_west: tile_pos.square_offset(&SquareDirection::SouthWest, tilemap_size).and_then(|pos| self.get_wall(pos)),
            south_west: tile_pos.square_offset(&SquareDirection::NorthWest, tilemap_size).and_then(|pos| self.get_wall(pos)),
            south_east: tile_pos.square_offset(&SquareDirection::NorthEast, tilemap_size).and_then(|pos| self.get_wall(pos)),
            north_east: tile_pos.square_offset(&SquareDirection::SouthEast, tilemap_size).and_then(|pos| self.get_wall(pos)),
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

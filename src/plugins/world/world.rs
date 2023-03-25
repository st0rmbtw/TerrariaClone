use bevy::prelude::Resource;
use bevy_ecs_tilemap::{tiles::TilePos, helpers::square_grid::neighbors::{Neighbors, SquareDirection}, prelude::{SquarePos, TilemapSize}};

use super::{generator::{BlockArray, WallArray}, block::Block, wall::Wall};

#[derive(Clone, Copy)]
pub(crate) struct Layer {
    pub(crate) surface: usize,
    pub(crate) underground: usize,
    pub(crate) cavern: usize,
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
pub(super) enum WorldSize {
    Tiny,
    Medium,
    Large
}

impl WorldSize {
    pub(super) fn size(&self) -> Size {
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
    pub(crate) blocks: BlockArray,
    pub(crate) walls: WallArray,
}

pub(crate) trait AsTilePos {
    fn x(&self) -> usize;
    fn y(&self) -> usize;

    fn yx(&self) -> (usize, usize) { (self.y(), self.x()) }
}

impl AsTilePos for TilePos {
    fn x(&self) -> usize { self.x as usize }
    fn y(&self) -> usize { self.y as usize }
}

impl AsTilePos for (usize, usize) {
    fn x(&self) -> usize { self.0 }
    fn y(&self) -> usize { self.1 }
}

impl AsTilePos for (u32, u32) {
    fn x(&self) -> usize { self.0 as usize }
    fn y(&self) -> usize { self.1 as usize }
}

impl WorldData {
    #[inline]
    pub(crate) fn get_block<Pos: AsTilePos>(&self, tile_pos: Pos) -> Option<&Block> {
        self.blocks.get(tile_pos.yx()).and_then(|b| b.as_ref())
    }

    #[inline]
    pub(crate) fn get_solid_block<Pos: AsTilePos>(&self, tile_pos: Pos) -> Option<&Block> {
        self.blocks.get(tile_pos.yx()).and_then(|b| b.as_ref()).filter(|b| b.is_solid())
    }

    #[inline]
    pub(crate) fn get_block_mut<Pos: AsTilePos>(&mut self, tile_pos: Pos) -> Option<&mut Block> {
        self.blocks.get_mut(tile_pos.yx()).and_then(|b| b.as_mut())
    }

    #[inline]
    pub(crate) fn get_wall<Pos: AsTilePos>(&self, tile_pos: Pos) -> Option<&Wall> {
        self.walls.get(tile_pos.yx()).and_then(|w| w.as_ref())
    }

    #[inline]
    pub(crate) fn get_wall_mut<Pos: AsTilePos>(&mut self, tile_pos: Pos) -> Option<&mut Wall> {
        self.walls.get_mut(tile_pos.yx()).and_then(|w| w.as_mut())
    }

    pub(crate) fn set_block<Pos: AsTilePos>(&mut self, tile_pos: Pos, block: &Block) {
        unsafe {
            if let Some(b) = self.blocks.get_mut_ptr(tile_pos.yx()) {
                *b = Some(*block);
            }
        }
    }

    pub(crate) fn set_wall<Pos: AsTilePos>(&mut self, tile_pos: Pos, wall: Wall) {
        unsafe {
            if let Some(w) = self.walls.get_mut_ptr(tile_pos.yx()) {
                *w = Some(wall);
            }
        }
    }

    pub(crate) fn remove_block<Pos: AsTilePos>(&mut self, tile_pos: Pos) {
        unsafe {
            if let Some(block) = self.blocks.get_mut_ptr(tile_pos.yx()) {
                *block = None;
            }
        }
    }

    #[inline]
    pub(crate) fn block_exists<Pos: AsTilePos>(&self, tile_pos: Pos) -> bool {
        self.get_block(tile_pos).is_some()
    }

    #[inline]
    pub(crate) fn wall_exists<Pos: AsTilePos>(&self, tile_pos: Pos) -> bool {
        self.get_wall(tile_pos).is_some()
    }

    #[inline]
    pub(crate) fn solid_block_exists<Pos: AsTilePos>(&self, tile_pos: Pos) -> bool {
        if let Some(block) = self.get_block(tile_pos) {
            return block.is_solid();
        }

        false
    }

    pub(crate) fn get_block_neighbors<Pos: AsTilePos>(&self, tile_pos: Pos, solid: bool) -> Neighbors<&Block> {
        let pos = TilePos::new(tile_pos.x() as u32, tile_pos.y() as u32);
        let map_size = TilemapSize {
            x: self.size.width as u32,
            y: self.size.height as u32,
        };

        let get_block = move |pos: TilePos| -> Option<&Block> {
            if solid {
                self.get_solid_block(pos)
            } else {
                self.get_block(pos)
            }
        };

        Neighbors {
            west: SquarePos::from(&pos).offset(&SquareDirection::West).as_tile_pos(&map_size)
                .and_then(get_block),

            east: SquarePos::from(&pos).offset(&SquareDirection::East).as_tile_pos(&map_size)
                .and_then(get_block),

            north: SquarePos::from(&pos).offset(&SquareDirection::South).as_tile_pos(&map_size)
                .and_then(get_block),

            south: SquarePos::from(&pos).offset(&SquareDirection::North).as_tile_pos(&map_size)
                .and_then(get_block),

            north_west: SquarePos::from(&pos).offset(&SquareDirection::SouthWest).as_tile_pos(&map_size)
                .and_then(get_block),

            south_west: SquarePos::from(&pos).offset(&SquareDirection::NorthWest).as_tile_pos(&map_size)
                .and_then(get_block),

            south_east: SquarePos::from(&pos).offset(&SquareDirection::NorthEast).as_tile_pos(&map_size)
                .and_then(get_block),
            
            north_east: SquarePos::from(&pos).offset(&SquareDirection::SouthEast).as_tile_pos(&map_size)
                .and_then(get_block),
        }
    }

    pub(crate) fn get_wall_neighbors<Pos: AsTilePos>(&self, tile_pos: Pos) -> Neighbors<&Wall> {
        let pos = TilePos::new(tile_pos.x() as u32, tile_pos.y() as u32);
        let map_size = TilemapSize {
            x: self.size.width as u32,
            y: self.size.height as u32
        };

        Neighbors {
            west: SquarePos::from(&pos).offset(&SquareDirection::West).as_tile_pos(&map_size)
                .and_then(|pos| self.get_wall(pos)),

            east: SquarePos::from(&pos).offset(&SquareDirection::East).as_tile_pos(&map_size)
                .and_then(|pos| self.get_wall(pos)),

            north: SquarePos::from(&pos).offset(&SquareDirection::South).as_tile_pos(&map_size)
                .and_then(|pos| self.get_wall(pos)),

            south: SquarePos::from(&pos).offset(&SquareDirection::North).as_tile_pos(&map_size)
                .and_then(|pos| self.get_wall(pos)),

            north_west: SquarePos::from(&pos).offset(&SquareDirection::SouthWest).as_tile_pos(&map_size)
                .and_then(|pos| self.get_wall(pos)),

            south_west: SquarePos::from(&pos).offset(&SquareDirection::NorthWest).as_tile_pos(&map_size)
                .and_then(|pos| self.get_wall(pos)),

            south_east: SquarePos::from(&pos).offset(&SquareDirection::NorthEast).as_tile_pos(&map_size)
                .and_then(|pos| self.get_wall(pos)),
            
            north_east: SquarePos::from(&pos).offset(&SquareDirection::SouthEast).as_tile_pos(&map_size)
                .and_then(|pos| self.get_wall(pos)),
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub(super) struct Point {
    pub(super) x: usize,
    pub(super) y: usize
}

impl Point {
    pub(super) const fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}

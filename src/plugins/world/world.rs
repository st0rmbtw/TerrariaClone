use bevy::prelude::Resource;
use bevy_ecs_tilemap::{tiles::TilePos, helpers::square_grid::neighbors::{Neighbors, SquareDirection}, prelude::{SquarePos, TilemapSize}};

use super::{generator::{BlockArray, WallArray}, block::Block, wall::Wall};

#[derive(Clone, Copy)]
pub struct Level {
    pub from: usize,
    pub to: usize
}

impl Level {
    pub const fn new(from: usize, to: usize) -> Self {
        Self { from, to }
    }
}

#[derive(Clone, Copy)]
pub struct Layer {
    pub sky: Level,
    pub dirt: Level,
    pub stone: Level,
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
    pub fn size(&self) -> Size {
        match self {
            WorldSize::Tiny => Size { width: 1750, height: 900 },
            WorldSize::Medium => Size { width: 6400, height: 1800 },
            WorldSize::Large => Size { width: 8400, height: 2400 }
        }
    }
}

#[derive(Resource)]
pub struct WorldData {
    pub size: Size,
    pub layer: Layer,
    pub spawn_point: TilePos,
    pub blocks: BlockArray,
    pub walls: WallArray,
}

pub trait AsTilePos {
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
    pub fn get_block<Pos: AsTilePos>(&self, tile_pos: Pos) -> Option<&Block> {
        self.blocks.get(tile_pos.yx()).and_then(|b| b.as_ref())
    }

    #[inline]
    pub fn get_block_mut<Pos: AsTilePos>(&mut self, tile_pos: Pos) -> Option<&mut Block> {
        self.blocks.get_mut(tile_pos.yx()).and_then(|b| b.as_mut())
    }

    #[inline]
    pub fn get_wall<Pos: AsTilePos>(&self, tile_pos: Pos) -> Option<&Wall> {
        self.walls.get(tile_pos.yx()).and_then(|w| w.as_ref())
    }

    #[inline]
    pub fn get_wall_mut<Pos: AsTilePos>(&mut self, tile_pos: Pos) -> Option<&mut Wall> {
        self.walls.get_mut(tile_pos.yx()).and_then(|w| w.as_mut())
    }

    #[inline]
    pub fn set_block<Pos: AsTilePos>(&mut self, tile_pos: Pos, block: &Block) {
        if let Some(b) = self.get_block_mut(tile_pos.yx()) {
            *b = *block;
        }
    }

    #[inline]
    pub fn set_wall<Pos: AsTilePos>(&mut self, tile_pos: Pos, wall: Wall) {
        if let Some(w) = self.get_wall_mut(tile_pos.yx()) {
            *w = wall;
        }
    }

    #[inline]
    pub fn remove_block<Pos: AsTilePos>(&mut self, tile_pos: Pos) {
        unsafe {
            if let Some(block) = self.blocks.get_mut_ptr(tile_pos.yx()) {
                *block = None;
            }
        }
    }

    #[inline]
    pub fn block_exists<Pos: AsTilePos>(&self, tile_pos: Pos) -> bool {
        self.get_block(tile_pos.yx()).is_some()
    }

    pub fn get_block_neighbors<Pos: AsTilePos>(&self, tile_pos: Pos) -> Neighbors<&Block> {
        let pos = TilePos::new(tile_pos.x() as u32, tile_pos.y() as u32);
        let map_size = TilemapSize {
            x: self.size.width as u32,
            y: self.size.height as u32,
        };

        Neighbors {
            west: SquarePos::from(&pos).offset(&SquareDirection::West).as_tile_pos(&map_size)
                .map(|pos| self.get_block(pos))
                .flatten()
                .map(|block| block),

            east: SquarePos::from(&pos).offset(&SquareDirection::East).as_tile_pos(&map_size)
                .map(|pos| self.get_block(pos))
                .flatten()
                .map(|block| block),

            north: SquarePos::from(&pos).offset(&SquareDirection::South).as_tile_pos(&map_size)
                .map(|pos| self.get_block(pos))
                .flatten()
                .map(|block| block),

            south: SquarePos::from(&pos).offset(&SquareDirection::North).as_tile_pos(&map_size)
                .map(|pos| self.get_block(pos))
                .flatten()
                .map(|block| block),

            north_west: SquarePos::from(&pos).offset(&SquareDirection::SouthWest).as_tile_pos(&map_size)
                .map(|pos| self.get_block(pos))
                .flatten()
                .map(|block| block),

            south_west: SquarePos::from(&pos).offset(&SquareDirection::NorthWest).as_tile_pos(&map_size)
                .map(|pos| self.get_block(pos))
                .flatten()
                .map(|block| block),

            south_east: SquarePos::from(&pos).offset(&SquareDirection::NorthEast).as_tile_pos(&map_size)
                .map(|pos| self.get_block(pos))
                .flatten()
                .map(|block| block),
            
            north_east: SquarePos::from(&pos).offset(&SquareDirection::SouthEast).as_tile_pos(&map_size)
                .map(|pos| self.get_block(pos))
                .flatten()
                .map(|block| block),   
        }
    }

    pub fn get_wall_neighbors<Pos: AsTilePos>(&self, tile_pos: Pos) -> Neighbors<Wall> {
        let pos = TilePos::new(tile_pos.x() as u32, tile_pos.y() as u32);
        let map_size = TilemapSize {
            x: self.size.width as u32,
            y: self.size.height as u32
        };

        Neighbors {
            west: SquarePos::from(&pos).offset(&SquareDirection::West).as_tile_pos(&map_size)
                .map(|pos| self.get_wall(pos))
                .flatten()
                .map(|wall| *wall),

            east: SquarePos::from(&pos).offset(&SquareDirection::East).as_tile_pos(&map_size)
                .map(|pos| self.get_wall(pos))
                .flatten()
                .map(|wall| *wall),

            north: SquarePos::from(&pos).offset(&SquareDirection::South).as_tile_pos(&map_size)
                .map(|pos| self.get_wall(pos))
                .flatten()
                .map(|wall| *wall),

            south: SquarePos::from(&pos).offset(&SquareDirection::North).as_tile_pos(&map_size)
                .map(|pos| self.get_wall(pos))
                .flatten()
                .map(|wall| *wall),

            north_west: SquarePos::from(&pos).offset(&SquareDirection::SouthWest).as_tile_pos(&map_size)
                .map(|pos| self.get_wall(pos))
                .flatten()
                .map(|wall| *wall),

            south_west: SquarePos::from(&pos).offset(&SquareDirection::NorthWest).as_tile_pos(&map_size)
                .map(|pos| self.get_wall(pos))
                .flatten()
                .map(|wall| *wall),

            south_east: SquarePos::from(&pos).offset(&SquareDirection::NorthEast).as_tile_pos(&map_size)
                .map(|pos| self.get_wall(pos))
                .flatten()
                .map(|wall| *wall),
            
            north_east: SquarePos::from(&pos).offset(&SquareDirection::SouthEast).as_tile_pos(&map_size)
                .map(|pos| self.get_wall(pos))
                .flatten()
                .map(|wall| *wall),
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Point {
    pub x: usize,
    pub y: usize
}

impl Point {
    pub const fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}

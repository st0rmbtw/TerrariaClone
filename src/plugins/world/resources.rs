use bevy::{utils::HashSet, prelude::IVec2};
use bevy_ecs_tilemap::tiles::TilePos;
use ndarray::Array2;

use crate::world_generator::{Cell, Neighbors, Tile};

use super::MAP_SIZE;

pub struct WorldData {
    pub width: u16,
    pub height: u16,
    pub tiles: Array2<Cell>,
}

impl WorldData {
    pub fn get_cell(&self, pos: TilePos) -> Option<&Cell> {
        self.tiles.get((pos.y as usize, pos.x as usize))
    }

    pub fn get_cell_mut(&mut self, pos: TilePos) -> Option<&mut Cell> {
        self.tiles.get_mut((pos.y as usize, pos.x as usize))
    }
    
    pub fn get_tile(&self, pos: TilePos) -> Option<&Tile> {
        self.tiles.get((pos.y as usize, pos.x as usize)).and_then(|cell| cell.tile.as_ref())
    }

    pub fn get_tile_mut(&mut self, pos: TilePos) -> Option<&mut Tile> {
        self.tiles.get_mut((pos.y as usize, pos.x as usize)).and_then(|cell| cell.tile.as_mut())
    }

    pub fn tile_exists(&self, pos: TilePos) -> bool {
        self.tiles.get((pos.y as usize, pos.x as usize)).and_then(|cell| cell.tile).is_some()
    }

    pub fn get_neighbours(&self, pos: TilePos) -> Neighbors {
        Neighbors { 
            left: pos.square_west().and_then(|p| self.get_tile(p)).is_some(),
            right: pos.square_east(&MAP_SIZE).and_then(|p| self.get_tile(p)).is_some(),
            top: pos.square_south().and_then(|p| self.get_tile(p)).is_some(),
            bottom: pos.square_north(&MAP_SIZE).and_then(|p| self.get_tile(p)).is_some()
        }
    }
}

#[derive(Default)]
pub struct ChunkManager {
    pub spawned_chunks: HashSet<IVec2>
}
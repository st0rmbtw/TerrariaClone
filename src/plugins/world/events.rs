use bevy::prelude::Vec2;
use bevy_ecs_tilemap::tiles::TilePos;

use crate::items::Block;

use super::ChunkPos;

pub enum BlockEvent {
    Place { tile_pos: Vec2, block: Block, inventory_item_index: usize },
    Break { tile_pos: Vec2 }
}

pub struct UpdateNeighborsEvent {
    pub tile_pos: TilePos,
    pub chunk_tile_pos: TilePos,
    pub chunk_pos: ChunkPos,
}
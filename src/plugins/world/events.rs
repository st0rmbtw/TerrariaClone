use bevy::prelude::{UVec2, Vec2};
use bevy_ecs_tilemap::tiles::TilePos;

use crate::{block::Block, world_generator::Tile};

use super::ChunkPos;

pub struct BlockBreakEvent {
    pub coords: UVec2,
}

pub struct BlockPlaceEvent {
    pub tile_pos: Vec2,
    pub block: Block,
    pub inventory_item_index: usize
}

pub struct UpdateNeighborsEvent {
    pub tile_pos: TilePos,
    pub chunk_tile_pos: TilePos,
    pub chunk_pos: ChunkPos,
    pub tile: Tile,
}
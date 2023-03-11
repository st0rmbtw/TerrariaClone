use bevy::prelude::Vec2;
use bevy_ecs_tilemap::tiles::TilePos;

use crate::items::Tool;

use super::{ChunkPos, Block};

pub struct BreakBlockEvent {
    pub tile_pos: Vec2
}

pub struct DigBlockEvent {
    pub tile_pos: Vec2,
    pub tool: Tool
}

pub struct PlaceBlockEvent {
    pub tile_pos: Vec2,
    pub block: Block,
    pub inventory_item_index: usize
}

pub struct UpdateNeighborsEvent {
    pub tile_pos: TilePos,
    pub chunk_tile_pos: TilePos,
    pub chunk_pos: ChunkPos,
}
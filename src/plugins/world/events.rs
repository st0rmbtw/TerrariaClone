use bevy_ecs_tilemap::tiles::TilePos;

use crate::items::{Tool, Seed};

use super::{Block, BlockType};

pub(crate) struct BreakBlockEvent {
    pub(crate) tile_pos: TilePos
}

pub(crate) struct DigBlockEvent {
    pub(crate) tile_pos: TilePos,
    pub(crate) tool: Tool
}

pub(crate) struct PlaceBlockEvent {
    pub(crate) tile_pos: TilePos,
    pub(crate) block: Block,
}

pub(super) struct UpdateNeighborsEvent {
    pub(super) tile_pos: TilePos
}

pub(crate) struct UpdateBlockEvent {
    pub(crate) tile_pos: TilePos,
    pub(crate) block_type: BlockType,
    pub(crate) update_neighbors: bool
}

pub(crate) struct SeedEvent {
    pub(crate) tile_pos: TilePos,
    pub(crate) seed: Seed
}
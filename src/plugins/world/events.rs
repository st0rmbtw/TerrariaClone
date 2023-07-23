use bevy::prelude::Event;
use bevy_ecs_tilemap::tiles::TilePos;

use crate::{items::{Tool, Seed}, world::block::{Block, BlockType}};

#[derive(Event)]
pub(crate) struct BreakBlockEvent {
    pub(crate) tile_pos: TilePos
}

#[derive(Event)]
pub(crate) struct DigBlockEvent {
    pub(crate) tile_pos: TilePos,
    pub(crate) tool: Tool
}

#[derive(Event)]
pub(crate) struct PlaceBlockEvent {
    pub(crate) tile_pos: TilePos,
    pub(crate) block: Block,
}

#[derive(Event)]
pub(super) struct UpdateNeighborsEvent {
    pub(super) tile_pos: TilePos
}

#[derive(Event)]
pub(crate) struct UpdateBlockEvent {
    pub(crate) tile_pos: TilePos,
    pub(crate) block_type: BlockType,
    pub(crate) update_neighbors: bool
}

#[derive(Event)]
pub(crate) struct SeedEvent {
    pub(crate) tile_pos: TilePos,
    pub(crate) seed: Seed
}
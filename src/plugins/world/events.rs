use bevy::prelude::Event;
use bevy_ecs_tilemap::tiles::TilePos;

use crate::{items::{ItemTool, ItemSeed}, world::{block::Block, wall::Wall}};

use super::TileType;

#[derive(Event)]
pub(crate) struct BreakTileEvent {
    pub(crate) tile_pos: TilePos,
    pub(crate) tile_type: TileType
}

#[derive(Event)]
pub(crate) struct DigBlockEvent {
    pub(crate) tile_pos: TilePos,
    pub(crate) tool: ItemTool
}

#[derive(Event)]
pub(crate) struct DigWallEvent {
    pub(crate) tile_pos: TilePos,
    pub(crate) tool: ItemTool
}

#[derive(Event)]
pub(crate) struct PlaceTileEvent {
    pub(crate) tile_pos: TilePos,
    pub(crate) tile_type: TileType,
}

#[derive(Event)]
pub(super) struct UpdateNeighborsEvent {
    pub(super) tile_pos: TilePos
}

#[derive(Event)]
pub(crate) struct UpdateBlockEvent {
    pub(crate) tile_pos: TilePos,
    pub(crate) block: Block,
}

#[derive(Event)]
pub(crate) struct UpdateWallEvent {
    pub(crate) tile_pos: TilePos,
    pub(crate) wall: Wall,
}

#[derive(Event)]
pub(crate) struct UpdateCracksEvent {
    pub(crate) tile_pos: TilePos,
    pub(crate) index: u32
}

#[derive(Event)]
pub(crate) struct SeedEvent {
    pub(crate) tile_pos: TilePos,
    pub(crate) seed: ItemSeed
}
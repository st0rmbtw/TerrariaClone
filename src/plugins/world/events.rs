use bevy::prelude::Vec2;
use bevy_ecs_tilemap::tiles::TilePos;

use crate::items::Tool;

use super::Block;

pub(crate) struct BreakBlockEvent {
    pub(crate) tile_pos: Vec2
}

pub(crate) struct DigBlockEvent {
    pub(crate) tile_pos: Vec2,
    pub(crate) tool: Tool
}

pub(crate) struct PlaceBlockEvent {
    pub(crate) tile_pos: Vec2,
    pub(crate) block: Block,
    pub(crate) inventory_item_index: usize
}

pub(super) struct UpdateNeighborsEvent {
    pub(super) tile_pos: TilePos
}
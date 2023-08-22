use bevy::prelude::Event;
use bevy_ecs_tilemap::tiles::TilePos;

#[derive(Event)]
pub(crate) struct UpdateLightEvent {
    pub(crate) tile_pos: TilePos
}

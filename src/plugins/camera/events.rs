use bevy_ecs_tilemap::tiles::TilePos;

pub(crate) struct UpdateLightEvent {
    pub(crate) tile_pos: TilePos
}
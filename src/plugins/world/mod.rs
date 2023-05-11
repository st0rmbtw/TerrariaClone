mod events;
mod resources;
mod systems;
mod utils;

use std::ops::RangeInclusive;

use systems::*;
use utils::*;
pub(crate) use events::*;
pub(crate) use resources::*;

use crate::common::state::GameState;
use bevy::prelude::{Plugin, App, IntoSystemAppConfig, OnEnter, IntoSystemConfigs, OnUpdate};
use bevy_ecs_tilemap::prelude::{TilemapSize, TilemapTileSize};

pub(crate) const TILE_SIZE: f32 = 16.;
pub(crate) const WALL_SIZE: f32 = 32.;
pub(super) const TREE_SIZE: TilemapTileSize = TilemapTileSize { x: 20., y: 20. };
pub(super) const TREE_BRANCHES_SIZE: TilemapTileSize = TilemapTileSize { x: 50., y: 40. };
pub(super) const TREE_TOPS_SIZE: TilemapTileSize = TilemapTileSize { x: 88., y: 148. };

const CHUNK_SIZE: f32 = 25.;
const CHUNK_SIZE_U: u32 = CHUNK_SIZE as u32;

const CHUNKMAP_SIZE: TilemapSize = TilemapSize {
    x: CHUNK_SIZE as u32,
    y: CHUNK_SIZE as u32,
};

pub(crate) struct WorldPlugin;
impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChunkManager>();
        app.add_event::<BreakBlockEvent>();
        app.add_event::<DigBlockEvent>();
        app.add_event::<PlaceBlockEvent>();
        app.add_event::<UpdateNeighborsEvent>();
        app.add_event::<UpdateBlockEvent>();
        app.add_event::<SeedEvent>();

        app.add_system(spawn_terrain.in_schedule(OnEnter(GameState::WorldLoading)));

        app.add_systems(
            (
                spawn_chunks,
                despawn_chunks,
                handle_dig_block_event,
                handle_place_block_event,
                handle_break_block_event,
                handle_update_neighbors_event,
                handle_update_block_event,
                handle_seed_event
            )
            .in_set(OnUpdate(GameState::InGame))
        );
    }
}

#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub(super) struct CameraFov {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub(super) struct ChunkRange {
    pub x: RangeInclusive<u32>,
    pub y: RangeInclusive<u32>
}
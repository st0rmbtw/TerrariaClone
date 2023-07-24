mod events;
mod resources;
mod systems;
mod utils;

use systems::*;
use utils::*;
pub(crate) use events::*;
pub(crate) use resources::*;

use crate::common::state::GameState;
use bevy::{prelude::{Plugin, App, OnEnter, IntoSystemConfigs, in_state, Update, Rect}, math::URect};
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

        app.add_systems(OnEnter(GameState::WorldLoading), spawn_terrain);

        app.add_systems(
            Update,
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
            .run_if(in_state(GameState::InGame))
        );

        #[cfg(feature = "debug")]
        app.add_systems(Update, set_tiles_visibility.run_if(in_state(GameState::InGame)));
    }
}

pub(super) type CameraFov = Rect;
pub(super) type ChunkRange = URect;
mod components;
mod events;
mod resources;
mod systems;
mod utils;

pub use components::*;
pub use events::*;
use iyes_loopless::prelude::{ConditionSet, AppLooplessStateExt};
pub use resources::*;
pub use systems::*;
pub use utils::*;

use crate::{world_generator::{WORLD_SIZE_X, WORLD_SIZE_Y}, state::GameState};
use bevy::prelude::{Plugin, App};
use bevy_ecs_tilemap::prelude::{TilemapSize, TilemapType};

pub const TILE_SIZE: f32 = 16.;
pub const WALL_SIZE: f32 = 32.;

const CHUNK_SIZE: f32 = 25.;
const CHUNK_SIZE_U: u32 = CHUNK_SIZE as u32;

const CHUNKMAP_SIZE: TilemapSize = TilemapSize {
    x: CHUNK_SIZE as u32,
    y: CHUNK_SIZE as u32,
};

pub const MAP_SIZE: TilemapSize = TilemapSize {
    x: WORLD_SIZE_X as u32,
    y: WORLD_SIZE_Y as u32
};

pub const MAP_TYPE: TilemapType = TilemapType::Square { diagonal_neighbors: false };

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<ChunkManager>()
            .add_event::<BlockPlaceEvent>()
            .add_event::<UpdateNeighborsEvent>()
            .add_enter_system(GameState::WorldLoading, spawn_terrain)
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::InGame)
                    .with_system(spawn_chunks)
                    .with_system(despawn_chunks)
                    .with_system(handle_block_place)
                    .with_system(update_neighbors)
                    .into(),
            );
        
        #[cfg(feature = "debug_grid")]
        app.add_system_set(
            ConditionSet::new()
                .run_in_state(GameState::InGame)
                .with_system(spawn_tile_grid)
                .with_system(spawn_pixel_grid)
                .into()
        );
    }
}
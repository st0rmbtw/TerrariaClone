mod components;
mod events;
mod resources;
mod systems;
mod utils;
pub mod generator;
mod world;
mod block;
mod wall;
mod tree;
mod light;

pub use components::*;
pub use events::*;
use iyes_loopless::prelude::{ConditionSet, AppLooplessStateExt};
pub use resources::*;
pub use systems::*;
pub use utils::*;
pub use world::*;
pub use block::*;
pub use wall::*;
pub use tree::*;
pub use light::*;

use crate::{state::GameState};
use bevy::prelude::{Plugin, App};
use bevy_ecs_tilemap::prelude::TilemapSize;

pub const TILE_SIZE: f32 = 16.;
pub const WALL_SIZE: f32 = 32.;

const CHUNK_SIZE: f32 = 25.;
const CHUNK_SIZE_U: u32 = CHUNK_SIZE as u32;

const CHUNKMAP_SIZE: TilemapSize = TilemapSize {
    x: CHUNK_SIZE as u32,
    y: CHUNK_SIZE as u32,
};

pub const DIRT_HILL_HEIGHT: f32 = 50.;
pub const STONE_HILL_HEIGHT: f32 = 15.;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<ChunkManager>()
            .add_event::<BlockEvent>()
            .add_event::<UpdateNeighborsEvent>()
            .add_enter_system(GameState::WorldLoading, spawn_terrain)
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::InGame)
                    .with_system(spawn_chunks)
                    .with_system(despawn_chunks)
                    .with_system(handle_block_event)
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

#[derive(Debug, Clone, Copy, Default)]
pub struct Frame {
    pub x: u16,
    pub y: u16
}

impl Frame {
    pub const fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }
}
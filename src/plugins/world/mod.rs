mod chunk;
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

pub use chunk::*;
pub use events::*;
pub use resources::*;
use systems::*;
use utils::*;
pub use world::*;
pub use block::*;
pub use wall::*;
pub use tree::*;

use crate::common::state::GameState;
use bevy::prelude::{Plugin, App, IntoSystemAppConfig, OnEnter, IntoSystemConfigs, OnUpdate};
use bevy_ecs_tilemap::prelude::{TilemapSize, TilemapTileSize};

pub const TILE_SIZE: f32 = 16.;
pub const WALL_SIZE: f32 = 32.;
pub const TREE_SIZE: TilemapTileSize = TilemapTileSize { x: 20., y: 20. };
pub const TREE_BRANCHES_SIZE: TilemapTileSize = TilemapTileSize { x: 50., y: 40. };
pub const TREE_TOPS_SIZE: TilemapTileSize = TilemapTileSize { x: 88., y: 148. };

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
        app.init_resource::<ChunkManager>();
        app.add_event::<BreakBlockEvent>();
        app.add_event::<DigBlockEvent>();
        app.add_event::<PlaceBlockEvent>();
        app.add_event::<UpdateNeighborsEvent>();

        app.add_system(spawn_terrain.in_schedule(OnEnter(GameState::WorldLoading)));

        app.add_systems(
            (
                spawn_chunks,
                despawn_chunks,
                handle_dig_block_event,
                handle_place_block_event,
                handle_break_block_event,
                update_neighbors,
            ).chain().in_set(OnUpdate(GameState::InGame))
        );
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct TerrariaFrame {
    pub x: u16,
    pub y: u16
}

impl TerrariaFrame {
    pub const fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }
}
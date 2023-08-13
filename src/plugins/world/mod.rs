pub(crate) mod events;
pub(crate) mod resources;
pub(crate) mod constants;
mod systems;
mod utils;

use crate::{common::state::GameState, lighting::compositing::TileMaterial};
use bevy::{prelude::{Plugin, App, OnEnter, IntoSystemConfigs, in_state, Update, Rect}, math::URect};
use bevy_ecs_tilemap::{prelude::MaterialTilemapPlugin, TilemapPlugin};

pub(crate) struct WorldPlugin;
impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((TilemapPlugin, MaterialTilemapPlugin::<TileMaterial>::default()));

        app.init_resource::<resources::ChunkManager>();
        app.add_event::<events::BreakBlockEvent>();
        app.add_event::<events::DigBlockEvent>();
        app.add_event::<events::PlaceBlockEvent>();
        app.add_event::<events::UpdateNeighborsEvent>();
        app.add_event::<events::UpdateBlockEvent>();
        app.add_event::<events::SeedEvent>();

        app.add_systems(OnEnter(GameState::WorldLoading), systems::spawn_terrain);

        app.add_systems(
            Update,
            (
                systems::spawn_chunks,
                systems::despawn_chunks,
                systems::handle_dig_block_event,
                systems::handle_place_block_event,
                systems::handle_break_block_event,
                systems::handle_update_neighbors_event,
                systems::handle_update_block_event,
                systems::handle_seed_event
            )
            .run_if(in_state(GameState::InGame))
        );

        #[cfg(feature = "debug")]
        app.add_systems(Update, systems::set_tiles_visibility.run_if(in_state(GameState::InGame)));
    }
}

pub(super) type CameraFov = Rect;
pub(super) type ChunkRange = URect;
pub(crate) mod events;
pub(crate) mod resources;
pub(crate) mod constants;
mod systems;
mod utils;

use crate::common::state::GameState;
use bevy::{prelude::{Plugin, App, OnEnter, IntoSystemConfigs, Update, Rect, OnExit}, math::URect, render::view::RenderLayers};
use bevy_ecs_tilemap::TilemapPlugin;

use super::InGameSystemSet;

pub(crate) const WORLD_RENDER_LAYER: RenderLayers = RenderLayers::layer(26);

pub(super) type CameraFov = Rect;
pub(super) type ChunkRange = URect;

pub(crate) struct WorldPlugin;
impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TilemapPlugin);

        app.add_event::<events::BreakBlockEvent>();
        app.add_event::<events::DigBlockEvent>();
        app.add_event::<events::PlaceBlockEvent>();
        app.add_event::<events::UpdateNeighborsEvent>();
        app.add_event::<events::UpdateBlockEvent>();
        app.add_event::<events::SeedEvent>();

        app.add_systems(OnEnter(GameState::WorldLoading), (systems::setup, systems::spawn_terrain).chain());
        app.add_systems(OnExit(GameState::InGame), systems::cleanup);

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
                systems::handle_seed_event,
            )
            .in_set(InGameSystemSet::Update)
        );

        #[cfg(feature = "debug")]
        app.add_systems(Update, systems::set_tiles_visibility.in_set(InGameSystemSet::Update));
    }
}
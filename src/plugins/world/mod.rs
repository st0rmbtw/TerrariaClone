pub(crate) mod events;
pub(crate) mod resources;
pub(crate) mod constants;
pub(crate) mod time;
mod utils;
mod systems;

use crate::{common::state::GameState, world::{block::BlockType, wall::WallType}};
use bevy::{prelude::{Plugin, App, OnEnter, IntoSystemConfigs, Update, Rect, OnExit, Resource, UVec2, Deref}, math::URect, render::view::RenderLayers};
use bevy_ecs_tilemap::TilemapPlugin;

use self::time::WorldTimePlugin;

use super::{InGameSystemSet, particles::ParticlePlugin, item::ItemPlugin};

pub(crate) const WORLD_RENDER_LAYER: RenderLayers = RenderLayers::layer(26);

pub(super) type CameraFov = Rect;
pub(super) type ChunkRange = URect;

#[derive(Resource, Deref, Clone, Copy)]
pub(crate) struct WorldSize(UVec2);

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum TileType {
    Block(Option<BlockType>),
    Wall(Option<WallType>)
}

pub(crate) struct WorldPlugin;
impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((TilemapPlugin, ParticlePlugin, ItemPlugin, WorldTimePlugin));

        app.add_event::<events::BreakTileEvent>();

        app.add_event::<events::TileRemovedEvent>();

        app.add_event::<events::DigBlockEvent>();
        app.add_event::<events::DigWallEvent>();

        app.add_event::<events::PlaceTileEvent>();

        app.add_event::<events::UpdateNeighborsEvent>();

        app.add_event::<events::UpdateBlockEvent>();
        app.add_event::<events::UpdateWallEvent>();
        app.add_event::<events::UpdateCracksEvent>();

        app.add_event::<events::SeedEvent>();

        app.add_systems(OnEnter(GameState::WorldLoading), (systems::setup, systems::spawn_terrain));
        app.add_systems(OnExit(GameState::InGame), systems::cleanup);

        app.add_systems(
            Update,
            (
                systems::spawn_chunks,
                systems::despawn_chunks,

                systems::handle_dig_block_event,
                systems::handle_dig_wall_event,

                systems::handle_place_tile_event,

                systems::handle_break_tile_event,

                systems::handle_update_neighbors_event,
                
                systems::handle_update_block_event,
                systems::handle_update_wall_event,

                systems::handle_seed_event,
                systems::handle_update_cracks_event,
            )
            .in_set(InGameSystemSet::Update)
        );

        #[cfg(feature = "debug")]
        app.add_systems(Update, systems::set_tiles_visibility.in_set(InGameSystemSet::Update));
    }
}
pub(crate) mod components;
pub(crate) mod resources;
mod systems;

use bevy::{prelude::{Plugin, App, OnExit, OnEnter, IntoSystemConfigs, not, resource_equals, in_state, Condition, resource_changed, Update}, ui::BackgroundColor};
use crate::{common::state::GameState, animation::{AnimationSystemSet, component_animator_system}};
use super::{ui::UiVisibility, settings::ShowTileGrid};

const CURSOR_SIZE: f32 = 22.;
const MAX_TILE_GRID_OPACITY: f32 = 0.8;
const MIN_TILE_GRID_OPACITY: f32 = 0.2;

pub struct CursorPlugin;
impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(resources::CursorPosition::default());

        app.add_systems(OnExit(GameState::AssetLoading), systems::setup);
        app.add_systems(OnEnter(GameState::InGame), systems::spawn_tile_grid);

        app.add_systems(
            Update,
            (
                systems::update_cursor_position,
                systems::update_cursor_info
            )
            .run_if(not(in_state(GameState::AssetLoading)))
        );

        app.add_systems(
            Update,
            systems::update_tile_grid_position
                .run_if(in_state(GameState::InGame))
                .run_if(resource_equals(UiVisibility::VISIBLE).and_then(resource_equals(ShowTileGrid(true))))
        );

        app.add_systems(
            Update,
            component_animator_system::<BackgroundColor>
                .run_if(not(in_state(GameState::AssetLoading)))
                .run_if(resource_equals(UiVisibility::VISIBLE))
                .in_set(AnimationSystemSet::AnimationUpdate),
        );

        app.add_systems(
            Update,
            systems::set_visibility::<components::CursorBackground>.run_if(in_state(GameState::InGame))
        );

        app.add_systems(
            Update,
            (
                systems::set_visibility::<components::TileGrid>,
                systems::update_tile_grid_visibility,
            )
            .run_if(in_state(GameState::InGame))
            .run_if(resource_changed::<ShowTileGrid>())
        );

        #[cfg(not(feature = "debug"))]
        app.add_systems(
            Update,
            systems::update_tile_grid_opacity
                .run_if(in_state(GameState::InGame))
                .run_if(resource_equals(ShowTileGrid(true)))
        );

        #[cfg(feature = "debug")] {
            use crate::plugins::debug::DebugConfiguration;
            use bevy::prelude::Res;
            app.add_systems(
                Update,
                systems::update_tile_grid_opacity
                    .run_if(in_state(GameState::InGame))
                    .run_if(resource_equals(ShowTileGrid(true)))
                    .run_if(|config: Res<DebugConfiguration>| !config.free_camera)
            );
        }
    }
}

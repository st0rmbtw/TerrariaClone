mod components;
mod resources;
mod systems;

use systems::*;
pub(crate) use components::*;
pub(crate) use resources::*;

use bevy::{prelude::{Plugin, App, OnExit, OnEnter, IntoSystemConfigs, not, resource_equals, in_state, Condition, resource_changed, Update}, ui::BackgroundColor};
use crate::{common::state::GameState, animation::{AnimationSystemSet, component_animator_system}};
use super::{ui::UiVisibility, settings::ShowTileGrid};

const CURSOR_SIZE: f32 = 22.;
const MAX_TILE_GRID_OPACITY: f32 = 0.8;
const MIN_TILE_GRID_OPACITY: f32 = 0.2;

pub struct CursorPlugin;
impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CursorPosition::default());

        app.add_systems(OnExit(GameState::AssetLoading), setup);
        app.add_systems(OnEnter(GameState::InGame), spawn_tile_grid);

        app.add_systems(
            Update,
            (
                update_cursor_position,
                update_cursor_info
            )
            .distributive_run_if(not(in_state(GameState::AssetLoading)))
        );

        app.add_systems(
            Update,
            update_tile_grid_position
                .run_if(in_state(GameState::InGame))
                .run_if(resource_equals(UiVisibility(true)).and_then(resource_equals(ShowTileGrid(true))))
        );

        app.add_systems(
            Update,
            component_animator_system::<BackgroundColor>
                .run_if(not(in_state(GameState::AssetLoading)))
                .run_if(resource_equals(UiVisibility(true)))
                .in_set(AnimationSystemSet::AnimationUpdate),
        );

        app.add_systems(
            Update,
            set_visibility::<CursorBackground>
                .run_if(in_state(GameState::InGame))
        );

        app.add_systems(
            Update,
            (
                set_visibility::<TileGrid>,
                update_tile_grid_visibility,
            )
            .distributive_run_if(resource_changed::<ShowTileGrid>())
            .run_if(in_state(GameState::InGame))
        );

        #[cfg(not(feature = "debug"))]
        app.add_systems(
            Update,
            update_tile_grid_opacity
                .run_if(resource_equals(ShowTileGrid(true)))
                .run_if(in_state(GameState::InGame))
        );

        #[cfg(feature = "debug")] {
            use crate::plugins::debug::DebugConfiguration;
            use bevy::prelude::Res;
            app.add_systems(
                Update,
                update_tile_grid_opacity
                    .run_if(in_state(GameState::InGame))
                    .run_if(resource_equals(ShowTileGrid(true)))
                    .run_if(|config: Res<DebugConfiguration>| !config.free_camera)
            );
        }
    }
}

mod components;
mod resources;
mod systems;
mod events;

use systems::*;
pub(crate) use components::*;
pub(crate) use resources::*;
pub(crate) use events::*;

use bevy::{prelude::{Plugin, App, IntoSystemConfig, OnExit, OnEnter, IntoSystemConfigs, not, IntoSystemAppConfig, resource_equals, OnUpdate, in_state, Res, State, Condition, resource_changed}, ui::BackgroundColor};
use crate::{common::state::GameState, animation::{AnimationSystemSet, component_animator_system}};
use super::{ui::UiVisibility, settings::ShowTileGrid};

const CURSOR_SIZE: f32 = 22.;
const MAX_TILE_GRID_OPACITY: f32 = 0.8;
const MIN_TILE_GRID_OPACITY: f32 = 0.2;

pub struct CursorPlugin;
impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CursorPosition::default());

        app.add_event::<UpdateHoverableInfoEvent>();

        app.add_system(setup.in_schedule(OnExit(GameState::AssetLoading)));
        app.add_system(spawn_tile_grid.in_schedule(OnEnter(GameState::InGame)));

        app.add_systems(
            (
                update_cursor_position,
                handle_update_hoverable_info_event
            )
            .distributive_run_if(|current_state: Res<State<GameState>>| current_state.0 != GameState::AssetLoading)
        );

        app.add_system(
            update_tile_grid_position
                .in_set(OnUpdate(GameState::InGame))
                .run_if(resource_equals(UiVisibility(true)).and_then(resource_equals(ShowTileGrid(true))))
        );

        app.add_system(
            component_animator_system::<BackgroundColor>
                .run_if(not(in_state(GameState::AssetLoading)))
                .run_if(resource_equals(UiVisibility(true)))
                .in_set(AnimationSystemSet::AnimationUpdate),
        );

        app.add_system(
            set_visibility::<CursorBackground>
                .in_set(OnUpdate(GameState::InGame))
        );

        app.add_systems(
            (
                set_visibility::<TileGrid>,
                update_tile_grid_visibility,
            )
            .distributive_run_if(resource_changed::<ShowTileGrid>())
            .in_set(OnUpdate(GameState::InGame))
        );

        #[cfg(not(feature = "debug"))]
        app.add_system(
            update_tile_grid_opacity
                .run_if(resource_equals(ShowTileGrid(true)))
                .in_set(OnUpdate(GameState::InGame))
        );

        #[cfg(feature = "debug")] {
            use crate::plugins::debug::DebugConfiguration;
            app.add_system(
                update_tile_grid_opacity
                    .run_if(resource_equals(ShowTileGrid(true)))
                    .run_if(|config: Res<DebugConfiguration>| !config.free_camera)
                    .in_set(OnUpdate(GameState::InGame))
            );
        }
    }
}

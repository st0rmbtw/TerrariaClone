mod components;
mod resources;
mod systems;

pub use components::*;
pub use resources::*;
pub use systems::*;

use bevy::{prelude::{Plugin, App, IntoSystemConfig, OnExit, OnEnter, IntoSystemConfigs, not, IntoSystemAppConfig, resource_equals, OnUpdate, in_state, Res, State, Condition}, ui::BackgroundColor};
use crate::{common::state::GameState, animation::{AnimationSystemSet, component_animator_system}, DebugConfiguration};
use super::{ui::UiVisibility, settings::ShowTileGrid};

const MAX_TILE_GRID_OPACITY: f32 = 0.8;
const MIN_TILE_GRID_OPACITY: f32 = 0.2;

pub struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(HoveredInfo::default());
        app.insert_resource(CursorPosition::default());

        app.add_system(setup.in_schedule(OnExit(GameState::AssetLoading)));
        app.add_system(spawn_tile_grid.in_schedule(OnEnter(GameState::InGame)));

        app.add_systems(
            (
                update_cursor_position,
                update_hovered_info,
            )
            .chain()
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

        app.add_systems(
            (
                set_visibility::<TileGrid>,
                set_visibility::<CursorBackground>,
                update_tile_grid_visibility,
                update_tile_grid_opacity.run_if(|config: Res<DebugConfiguration>| !config.free_camera)
            )
            .chain()
            .in_set(OnUpdate(GameState::InGame))
        );
    }
}

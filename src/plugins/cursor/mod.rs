pub(crate) mod components;
pub(crate) mod resources;
mod systems;

use bevy::{prelude::{Plugin, App, OnExit, OnEnter, IntoSystemConfigs, not, resource_equals, in_state, Condition, Update, PreUpdate}, ui::BackgroundColor};
use crate::{common::{state::GameState, systems::set_visibility}, animation::{AnimationSystemSet, component_animator_system}};
use super::{ui::UiVisibility, settings::ShowTileGrid};

const CURSOR_SIZE: f32 = 22.;
const MAX_TILE_GRID_OPACITY: f32 = 0.8;
const MIN_TILE_GRID_OPACITY: f32 = 0.2;

#[cfg(feature = "debug")]
use crate::plugins::debug::DebugConfiguration;
#[cfg(feature = "debug")]
use bevy::prelude::Res;

pub struct CursorPlugin;
impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(resources::CursorPosition::default());

        app.add_systems(OnExit(GameState::AssetLoading), systems::setup);
        app.add_systems(OnEnter(GameState::InGame), systems::spawn_tile_grid);

        app.add_systems(
            PreUpdate,
            systems::update_cursor_position
                .run_if(not(in_state(GameState::AssetLoading)))
        );

        app.add_systems(
            Update,
            systems::update_cursor_info
                .run_if(not(in_state(GameState::AssetLoading)))
        );

        app.add_systems(
            Update,
            systems::update_tile_grid_position
                .run_if(in_state(GameState::InGame))
                .run_if(resource_equals(UiVisibility::VISIBLE).and_then(
                        resource_equals(ShowTileGrid(true))))
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
            set_visibility::<components::CursorBackground, UiVisibility>.run_if(in_state(GameState::InGame))
        );

        app.add_systems(
            Update,
            (
                set_visibility::<components::TileGrid, UiVisibility>,
                set_visibility::<components::TileGrid, ShowTileGrid>,
            )
            .run_if(in_state(GameState::InGame))
        );

        let update_tile_grid_opacity = systems::update_tile_grid_opacity
            .run_if(in_state(GameState::InGame))
            .run_if(resource_equals(ShowTileGrid(true)));

        #[cfg(feature = "debug")]
        let update_tile_grid_opacity = update_tile_grid_opacity.run_if(|config: Res<DebugConfiguration>| !config.free_camera);
        
        app.add_systems(Update, update_tile_grid_opacity);
    }
}

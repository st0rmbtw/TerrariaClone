mod components;
mod resources;
mod systems;

pub use components::*;
pub use resources::*;
pub use systems::*;

use iyes_loopless::prelude::*;
use bevy::{prelude::{Plugin, App}, ui::BackgroundColor};
use crate::{state::GameState, animation::{AnimationSystem, component_animator_system}};
use super::{ui::UiVisibility, settings::ShowTileGrid};

const MAX_TILE_GRID_OPACITY: f32 = 0.8;
const MIN_TILE_GRID_OPACITY: f32 = 0.2;

pub struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(HoveredInfo::default())
            .insert_resource(CursorPosition::default())
            .add_exit_system(GameState::AssetLoading, setup)
            .add_enter_system(GameState::InGame, spawn_tile_grid)
            .add_system_set(
                ConditionSet::new()
                    .run_not_in_state(GameState::AssetLoading)
                    .run_if_resource_equals(UiVisibility(true))
                    .with_system(update_cursor_position)
                    .with_system(update_hovered_info)
                    .into(),
            )
            .add_system(
                update_tile_grid_position
                    .run_in_state(GameState::InGame)
                    .run_if_resource_equals(UiVisibility(true))
                    .run_if_resource_equals(ShowTileGrid(true))
            )
            .add_system(
                component_animator_system::<BackgroundColor>
                    .run_not_in_state(GameState::AssetLoading)
                    .label(AnimationSystem::AnimationUpdate),
            );

        let mut set = ConditionSet::new()
            .run_in_state(GameState::InGame)
            .with_system(set_visibility::<TileGrid>)
            .with_system(set_visibility::<CursorBackground>)
            .with_system(update_tile_grid_visibility);

        #[cfg(not(feature = "free_camera"))]
        set.add_system(update_tile_grid_opacity);

        app.add_system_set(set.into());
    }
}

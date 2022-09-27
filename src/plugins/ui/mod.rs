mod components;
mod resources;
mod systems;

pub use components::*;
pub use resources::*;
pub use systems::*;

use bevy::prelude::{Plugin, App};
use iyes_loopless::prelude::{AppLooplessStateExt, ConditionSet};
use crate::state::GameState;

pub const SPAWN_UI_CONTAINER_LABEL: &str = "spawn_ui_container";

pub struct PlayerUiPlugin;

impl Plugin for PlayerUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ToggleExtraUiEvent>()
            .init_resource::<ExtraUiVisibility>()
            .init_resource::<UiVisibility>()
            .add_enter_system(GameState::InGame, spawn_ui_container)
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::InGame)
                    .with_system(toggle_extra_ui)
                    .with_system(toggle_ui)
                    .with_system(set_main_container_visibility)
                    .into(),
            );
    }
}
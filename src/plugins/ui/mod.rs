mod components;
mod resources;
mod systems;
mod events;

pub use components::*;
pub use resources::*;
pub use systems::*;
pub use events::*;

use bevy::{prelude::{Plugin, App, IntoSystemAppConfig, OnEnter, OnUpdate, IntoSystemConfig, KeyCode}, input::common_conditions::input_just_pressed};
use crate::common::state::GameState;

pub const SPAWN_UI_CONTAINER_LABEL: &str = "spawn_ui_container";

pub struct PlayerUiPlugin;

impl Plugin for PlayerUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ToggleExtraUiEvent>();
        app.init_resource::<ExtraUiVisibility>();
        app.init_resource::<UiVisibility>();
        app.add_system(spawn_ui_container.in_schedule(OnEnter(GameState::InGame)));

        app.add_system(
            toggle_extra_ui
                .run_if(input_just_pressed(KeyCode::Escape))
                .in_set(OnUpdate(GameState::InGame))
        );
        app.add_system(
            toggle_ui
                .run_if(input_just_pressed(KeyCode::F11))
                .in_set(OnUpdate(GameState::InGame))
        );
        app.add_system(set_main_container_visibility.in_set(OnUpdate(GameState::InGame)));
    }
}
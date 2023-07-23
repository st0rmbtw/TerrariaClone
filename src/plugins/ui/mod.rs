mod components;
mod resources;
mod systems;
mod events;

use components::*;
pub(crate) use events::*;
pub(crate) use resources::*;
pub(crate) use systems::*;

use bevy::{prelude::{Plugin, App, OnEnter, KeyCode, Update, in_state, IntoSystemConfigs}, input::common_conditions::input_just_pressed};
use crate::common::state::GameState;

pub(crate) struct PlayerUiPlugin;
impl Plugin for PlayerUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ToggleExtraUiEvent>();
        app.init_resource::<ExtraUiVisibility>();
        app.init_resource::<UiVisibility>();

        app.add_systems(OnEnter(GameState::InGame), spawn_ui_container);
        app.add_systems(Update,
            (
                toggle_extra_ui_visibility.run_if(input_just_pressed(KeyCode::Escape)),
                toggle_ui_visibility.run_if(input_just_pressed(KeyCode::F11)),
                set_main_container_visibility
            )
            .run_if(in_state(GameState::InGame))
        );
    }
}
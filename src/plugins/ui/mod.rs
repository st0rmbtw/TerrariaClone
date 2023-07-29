mod components;
mod resources;
mod systems;
mod events;

pub(crate) use events::*;
pub(crate) use resources::*;

use bevy::{prelude::{Plugin, App, KeyCode, Update, in_state, IntoSystemConfigs, resource_changed, OnExit}, input::common_conditions::input_just_pressed};
use crate::common::state::GameState;

pub(crate) struct PlayerUiPlugin;
impl Plugin for PlayerUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ToggleExtraUiEvent>();
        app.init_resource::<ExtraUiVisibility>();
        app.init_resource::<UiVisibility>();

        app.add_systems(OnExit(GameState::WorldLoading), systems::spawn_ui_container);
        app.add_systems(Update,
            (
                systems::toggle_extra_ui_visibility.run_if(input_just_pressed(KeyCode::Escape)),
                systems::toggle_ui_visibility.run_if(input_just_pressed(KeyCode::F11)),
                systems::set_main_container_visibility.run_if(resource_changed::<UiVisibility>())
            )
            .run_if(in_state(GameState::InGame))
        );
    }
}
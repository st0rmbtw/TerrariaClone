mod components;
mod resources;
mod systems;

pub(crate) use resources::*;

use bevy::{prelude::{Plugin, App, KeyCode, Update, in_state, IntoSystemConfigs, OnExit}, input::common_conditions::input_just_pressed};
use crate::common::{state::GameState, systems::set_visibility};

use self::components::MainUiContainer;

pub(crate) struct PlayerUiPlugin;
impl Plugin for PlayerUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ExtraUiVisibility>();
        app.init_resource::<UiVisibility>();

        app.add_systems(OnExit(GameState::WorldLoading), systems::spawn_ui_container);
        app.add_systems(Update,
            (
                systems::toggle_extra_ui_visibility.run_if(input_just_pressed(KeyCode::Escape)),
                systems::toggle_ui_visibility.run_if(input_just_pressed(KeyCode::F11)),
                set_visibility::<MainUiContainer, UiVisibility>
            )
            .run_if(in_state(GameState::InGame))
        );
    }
}
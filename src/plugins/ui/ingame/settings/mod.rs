pub(crate) mod systems;
mod components;

use bevy::{prelude::{Plugin, App, Update, IntoSystemConfigs, OnEnter, KeyCode, Condition, Commands, OnExit, resource_exists_and_equals}, input::common_conditions::input_just_pressed};

use crate::{common::{systems::{set_visibility, toggle_resource, set_state, set_display}, state::GameState, conditions::on_click}, plugins::{InGameSystemSet, ui::{InventoryUiVisibility, SettingsMenuVisibility, systems::play_sound_on_toggle}}};

use self::components::{InGameSettingsButton, SaveAndExitButton, CloseMenuButton};

pub(crate) struct InGameSettingsUiPlugin;
impl Plugin for InGameSettingsUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::InGame), (setup, systems::spawn_settings_menu));
        app.add_systems(OnExit(GameState::InGame), cleanup);

        app.add_systems(
            Update,
            (
                set_visibility::<components::InGameSettingsButtonContainer, InventoryUiVisibility>,
                set_display::<components::InGameSettingsMenuContainer, SettingsMenuVisibility>,
                toggle_resource::<SettingsMenuVisibility>.run_if(on_click::<InGameSettingsButton>),
                (
                    toggle_resource::<SettingsMenuVisibility>,
                    play_sound_on_toggle::<SettingsMenuVisibility>
                )
                .chain()
                .run_if(
                    resource_exists_and_equals(SettingsMenuVisibility(true)).and_then(input_just_pressed(KeyCode::Escape))
                ),
            )
            .in_set(InGameSystemSet::Update)
        );

        app.add_systems(
            Update,
            (
                (
                    toggle_resource::<SettingsMenuVisibility>,
                    play_sound_on_toggle::<SettingsMenuVisibility>
                ).run_if(on_click::<CloseMenuButton>),
                set_state(GameState::Menu).run_if(on_click::<SaveAndExitButton>),
            )
            .in_set(InGameSystemSet::Update)
        );
    }
}

fn setup(mut commands: Commands) {
    commands.init_resource::<SettingsMenuVisibility>();
}

fn cleanup(mut commands: Commands) {
    commands.remove_resource::<SettingsMenuVisibility>();
}
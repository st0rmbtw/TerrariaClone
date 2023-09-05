pub(crate) mod systems;
mod components;
mod menus;

use bevy::{prelude::{Plugin, App, Update, IntoSystemConfigs, OnEnter, KeyCode, Condition, Commands, OnExit, resource_exists_and_equals, not, Component, Resource}, input::common_conditions::input_just_pressed};

use crate::{common::{systems::{set_visibility, toggle_resource, set_state, set_display, despawn_with, set_resource}, state::GameState, conditions::on_click}, plugins::{InGameSystemSet, ui::{InventoryUiVisibility, SettingsMenuVisibility, systems::play_sound_on_toggle}}};

use self::{components::{SettingsButton, buttons::SaveAndExitButton, buttons::{CloseMenuButton, GeneralButton}, TabMenu}, systems::{spawn_general_menu, update_tab_buttons}};

#[derive(Component, Resource, Clone, Copy, Default, PartialEq)]
enum SelectedTab {
    #[default]
    General,
    Interface,
    Video,
    Cursor
}

const TAB_BUTTON_TEXT_SIZE: f32 = 36.;

pub(crate) struct InGameSettingsUiPlugin;
impl Plugin for InGameSettingsUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::InGame), (setup, systems::spawn_settings_menu));
        app.add_systems(OnExit(GameState::InGame), cleanup);

        app.add_systems(
            Update,
            (
                systems::animate_button_scale,
                set_visibility::<components::SettingsButtonContainer, InventoryUiVisibility>,
                set_display::<components::MenuContainer, SettingsMenuVisibility>,
                (
                    toggle_resource::<SettingsMenuVisibility>,
                    spawn_general_menu
                ).run_if(on_click::<SettingsButton>),

                (
                    (
                        toggle_resource::<SettingsMenuVisibility>,
                        play_sound_on_toggle::<SettingsMenuVisibility>
                    )
                    .chain()
                    .run_if(input_just_pressed(KeyCode::Escape)),

                    update_tab_buttons
                )
                .run_if(resource_exists_and_equals(SettingsMenuVisibility(true)))
            )
            .in_set(InGameSystemSet::Update)
        );

        app.add_systems(
            Update,
            (
                (
                    set_resource(SelectedTab::General),
                    despawn_with::<TabMenu>,
                    systems::spawn_general_menu
                )
                .chain()
                .run_if(not(resource_exists_and_equals(SelectedTab::General)).and_then(on_click::<GeneralButton>)),

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
    commands.init_resource::<SelectedTab>();
}

fn cleanup(mut commands: Commands) {
    commands.remove_resource::<SettingsMenuVisibility>();
    commands.remove_resource::<SelectedTab>();
}
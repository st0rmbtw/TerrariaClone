pub(crate) mod systems;
mod components;
mod menus;

use bevy::{prelude::{Plugin, App, Update, IntoSystemConfigs, OnEnter, KeyCode, Condition, Commands, OnExit, resource_exists_and_equals, not, Component, Resource, apply_deferred, Color, resource_exists_and_changed}, input::common_conditions::input_just_pressed};

use crate::{common::{systems::{bind_visibility_to, set_state, set_display, despawn_with, set_resource, animate_button_color, toggle_resource, play_sound}, state::GameState, conditions::{on_click, is_visible}}, plugins::{InGameSystemSet, ui::{systems::{play_sound_on_toggle, update_toggle_tile_grid_button_text, play_sound_on_hover}, menu::MENU_BUTTON_COLOR, components::ToggleTileGridButton, resources::IsVisible, InventoryUi, SettingsMenu}, config::ShowTileGrid, audio::SoundType}};

use self::{components::{SettingsButton, buttons::SaveAndExitButton, buttons::{CloseMenuButton, GeneralButton, InterfaceButton}, TabMenu, TabButton, TabMenuButton}, systems::{spawn_general_menu, update_tab_buttons, bind_zoom_slider_to_output, update_zoom}};

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
                play_sound(SoundType::MenuOpen).run_if(resource_exists_and_changed::<SelectedTab>()),
                systems::animate_button_scale,
                bind_visibility_to::<InventoryUi, components::SettingsButtonContainer>,
                set_display::<components::MenuContainer, IsVisible<SettingsMenu>>,

                (
                    spawn_general_menu,
                    (
                        toggle_resource::<IsVisible<SettingsMenu>>,
                        play_sound_on_toggle::<IsVisible<SettingsMenu>>
                    )
                    .chain()
                )
                .run_if(on_click::<SettingsButton>),

                (
                    (
                        set_resource(IsVisible::<SettingsMenu>::hidden()),
                        play_sound(SoundType::MenuClose),
                    )
                    .run_if(input_just_pressed(KeyCode::Escape)),

                    update_tab_buttons,
                    play_sound_on_hover::<TabButton>,
                    animate_button_color::<TabMenuButton>(MENU_BUTTON_COLOR, Color::rgb(0.9, 0.9, 0.9)),
                    bind_zoom_slider_to_output,
                    update_zoom,
                    update_toggle_tile_grid_button_text,
                    toggle_resource::<ShowTileGrid>.run_if(on_click::<ToggleTileGridButton>)
                )
                .run_if(is_visible::<SettingsMenu>)
            )
            .in_set(InGameSystemSet::Update)
        );

        app.add_systems(
            Update,
            (
                (
                    set_resource(SelectedTab::General),
                    (
                        despawn_with::<TabMenu>,
                        apply_deferred,
                        systems::spawn_general_menu
                    )
                    .chain()
                )
                .run_if(not(resource_exists_and_equals(SelectedTab::General)).and_then(on_click::<GeneralButton>)),

                (
                    set_resource(SelectedTab::Interface),
                    (
                        despawn_with::<TabMenu>,
                        apply_deferred,
                        systems::spawn_interface_menu
                    )
                    .chain()
                )
                .run_if(not(resource_exists_and_equals(SelectedTab::Interface)).and_then(on_click::<InterfaceButton>)),

                (
                    set_resource(IsVisible::<SettingsMenu>::hidden()),
                    play_sound_on_toggle::<IsVisible<SettingsMenu>>
                ).run_if(on_click::<CloseMenuButton>),

                set_state(GameState::Menu).run_if(on_click::<SaveAndExitButton>),
            )
            .run_if(is_visible::<SettingsMenu>)
            .in_set(InGameSystemSet::Update)
        );
    }
}

fn setup(mut commands: Commands) {
    commands.insert_resource(IsVisible::<SettingsMenu>::hidden());
    commands.init_resource::<SelectedTab>();
}

fn cleanup(mut commands: Commands) {
    commands.remove_resource::<SelectedTab>();
}
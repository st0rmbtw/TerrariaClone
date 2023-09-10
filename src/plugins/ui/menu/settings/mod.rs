mod interface;
mod video;
mod volume;
mod language;

use bevy::{prelude::{Commands, Res, Plugin, App, OnEnter, OnExit, IntoSystemConfigs, Query, Entity, With, Update, in_state, Component}, text::TextStyle};

use crate::{plugins::assets::FontAssets, language::LanguageContent, common::{conditions::on_click, state::{SettingsMenuState, MenuState}, systems::send_event}};

use self::{interface::InterfaceMenuPlugin, video::VideoMenuPlugin, volume::VolumeMenuPlugin, language::LanguageMenuPlugin};

use super::{despawn_with, MenuContainer, MENU_BUTTON_COLOR, BackButton, MENU_BUTTON_FONT_SIZE, builders::{menu, menu_button, control_buttons_layout, control_button}, events::EnterMenu, components::MenuButton};

pub(super) struct SettingsMenuPlugin;
impl Plugin for SettingsMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((InterfaceMenuPlugin, VideoMenuPlugin, VolumeMenuPlugin, LanguageMenuPlugin));

        app.add_systems(
            OnEnter(MenuState::Settings(SettingsMenuState::Main)),
            setup_settings_menu
        );
        app.add_systems(
            OnExit(MenuState::Settings(SettingsMenuState::Main)),
            despawn_with::<SettingsMenu>
        );

        app.add_systems(
            Update,
            (
                send_event(EnterMenu(MenuState::Settings(SettingsMenuState::Interface)))
                    .run_if(on_click::<InterfaceButton>),
                send_event(EnterMenu(MenuState::Settings(SettingsMenuState::Video)))
                    .run_if(on_click::<VideoButton>),
                send_event(EnterMenu(MenuState::Settings(SettingsMenuState::Volume)))
                    .run_if(on_click::<VolumeButton>),
                send_event(EnterMenu(MenuState::Settings(SettingsMenuState::Language)))
                    .run_if(on_click::<LanguageButton>),
            )
            .run_if(in_state(MenuState::Settings(SettingsMenuState::Main)))
        );
    }
}

#[derive(Component)]
pub(super) struct SettingsMenu;

#[derive(Component)]
struct InterfaceButton;

#[derive(Component)]
struct VideoButton;

#[derive(Component)]
struct VolumeButton;

#[derive(Component)]
struct CursorButton;

#[derive(Component)]
struct LanguageButton;

fn setup_settings_menu(
    mut commands: Commands,
    fonts: Res<FontAssets>,
    language_content: Res<LanguageContent>,
    query_container: Query<Entity, With<MenuContainer>>
) {
    let text_style = TextStyle {
        font: fonts.andy_bold.clone_weak(),
        font_size: MENU_BUTTON_FONT_SIZE,
        color: MENU_BUTTON_COLOR,
    };

    let container = query_container.single();

    menu(SettingsMenu, &mut commands, container, 5., |builder| {
        menu_button(builder, text_style.clone(), &language_content.ui.interface, (MenuButton, InterfaceButton));
        menu_button(builder, text_style.clone(), &language_content.ui.video, (MenuButton, VideoButton));
        menu_button(builder, text_style.clone(), &language_content.ui.volume, (MenuButton, VolumeButton));
        menu_button(builder, text_style.clone(), &language_content.ui.cursor, (MenuButton, CursorButton));
        menu_button(builder, text_style.clone(), &language_content.ui.language, (MenuButton, LanguageButton));

        control_buttons_layout(builder, |control_button_builder| {
            control_button(control_button_builder, text_style.clone(), &language_content.ui.back, (MenuButton, BackButton));
        });
    });
}

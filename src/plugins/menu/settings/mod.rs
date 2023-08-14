mod interface;
mod video;
mod volume;
mod language;

use bevy::{prelude::{Commands, Res, Plugin, App, OnEnter, OnExit, IntoSystemConfigs, Query, Entity, With, Update, in_state, Component}, text::TextStyle};

use crate::{plugins::assets::FontAssets, language::LanguageContent, common::{conditions::on_btn_clicked, state::{SettingsMenuState, GameState, MenuState}}};

use self::{interface::InterfaceMenuPlugin, video::VideoMenuPlugin, volume::VolumeMenuPlugin, language::LanguageMenuPlugin};

use super::{despawn_with, MenuContainer, TEXT_COLOR, BackButton, MENU_BUTTON_FONT_SIZE, systems::send_enter_event, builders::{menu, menu_button, control_buttons_layout, control_button}};

pub(super) struct SettingsMenuPlugin;
impl Plugin for SettingsMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((InterfaceMenuPlugin, VideoMenuPlugin, VolumeMenuPlugin, LanguageMenuPlugin));

        app.add_systems(
            OnEnter(GameState::Menu(MenuState::Settings(SettingsMenuState::Main))),
            setup_settings_menu
        );
        app.add_systems(
            OnExit(GameState::Menu(MenuState::Settings(SettingsMenuState::Main))),
            despawn_with::<SettingsMenu>
        );

        app.add_systems(
            Update,
            (
                send_enter_event(GameState::Menu(MenuState::Settings(SettingsMenuState::Interface)))
                    .run_if(on_btn_clicked::<InterfaceButton>),
                send_enter_event(GameState::Menu(MenuState::Settings(SettingsMenuState::Video)))
                    .run_if(on_btn_clicked::<VideoButton>),
                send_enter_event(GameState::Menu(MenuState::Settings(SettingsMenuState::Volume)))
                    .run_if(on_btn_clicked::<VolumeButton>),
                send_enter_event(GameState::Menu(MenuState::Settings(SettingsMenuState::Language)))
                    .run_if(on_btn_clicked::<LanguageButton>),
                // send_enter_event(GameState::Menu(MenuState::Settings(SettingsMenuState::Cursor)))
                //     .run_if(on_btn_clicked::<CursorButton>),
            )
            .run_if(in_state(GameState::Menu(MenuState::Settings(SettingsMenuState::Main))))
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
        color: TEXT_COLOR,
    };

    let container = query_container.single();

    menu(SettingsMenu, &mut commands, container, 50., |builder| {
        menu_button(builder, text_style.clone(), language_content.ui.interface.clone(), InterfaceButton);
        menu_button(builder, text_style.clone(), language_content.ui.video.clone(), VideoButton);
        menu_button(builder, text_style.clone(), language_content.ui.volume.clone(), VolumeButton);
        menu_button(builder, text_style.clone(), language_content.ui.cursor.clone(), CursorButton);
        menu_button(builder, text_style.clone(), language_content.ui.language.clone(), LanguageButton);

        control_buttons_layout(builder, |control_button_builder| {
            control_button(control_button_builder, text_style.clone(), language_content.ui.back.clone(), BackButton);
        });
    });
}

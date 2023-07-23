pub(super) mod interface;
pub(super) mod video;

use autodefault::autodefault;
use bevy::{prelude::{Commands, Res, Plugin, App, Component, OnEnter, OnExit, IntoSystemConfigs, NextState, ResMut, Query, Entity, With, Update, in_state}, text::TextStyle};

use crate::{plugins::{assets::FontAssets, menu::{menu_button, control_buttons_layout, control_button}}, language::LanguageContent, common::{conditions::on_btn_clicked, state::{SettingsMenuState, GameState, MenuState}}};

use self::{interface::InterfaceMenuPlugin, video::VideoMenuPlugin};

use super::{despawn_with, menu, MenuContainer, TEXT_COLOR};

#[derive(Component)]
pub(super) struct SettingsMenu;

#[derive(Component)]
pub(super) struct BackButton;

#[derive(Component)]
pub(super) struct ApplyButton;

pub(super) const MENU_BUTTON_FONT_SIZE: f32 = 42.;

pub(super) struct SettingsMenuPlugin;
impl Plugin for SettingsMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(InterfaceMenuPlugin);
        app.add_plugin(VideoMenuPlugin);

        app.add_systems(OnEnter(GameState::Menu(MenuState::Settings(SettingsMenuState::Main))), setup_settings_menu);
        app.add_systems(OnExit(GameState::Menu(MenuState::Settings(SettingsMenuState::Main))), despawn_with::<SettingsMenu>);

        app.add_systems(
            Update,
            (
                interface_clicked.run_if(on_btn_clicked::<InterfaceButton>),
                video_clicked.run_if(on_btn_clicked::<VideoButton>),
                cursor_clicked.run_if(on_btn_clicked::<CursorButton>),
                back_clicked.run_if(on_btn_clicked::<BackButton>),
            )
            .run_if(in_state(GameState::Menu(MenuState::Settings(SettingsMenuState::Main))))
        );
    }
}

#[derive(Component)]
struct InterfaceButton;

#[derive(Component)]
struct VideoButton;

#[derive(Component)]
struct CursorButton;

#[autodefault]
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
        menu_button(builder, text_style.clone(), language_content.ui.cursor.clone(), CursorButton);

        control_buttons_layout(builder, |control_button_builder| {
            control_button(control_button_builder, text_style.clone(), language_content.ui.back.clone(), BackButton);
        });
    });
}


fn interface_clicked(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::Menu(MenuState::Settings(SettingsMenuState::Interface)));
}

fn video_clicked(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::Menu(MenuState::Settings(SettingsMenuState::Video)));
}

fn cursor_clicked(/* mut commands: Commands */) {
    // TODO: Implement Cursor menu
}

fn back_clicked(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::Menu(MenuState::Main));
}
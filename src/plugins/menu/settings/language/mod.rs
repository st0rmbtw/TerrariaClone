use bevy::{prelude::{Plugin, App, Commands, OnEnter, OnExit, Component, Res, Entity, With, Query, Color}, text::TextStyle};

use crate::{common::state::{GameState, MenuState, SettingsMenuState}, plugins::{menu::{systems::despawn_with, builders::{menu, menu_text, control_buttons_layout, control_button}, components::MenuContainer, MENU_BUTTON_FONT_SIZE, TEXT_COLOR, BackButton}, assets::FontAssets}, language::LanguageContent};

pub(super) struct LanguageMenuPlugin;
impl Plugin for LanguageMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Menu(MenuState::Settings(SettingsMenuState::Language))),
            setup_language_menu
        );

        app.add_systems(
            OnExit(GameState::Menu(MenuState::Settings(SettingsMenuState::Language))),
            despawn_with::<LanguageMenu>
        );
    }
}

#[derive(Component)]
struct LanguageMenu;

fn setup_language_menu(
    mut commands: Commands,
    fonts: Res<FontAssets>,
    language_content: Res<LanguageContent>,
    query_container: Query<Entity, With<MenuContainer>>,
) {
    let container = query_container.single();

    let title_text_style = TextStyle {
        font: fonts.andy_bold.clone_weak(),
        font_size: MENU_BUTTON_FONT_SIZE,
        color: Color::WHITE,
    };

    let button_text_style = TextStyle {
        font: fonts.andy_bold.clone_weak(),
        font_size: MENU_BUTTON_FONT_SIZE,
        color: TEXT_COLOR,
    };

    menu(LanguageMenu, &mut commands, container, 50., |builder| {
        menu_text(builder, title_text_style.clone(), "Select Language");

        menu_text(builder, title_text_style, "To do...");

        control_buttons_layout(builder, |control_button_builder| {
            control_button(control_button_builder, button_text_style, language_content.ui.back.clone(), BackButton);
        });
    });
}
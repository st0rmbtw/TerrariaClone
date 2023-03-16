mod buttons;

pub use buttons::*;

use autodefault::autodefault;
use bevy::{prelude::{Commands, Res, With, Query, ResMut, Component, NextState, Entity}, text::{TextStyle, Text}};

use crate::{plugins::{assets::FontAssets, menu::{menu_button, control_buttons_layout, control_button, menu, MenuContainer}, settings::ShowTileGrid}, language::LanguageContent, TEXT_COLOR, state::{SettingsMenuState, GameState, MenuState}};

use super::{MENU_BUTTON_FONT_SIZE, BackButton};

#[derive(Component)]
pub struct InterfaceMenu;

#[autodefault]
pub fn setup_interface_menu(
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

    menu(InterfaceMenu, &mut commands, container, 50., |builder| {
        menu_button(builder, text_style.clone(), language_content.ui.tile_grid.clone(), ToggleTileGridButton);

        control_buttons_layout(builder, |control_button_builder| {
            control_button(control_button_builder, text_style.clone(), language_content.ui.back.clone(), BackButton);
        });
    });
}

pub fn toggle_tile_grid_clicked(mut show_tile_grid: ResMut<ShowTileGrid>) {
    show_tile_grid.0 = !show_tile_grid.0;
}

pub fn back_clicked(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::Menu(MenuState::Settings(SettingsMenuState::Main)));
}

pub fn update_toggle_tile_grid_button_text(
    mut query: Query<&mut Text, With<ToggleTileGridButton>>,
    show_tile_grid: Res<ShowTileGrid>,
    language_content: Res<LanguageContent>
) {
    let mut text = query.single_mut();

    let status = if show_tile_grid.0 { language_content.ui.on.clone() } else { language_content.ui.off.clone() } ;

    text.sections[0].value = format!("{} {}", language_content.ui.tile_grid, status);
}
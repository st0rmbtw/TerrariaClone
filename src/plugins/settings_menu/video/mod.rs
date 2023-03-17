pub mod resolution;
mod buttons;

pub use buttons::*;

use autodefault::autodefault;
use bevy::{prelude::{Commands, Res, Component, ResMut, Query, With, NextState, Entity}, text::{TextStyle, Text}, window::Window};

use crate::{plugins::{assets::FontAssets, menu::{menu_button, control_buttons_layout, control_button, menu, MenuContainer}, settings::VSync}, language::LanguageContent, TEXT_COLOR, common::state::{SettingsMenuState, MenuState, GameState}};

use super::{MENU_BUTTON_FONT_SIZE, BackButton};

#[derive(Component)]
pub struct VideoMenu;

#[autodefault]
pub fn setup_video_menu(
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

    menu(VideoMenu, &mut commands, container, 50., |builder| {
        menu_button(builder, text_style.clone(), language_content.ui.resolution.clone(), ResolutionButton);
        menu_button(builder, text_style.clone(), language_content.ui.vsync.clone(), VSyncButton);

        control_buttons_layout(builder, |control_button_builder| {
            control_button(control_button_builder, text_style.clone(), language_content.ui.back.clone(), BackButton);
        });
    });
}

pub fn resolution_clicked(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::Menu(MenuState::Settings(SettingsMenuState::Resolution)));
}

pub fn back_clicked(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::Menu(MenuState::Settings(SettingsMenuState::Main)));
}

pub fn vsync_clicked(mut window: Query<&mut Window>, mut vsync: ResMut<VSync>) {
    vsync.0 = !vsync.0;
    window.single_mut().present_mode = vsync.as_present_mode();
}

pub fn update_vsync_button_text(
    mut query: Query<&mut Text, With<VSyncButton>>,
    vsync: Res<VSync>,
    language_content: Res<LanguageContent>
) {
    let mut text = query.single_mut();

    let status = if vsync.0 { language_content.ui.on.clone() } else { language_content.ui.off.clone() } ;

    text.sections[0].value = format!("{} {}", language_content.ui.vsync, status);
}
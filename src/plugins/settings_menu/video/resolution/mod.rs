mod buttons;
pub use buttons::*;

use autodefault::autodefault;
use bevy::{prelude::{Component, Commands, Res, NodeBundle, BuildChildren, ResMut, Query, With, Local, NextState}, text::{TextStyle, Text}, ui::{Style, Size, Val, JustifyContent, AlignItems, FlexDirection}, window::{Window, WindowResolution}};
use strum::EnumCount;

use crate::{plugins::{assets::FontAssets, settings_menu::{MENU_BUTTON_FONT_SIZE, BackButton, ApplyButton, SettingsMenuState}, menu::{menu_button, control_buttons_layout, control_button}, settings::{FullScreen, Resolution}}, language::LanguageContent, TEXT_COLOR, state::{GameState, MenuState}};

#[derive(Component)]
pub struct ResolutionMenu;

#[autodefault]
pub fn setup_resolution_menu(
    mut commands: Commands,
    fonts: Res<FontAssets>,
    language_content: Res<LanguageContent>
) {
    let text_style = TextStyle {
        font: fonts.andy_bold.clone_weak(),
        font_size: MENU_BUTTON_FONT_SIZE,
        color: TEXT_COLOR,
    };

    commands.spawn(NodeBundle {
        style: Style {
            size: Size {
                width: Val::Percent(100.),
                height: Val::Percent(100.),
            },
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
        }
    })
    .insert(ResolutionMenu)
    .with_children(|builder| {
        menu_button(builder, text_style.clone(), language_content.ui.full_screen_resolution.clone(), FullScreenResolutionButton, None);
        menu_button(builder, text_style.clone(), language_content.ui.full_screen.clone(), FullScreenButton, None);

        control_buttons_layout(builder, |control_button_builder| {
            control_button(control_button_builder, text_style.clone(), language_content.ui.apply.clone(), ApplyButton);
            control_button(control_button_builder, text_style.clone(), language_content.ui.back.clone(), BackButton);
        });
    });
}

pub fn fullscreen_resolution_clicked(
    mut resolution_index: Local<usize>,
    mut resolution: ResMut<Resolution> 
) {
    *resolution_index = (*resolution_index + 1) % Resolution::COUNT;

    *resolution = Resolution::from_repr(*resolution_index).unwrap();
}

pub fn fullscreen_clicked(mut fullscreen: ResMut<FullScreen>) {
    fullscreen.0 = !fullscreen.0;
}

pub fn back_clicked(mut commands: Commands) {
    commands.insert_resource(NextState(Some(GameState::Menu(MenuState::Settings(SettingsMenuState::Video)))));
}

pub fn apply_clicked(
    mut window: Query<&mut Window>,
    fullscreen: Res<FullScreen>,
    resolution: Res<Resolution>
) {
    let mut primary_window = window.single_mut();
    let resolution_data = resolution.data();
    
    primary_window.mode = fullscreen.as_window_mode();
    primary_window.resolution = WindowResolution::new(resolution_data.width, resolution_data.height);
}

pub fn update_fullscreen_resolution_button_text(
    mut query: Query<&mut Text, With<FullScreenResolutionButton>>,
    resolution: Res<Resolution>,
    language_content: Res<LanguageContent>
) {
    let mut text = query.single_mut();
    let resolution_data = resolution.data();

    let resolution_str = format!("{}x{}", resolution_data.width, resolution_data.height);

    text.sections[0].value = format!("{} {}", language_content.ui.full_screen_resolution, resolution_str);
}

pub fn update_resolution_button_text(
    mut query: Query<&mut Text, With<FullScreenButton>>,
    fullscreen: Res<FullScreen>,
    language_content: Res<LanguageContent>
) {
    let mut text = query.single_mut();

    let status = if fullscreen.0 { language_content.ui.on.to_string() } else { language_content.ui.off.to_string() };

    text.sections[0].value = format!("{} {}", language_content.ui.full_screen, status);
}
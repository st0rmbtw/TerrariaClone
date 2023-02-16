mod buttons;
pub use buttons::*;

use autodefault::autodefault;
use bevy::{prelude::{Component, Commands, Res, NodeBundle, BuildChildren, ResMut, Query, With, Local}, text::{TextStyle, Text}, ui::{Style, Size, Val, JustifyContent, AlignItems, FlexDirection}, window::Windows};
use iyes_loopless::state::NextState;

use crate::{plugins::{assets::FontAssets, settings_menu::{MENU_BUTTON_FONT_SIZE, BackButton, ApplyButton, SettingsMenuState}, menu::{menu_button, control_buttons_layout, control_button}, settings::{FullScreen, Resolution, RESOLUTIONS}}, language::LanguageContent, TEXT_COLOR};

#[derive(Component)]
pub struct ResolutionMenu;

#[autodefault]
pub fn setup_resolution_menu(
    mut commands: Commands,
    fonts: Res<FontAssets>,
    language_content: Res<LanguageContent>
) {
    let text_style = TextStyle {
        font: fonts.andy_bold.clone(),
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
        menu_button(builder, text_style.clone(), language_content.ui.full_screen_resolution.clone(), FullScreenResolutionButton);
        menu_button(builder, text_style.clone(), language_content.ui.full_screen.clone(), FullScreenButton);

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
    *resolution_index = (*resolution_index + 1) % RESOLUTIONS.len();

    *resolution = RESOLUTIONS[*resolution_index];
}

pub fn fullscreen_clicked(mut fullscreen: ResMut<FullScreen>) {
    fullscreen.0 = !fullscreen.0;
}

pub fn back_clicked(mut commands: Commands) {
    commands.insert_resource(NextState(SettingsMenuState::Video));
}

pub fn apply_clicked(
    mut window: ResMut<Windows>,
    fullscreen: Res<FullScreen>,
    resolution: Res<Resolution>
) {
    let primary_window = window.get_primary_mut().unwrap();
    
    primary_window.set_mode(fullscreen.as_window_mode());
    primary_window.set_resolution(resolution.width, resolution.height);
}

pub fn update_fullscreen_resolution_button_text(
    mut query: Query<&mut Text, With<FullScreenResolutionButton>>,
    resolution: Res<Resolution>,
    language_content: Res<LanguageContent>
) {
    let mut text = query.single_mut();

    let resolution_str = format!("{}x{}", resolution.width, resolution.height);

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
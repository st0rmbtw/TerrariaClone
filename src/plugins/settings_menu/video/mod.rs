pub mod resolution;
mod buttons;

pub use buttons::*;

use autodefault::autodefault;
use bevy::{prelude::{Commands, Res, NodeBundle, BuildChildren, Component, ResMut, Query, With}, text::{TextStyle, Text}, ui::{Style, Size, Val, JustifyContent, AlignItems, FlexDirection}, window::Windows};
use iyes_loopless::state::NextState;

use crate::{plugins::{assets::FontAssets, menu::{menu_button, control_buttons_layout, control_button}, settings::VSync}, language::LanguageContent, TEXT_COLOR, state::GameState};

use super::{SettingsMenuState, MENU_BUTTON_FONT_SIZE, BackButton};

#[derive(Component)]
pub struct VideoMenu;

#[autodefault]
pub fn setup_video_menu(
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
    .insert(VideoMenu)
    .with_children(|builder| {
        menu_button(builder, text_style.clone(), language_content.ui.resolution.clone(), ResolutionButton, None);
        menu_button(builder, text_style.clone(), language_content.ui.vsync.clone(), VSyncButton, None);

        control_buttons_layout(builder, |control_button_builder| {
            control_button(control_button_builder, text_style.clone(), language_content.ui.back.clone(), BackButton);
        });
    });
}

pub fn resolution_clicked(mut commands: Commands) {
    commands.insert_resource(NextState(SettingsMenuState::Resolution));
}

pub fn back_clicked(mut commands: Commands) {
    commands.insert_resource(NextState(SettingsMenuState::None));
    commands.insert_resource(NextState(GameState::Settings));
}

pub fn vsync_clicked(mut window: ResMut<Windows>, mut vsync: ResMut<VSync>) {
    vsync.0 = !vsync.0;

    window
        .get_primary_mut()
        .unwrap()
        .set_present_mode(vsync.as_present_mode());
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
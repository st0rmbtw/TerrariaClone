mod buttons;

pub use buttons::*;

use autodefault::autodefault;
use bevy::{prelude::{Commands, Res, NodeBundle, BuildChildren, With, Query, ResMut, Component}, text::{TextStyle, Text}, ui::{Style, Size, Val, JustifyContent, AlignItems, FlexDirection}};
use iyes_loopless::state::NextState;

use crate::{plugins::{assets::FontAssets, menu::menu_button, settings::ShowTileGrid}, language::LanguageContent, TEXT_COLOR, state::GameState};

use super::{BackButton, SettingsMenuState};

#[derive(Component)]
pub struct InterfaceMenu;

#[autodefault]
pub fn setup_interface_menu(
    mut commands: Commands,
    fonts: Res<FontAssets>,
    language_content: Res<LanguageContent>
) {
    let text_style = TextStyle {
        font: fonts.andy_bold.clone(),
        font_size: 44.,
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
    .insert(InterfaceMenu)
    .with_children(|builder| {
        menu_button(builder, text_style.clone(), language_content.ui.tile_grid.clone(), ToggleTileGridButton);
        menu_button(builder, text_style.clone(), language_content.ui.back.clone(), BackButton);
    });
}

pub fn toggle_tile_grid_clicked(mut show_tile_grid: ResMut<ShowTileGrid>) {
    show_tile_grid.0 = !show_tile_grid.0;
}

pub fn back_clicked(mut commands: Commands) {
    commands.insert_resource(NextState(SettingsMenuState::None));
    commands.insert_resource(NextState(GameState::Settings));
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
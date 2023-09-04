use autodefault::autodefault;
use bevy::{prelude::{Commands, Res, With, Query, ResMut, Component, Entity, Plugin, App, OnEnter, OnExit, IntoSystemConfigs, Update, in_state}, text::{TextStyle, Text}};

use crate::{
    plugins::{
        assets::FontAssets,
        ui::menu::{MenuContainer, despawn_with, TEXT_COLOR, builders::{menu, menu_button, control_buttons_layout, control_button}},
        config::ShowTileGrid
    },
    language::LanguageContent,
    common::{state::{SettingsMenuState, MenuState}, conditions::on_click}
};

use super::{MENU_BUTTON_FONT_SIZE, BackButton};

pub(super) struct InterfaceMenuPlugin;
impl Plugin for InterfaceMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(MenuState::Settings(SettingsMenuState::Interface)),
            setup_interface_menu
        );
        app.add_systems(
            OnExit(MenuState::Settings(SettingsMenuState::Interface)),
            despawn_with::<InterfaceMenu>
        );

        app.add_systems(
            Update,
            (
                update_toggle_tile_grid_button_text,
                toggle_tile_grid_clicked.run_if(on_click::<ToggleTileGridButton>),
            )
            .run_if(in_state(MenuState::Settings(SettingsMenuState::Interface)))
        );
    }
}

#[derive(Component)]
struct InterfaceMenu;

#[derive(Component)]
struct ToggleTileGridButton;

#[autodefault]
fn setup_interface_menu(
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

    menu(InterfaceMenu, &mut commands, container, 5., |builder| {
        menu_button(builder, text_style.clone(), language_content.ui.tile_grid.clone(), ToggleTileGridButton);

        control_buttons_layout(builder, |control_button_builder| {
            control_button(control_button_builder, text_style, language_content.ui.back.clone(), BackButton);
        });
    });
}

fn toggle_tile_grid_clicked(mut show_tile_grid: ResMut<ShowTileGrid>) {
    show_tile_grid.0 = !show_tile_grid.0;
}

fn update_toggle_tile_grid_button_text(
    mut query: Query<&mut Text, With<ToggleTileGridButton>>,
    show_tile_grid: Res<ShowTileGrid>,
    language_content: Res<LanguageContent>
) {
    let mut text = query.single_mut();

    let status = if show_tile_grid.0 { language_content.ui.on.clone() } else { language_content.ui.off.clone() } ;

    text.sections[0].value = format!("{} {}", language_content.ui.tile_grid, status);
}
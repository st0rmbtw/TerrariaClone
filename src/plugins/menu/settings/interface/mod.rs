use autodefault::autodefault;
use bevy::{prelude::{Commands, Res, With, Query, ResMut, Component, NextState, Entity, Plugin, App, OnUpdate, IntoSystemAppConfig, OnEnter, OnExit, IntoSystemConfig, IntoSystemConfigs}, text::{TextStyle, Text}};

use crate::{plugins::{assets::FontAssets, menu::{menu_button, control_buttons_layout, control_button, menu, MenuContainer, despawn_with}, settings::ShowTileGrid}, language::LanguageContent, TEXT_COLOR, common::{state::{SettingsMenuState, GameState, MenuState}, conditions::on_btn_clicked}};

use super::{MENU_BUTTON_FONT_SIZE, BackButton};

pub(super) struct InterfaceMenuPlugin;
impl Plugin for InterfaceMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(setup_interface_menu.in_schedule(OnEnter(GameState::Menu(MenuState::Settings(SettingsMenuState::Interface)))));
        app.add_system(despawn_with::<InterfaceMenu>.in_schedule(OnExit(GameState::Menu(MenuState::Settings(SettingsMenuState::Interface)))));

        app.add_systems(
            (
                update_toggle_tile_grid_button_text,
                toggle_tile_grid_clicked.run_if(on_btn_clicked::<ToggleTileGridButton>),
                back_clicked.run_if(on_btn_clicked::<BackButton>),
            )
            .chain()
            .in_set(OnUpdate(GameState::Menu(MenuState::Settings(SettingsMenuState::Interface))))
        );
    }
}

#[derive(Component)]
struct InterfaceMenu;

#[derive(Component)]
pub struct ToggleTileGridButton;

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

    menu(InterfaceMenu, &mut commands, container, 50., |builder| {
        menu_button(builder, text_style.clone(), language_content.ui.tile_grid.clone(), ToggleTileGridButton);

        control_buttons_layout(builder, |control_button_builder| {
            control_button(control_button_builder, text_style, language_content.ui.back.clone(), BackButton);
        });
    });
}

fn toggle_tile_grid_clicked(mut show_tile_grid: ResMut<ShowTileGrid>) {
    show_tile_grid.0 = !show_tile_grid.0;
}

fn back_clicked(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::Menu(MenuState::Settings(SettingsMenuState::Main)));
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
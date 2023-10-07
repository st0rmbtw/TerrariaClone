use bevy::{prelude::{Commands, Res, With, Query, Component, Entity, Plugin, App, OnEnter, OnExit, IntoSystemConfigs, Update, in_state}, text::TextStyle};

use crate::{
    plugins::{
        assets::FontAssets,
        ui::{menu::{MenuContainer, despawn_with, MENU_BUTTON_COLOR, builders::{menu, menu_button, control_buttons_layout, control_button}, components::MenuButton}, components::ToggleTileGridButton, systems::update_toggle_tile_grid_button_text}, config::ShowTileGrid,
    },
    common::{state::{SettingsMenuState, MenuState}, conditions::on_click, systems::toggle_resource}, language::keys::UIStringKey
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
                toggle_resource::<ShowTileGrid>.run_if(on_click::<ToggleTileGridButton>),
            )
            .run_if(in_state(MenuState::Settings(SettingsMenuState::Interface)))
        );
    }
}

#[derive(Component)]
struct InterfaceMenu;

fn setup_interface_menu(
    mut commands: Commands,
    fonts: Res<FontAssets>,
    query_container: Query<Entity, With<MenuContainer>>
) {
    let text_style = TextStyle {
        font: fonts.andy_bold.clone_weak(),
        font_size: MENU_BUTTON_FONT_SIZE,
        color: MENU_BUTTON_COLOR,
    };

    let container = query_container.single();

    menu(InterfaceMenu, &mut commands, container, 5., |builder| {
        menu_button(builder, text_style.clone(), UIStringKey::TileGrid, (MenuButton, ToggleTileGridButton));

        control_buttons_layout(builder, |control_button_builder| {
            control_button(control_button_builder, text_style, UIStringKey::Back, (MenuButton, BackButton));
        });
    });
}
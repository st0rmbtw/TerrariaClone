mod buttons;
pub mod interface;

use autodefault::autodefault;
use bevy::{prelude::{Commands, Res, NodeBundle, BuildChildren, Plugin, App, Component}, text::{TextStyle}, ui::{Style, Size, Val, JustifyContent, AlignItems, FlexDirection}};
use iyes_loopless::{state::NextState, prelude::{AppLooplessStateExt, IntoConditionalSystem, ConditionSet}};

use crate::{plugins::{assets::FontAssets, menu::menu_button}, language::LanguageContent, TEXT_COLOR, state::{GameState}, util::on_btn_clicked};

use self::buttons::{InterfaceButton, VideoButton, CursorButton};

use super::menu::despawn_with;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SettingsMenuState {
    Interface,
    Video,
    Cursor,
    None
}

#[derive(Component)]
pub struct SettingsMenu;

#[derive(Component)]
pub struct BackButton;

pub struct SettingsMenuPlugin;

impl Plugin for SettingsMenuPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_enter_system(GameState::Settings, setup_settings_menu)
            .add_exit_system(GameState::Settings, despawn_with::<SettingsMenu>)
            .add_exit_system(SettingsMenuState::None, despawn_with::<SettingsMenu>)

            .add_system_set(
                ConditionSet::new()
                    .run_in_state(SettingsMenuState::None)
                    .with_system(interface_clicked.run_if(on_btn_clicked::<InterfaceButton>))
                    .with_system(video_clicked.run_if(on_btn_clicked::<VideoButton>))
                    .with_system(cursor_clicked.run_if(on_btn_clicked::<CursorButton>))
                    .with_system(back_clicked.run_if(on_btn_clicked::<BackButton>))
                    .into()
            )
            
            // ---- Interface -----
            .add_enter_system(SettingsMenuState::Interface, interface::setup_interface_menu)
            .add_exit_system(SettingsMenuState::Interface, despawn_with::<interface::InterfaceMenu>)
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(SettingsMenuState::Interface)
                    .with_system(interface::update_toggle_tile_grid_button_text)
                    .with_system(interface::toggle_tile_grid_clicked.run_if(on_btn_clicked::<interface::ToggleTileGridButton>))
                    .with_system(interface::back_clicked.run_if(on_btn_clicked::<BackButton>))
                    .into()
            );
    }
}

#[autodefault]
pub fn setup_settings_menu(
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
    .insert(SettingsMenu)
    .with_children(|builder| {
        use buttons::*;
        menu_button(builder, text_style.clone(), language_content.ui.interface.clone(), InterfaceButton);
        menu_button(builder, text_style.clone(), language_content.ui.video.clone(), VideoButton);
        menu_button(builder, text_style.clone(), language_content.ui.cursor.clone(), CursorButton);
        menu_button(builder, text_style.clone(), language_content.ui.back.clone(), BackButton)
    });
}


pub fn interface_clicked(mut commands: Commands) {
    commands.insert_resource(NextState(SettingsMenuState::Interface));
}

pub fn video_clicked(/* mut commands: Commands */) {
    // TODO: Implement Video menu
    // commands.insert_resource(NextState(SettingsMenuState::Video));
}

pub fn cursor_clicked(/* mut commands: Commands */) {
    // TODO: Implement Cursor menu
    // commands.insert_resource(NextState(SettingsMenuState::Cursor));
}

pub fn back_clicked(mut commands: Commands) {
    commands.insert_resource(NextState(GameState::MainMenu));
}
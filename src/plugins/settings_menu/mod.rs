mod buttons;
pub mod interface;
pub mod video;

use autodefault::autodefault;
use bevy::{prelude::{Commands, Res, NodeBundle, BuildChildren, Plugin, App, Component, IntoSystemAppConfig, OnEnter, OnExit, OnUpdate, IntoSystemConfig, IntoSystemConfigs, NextState}, text::TextStyle, ui::{Style, Size, Val, JustifyContent, AlignItems, FlexDirection}};

use crate::{plugins::{assets::FontAssets, menu::{menu_button, control_buttons_layout, control_button}}, language::LanguageContent, TEXT_COLOR, state::GameState, util::on_btn_clicked};

use self::buttons::{InterfaceButton, VideoButton, CursorButton};

use super::menu::despawn_with;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SettingsMenuState {
    Interface,
    Video,
    Cursor,
    Resolution,
    #[default]
    None
}

#[derive(Component)]
pub struct SettingsMenu;

#[derive(Component)]
pub struct BackButton;

#[derive(Component)]
pub struct ApplyButton;

pub struct SettingsMenuPlugin;

pub const MENU_BUTTON_FONT_SIZE: f32 = 42.;

impl Plugin for SettingsMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(setup_settings_menu.in_schedule(OnEnter(GameState::Settings(SettingsMenuState::None))));
        app.add_system(despawn_with::<SettingsMenu>.in_schedule(OnExit(GameState::Settings(SettingsMenuState::None))));

        app.add_systems(
            (
                interface_clicked.run_if(on_btn_clicked::<InterfaceButton>),
                video_clicked.run_if(on_btn_clicked::<VideoButton>),
                cursor_clicked.run_if(on_btn_clicked::<CursorButton>),
                back_clicked.run_if(on_btn_clicked::<BackButton>),
            )
            .chain()
            .in_set(OnUpdate(GameState::Settings(SettingsMenuState::None)))
        );
            
        // ---- Interface -----
        app.add_system(interface::setup_interface_menu.in_schedule(OnEnter(GameState::Settings(SettingsMenuState::Interface))));
        app.add_system(despawn_with::<interface::InterfaceMenu>.in_schedule(OnExit(GameState::Settings(SettingsMenuState::Interface))));

        app.add_systems(
            (
                interface::update_toggle_tile_grid_button_text,
                interface::toggle_tile_grid_clicked.run_if(on_btn_clicked::<interface::ToggleTileGridButton>),
                interface::back_clicked.run_if(on_btn_clicked::<BackButton>),
            )
            .chain()
            .in_set(OnUpdate(GameState::Settings(SettingsMenuState::Interface)))
        );
            
        // ------ Video -------
        app.add_system(video::setup_video_menu.in_schedule(OnEnter(GameState::Settings(SettingsMenuState::Video))));
        app.add_system(despawn_with::<video::VideoMenu>.in_schedule(OnExit(GameState::Settings(SettingsMenuState::Video))));

        app.add_systems(
            (
                video::update_vsync_button_text,
                video::resolution_clicked.run_if(on_btn_clicked::<video::ResolutionButton>),
                video::vsync_clicked.run_if(on_btn_clicked::<video::VSyncButton>),
                video::back_clicked.run_if(on_btn_clicked::<BackButton>),
            )
            .chain()
            .in_set(OnUpdate(GameState::Settings(SettingsMenuState::Video)))
        );
            
        // ----- Resolution -----
        app.add_system(video::resolution::setup_resolution_menu.in_schedule(OnEnter(GameState::Settings(SettingsMenuState::Resolution))));
        app.add_system(despawn_with::<video::resolution::ResolutionMenu>.in_schedule(OnExit(GameState::Settings(SettingsMenuState::Resolution))));

        app.add_systems(
            (
                video::resolution::update_fullscreen_resolution_button_text,
                video::resolution::update_resolution_button_text,
                video::resolution::fullscreen_resolution_clicked.run_if(on_btn_clicked::<video::resolution::FullScreenResolutionButton>),
                video::resolution::fullscreen_clicked.run_if(on_btn_clicked::<video::resolution::FullScreenButton>),
                video::resolution::apply_clicked.run_if(on_btn_clicked::<ApplyButton>),
                video::resolution::back_clicked.run_if(on_btn_clicked::<BackButton>),
            )
            .chain()
            .in_set(OnUpdate(GameState::Settings(SettingsMenuState::Resolution)))
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
    .insert(SettingsMenu)
    .with_children(|builder| {
        use buttons::*;
        menu_button(builder, text_style.clone(), language_content.ui.interface.clone(), InterfaceButton, None);
        menu_button(builder, text_style.clone(), language_content.ui.video.clone(), VideoButton, None);
        menu_button(builder, text_style.clone(), language_content.ui.cursor.clone(), CursorButton, None);

        control_buttons_layout(builder, |control_button_builder| {
            control_button(control_button_builder, text_style.clone(), language_content.ui.back.clone(), BackButton);
        });
    });
}


pub fn interface_clicked(mut commands: Commands) {
    commands.insert_resource(NextState(Some(GameState::Settings(SettingsMenuState::Interface))));
}

pub fn video_clicked(mut commands: Commands) {
    commands.insert_resource(NextState(Some(GameState::Settings(SettingsMenuState::Video))));
}

pub fn cursor_clicked(/* mut commands: Commands */) {
    // TODO: Implement Cursor menu
    // commands.insert_resource(NextState(SettingsMenuState::Cursor));
}

pub fn back_clicked(mut commands: Commands) {
    commands.insert_resource(NextState(Some(GameState::MainMenu)));
}
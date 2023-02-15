mod buttons;
pub mod interface;
pub mod video;

use autodefault::autodefault;
use bevy::{prelude::{Commands, Res, NodeBundle, BuildChildren, Plugin, App, Component}, text::TextStyle, ui::{Style, Size, Val, JustifyContent, AlignItems, FlexDirection}};
use iyes_loopless::{state::NextState, prelude::{AppLooplessStateExt, IntoConditionalSystem, ConditionSet}};

use crate::{plugins::{assets::FontAssets, menu::{menu_button, control_buttons_layout, control_button}}, language::LanguageContent, TEXT_COLOR, state::GameState, util::on_btn_clicked};

use self::buttons::{InterfaceButton, VideoButton, CursorButton};

use super::menu::despawn_with;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SettingsMenuState {
    Interface,
    Video,
    Cursor,
    Resolution,
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
            )
            
            // ------ Video -------
            .add_enter_system(SettingsMenuState::Video, video::setup_video_menu)
            .add_exit_system(SettingsMenuState::Video, despawn_with::<video::VideoMenu>)
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(SettingsMenuState::Video)
                    .with_system(video::update_vsync_button_text)
                    .with_system(video::resolution_clicked.run_if(on_btn_clicked::<video::ResolutionButton>))
                    .with_system(video::vsync_clicked.run_if(on_btn_clicked::<video::VSyncButton>))
                    .with_system(video::back_clicked.run_if(on_btn_clicked::<BackButton>))
                    .into()
            )
            
            // ----- Resolution -----
            .add_enter_system(SettingsMenuState::Resolution, video::resolution::setup_resolution_menu)
            .add_exit_system(SettingsMenuState::Resolution, despawn_with::<video::resolution::ResolutionMenu>)
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(SettingsMenuState::Resolution)
                    .with_system(video::resolution::update_fullscreen_resolution_button_text)
                    .with_system(video::resolution::update_resolution_button_text)
                    .with_system(video::resolution::fullscreen_resolution_clicked.run_if(on_btn_clicked::<video::resolution::FullScreenResolutionButton>))
                    .with_system(video::resolution::fullscreen_clicked.run_if(on_btn_clicked::<video::resolution::FullScreenButton>))
                    .with_system(video::resolution::apply_clicked.run_if(on_btn_clicked::<ApplyButton>))
                    .with_system(video::resolution::back_clicked.run_if(on_btn_clicked::<BackButton>))
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
        menu_button(builder, text_style.clone(), language_content.ui.interface.clone(), InterfaceButton);
        menu_button(builder, text_style.clone(), language_content.ui.video.clone(), VideoButton);
        menu_button(builder, text_style.clone(), language_content.ui.cursor.clone(), CursorButton);

        control_buttons_layout(builder, |control_button_builder| {
            control_button(control_button_builder, text_style.clone(), language_content.ui.back.clone(), BackButton);
        });
    });
}


pub fn interface_clicked(mut commands: Commands) {
    commands.insert_resource(NextState(SettingsMenuState::Interface));
}

pub fn video_clicked(mut commands: Commands) {
    commands.insert_resource(NextState(SettingsMenuState::Video));
}

pub fn cursor_clicked(/* mut commands: Commands */) {
    // TODO: Implement Cursor menu
    // commands.insert_resource(NextState(SettingsMenuState::Cursor));
}

pub fn back_clicked(mut commands: Commands) {
    commands.insert_resource(NextState(GameState::MainMenu));
}
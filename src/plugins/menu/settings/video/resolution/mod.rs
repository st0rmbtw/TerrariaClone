use autodefault::autodefault;
use bevy::{prelude::{Component, Commands, Res, ResMut, Query, With, Local, NextState, Entity, Plugin, App, OnEnter, OnExit, OnUpdate, IntoSystemConfig, IntoSystemAppConfig, IntoSystemConfigs}, text::{TextStyle, Text}, window::{Window, WindowResolution}};

use crate::{
    language::LanguageContent,
    common::{state::{SettingsMenuState, GameState, MenuState}, conditions::on_btn_clicked},
    plugins::{
        assets::FontAssets, 
        menu::{
            menu_button, control_buttons_layout, control_button, menu, MenuContainer,
            settings::{BackButton, ApplyButton, MENU_BUTTON_FONT_SIZE}, despawn_with
        }, 
        settings::{FullScreen, Resolution, RESOLUTIONS}
    }, 
    TEXT_COLOR
};

pub(super) struct ResolutionMenuPlugin;
impl Plugin for ResolutionMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(setup_resolution_menu.in_schedule(OnEnter(GameState::Menu(MenuState::Settings(SettingsMenuState::Resolution)))));
        app.add_system(despawn_with::<ResolutionMenu>.in_schedule(OnExit(GameState::Menu(MenuState::Settings(SettingsMenuState::Resolution)))));

        app.add_systems(
            (
                update_fullscreen_resolution_button_text,
                update_resolution_button_text,
                fullscreen_resolution_clicked.run_if(on_btn_clicked::<FullScreenResolutionButton>),
                fullscreen_clicked.run_if(on_btn_clicked::<FullScreenButton>),
                apply_clicked.run_if(on_btn_clicked::<ApplyButton>),
                back_clicked.run_if(on_btn_clicked::<BackButton>),
            )
            .in_set(OnUpdate(GameState::Menu(MenuState::Settings(SettingsMenuState::Resolution))))
        );
    }
}

#[derive(Component)]
struct ResolutionMenu;

#[derive(Component, Clone)]
struct FullScreenResolutionButton;

#[derive(Component, Clone)]
struct FullScreenButton;

#[autodefault]
fn setup_resolution_menu(
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

    menu(ResolutionMenu, &mut commands, container, 50., |builder| {
        menu_button(builder, text_style.clone(), language_content.ui.full_screen_resolution.clone(), FullScreenResolutionButton);
        menu_button(builder, text_style.clone(), language_content.ui.full_screen.clone(), FullScreenButton);

        control_buttons_layout(builder, |control_button_builder| {
            control_button(control_button_builder, text_style.clone(), language_content.ui.apply.clone(), ApplyButton);
            control_button(control_button_builder, text_style, language_content.ui.back.clone(), BackButton);
        });
    });
}

fn fullscreen_resolution_clicked(
    mut resolution_index: Local<usize>,
    mut resolution: ResMut<Resolution> 
) {
    let index = RESOLUTIONS.iter().position(|res| *res == *resolution).unwrap_or(*resolution_index);
    *resolution_index = (index + 1) % RESOLUTIONS.len();
    *resolution = RESOLUTIONS[*resolution_index];
}

fn fullscreen_clicked(mut fullscreen: ResMut<FullScreen>) {
    fullscreen.0 = !fullscreen.0;
}

fn back_clicked(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::Menu(MenuState::Settings(SettingsMenuState::Video)));
}

fn apply_clicked(
    mut next_state: ResMut<NextState<GameState>>,
    mut window: Query<&mut Window>,
    fullscreen: Res<FullScreen>,
    resolution: Res<Resolution>
) {
    let mut primary_window = window.single_mut();
    primary_window.mode = fullscreen.as_window_mode();
    primary_window.resolution = WindowResolution::new(resolution.width, resolution.height);
    
    next_state.set(GameState::Menu(MenuState::Settings(SettingsMenuState::Video)));
}

fn update_fullscreen_resolution_button_text(
    mut query: Query<&mut Text, With<FullScreenResolutionButton>>,
    resolution: Res<Resolution>,
    language_content: Res<LanguageContent>
) {
    let mut text = query.single_mut();

    let resolution_str = format!("{}x{}", resolution.width, resolution.height);

    text.sections[0].value = format!("{} {}", language_content.ui.full_screen_resolution, resolution_str);
}

fn update_resolution_button_text(
    mut query: Query<&mut Text, With<FullScreenButton>>,
    fullscreen: Res<FullScreen>,
    language_content: Res<LanguageContent>
) {
    let mut text = query.single_mut();

    let status = if fullscreen.0 { language_content.ui.on.to_string() } else { language_content.ui.off.to_string() };

    text.sections[0].value = format!("{} {}", language_content.ui.full_screen, status);
}
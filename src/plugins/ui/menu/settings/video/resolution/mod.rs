use autodefault::autodefault;
use bevy::{prelude::{Component, Commands, Res, ResMut, Query, With, Local, Entity, Plugin, App, OnEnter, OnExit,IntoSystemConfigs, in_state, Update, EventWriter, DetectChanges}, text::{TextStyle, Text}, window::{Window, WindowResolution}};

use crate::{
    language::LanguageContent,
    common::{state::{SettingsMenuState, MenuState}, conditions::on_click},
    plugins::{
        assets::FontAssets, 
        ui::menu::{
            MenuContainer, MENU_BUTTON_FONT_SIZE, despawn_with, 
            MENU_BUTTON_COLOR, ApplyButton, BackButton, Back, builders::{menu, menu_button, control_buttons_layout, control_button}, components::MenuButton
        }, 
        config::{FullScreen, Resolution, RESOLUTIONS}
    }, 
};

pub(super) struct ResolutionMenuPlugin;
impl Plugin for ResolutionMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(MenuState::Settings(SettingsMenuState::Resolution)),
            setup_resolution_menu
        );
        app.add_systems(
            OnExit(MenuState::Settings(SettingsMenuState::Resolution)),
            despawn_with::<ResolutionMenu>
        );

        app.add_systems(
            Update,
            (
                update_fullscreen_resolution_button_text,
                update_resolution_button_text,
                fullscreen_resolution_clicked.run_if(on_click::<FullScreenResolutionButton>),
                fullscreen_clicked.run_if(on_click::<FullScreenButton>),
                apply_clicked.run_if(on_click::<ApplyButton>),
            )
            .run_if(in_state(MenuState::Settings(SettingsMenuState::Resolution)))
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
    fullscreen: Res<FullScreen>,
    resolution: Res<Resolution>,
    query_container: Query<Entity, With<MenuContainer>>
) {
    let text_style = TextStyle {
        font: fonts.andy_bold.clone_weak(),
        font_size: MENU_BUTTON_FONT_SIZE,
        color: MENU_BUTTON_COLOR,
    };

    let container = query_container.single();

    menu(ResolutionMenu, &mut commands, container, 50., |builder| {
        menu_button(
            builder,
            text_style.clone(),
            resolution_btn_name(&language_content, &resolution),
            (MenuButton, FullScreenResolutionButton)
        );
        menu_button(
            builder,
            text_style.clone(),
            fullscreen_btn_name(&language_content, fullscreen.0),
            (MenuButton, FullScreenButton)
        );

        control_buttons_layout(builder, |control_button_builder| {
            control_button(control_button_builder, text_style.clone(), &language_content.ui.apply, (MenuButton, ApplyButton));
            control_button(control_button_builder, text_style, &language_content.ui.back, (MenuButton, BackButton));
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

fn apply_clicked(
    mut back_events: EventWriter<Back>,
    mut window: Query<&mut Window>,
    fullscreen: Res<FullScreen>,
    resolution: Res<Resolution>
) {
    let mut primary_window = window.single_mut();
    primary_window.mode = fullscreen.as_window_mode();
    primary_window.resolution = WindowResolution::new(resolution.width, resolution.height);
    
    back_events.send(Back);
}

fn update_fullscreen_resolution_button_text(
    mut query: Query<&mut Text, With<FullScreenResolutionButton>>,
    resolution: Res<Resolution>,
    language_content: Res<LanguageContent>
) {
    if resolution.is_changed() {
        let mut text = query.single_mut();

        text.sections[0].value = resolution_btn_name(&language_content, &resolution);
    }
}

fn update_resolution_button_text(
    mut query: Query<&mut Text, With<FullScreenButton>>,
    fullscreen: Res<FullScreen>,
    language_content: Res<LanguageContent>
) {
    if fullscreen.is_changed() {
        let mut text = query.single_mut();

        text.sections[0].value = fullscreen_btn_name(&language_content, fullscreen.0);
    }
}

#[inline]
fn fullscreen_btn_name(language_content: &LanguageContent, fullscreen: bool) -> String {
    format!("{} {}", language_content.ui.fullscreen, language_content.on_off(fullscreen))
}

#[inline]
fn resolution_btn_name(language_content: &LanguageContent, resolution: &Resolution) -> String {
    format!("{} {}x{}", language_content.ui.fullscreen_resolution, resolution.width, resolution.height)
}
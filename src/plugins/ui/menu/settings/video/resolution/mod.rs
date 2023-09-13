use autodefault::autodefault;
use bevy::{prelude::{Component, Commands, Res, ResMut, Query, With, Local, Entity, Plugin, App, OnEnter, OnExit,IntoSystemConfigs, in_state, Update, EventWriter, DetectChanges}, text::TextStyle, window::{Window, WindowResolution}};

use crate::{
    language::{keys::UIStringKey, LocalizedText, args},
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
            resolution_btn_text(&resolution),
            (MenuButton, FullScreenResolutionButton)
        );
        menu_button(
            builder,
            text_style.clone(),
            fullscreen_btn_text(fullscreen.0),
            (MenuButton, FullScreenButton)
        );

        control_buttons_layout(builder, |control_button_builder| {
            control_button(control_button_builder, text_style.clone(), UIStringKey::Apply, (MenuButton, ApplyButton));
            control_button(control_button_builder, text_style, UIStringKey::Back, (MenuButton, BackButton));
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
    mut query: Query<&mut LocalizedText, With<FullScreenResolutionButton>>,
    resolution: Res<Resolution>,
) {
    if resolution.is_changed() {
        let mut localized_text = query.single_mut();

        *localized_text = resolution_btn_text(&resolution);
    }
}

fn update_resolution_button_text(
    mut query: Query<&mut LocalizedText, With<FullScreenButton>>,
    fullscreen: Res<FullScreen>,
) {
    if fullscreen.is_changed() {
        let mut localized_text = query.single_mut();

        *localized_text = fullscreen_btn_text(fullscreen.0);
    }
}

#[inline(always)]
fn fullscreen_btn_text(fullscreen: bool) -> LocalizedText {
    let status = if fullscreen { UIStringKey::On } else { UIStringKey::Off };

    LocalizedText::new(UIStringKey::Fullscreen, "{} {}", args![status])
}

#[inline(always)]
fn resolution_btn_text(resolution: &Resolution) -> LocalizedText {
    LocalizedText::new(UIStringKey::FullscreenResolution, "{} {}x{}", args![resolution.width, resolution.height])
}
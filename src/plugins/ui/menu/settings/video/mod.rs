mod resolution;

use autodefault::autodefault;
use bevy::{prelude::{Commands, Res, ResMut, Query, With, Entity, Plugin, OnEnter, OnExit, IntoSystemConfigs, App, in_state, Update, Component, DetectChanges, Local}, text::TextStyle, window::Window};

use crate::{
    plugins::{
        assets::FontAssets,
        config::{VSync, LightSmoothness},
        ui::menu::{MenuContainer, despawn_with, MENU_BUTTON_COLOR, EnterMenu, builders::{menu, menu_button, control_buttons_layout, control_button}, components::MenuButton}
    },
    language::{keys::UIStringKey, LocalizedText, args},
    common::{state::{SettingsMenuState, MenuState}, conditions::on_click, systems::send_event},

};
use self::resolution::ResolutionMenuPlugin;

use super::{MENU_BUTTON_FONT_SIZE, BackButton};

pub(super) struct VideoMenuPlugin;
impl Plugin for VideoMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ResolutionMenuPlugin);

        app.add_systems(
            OnEnter(MenuState::Settings(SettingsMenuState::Video)),
            setup_video_menu
        );
        app.add_systems(
            OnExit(MenuState::Settings(SettingsMenuState::Video)),
            despawn_with::<VideoMenu>
        );

        app.add_systems(
            Update,
            (
                update_vsync_button_text,
                update_light_smoothness_button_text,
                send_event(EnterMenu(MenuState::Settings(SettingsMenuState::Resolution))).run_if(on_click::<ResolutionButton>),
                vsync_clicked.run_if(on_click::<VSyncButton>),
                light_smoothness_clicked.run_if(on_click::<LightSmoothnessButton>),
            )
            .run_if(in_state(MenuState::Settings(SettingsMenuState::Video)))
        );
    }
}

#[derive(Component)]
struct VideoMenu;

#[derive(Component, Clone)]
pub(super) struct ResolutionButton;

#[derive(Component, Clone)]
pub(super) struct VSyncButton;

#[derive(Component, Clone)]
pub(super) struct LightSmoothnessButton;

#[autodefault]
fn setup_video_menu(
    mut commands: Commands,
    fonts: Res<FontAssets>,
    vsync: Res<VSync>,
    light_smoothness: Res<LightSmoothness>,
    query_container: Query<Entity, With<MenuContainer>>
) {
    let text_style = TextStyle {
        font: fonts.andy_bold.clone_weak(),
        font_size: MENU_BUTTON_FONT_SIZE,
        color: MENU_BUTTON_COLOR,
    };

    let container = query_container.single();

    menu(VideoMenu, &mut commands, container, 5., |builder| {
        menu_button(
            builder,
            text_style.clone(),
            UIStringKey::Resolution,
            (MenuButton, ResolutionButton)
        );
        menu_button(
            builder,
            text_style.clone(),
            vsync_btn_text(vsync.0),
            (MenuButton, VSyncButton)
        );

        menu_button(
            builder,
            text_style.clone(),
            light_smoothness_btn_text(*light_smoothness),
            (MenuButton, LightSmoothnessButton)
        );

        control_buttons_layout(builder, |control_button_builder| {
            control_button(control_button_builder, text_style, UIStringKey::Back, (MenuButton, BackButton));
        });
    });
}

fn vsync_clicked(mut window: Query<&mut Window>, mut vsync: ResMut<VSync>) {
    vsync.0 = !vsync.0;
    window.single_mut().present_mode = vsync.as_present_mode();
}

fn light_smoothness_clicked(
    mut index: Local<u8>,
    mut light_smoothness: ResMut<LightSmoothness>
) {
    *index = (light_smoothness.to_u8() + 1) % LightSmoothness::length();
    *light_smoothness = LightSmoothness::new(*index);
}

fn update_vsync_button_text(
    vsync: Res<VSync>,
    mut query: Query<&mut LocalizedText, With<VSyncButton>>,
) {
    if vsync.is_changed() {
        let mut localized_text = query.single_mut();
        *localized_text = vsync_btn_text(vsync.0);
    }
}

fn update_light_smoothness_button_text(
    light_smoothness: Res<LightSmoothness>,
    mut query: Query<&mut LocalizedText, With<LightSmoothnessButton>>,
) {
    if light_smoothness.is_changed() {
        let mut localized_text = query.single_mut();
        *localized_text = light_smoothness_btn_text(*light_smoothness);
    }
}

#[inline(always)]
fn vsync_btn_text(vsync: bool) -> LocalizedText {
    let status = if vsync { UIStringKey::On } else { UIStringKey::Off };
    LocalizedText::new(UIStringKey::Vsync, "{} {}", args![status])
}

#[inline(always)]
fn light_smoothness_btn_text(light_smoothness: LightSmoothness) -> LocalizedText {
    LocalizedText::new(UIStringKey::LightSmoothness, "{} {}", args![light_smoothness.name()])
}
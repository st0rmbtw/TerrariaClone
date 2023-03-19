pub mod resolution;

use autodefault::autodefault;
use bevy::{prelude::{Commands, Res, Component, ResMut, Query, With, NextState, Entity, Plugin, IntoSystemAppConfig, OnEnter, OnExit, IntoSystemConfig, OnUpdate, IntoSystemConfigs, App}, text::{TextStyle, Text}, window::Window};

use crate::{plugins::{assets::FontAssets, menu::{menu_button, control_buttons_layout, control_button, menu, MenuContainer, despawn_with}, settings::VSync}, language::LanguageContent, TEXT_COLOR, common::{state::{SettingsMenuState, MenuState, GameState}, conditions::on_btn_clicked}};
use self::resolution::ResolutionMenuPlugin;

use super::{MENU_BUTTON_FONT_SIZE, BackButton};

pub(super) struct VideoMenuPlugin;
impl Plugin for VideoMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(ResolutionMenuPlugin);

        app.add_system(setup_video_menu.in_schedule(OnEnter(GameState::Menu(MenuState::Settings(SettingsMenuState::Video)))));
        app.add_system(despawn_with::<VideoMenu>.in_schedule(OnExit(GameState::Menu(MenuState::Settings(SettingsMenuState::Video)))));

        app.add_systems(
            (
                update_vsync_button_text,
                resolution_clicked.run_if(on_btn_clicked::<ResolutionButton>),
                vsync_clicked.run_if(on_btn_clicked::<VSyncButton>),
                back_clicked.run_if(on_btn_clicked::<BackButton>),
            )
            .chain()
            .in_set(OnUpdate(GameState::Menu(MenuState::Settings(SettingsMenuState::Video))))
        );
    }
}

#[derive(Component)]
struct VideoMenu;

#[derive(Component, Clone)]
pub(super) struct ResolutionButton;

#[derive(Component, Clone)]
pub(super) struct VSyncButton;

#[autodefault]
fn setup_video_menu(
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

    menu(VideoMenu, &mut commands, container, 50., |builder| {
        menu_button(builder, text_style.clone(), language_content.ui.resolution.clone(), ResolutionButton);
        menu_button(builder, text_style.clone(), language_content.ui.vsync.clone(), VSyncButton);

        control_buttons_layout(builder, |control_button_builder| {
            control_button(control_button_builder, text_style, language_content.ui.back.clone(), BackButton);
        });
    });
}

fn resolution_clicked(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::Menu(MenuState::Settings(SettingsMenuState::Resolution)));
}

fn back_clicked(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::Menu(MenuState::Settings(SettingsMenuState::Main)));
}

fn vsync_clicked(mut window: Query<&mut Window>, mut vsync: ResMut<VSync>) {
    vsync.0 = !vsync.0;
    window.single_mut().present_mode = vsync.as_present_mode();
}

fn update_vsync_button_text(
    mut query: Query<&mut Text, With<VSyncButton>>,
    vsync: Res<VSync>,
    language_content: Res<LanguageContent>
) {
    let mut text = query.single_mut();

    let status = if vsync.0 { language_content.ui.on.clone() } else { language_content.ui.off.clone() } ;

    text.sections[0].value = format!("{} {}", language_content.ui.vsync, status);
}
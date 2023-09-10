mod resolution;

use autodefault::autodefault;
use bevy::{prelude::{Commands, Res, ResMut, Query, With, Entity, Plugin, OnEnter, OnExit, IntoSystemConfigs, App, in_state, Update, Component}, text::{TextStyle, Text}, window::Window};

use crate::{
    plugins::{
        assets::FontAssets,
        config::VSync,
        ui::menu::{MenuContainer, despawn_with, MENU_BUTTON_COLOR, EnterMenu, builders::{menu, menu_button, control_buttons_layout, control_button}, components::MenuButton}
    },
    language::LanguageContent,
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
                send_event(EnterMenu(MenuState::Settings(SettingsMenuState::Resolution))).run_if(on_click::<ResolutionButton>),
                vsync_clicked.run_if(on_click::<VSyncButton>),
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
        color: MENU_BUTTON_COLOR,
    };

    let container = query_container.single();

    menu(VideoMenu, &mut commands, container, 5., |builder| {
        menu_button(builder, text_style.clone(), &language_content.ui.resolution, (MenuButton, ResolutionButton));
        menu_button(builder, text_style.clone(), &language_content.ui.vsync, (MenuButton, VSyncButton));

        control_buttons_layout(builder, |control_button_builder| {
            control_button(control_button_builder, text_style, &language_content.ui.back, (MenuButton, BackButton));
        });
    });
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

    let status = if vsync.0 { &language_content.ui.on } else { &language_content.ui.off };

    text.sections[0].value = format!("{} {}", language_content.ui.vsync, status);
}
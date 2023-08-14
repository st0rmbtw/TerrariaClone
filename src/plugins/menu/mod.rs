mod settings;
mod celestial_body;
mod components;
mod systems;
mod role;

use components::*;
use systems::*;

use bevy::{prelude::{Plugin, App, IntoSystemConfigs, OnEnter, OnExit, Color, Component, Startup, Update, Event, KeyCode, PostUpdate, Button}, input::common_conditions::input_just_pressed};

use crate::{common::{state::{GameState, MenuState, SettingsMenuState}, conditions::{on_btn_clicked, in_menu_state}, systems::{animate_button_scale, play_sound_on_hover}}, parallax::{parallax_animation_system, ParallaxSet}};

use self::{settings::SettingsMenuPlugin, celestial_body::CelestialBodyPlugin};

use super::slider::Slider;

pub(crate) const TEXT_COLOR: Color = Color::rgb(0.58, 0.58, 0.58);
pub(super) const MENU_BUTTON_FONT_SIZE: f32 = 42.;

pub(crate) struct MenuPlugin;
impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<BackEvent>();
        app.add_event::<EnterEvent>();

        app.add_plugins((CelestialBodyPlugin, SettingsMenuPlugin));

        app.add_systems(Startup, setup_camera);
        app.add_systems(
            OnExit(GameState::AssetLoading),
            (
                spawn_menu_container,
                play_music
            )
        );

        app.add_systems(OnEnter(GameState::Menu(MenuState::Main)), setup_main_menu);
        app.add_systems(OnExit(GameState::Menu(MenuState::Main)), despawn_with::<Menu>);

        app.add_systems(OnEnter(GameState::InGame), despawn_with::<DespawnOnMenuExit>);

        app.add_systems(
            Update,
            (
                send_back_event.run_if(on_btn_clicked::<BackButton>),
                send_back_event.run_if(input_just_pressed(KeyCode::Escape)),
            )
        );

        app.add_systems(
            PostUpdate,
            (
                handle_back_event,
                handle_enter_event    
            ).run_if(in_menu_state)
        );
        
        app.add_systems(
            Update,
            (
                parallax_animation_system(150.).in_set(ParallaxSet::FollowCamera),
                animate_button_scale::<Button>,
                animate_button_color,
                animate_slider_border_color,
                play_sound_on_hover::<Button>,
                play_sound_on_hover::<Slider>,
            )
            .run_if(in_menu_state)
        );

        app.add_systems(
            Update,
            (
                send_enter_event(GameState::WorldLoading)
                    .run_if(on_btn_clicked::<SinglePlayerButton>),
                send_enter_event(GameState::Menu(MenuState::Settings(SettingsMenuState::Main)))
                    .run_if(on_btn_clicked::<SettingsButton>),
                exit_clicked.run_if(on_btn_clicked::<ExitButton>),
            )
            .run_if(in_menu_state)
        );
    }
}

#[derive(Component)]
pub(crate) struct DespawnOnMenuExit;

#[derive(Component)]
pub(super) struct BackButton;

#[derive(Component)]
pub(super) struct ApplyButton;

#[derive(Event)]
pub(super) struct BackEvent;

#[derive(Event)]
pub(super) struct EnterEvent(pub(super) GameState);
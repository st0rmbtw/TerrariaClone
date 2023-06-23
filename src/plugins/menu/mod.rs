mod settings;
mod celestial_body;
mod components;
mod systems;
mod role;

use components::*;
use systems::*;

use bevy::prelude::{Plugin, App, IntoSystemAppConfig, IntoSystemConfigs, IntoSystemConfig, OnEnter, OnExit, Color, Component};

use crate::{common::{state::{GameState, MenuState}, conditions::{on_btn_clicked, in_menu_state}}, parallax::{parallax_animation_system, ParallaxSet}};

use self::{settings::SettingsMenuPlugin, celestial_body::CelestialBodyPlugin};

pub(crate) const TEXT_COLOR: Color = Color::rgb(0.58, 0.58, 0.58);

#[derive(Component)]
pub(crate) struct DespawnOnMenuExit;

pub(crate) struct MenuPlugin;
impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(CelestialBodyPlugin);
        app.add_plugin(SettingsMenuPlugin);

        app.add_system(setup_camera.on_startup());
        app.add_system(spawn_menu_container.in_schedule(OnExit(GameState::AssetLoading)));

        app.add_system(setup_main_menu.in_schedule(OnEnter(GameState::Menu(MenuState::Main))));
        app.add_system(despawn_with::<Menu>.in_schedule(OnExit(GameState::Menu(MenuState::Main))));

        app.add_system(
            despawn_with::<DespawnOnMenuExit>.in_schedule(OnEnter(GameState::InGame))
        );
        
        app.add_systems(
            (
                parallax_animation_system(150.).in_set(ParallaxSet::FollowCamera),
                update_buttons
            )
            .distributive_run_if(in_menu_state)
        );

        app.add_systems(
            (
                single_player_clicked.run_if(on_btn_clicked::<SinglePlayerButton>),
                settings_clicked.run_if(on_btn_clicked::<SettingsButton>),
                exit_clicked.run_if(on_btn_clicked::<ExitButton>),
            )
            .distributive_run_if(in_menu_state)
        );
    }
}
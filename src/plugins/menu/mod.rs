mod settings;
mod celestial_body;
mod components;
mod systems;
mod role;

use components::*;
use systems::*;

use bevy::prelude::{Plugin, App, IntoSystemConfigs, OnEnter, OnExit, Color, Component, Startup, Update};

use crate::{common::{state::{GameState, MenuState}, conditions::{on_btn_clicked, in_menu_state}}, parallax::{parallax_animation_system, ParallaxSet}};

use self::{settings::SettingsMenuPlugin, celestial_body::CelestialBodyPlugin};

pub(crate) const TEXT_COLOR: Color = Color::rgb(0.58, 0.58, 0.58);

#[derive(Component)]
pub(crate) struct DespawnOnMenuExit;

pub(crate) struct MenuPlugin;
impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((CelestialBodyPlugin, SettingsMenuPlugin));

        app.add_systems(Startup, setup_camera);
        app.add_systems(OnExit(GameState::AssetLoading), spawn_menu_container);

        app.add_systems(OnEnter(GameState::Menu(MenuState::Main)), setup_main_menu);
        app.add_systems(OnExit(GameState::Menu(MenuState::Main)), despawn_with::<Menu>);

        app.add_systems(OnEnter(GameState::InGame), despawn_with::<DespawnOnMenuExit>);
        
        app.add_systems(
            Update,
            (
                parallax_animation_system(150.).in_set(ParallaxSet::FollowCamera),
                update_buttons
            )
            .run_if(in_menu_state)
        );

        app.add_systems(
            Update,
            (
                single_player_clicked.run_if(on_btn_clicked::<SinglePlayerButton>),
                settings_clicked.run_if(on_btn_clicked::<SettingsButton>),
                exit_clicked.run_if(on_btn_clicked::<ExitButton>),
            )
            .run_if(in_menu_state)
        );
    }
}
mod settings;
mod celestial_body;
mod components;
mod systems;
mod role;

pub use components::*;
pub use systems::*;

use bevy::prelude::{Plugin, App, IntoSystemAppConfig, IntoSystemConfigs, IntoSystemConfig, OnEnter, OnExit};

use crate::{common::{state::{GameState, MenuState}, conditions::{on_btn_clicked, in_menu_state}}, parallax::move_background_system};

use self::{celestial_body::CelestialBodyPlugin, settings::SettingsMenuPlugin};

use super::{camera::MainCamera, fps::FpsText, background::Star};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(CelestialBodyPlugin);
        app.add_plugin(SettingsMenuPlugin);

        app.add_system(setup_camera.on_startup());
        app.add_system(spawn_menu_container.in_schedule(OnExit(GameState::AssetLoading)));
        app.add_system(setup_main_menu.in_schedule(OnEnter(GameState::Menu(MenuState::Main))));

        app.add_system(despawn_with::<MainCamera>.in_schedule(OnEnter(GameState::InGame)));
        app.add_system(despawn_with::<MenuContainer>.in_schedule(OnEnter(GameState::InGame)));
        app.add_system(despawn_with::<FpsText>.in_schedule(OnEnter(GameState::InGame)));
        app.add_system(despawn_with::<Star>.in_schedule(OnEnter(GameState::InGame)));

        app.add_system(despawn_with::<Menu>.in_schedule(OnExit(GameState::Menu(MenuState::Main))));

        app.add_system(move_background_system().run_if(in_menu_state));
        app.add_system(update_buttons.run_if(in_menu_state));

        app.add_systems(
            (
                single_player_clicked.run_if(on_btn_clicked::<SinglePlayerButton>),
                settings_clicked.run_if(on_btn_clicked::<SettingsButton>),
                exit_clicked.run_if(on_btn_clicked::<ExitButton>),
            )
            .chain()
            .distributive_run_if(in_menu_state)
        );
    }
}
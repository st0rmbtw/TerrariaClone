mod components;
mod systems;
mod celestial_body;

pub use components::*;
pub use systems::*;

use bevy::prelude::{Plugin, App, Res, IntoSystemAppConfig, OnUpdate, State, IntoSystemConfigs, IntoSystemConfig, OnEnter, OnExit};

use crate::{state::GameState, util::on_btn_clicked, parallax::move_background_system};

use self::celestial_body::CelestialBodyPlugin;

use super::camera::MainCamera;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(CelestialBodyPlugin);

        app.add_system(setup_camera.on_startup());
        app.add_system(setup_main_menu.in_schedule(OnEnter(GameState::MainMenu)));

        app.add_system(despawn_with::<MainCamera>.in_schedule(OnEnter(GameState::InGame)));
        app.add_system(despawn_with::<Menu>.in_schedule(OnExit(GameState::MainMenu)));

        app.add_systems(
            (
                move_background_system(),
                update_buttons
            )
            .chain()
            .distributive_run_if(in_menu_state)
        );

        app.add_systems(
            (
                single_player_clicked.run_if(on_btn_clicked::<SinglePlayerButton>),
                settings_clicked.run_if(on_btn_clicked::<SettingsButton>),
                exit_clicked.run_if(on_btn_clicked::<ExitButton>),
            ).chain().in_set(OnUpdate(GameState::MainMenu))
        );
    }
}

fn in_menu_state(state: Res<State<GameState>>) -> bool {
    matches!(&state.0, GameState::MainMenu | GameState::Settings(_))
}
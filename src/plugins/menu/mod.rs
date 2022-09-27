mod components;
mod systems;

pub use components::*;
use iyes_loopless::prelude::{ConditionSet, AppLooplessStateExt, IntoConditionalSystem};
pub use systems::*;

use bevy::prelude::{Plugin, App};

use crate::{state::GameState, util::on_btn_clicked, parallax::move_background_system};

use super::camera::MainCamera;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_camera)
            .add_enter_system(GameState::MainMenu, setup_main_menu)
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::MainMenu)
                    .with_system(move_background_system())
                    .with_system(update_buttons)
                    .with_system(single_player_btn.run_if(on_btn_clicked::<SinglePlayerButton>))
                    .with_system(exit_btn.run_if(on_btn_clicked::<ExitButton>))
                    .into(),
            )
            .add_exit_system(GameState::MainMenu, despawn_with::<MainCamera>)
            .add_exit_system(GameState::MainMenu, despawn_with::<Menu>);
    }
}
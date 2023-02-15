mod components;
mod systems;

pub use components::*;
use iyes_loopless::{prelude::{ConditionSet, AppLooplessStateExt, IntoConditionalSystem}, state::CurrentState};
pub use systems::*;

use bevy::{prelude::{Plugin, App, Res}, text::Text};

use crate::{state::GameState, util::on_btn_clicked, parallax::move_background_system, animation::{component_animator_system, AnimationSystem}};

use super::camera::MainCamera;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_startup_system(setup_camera)
            .add_enter_system(GameState::MainMenu, setup_main_menu)
            .add_system_set(
                ConditionSet::new()
                    .run_if(in_menu_state)
                    .with_system(move_background_system())
                    .with_system(update_buttons)
                    .into(),
            )
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::MainMenu)
                    .with_system(single_player_clicked.run_if(on_btn_clicked::<SinglePlayerButton>))
                    .with_system(settings_clicked.run_if(on_btn_clicked::<SettingsButton>))
                    .with_system(exit_clicked.run_if(on_btn_clicked::<ExitButton>))
                    .into()
            )
            .add_system(
                component_animator_system::<Text>
                    .run_if(in_menu_state)
                    .label(AnimationSystem::AnimationUpdate)
            )
            .add_enter_system(GameState::InGame, despawn_with::<MainCamera>)
            .add_exit_system(GameState::MainMenu, despawn_with::<Menu>);
    }
}

fn in_menu_state(state: Res<CurrentState<GameState>>) -> bool {
    match &state.0 {
        GameState::MainMenu | GameState::Settings => true,
        _ => false
    }
}
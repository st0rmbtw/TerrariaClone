use bevy::prelude::{Plugin, App};
use iyes_loopless::prelude::{AppLooplessStateExt, IntoConditionalSystem};
use leafwing_input_manager::prelude::InputManagerPlugin;

use crate::{state::GameState, labels::PlayerLabel};

pub use components::*;
pub use systems::*;

mod components;
mod systems;

const MAX_CAMERA_ZOOM: f32 = 1.1;
const MIN_CAMERA_ZOOM: f32 = 0.46;
const CAMERA_ZOOM_STEP: f32 = 0.3;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugin(InputManagerPlugin::<MouseAction>::default())
            .add_enter_system(GameState::InGame, setup_camera)
            .add_system(zoom.run_in_state(GameState::InGame))
            .add_system(
                move_camera
                    .run_in_state(GameState::InGame)
                    .after(PlayerLabel::Update)
            );
    }
}
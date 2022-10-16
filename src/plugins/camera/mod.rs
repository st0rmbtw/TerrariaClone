use bevy::prelude::{Plugin, App};
use iyes_loopless::prelude::{ConditionSet, AppLooplessStateExt};

use crate::{state::GameState, labels::PlayerLabel};

pub use components::*;
pub use systems::*;

mod components;
mod systems;

const MAX_CAMERA_ZOOM: f32 = 1.1;
const MIN_CAMERA_ZOOM: f32 = 0.2;
const CAMERA_ZOOM_STEP: f32 = 0.3;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_enter_system(GameState::InGame, setup_camera)
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::InGame)
                    .with_system(zoom)
                    .into()
            )
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::InGame)
                    .after(PlayerLabel::MovePlayer)
                    .with_system(move_camera)
                    .into(),
            );
    }
}
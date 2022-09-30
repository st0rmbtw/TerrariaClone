use bevy::prelude::{Plugin, App, CoreStage};
use iyes_loopless::prelude::{ConditionSet, AppLooplessStateExt};

use crate::state::GameState;

pub use components::*;
pub use systems::*;

mod components;
mod systems;

const MAX_CAMERA_ZOOM: f32 = 1.1;
const MIN_CAMERA_ZOOM: f32 = 0.5;
const CAMERA_ZOOM_STEP: f32 = 0.3;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system(zoom)
            .add_enter_system(GameState::InGame, setup_camera)
            .add_system_set_to_stage(
                CoreStage::PostUpdate,
                ConditionSet::new()
                    .run_in_state(GameState::InGame)
                    .with_system(move_camera)
                    .into(),
            );
    }
}
use bevy::prelude::{Plugin, App, Component, CoreStage};
use iyes_loopless::prelude::{AppLooplessStateExt, IntoConditionalSystem};

use crate::state::GameState;

pub use components::*;
pub use systems::*;
pub use events::*;

mod components;
mod systems;
mod events;

const MAX_CAMERA_ZOOM: f32 = 2.;
const MIN_CAMERA_ZOOM: f32 = 0.2;
const CAMERA_ZOOM_STEP: f32 = 0.3;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_enter_system(GameState::InGame, setup_camera)
            .add_system(zoom.run_in_state(GameState::InGame))
            .add_system(control_mouse_light.run_in_state(GameState::InGame))
            .add_system_to_stage(
                CoreStage::PostUpdate,
                move_camera
                    .run_in_state(GameState::InGame)
            );
    }
}
#[derive(Component)]
pub struct MouseLight;
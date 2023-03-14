use bevy::prelude::{Plugin, App, Component, OnUpdate, IntoSystemConfig, CoreSet, in_state, IntoSystemAppConfig, OnEnter};

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
        app.add_system(setup_camera.in_schedule(OnEnter(GameState::InGame)));
        app.add_system(zoom.in_set(OnUpdate(GameState::InGame)));
        app.add_system(control_mouse_light.in_set(OnUpdate(GameState::InGame)));
        app.add_system(move_camera.in_set(OnUpdate(GameState::InGame)));
    }
}
#[derive(Component)]
pub struct MouseLight;
use bevy::{prelude::{Plugin, App, Component, OnUpdate, IntoSystemConfig, IntoSystemAppConfig, OnEnter, Res, KeyCode}, input::common_conditions::input_just_pressed};

use crate::{common::{state::GameState, helpers::toggle_visibility}, DebugConfiguration};

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
        app.add_system(
            toggle_visibility::<MouseLight>
                .run_if(input_just_pressed(KeyCode::F1))
                .in_set(OnUpdate(GameState::InGame))
        );

        app.add_system(
            follow_player
                .in_set(OnUpdate(GameState::InGame))
                .run_if(|debug_config: Res<DebugConfiguration>| !debug_config.free_camera)
        );

        #[cfg(feature = "debug")]
        app.add_system(
            free_camera
                .in_set(OnUpdate(GameState::InGame))
                .run_if(|debug_config: Res<DebugConfiguration>| debug_config.free_camera)
        );
    }
}

#[derive(Component)]
pub struct MouseLight;
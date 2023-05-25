use bevy::{prelude::{Plugin, App, OnUpdate, IntoSystemConfig, IntoSystemAppConfig, OnEnter, Res, SystemSet, IntoSystemSetConfig, CoreSet, in_state}};

use crate::{common::state::GameState, DebugConfiguration};

pub(crate) use components::*;
pub(crate) use events::*;
use systems::*;

mod components;
mod systems;
mod events;

const MAX_CAMERA_ZOOM: f32 = 2.;
const MIN_CAMERA_ZOOM: f32 = 0.2;
const CAMERA_ZOOM_STEP: f32 = 0.5;

#[cfg(feature = "debug")]
const CAMERA_MOVE_SPEED: f32 = 1000.;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub(crate) enum CameraSet {
    MoveCamera
}

pub(crate) struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(setup_camera.in_schedule(OnEnter(GameState::InGame)));
        
        app.add_system(zoom.in_set(OnUpdate(GameState::InGame)));
        
        app.configure_set(
            CameraSet::MoveCamera
                .run_if(in_state(GameState::InGame))
                .in_base_set(CoreSet::PostUpdate)
        );

        app.add_system(
            follow_player
                .in_set(CameraSet::MoveCamera)
                .run_if(|debug_config: Res<DebugConfiguration>| !debug_config.free_camera)
        );

        #[cfg(feature = "debug")]
        app.add_system(
            free_camera
                .in_set(CameraSet::MoveCamera)
                .run_if(|debug_config: Res<DebugConfiguration>| debug_config.free_camera)
        );
    }
}

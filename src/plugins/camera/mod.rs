use bevy::{prelude::{Plugin, App, OnUpdate, IntoSystemConfig, IntoSystemAppConfig, OnEnter, SystemSet, IntoSystemSetConfig, CoreSet, in_state}, transform::TransformSystem};

use crate::common::state::GameState;

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
                .before(TransformSystem::TransformPropagate)
        );

        #[cfg(not(feature = "debug"))]
        app.add_system(follow_player.in_set(CameraSet::MoveCamera));
        
        #[cfg(feature = "debug")] {
            use crate::plugins::debug::DebugConfiguration;
            use bevy::prelude::Res;

            app.add_system(
                follow_player
                    .in_set(CameraSet::MoveCamera)
                    .run_if(|debug_config: Res<DebugConfiguration>| !debug_config.free_camera)
            );

            app.add_system(
                free_camera
                    .in_set(CameraSet::MoveCamera)
                    .run_if(|debug_config: Res<DebugConfiguration>| debug_config.free_camera)
            );
        }
    }
}

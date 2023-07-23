use bevy::{prelude::{Plugin, App, OnEnter, SystemSet, IntoSystemSetConfig, in_state, IntoSystemConfigs, Update, PostUpdate}, transform::TransformSystem};

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
#[cfg(feature = "debug")]
const CAMERA_MOVE_SPEED_FASTER: f32 = CAMERA_MOVE_SPEED * 2.;
#[cfg(feature = "debug")]
const CAMERA_MOVE_SPEED_SLOWER: f32 = CAMERA_MOVE_SPEED / 2.;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub(crate) enum CameraSet {
    MoveCamera
}

pub(crate) struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.configure_set(PostUpdate,
            CameraSet::MoveCamera
                .run_if(in_state(GameState::InGame))
                .before(TransformSystem::TransformPropagate)
        );

        app.add_systems(OnEnter(GameState::InGame), setup_camera);
        app.add_systems(Update, zoom.run_if(in_state(GameState::InGame)));
        app.add_systems(
            PostUpdate,
            (
                move_camera,
                keep_camera_inside_world_bounds
            )
            .chain()
            .in_set(CameraSet::MoveCamera)
        );
    }
}

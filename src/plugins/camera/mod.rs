use bevy::{prelude::{Plugin, App, SystemSet, PostUpdate, IntoSystemConfigs, OnExit, Update, OnEnter}, transform::TransformSystem};

use crate::common::state::GameState;

use super::InGameSystemSet;

#[cfg(feature = "debug")]
use super::debug::DebugConfiguration;
#[cfg(feature = "debug")]
use bevy::prelude::Res;

pub(crate) mod components;
pub(crate) mod resources;
mod systems;

const MIN_CAMERA_ZOOM: f32 = 0.5;
const MAX_CAMERA_ZOOM: f32 = 1.;
const CAMERA_ZOOM_STEP: f32 = 1.;

#[cfg(feature = "debug")]
const CAMERA_MOVE_SPEED: f32 = 1000.;
#[cfg(feature = "debug")]
const CAMERA_MOVE_SPEED_FASTER: f32 = CAMERA_MOVE_SPEED * 3.;
#[cfg(feature = "debug")]
const CAMERA_MOVE_SPEED_SLOWER: f32 = CAMERA_MOVE_SPEED / 2.;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub(crate) enum CameraSet {
    MoveCamera
}

pub(crate) struct CameraPlugin;
impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnExit(GameState::WorldLoading),
            (systems::setup_main_camera, systems::setup_world_camera)
        );

        app.add_systems(OnEnter(GameState::InGame), systems::init_camera_position);

        app.add_systems(
            Update,
            (
                systems::zoom,
                systems::update_camera_scale
            )
            .chain()
            .in_set(InGameSystemSet::Update)
        );

        #[cfg(feature = "debug")]
        let move_camera = (
            systems::follow_player.run_if(|config: Res<DebugConfiguration>| !config.free_camera),
            systems::free_camera.run_if(|config: Res<DebugConfiguration>| config.free_camera)
        );

        #[cfg(not(feature = "debug"))]
        let move_camera = systems::follow_player;

        app.add_systems(
            PostUpdate,
            (
                move_camera,
                systems::keep_camera_inside_world_bounds
            )
            .chain()
            .before(TransformSystem::TransformPropagate)
            .in_set(CameraSet::MoveCamera)
            .in_set(InGameSystemSet::PostUpdate)
        );
    }
}

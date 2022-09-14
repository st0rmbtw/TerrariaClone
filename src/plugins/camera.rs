use autodefault::autodefault;
use bevy::prelude::*;
use iyes_loopless::prelude::{AppLooplessStateExt, ConditionSet};

use crate::{parallax::ParallaxCameraComponent, state::GameState, world_generator::WORLD_SIZE_X};

use super::{Player, TILE_SIZE};

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

#[derive(Clone, Copy)]
pub struct CameraEntity {
    pub main_camera: Entity
}

#[derive(Component)]
pub struct MainCamera;

const MAX_CAMERA_ZOOM: f32 = 1.1;
const MIN_CAMERA_ZOOM: f32 = 0.5;
const CAMERA_ZOOM_STEP: f32 = 0.3;

#[autodefault(except(CameraEntity))]
fn setup_camera(mut commands: Commands) {
    let main_camera = commands
        .spawn()
        .insert_bundle(Camera2dBundle {
            projection: OrthographicProjection { scale: 0.9 },
            transform: Transform::from_xyz(0., 0., 500.),
        })
        .insert(ParallaxCameraComponent)
        .insert(MainCamera)
        .id();

    commands.insert_resource(CameraEntity {
        main_camera
    })
}

fn zoom(
    time: Res<Time>,
    input: Res<Input<KeyCode>>,
    mut camera_query: Query<&mut OrthographicProjection, With<MainCamera>>,
) {
    if let Ok(mut projection) = camera_query.get_single_mut() {
        if input.pressed(KeyCode::Equals) {
            let scale = projection.scale - (CAMERA_ZOOM_STEP * time.delta_seconds());

            projection.scale = scale.max(MIN_CAMERA_ZOOM);
        }

        if input.pressed(KeyCode::Minus) {
            let scale = projection.scale + (CAMERA_ZOOM_STEP * time.delta_seconds());

            projection.scale = scale.min(MAX_CAMERA_ZOOM);
        }
    }
}

fn move_camera(
    mut player: Query<&Transform, With<Player>>,
    mut camera: Query<(&mut GlobalTransform, &OrthographicProjection), With<MainCamera>>,
) {
    if let Ok((mut camera_transform, projection)) = camera.get_single_mut() {
        if let Ok(player_transform) = player.get_single_mut() {
            let camera_translation = camera_transform.translation_mut();

            let projection_left = projection.left * projection.scale;
            let projection_right = projection.right * projection.scale;
            
            {
                let min = projection_left.abs() - TILE_SIZE / 2.;
                let max = (WORLD_SIZE_X as f32 * 16.) - projection_right - TILE_SIZE / 2.;
                camera_translation.x = player_transform.translation.x.clamp(min, max);
            }

            camera_translation.y = player_transform.translation.y;
        }
    }
}

use autodefault::autodefault;
use bevy::{
    prelude::{
        Commands, Camera2dBundle, OrthographicProjection, Transform, Res, Input, KeyCode, Query, 
        With, GlobalTransform
    }, 
    time::Time, 
    render::camera::DepthCalculation
};

use crate::{parallax::ParallaxCameraComponent, plugins::{player::Player, world::TILE_SIZE}, world_generator::{WORLD_SIZE_X, WORLD_SIZE_Y}};

use super::{MainCamera, CAMERA_ZOOM_STEP, MIN_CAMERA_ZOOM, MAX_CAMERA_ZOOM};

#[autodefault]
pub fn setup_camera(mut commands: Commands) {
    commands
        .spawn()
        .insert_bundle(Camera2dBundle {
            projection: OrthographicProjection { 
                scale: 0.9, 
                depth_calculation: DepthCalculation::ZDifference 
            },
            transform: Transform::from_xyz(0., 0., 500.),
        })
        .insert(ParallaxCameraComponent)
        .insert(MainCamera);
}

pub fn zoom(
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

pub fn move_camera(
    mut player: Query<&Transform, With<Player>>,
    mut camera: Query<(&mut GlobalTransform, &OrthographicProjection), With<MainCamera>>,
) {
    if let Ok((mut camera_transform, projection)) = camera.get_single_mut() {
        if let Ok(player_transform) = player.get_single_mut() {
            let camera_translation = camera_transform.translation_mut();

            let projection_left = projection.left * projection.scale;
            let projection_right = projection.right * projection.scale;
            let projection_top = projection.top * projection.scale;
            
            {
                let min = projection_left.abs() - TILE_SIZE / 2.;
                let max = (WORLD_SIZE_X as f32 * 16.) - projection_right - TILE_SIZE / 2.;
                camera_translation.x = player_transform.translation.x.clamp(min, max);
            }
            {
                let max = -(projection_top - TILE_SIZE / 2.);
                let min = -((WORLD_SIZE_Y as f32 * 16.) + projection_top + TILE_SIZE / 2.);
                camera_translation.y = player_transform.translation.y.clamp(min, max);
            }
        }
    }
}
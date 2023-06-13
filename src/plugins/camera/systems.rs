use autodefault::autodefault;
use bevy::{
    prelude::{
        Commands, Camera2dBundle, OrthographicProjection, Transform, Res, KeyCode, Query, 
        With, Input,
        Without, Changed, Camera2d,
    }, 
    time::Time, core_pipeline::clear_color::ClearColorConfig
};
use interpolation::Lerp;

use crate::{plugins::world::TILE_SIZE, common::helpers::tile_pos_to_world_coords, world::WorldData};

use crate::plugins::player::Player;

use super::{MainCamera, CAMERA_ZOOM_STEP, MIN_CAMERA_ZOOM, MAX_CAMERA_ZOOM, LightMapCamera, BackgroundCamera};

#[autodefault(except(TextureDescriptor, ShadowMapMaterial, LightMapMaterial, SunMaterial, LightingMaterial))]
pub(super) fn setup_camera(
    mut commands: Commands,
    world_data: Res<WorldData>
) {
    let player_spawn_point = tile_pos_to_world_coords(world_data.spawn_point);

    commands
        .spawn((
            MainCamera,
            LightMapCamera,
            Camera2dBundle {
                projection: OrthographicProjection { 
                    scale: 0.9
                },
                transform: Transform::from_xyz(player_spawn_point.x, player_spawn_point.y, 500.),
                camera_2d: Camera2d {
                    clear_color: ClearColorConfig::None
                }
            }
        ));
}

pub(super) fn zoom(
    time: Res<Time>,
    input: Res<Input<KeyCode>>,
    mut camera_query: Query<&mut OrthographicProjection, With<MainCamera>>,
) {
    let mut projection = camera_query.single_mut();

    if input.pressed(KeyCode::Equals) {
        let scale = projection.scale - (CAMERA_ZOOM_STEP * time.delta_seconds());

        projection.scale = scale.max(MIN_CAMERA_ZOOM);
    }

    if input.pressed(KeyCode::Minus) {
        let scale = projection.scale + (CAMERA_ZOOM_STEP * time.delta_seconds());

        projection.scale = scale.min(MAX_CAMERA_ZOOM);
    }
}

pub(super) fn follow_player(
    mut player: Query<&Transform, (With<Player>, Without<MainCamera>, Changed<Transform>)>,
    mut query_main_camera: Query<(&mut Transform, &OrthographicProjection), (With<MainCamera>, Without<Player>)>,
    mut background_camera: Query<&mut Transform, (With<BackgroundCamera>, Without<MainCamera>, Without<Player>)>,
    world_data: Res<WorldData>
) {
    if let Ok((mut camera_transform, projection)) = query_main_camera.get_single_mut() {
        let background_camera_transform = background_camera.get_single_mut().ok();

        if let Ok(player_transform) = player.get_single_mut() {
            let projection_left = projection.area.min.x;
            let projection_right = projection.area.max.x;
            let projection_top = projection.area.max.y;
            
            {
                let min = projection_left.abs() - TILE_SIZE / 2.;
                let max = (world_data.size.width as f32 * 16.) - projection_right - TILE_SIZE / 2.;
                camera_transform.translation.x = camera_transform.translation.x
                    .lerp(&player_transform.translation.x.clamp(min, max), &0.5);
            }
            {
                let min = -(world_data.size.height as f32 * 16.) - projection_top - TILE_SIZE / 2.;
                let max = -projection_top - TILE_SIZE / 2.;
                camera_transform.translation.y = camera_transform.translation.y
                    .lerp(&player_transform.translation.y.clamp(min, max), &0.5);
            }

            if let Some(mut transform) = background_camera_transform {
                transform.translation = camera_transform.translation;
            }
        }
    }
}

#[cfg(feature = "debug")]
pub(super) fn free_camera(
    time: Res<Time>,
    mut query_main_camera: Query<(&mut Transform, &OrthographicProjection), With<MainCamera>>,
    mut query_background_camera: Query<&mut Transform, (With<BackgroundCamera>, Without<MainCamera>)>,
    input: Res<bevy::prelude::Input<KeyCode>>,
    world_data: Res<WorldData>
) {
    use bevy::prelude::Vec2;

    use super::CAMERA_MOVE_SPEED;

    if let Ok((mut main_camera_transform, projection)) = query_main_camera.get_single_mut() {
        let background_camera_transform = query_background_camera.get_single_mut().ok();

        let camera_speed = if input.pressed(KeyCode::LShift) {
            CAMERA_MOVE_SPEED * 2.
        } else if input.pressed(KeyCode::LAlt) {
            CAMERA_MOVE_SPEED / 2.
        } else {
            CAMERA_MOVE_SPEED
        };

        let mut move_direction = Vec2::new(0., 0.);

        if input.pressed(KeyCode::A) {
            move_direction.x = -1.;
        }
        if input.pressed(KeyCode::D) {
            move_direction.x = 1.;
        }
        if input.pressed(KeyCode::W) {
            move_direction.y = 1.;
        }
        if input.pressed(KeyCode::S) {
            move_direction.y = -1.;
        }

        let projection_left = projection.area.min.x;
        let projection_right = projection.area.max.x;
        let projection_top = projection.area.max.y;

        {
            let min = projection_left.abs() - TILE_SIZE / 2.;
            let max = (world_data.size.width as f32 * 16.) - projection_right - TILE_SIZE / 2.;

            let velocity = move_direction.x * camera_speed * time.delta_seconds();
            let new_x = (main_camera_transform.translation.x + velocity).clamp(min, max);

            main_camera_transform.translation.x = new_x;
        }
        {
            let min = -(world_data.size.height as f32 * 16.) - projection_top - TILE_SIZE / 2.;
            let max = -projection_top - TILE_SIZE / 2.;

            let velocity = move_direction.y * camera_speed * time.delta_seconds();
            let new_y = (main_camera_transform.translation.y + velocity).clamp(min, max);

            main_camera_transform.translation.y = new_y;
        }

        if let Some(mut transform) = background_camera_transform {
            transform.translation = main_camera_transform.translation;
        }
    }
}
use autodefault::autodefault;
use bevy::{
    prelude::{
        Commands, Camera2dBundle, OrthographicProjection, Transform, Res, KeyCode, Query, 
        With, Input,
        Without, Changed
    }, 
    time::Time
};
use interpolation::Lerp;

use crate::{parallax::ParallaxCameraComponent, plugins::{world::TILE_SIZE}, common::helpers::tile_pos_to_world_coords, world::WorldData};

use crate::plugins::player::Player;

use super::{MainCamera, CAMERA_ZOOM_STEP, MIN_CAMERA_ZOOM, MAX_CAMERA_ZOOM, LightMapCamera};

#[autodefault(except(TextureDescriptor, ShadowMapMaterial, LightMapMaterial, SunMaterial, LightingMaterial))]
pub(super) fn setup_camera(
    mut commands: Commands,
    world_data: Res<WorldData>
) {
    let player_spawn_point = tile_pos_to_world_coords(world_data.spawn_point);

    commands
        .spawn((
            MainCamera,
            ParallaxCameraComponent,
            LightMapCamera,
            Camera2dBundle {
                projection: OrthographicProjection { 
                    scale: 0.9
                },
                transform: Transform::from_xyz(player_spawn_point.x, player_spawn_point.y, 500.)
            }
        ));
}

pub(super) fn zoom(
    time: Res<Time>,
    input: Res<Input<KeyCode>>,
    mut camera_query: Query<&mut OrthographicProjection>,
) {
    for mut projection in &mut camera_query {
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

pub(super) fn follow_player(
    mut player: Query<&Transform, (With<Player>, Without<MainCamera>, Changed<Transform>)>,
    mut camera: Query<(&mut Transform, &OrthographicProjection), (With<MainCamera>, Without<Player>)>,
    world_data: Res<WorldData>
) {
    if let Ok((mut camera_transform, projection)) = camera.get_single_mut() {
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
        }
    }
}

#[cfg(feature = "debug")]
pub(super) fn free_camera(
    time: Res<Time>,
    mut camera: Query<(&mut Transform, &OrthographicProjection), With<MainCamera>>,
    input: Res<bevy::prelude::Input<KeyCode>>,
    world_data: Res<WorldData>
) {
    use bevy::prelude::Vec2;

    use super::CAMERA_MOVE_SPEED;

    if let Ok((mut camera_transform, projection)) = camera.get_single_mut() {
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
            let velocity = move_direction.x * camera_speed * time.delta_seconds();
            let new_position = camera_transform.translation.x + velocity;

            let min = projection_left.abs() - TILE_SIZE / 2.;
            let max = (world_data.size.width as f32 * 16.) - projection_right - TILE_SIZE / 2.;

            camera_transform.translation.x = new_position.clamp(min, max);
        }
        {
            let velocity = move_direction.y * camera_speed * time.delta_seconds();
            let new_position = camera_transform.translation.y + velocity;

            let min = -(world_data.size.height as f32 * 16.) - projection_top - TILE_SIZE / 2.;
            let max = -projection_top - TILE_SIZE / 2.;

            camera_transform.translation.y = new_position.clamp(min, max);
        }
    }
}
use autodefault::autodefault;
use bevy::{
    prelude::{
        Commands, Camera2dBundle, OrthographicProjection, Transform, Res, KeyCode, Query, 
        With, Input,
        Without, Camera2d, Name, Mut, GlobalTransform,
    }, 
    time::Time, core_pipeline::clear_color::ClearColorConfig
};

use crate::{plugins::world::constants::TILE_SIZE, common::helpers::tile_pos_to_world_coords, world::WorldData};

use crate::plugins::player::Player;

use super::{CAMERA_ZOOM_STEP, MIN_CAMERA_ZOOM, MAX_CAMERA_ZOOM, components::{MainCamera, LightMapCamera, BackgroundCamera}, INITIAL_ZOOM};

#[autodefault(except(TextureDescriptor, ShadowMapMaterial, LightMapMaterial, SunMaterial, LightingMaterial))]
pub(super) fn setup_camera(
    mut commands: Commands,
    world_data: Res<WorldData>
) {
    let player_spawn_point = tile_pos_to_world_coords(world_data.spawn_point);

    commands
        .spawn((
            Name::new("MainCamera"),
            MainCamera,
            LightMapCamera,
            Camera2dBundle {
                projection: OrthographicProjection { 
                    scale: INITIAL_ZOOM
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
    mut query_camera: Query<&mut OrthographicProjection, With<MainCamera>>,
) {
    let mut projection = query_camera.single_mut();

    if input.pressed(KeyCode::Equals) {
        let scale = projection.scale - (CAMERA_ZOOM_STEP * time.delta_seconds());

        projection.scale = scale.max(MIN_CAMERA_ZOOM);
    }

    if input.pressed(KeyCode::Minus) {
        let scale = projection.scale + (CAMERA_ZOOM_STEP * time.delta_seconds());

        projection.scale = scale.min(MAX_CAMERA_ZOOM);
    }
}

pub(super) fn move_camera(
    mut query_main_camera: Query<&mut Transform, (With<MainCamera>, Without<Player>)>,
    mut query_background_camera: Query<&mut Transform, (With<BackgroundCamera>, Without<MainCamera>, Without<Player>)>,
    query_player: Query<&Transform, (With<Player>, Without<MainCamera>)>,
    #[cfg(feature = "debug")]
    time: Res<Time>,
    #[cfg(feature = "debug")]
    input: Res<Input<KeyCode>>,
    #[cfg(feature = "debug")]
    debug_config: Res<crate::plugins::debug::DebugConfiguration>
) {
    let main_camera_transform = query_main_camera.get_single_mut().ok();
    let background_camera_transform = query_background_camera.get_single_mut().ok();

    #[cfg(not(feature = "debug"))] {
        if let Ok(player_transform) = query_player.get_single() {
            follow_player(player_transform, main_camera_transform, background_camera_transform);
        }
    }

    #[cfg(feature = "debug")] {
        if debug_config.free_camera {
            free_camera(time, input, main_camera_transform, background_camera_transform);
        } else {
            if let Ok(player_transform) = query_player.get_single() {
                follow_player(player_transform, main_camera_transform, background_camera_transform);
            }
        }
    }
}

pub(super) fn follow_player(
    player_transform: &Transform,
    main_camera_transform: Option<Mut<Transform>>,
    background_camera_transform: Option<Mut<Transform>>,
) {
    let player_pos = player_transform.translation.truncate();

    if let Some(mut transform) = main_camera_transform {
        transform.translation.x = player_pos.x;
        transform.translation.y = player_pos.y;
    }

    if let Some(mut transform) = background_camera_transform {
        transform.translation.x = player_pos.x;
        transform.translation.y = player_pos.y;
    }
}
#[cfg(feature = "debug")]
pub(super) fn free_camera(
    time: Res<Time>,
    input: Res<bevy::prelude::Input<KeyCode>>,
    main_camera_transform: Option<Mut<Transform>>,
    background_camera_transform: Option<Mut<Transform>>,
) {
    use bevy::prelude::Vec2;

    use super::{CAMERA_MOVE_SPEED, CAMERA_MOVE_SPEED_SLOWER, CAMERA_MOVE_SPEED_FASTER};

    let camera_speed = if input.pressed(KeyCode::ShiftLeft) {
        CAMERA_MOVE_SPEED_FASTER
    } else if input.pressed(KeyCode::AltLeft) {
        CAMERA_MOVE_SPEED_SLOWER
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

    let velocity = move_direction * camera_speed * time.delta_seconds();

    if let Some(mut transform) = main_camera_transform {
        transform.translation.x += velocity.x;
        transform.translation.y += velocity.y;
    }

    if let Some(mut transform) = background_camera_transform {
        transform.translation.x += velocity.x;
        transform.translation.y += velocity.y;
    }
}

pub(super) fn keep_camera_inside_world_bounds(
    mut query_main_camera: Query<(&mut Transform, &OrthographicProjection), (With<MainCamera>, Without<Player>)>,
    mut query_background_camera: Query<&mut Transform, (With<BackgroundCamera>, Without<MainCamera>, Without<Player>)>,
    world_data: Res<WorldData>
) {
    let Ok((mut main_camera_transform, projection)) = query_main_camera.get_single_mut() else { return; };
    
    let projection_left = projection.area.min.x;
    let projection_right = projection.area.max.x;
    let projection_top = projection.area.max.y;

    let x_min = projection_left.abs() - TILE_SIZE / 2.;
    let x_max = (world_data.size.width as f32 * 16.) - projection_right - TILE_SIZE / 2.;

    let y_min = -(world_data.size.height as f32 * 16.) - projection_top - TILE_SIZE / 2.;
    let y_max = -projection_top - TILE_SIZE / 2.;

    main_camera_transform.translation.x = main_camera_transform.translation.x.clamp(x_min, x_max);
    main_camera_transform.translation.y = main_camera_transform.translation.y.clamp(y_min, y_max);

    if let Ok(mut background_camera_transform) = query_background_camera.get_single_mut() {
        background_camera_transform.translation.x = background_camera_transform.translation.x.clamp(x_min, x_max);
        background_camera_transform.translation.y = background_camera_transform.translation.y.clamp(y_min, y_max);
    }
}

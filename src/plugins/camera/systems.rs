use autodefault::autodefault;
use bevy::{
    prelude::{
        Commands, Camera2dBundle, OrthographicProjection, Transform, Res, KeyCode, Query, 
        With, Input,
        Without, Camera2d, Name, Mut, Color, UiCameraConfig, default, ResMut,
    }, 
    time::Time, core_pipeline::clear_color::ClearColorConfig
};

use crate::{plugins::{world::{constants::TILE_SIZE, WORLD_RENDER_LAYER}, DespawnOnGameExit}, common::{helpers::tile_pos_to_world_coords, math::map_range_f32}, world::WorldData};

use crate::plugins::player::Player;

use super::{CAMERA_ZOOM_STEP, MIN_CAMERA_ZOOM, MAX_CAMERA_ZOOM, components::{MainCamera, WorldCamera, ZoomableCamera, MoveCamera}, resources::Zoom};

#[autodefault]
pub(super) fn setup_main_camera(
    mut commands: Commands,
    world_data: Res<WorldData>
) {
    let player_spawn_point = tile_pos_to_world_coords(world_data.spawn_point);

    commands
        .spawn((
            Name::new("MainCamera"),
            DespawnOnGameExit,
            MainCamera,
            ZoomableCamera,
            MoveCamera,
            UiCameraConfig { show_ui: false },
            Camera2dBundle {
                transform: Transform::from_xyz(player_spawn_point.x, player_spawn_point.y, 500.),
                camera_2d: Camera2d {
                    clear_color: ClearColorConfig::Custom(Color::NONE)
                }
            }
        ));
}

pub(super) fn setup_world_camera(
    mut commands: Commands,
    world_data: Res<WorldData>
) {
    let player_spawn_point = tile_pos_to_world_coords(world_data.spawn_point);

    commands.spawn((
        Name::new("WorldCamera"),
        DespawnOnGameExit,
        WorldCamera,
        ZoomableCamera,
        MoveCamera,
        UiCameraConfig { show_ui: false },
        Camera2dBundle {
            transform: Transform::from_xyz(player_spawn_point.x, player_spawn_point.y, 500.),
            camera_2d: Camera2d {
                clear_color: ClearColorConfig::Custom(Color::NONE)
            },
            ..default()
        },
        WORLD_RENDER_LAYER
    ));
}

pub(super) fn zoom(
    time: Res<Time>,
    input: Res<Input<KeyCode>>,
    mut zoom: ResMut<Zoom>,
) {
    let scale = zoom.get();

    let new_scale = map_range_f32(0., 1., 1.5, 0.5, scale) * CAMERA_ZOOM_STEP * time.delta_seconds();

    if input.pressed(KeyCode::Equals) {
        zoom.set((scale + new_scale).min(1.));
    }

    if input.pressed(KeyCode::Minus) {
        zoom.set((scale - new_scale).max(0.));
    }
}

pub(super) fn move_camera(
    mut query_move_camera: Query<&mut Transform, With<MoveCamera>>,
    query_player: Query<&Transform, (With<Player>, Without<MoveCamera>)>,
    #[cfg(feature = "debug")]
    time: Res<Time>,
    #[cfg(feature = "debug")]
    input: Res<Input<KeyCode>>,
    #[cfg(feature = "debug")]
    debug_config: Res<crate::plugins::debug::DebugConfiguration>
) {
    for camera_transform in &mut query_move_camera {
        #[cfg(not(feature = "debug"))] {
            if let Ok(player_transform) = query_player.get_single() {
                follow_player(player_transform, camera_transform);
            }
        }

        #[cfg(feature = "debug")] {
            if debug_config.free_camera {
                free_camera(&time, &input, camera_transform);
            } else {
                if let Ok(player_transform) = query_player.get_single() {
                    follow_player(player_transform, camera_transform);
                }
            }
        }
    }
}

pub(super) fn follow_player(
    player_transform: &Transform,
    mut camera_transform: Mut<Transform>,
) {
    let player_pos = player_transform.translation.truncate();
    camera_transform.translation.x = player_pos.x;
    camera_transform.translation.y = player_pos.y;
}

#[cfg(feature = "debug")]
pub(super) fn free_camera(
    time: &Res<Time>,
    input: &Res<bevy::prelude::Input<KeyCode>>,
    mut camera_transform: Mut<Transform>,
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

    camera_transform.translation.x += velocity.x;
    camera_transform.translation.y += velocity.y;
}

pub(super) fn keep_camera_inside_world_bounds(
    world_data: Res<WorldData>,
    mut query_main_camera: Query<(&mut Transform, &OrthographicProjection), With<MoveCamera>>,
) {
    for (mut camera_transform, projection) in &mut query_main_camera {
        let projection_left = projection.area.min.x;
        let projection_right = projection.area.max.x;
        let projection_top = projection.area.max.y;
        let projection_bottom = projection.area.min.y;

        let x_min = projection_left.abs() - TILE_SIZE / 2.;
        let x_max = (world_data.size.width as f32 * 16.) - projection_right - TILE_SIZE / 2.;

        let y_min = -(world_data.size.height as f32 * 16.) - projection_bottom - TILE_SIZE / 2.;
        let y_max = -projection_top - TILE_SIZE / 2.;

        camera_transform.translation.x = camera_transform.translation.x.clamp(x_min, x_max);
        camera_transform.translation.y = camera_transform.translation.y.clamp(y_min, y_max);
    }
}

pub(super) fn update_camera_scale(
    zoom: Res<Zoom>,
    mut query_camera: Query<&mut OrthographicProjection, With<ZoomableCamera>>
) {
    for mut projection in &mut query_camera {
        projection.scale = map_range_f32(0., 1., MAX_CAMERA_ZOOM, MIN_CAMERA_ZOOM, zoom.get());
    }
}
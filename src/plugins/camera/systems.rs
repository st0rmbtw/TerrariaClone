use bevy::{
    prelude::{
        Commands, Camera2dBundle, OrthographicProjection, Transform, Res, KeyCode, Query, 
        With, Input,
        Without, Camera2d, Name, Mut, UiCameraConfig, default, ResMut, Camera, Vec2, Changed,
    }, 
    time::Time, core_pipeline::{clear_color::ClearColorConfig, tonemapping::Tonemapping}, math::Vec3Swizzles
};

use crate::{plugins::{world::{constants::TILE_SIZE, WORLD_RENDER_LAYER}, DespawnOnGameExit}, common::{helpers::tile_to_world_pos, math::map_range_f32, components::EntityRect}, world::WorldData};

use crate::plugins::player::Player;

use super::{CAMERA_ZOOM_STEP, MIN_CAMERA_ZOOM, MAX_CAMERA_ZOOM, components::{MainCamera, WorldCamera, ZoomableCamera, MoveCamera}, resources::Zoom};

pub(super) fn setup_main_camera(
    mut commands: Commands,
    zoom: Res<Zoom>
) {
    commands
        .spawn((
            Name::new("MainCamera"),
            DespawnOnGameExit,
            MainCamera,
            ZoomableCamera,
            MoveCamera,
            UiCameraConfig::default(),
            Camera2dBundle {
                projection: OrthographicProjection {
                    scale: zoom.get(),
                    ..default()
                },
                camera: Camera {
                    order: 1,
                    msaa_writeback: false,
                    ..default()
                },
                transform: Transform::from_xyz(0., 0., 500.),
                camera_2d: Camera2d {
                    clear_color: ClearColorConfig::None
                },
                tonemapping: Tonemapping::None,
                ..default()
            }
        ));
}

pub(super) fn setup_world_camera(
    mut commands: Commands,
    zoom: Res<Zoom>
) {
    commands.spawn((
        Name::new("WorldCamera"),
        DespawnOnGameExit,
        WorldCamera,
        ZoomableCamera,
        MoveCamera,
        UiCameraConfig { show_ui: false },
        Camera2dBundle {
            projection: OrthographicProjection {
                scale: zoom.get(),
                ..default()
            },
            camera: Camera {
                order: 0,
                msaa_writeback: false,
                ..default()
            },
            transform: Transform::from_xyz(0., 0., 500.),
            camera_2d: Camera2d {
                clear_color: ClearColorConfig::None
            },
            tonemapping: Tonemapping::None,
            ..default()
        },
        WORLD_RENDER_LAYER
    ));
}

pub(super) fn init_camera_position(
    world_data: Res<WorldData>,
    mut query_move_camera: Query<&mut Transform, With<MoveCamera>>,
) {
    let player_spawn_point = tile_to_world_pos(world_data.spawn_point);

    query_move_camera.for_each_mut(|mut transform| {
        transform.translation.x = player_spawn_point.x;
        transform.translation.y = player_spawn_point.y;
    });
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
    query_player: Query<&EntityRect, (With<Player>, Changed<EntityRect>, Without<MoveCamera>)>,
    #[cfg(feature = "debug")]
    time: Res<Time>,
    #[cfg(feature = "debug")]
    input: Res<Input<KeyCode>>,
    #[cfg(feature = "debug")]
    debug_config: Res<crate::plugins::debug::DebugConfiguration>
) {
    #[cfg(not(feature = "debug"))] {
        let Ok(player_rect) = query_player.get_single() else { return; };
        let player_pos = player_rect.center();

        query_move_camera.for_each_mut(|camera_transform| {
            follow_player(player_pos, camera_transform);
        });
    }

    #[cfg(feature = "debug")] {
        if debug_config.free_camera {
            query_move_camera.for_each_mut(|camera_transform| {
                free_camera(&time, &input, camera_transform);
            });
        } else {
            let Ok(player_rect) = query_player.get_single() else { return; };
            let player_pos = player_rect.center();

            query_move_camera.for_each_mut(|camera_transform| {
                follow_player(player_pos, camera_transform);
            });
        }
    }
}

#[inline(always)]
pub(super) fn follow_player(
    player_pos: Vec2,
    mut camera_transform: Mut<Transform>,
) {
    let camera_pos = camera_transform.translation.xy();

    let new_pos = camera_pos.lerp(player_pos, 0.4);

    camera_transform.translation.x = new_pos.x;
    camera_transform.translation.y = new_pos.y;
}

#[cfg(feature = "debug")]
pub(super) fn free_camera(
    time: &Res<Time>,
    input: &Res<bevy::prelude::Input<KeyCode>>,
    mut camera_transform: Mut<Transform>,
) {
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
    mut query_camera: Query<(&mut Transform, &OrthographicProjection), With<MoveCamera>>,
) {
    query_camera.for_each_mut(|(mut camera_transform, projection)| {
        let proj_left = projection.area.min.x;
        let proj_right = projection.area.max.x;
        let proj_top = projection.area.max.y;
        let proj_bottom = projection.area.min.y;

        let x_min = proj_left.abs() - TILE_SIZE / 2.;
        let x_max = (world_data.size.width as f32 * TILE_SIZE) - proj_right - TILE_SIZE / 2.;

        let y_min = -(world_data.size.height as f32 * TILE_SIZE) - proj_bottom + TILE_SIZE / 2.;
        let y_max = -proj_top - TILE_SIZE / 2.;

        camera_transform.translation.x = camera_transform.translation.x.clamp(x_min, x_max);
        camera_transform.translation.y = camera_transform.translation.y.clamp(y_min, y_max);
    });
}

pub(super) fn update_camera_scale(
    zoom: Res<Zoom>,
    mut query_camera: Query<&mut OrthographicProjection, With<ZoomableCamera>>
) {
    query_camera.for_each_mut(|mut projection| {
        projection.scale = map_range_f32(0., 1., MAX_CAMERA_ZOOM, MIN_CAMERA_ZOOM, zoom.get());
    });
}
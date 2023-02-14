use autodefault::autodefault;
use bevy::{
    prelude::{
        Commands, Camera2dBundle, OrthographicProjection, Transform, Res, KeyCode, Query, 
        With, GlobalTransform
    }, 
    time::Time
};
use leafwing_input_manager::{InputManagerBundle, prelude::{ActionState, InputMap}};

use crate::{parallax::ParallaxCameraComponent, plugins::world::TILE_SIZE, world_generator::{WORLD_SIZE_X, WORLD_SIZE_Y}};

#[cfg(not(feature = "free_camera"))]
use crate::plugins::player::Player;

use super::{MainCamera, CAMERA_ZOOM_STEP, MIN_CAMERA_ZOOM, MAX_CAMERA_ZOOM, MouseAction};

#[autodefault]
pub fn setup_camera(mut commands: Commands) {
    commands
        .spawn((
            MainCamera,
            ParallaxCameraComponent,
            Camera2dBundle {
                projection: OrthographicProjection { 
                    scale: 0.9
                },
                transform: Transform::from_xyz(0., 0., 500.),
            },
            InputManagerBundle::<MouseAction> {
                action_state: ActionState::default(),
                input_map: InputMap::new([
                    (KeyCode::Equals, MouseAction::ZoomIn),
                    (KeyCode::Minus, MouseAction::ZoomOut),
                ])
            }   
        ));
}

pub fn zoom(
    time: Res<Time>,
    mut camera_query: Query<(&mut OrthographicProjection, &ActionState<MouseAction>), With<MainCamera>>,
) {
    if let Ok((mut projection, input)) = camera_query.get_single_mut() {
        if input.pressed(MouseAction::ZoomIn) {
            let scale = projection.scale - (CAMERA_ZOOM_STEP * time.delta_seconds());

            projection.scale = scale.max(MIN_CAMERA_ZOOM);
        }

        if input.pressed(MouseAction::ZoomOut) {
            let scale = projection.scale + (CAMERA_ZOOM_STEP * time.delta_seconds());

            projection.scale = scale.min(MAX_CAMERA_ZOOM);
        }
    }
}

#[cfg(not(feature = "free_camera"))]
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

#[cfg(feature = "free_camera")]
pub fn move_camera(
    mut camera: Query<&mut GlobalTransform, With<MainCamera>>,
    input: Res<Input<KeyCode>>
) {
    use bevy::prelude::Vec2;

    const CAMERA_MOVE_SPEED: f32 = 15.;

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

    if let Ok(mut camera_transform) = camera.get_single_mut() {
        camera_transform.translation_mut().x += move_direction.x * CAMERA_MOVE_SPEED;
        camera_transform.translation_mut().y += move_direction.y * CAMERA_MOVE_SPEED;
    }
}
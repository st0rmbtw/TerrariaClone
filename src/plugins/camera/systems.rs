use autodefault::autodefault;
use bevy::{
    prelude::{
        Commands, Camera2dBundle, OrthographicProjection, Transform, Res, KeyCode, Query, 
        With, GlobalTransform, default,ResMut, Assets, Vec3, Mesh, shape, Input, Color, Name,
    }, 
    time::Time, sprite::{MaterialMesh2dBundle, ColorMaterial}
};

use crate::{parallax::ParallaxCameraComponent, plugins::{world::{TILE_SIZE, WorldData}, cursor::CursorPosition}, world_generator::{WORLD_SIZE_X, WORLD_SIZE_Y}, util::tile_to_world_coords, lighting::{compositing::LightMapCamera, types::OmniLightSource2D}};

#[cfg(not(feature = "free_camera"))]
use crate::plugins::player::Player;

use super::{MainCamera, CAMERA_ZOOM_STEP, MIN_CAMERA_ZOOM, MAX_CAMERA_ZOOM, MouseLight};

#[autodefault(except(TextureDescriptor, ShadowMapMaterial, LightMapMaterial, SunMaterial, LightingMaterial))]
pub fn setup_camera(
    mut commands: Commands,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    world_data: Res<WorldData>
) {
    let spawn_point = tile_to_world_coords(world_data.spawn_point);

    commands
        .spawn((
            MainCamera,
            ParallaxCameraComponent,
            LightMapCamera,
            Camera2dBundle {
                projection: OrthographicProjection { 
                    scale: 0.9
                },
                transform: Transform::from_xyz(spawn_point.x + TILE_SIZE / 2., spawn_point.y + TILE_SIZE / 2., 500.)
            },
            // InputManagerBundle::<MouseAction> {
            //     action_state: ActionState::default(),
            //     input_map: InputMap::new([
            //         (KeyCode::Equals, MouseAction::ZoomIn),
            //         (KeyCode::Minus, MouseAction::ZoomOut),
            //     ])
            // },
        ));

    // commands
    //     .spawn(MaterialMesh2dBundle {
    //         mesh: meshes.add(Mesh::from(shape::Circle::new(57.))).into(),
    //         material: standard_materials.add(StandardMaterial {
    //             base_color_texture: Some(background_assets.sun.clone()),
    //             alpha_mode: AlphaMode::Blend,
    //             ..default()
    //         }),
    //         transform: Transform {
    //             translation: Vec3::new(14000., -2800., 1.),
    //             ..default()
    //         },
    //         ..default()
    //     })
    //     .insert(OmniLightSource2D {
    //         intensity: 10.0,
    //         color:     Color::YELLOW,
    //         falloff:   Vec3::new(50.0, 20.0, 0.05),
    //         ..default()
    //     });

    let block_mesh = meshes.add(Mesh::from(shape::Quad::default()));

    commands
        .spawn(MaterialMesh2dBundle {
            mesh: block_mesh.clone().into(),
            material: color_materials.add(ColorMaterial::from(Color::YELLOW)).into(),
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 1000.0),
                scale:       Vec3::splat(8.0),
                ..default()
            },
            ..default()
        })
        .insert(Name::new("cursor_light"))
        .insert(OmniLightSource2D {
            intensity: 10.,
            color:     Color::rgb_u8(254, 100, 34),
            falloff:   Vec3::new(10.0, 10.0, 0.05),
            jitter_intensity: 0.7,
            jitter_translation: 1.,
            ..default()
        })
        .insert(MouseLight);
}

pub fn zoom(
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

pub fn move_mouse_light(
    mut query: Query<&mut Transform, With<MouseLight>>,
    cursor_position: Res<CursorPosition>
) {
    for mut transform in &mut query {
        transform.translation = cursor_position.world_position.extend(10.);
    }
}

#[cfg(feature = "free_camera")]
pub fn move_camera(
    mut camera: Query<&mut GlobalTransform, With<MainCamera>>,
    input: Res<bevy::prelude::Input<KeyCode>>
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
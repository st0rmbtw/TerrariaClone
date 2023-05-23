// Based on https://github.com/Corrosive-Games/bevy-parallax

use std::cmp::max;

use bevy::{prelude::*, window::PrimaryWindow};

mod layer;

pub(crate) use layer::*;

use crate::plugins::camera::MainCamera;

pub struct ParallaxPlugin {
    pub initial_speed: f32,
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub(crate) enum ParallaxSet {
    FollowCamera
}

impl Plugin for ParallaxPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ParallaxMoveSpeed {
            speed: self.initial_speed * 1000.,
        });

        app.add_system(parallax_container_added);

        app.add_systems(
            (
                // update_window_size,
                update_layer_textures_system.after(ParallaxSet::FollowCamera),
            )
        );
    }
}

#[derive(Component, Default)]
pub(crate) struct ParallaxContainer {
    /// Data to describe each layer of parallax
    layer_data: Vec<LayerData>,
    /// Parallax layer entities
    layer_entities: Vec<Entity>,
    
    processed: bool,
}

impl ParallaxContainer {
    pub(crate) fn new(layers: Vec<LayerData>) -> Self {
        Self {
            layer_data: layers,
            layer_entities: Vec::new(),
            processed: false
        }
    }
}

#[derive(Resource)]
struct ParallaxMoveSpeed {
    speed: f32,
}

fn parallax_container_added(
    mut commands: Commands,
    mut query_parallax_container: Query<(&mut ParallaxContainer, Entity)>,
    images: Res<Assets<Image>>,
    query_window: Query<&Window, With<PrimaryWindow>>
) {
    let window = query_window.single();
    let window_width = window.width();
    let window_height = window.height();

    for (mut parallax_container, entity) in &mut query_parallax_container {
        if parallax_container.processed { continue; }
        
        commands.entity(entity)
            .insert(SpatialBundle::default())
            .with_children(|children| {
            // Spawn new layers using layer_data
            for i in 0..parallax_container.layer_data.len() {
                let layer = &parallax_container.layer_data[i];

                let texture = images.get(&layer.image).unwrap();

                let spritesheet_bundle = SpriteBundle {
                    sprite: Sprite {
                        custom_size: if layer.fill_screen_height { Some(Vec2::new(texture.size().x, window_height)) } else { None },
                        anchor: layer.anchor.clone(),
                        ..default()
                    },
                    texture: layer.image.clone(),
                    ..Default::default()
                };

                let x_max_index = match layer.speed {
                    LayerSpeed::Horizontal(_) | LayerSpeed::Bidirectional(..) => max(
                        (window_width / (texture.size().x * layer.scale / 2.) + 1.0) as i32,
                        1,
                    ),
                    LayerSpeed::Vertical(_) => 0,
                };

                let texture_count = 2.0 * x_max_index as f32 + 1.0;

                let layer_entity = children.spawn((
                    Name::new(format!("Parallax Layer ({})", i)),
                    SpatialBundle {
                        transform: Transform {
                            translation: Vec3::new(layer.position.x, layer.position.y, layer.z),
                            scale: Vec3::new(layer.scale, layer.scale, layer.scale),
                            ..default()
                        },
                        ..default()
                    },
                    LayerComponent {
                        speed: match layer.speed {
                            LayerSpeed::Horizontal(vx) => Vec2::new(vx, 1.0),
                            LayerSpeed::Vertical(vy) => Vec2::new(1.0, vy),
                            LayerSpeed::Bidirectional(vx, vy) => Vec2::new(vx, vy),
                        },
                        texture_count,
                        transition_factor: layer.transition_factor,
                        index: i
                    }
                ))
                .with_children(|parent| {
                    for x in -x_max_index..=x_max_index {
                        let mut adjusted_spritesheet_bundle = spritesheet_bundle.clone();
                        adjusted_spritesheet_bundle.transform.translation.x = texture.size().x * x as f32;
                        parent.spawn(adjusted_spritesheet_bundle).insert(
                            LayerTextureComponent {
                                width: texture.size().x,
                            },
                        );
                    }
                })
                .id();

                parallax_container.layer_entities.push(layer_entity);
            }
        });

        parallax_container.processed = true;
    }
}

#[derive(Component)]
pub(crate) struct ParallaxCameraComponent;

#[inline(always)]
pub(crate) fn move_background_system() -> impl IntoSystemConfig<()> {
    parallax_animation_system
        .in_set(ParallaxSet::FollowCamera)
}

/// Move camera and background layers
fn parallax_animation_system(
    time: Res<Time>,
    mut camera_query: Query<&mut Transform, With<ParallaxCameraComponent>>,
    mut layer_query: Query<(&mut Transform, &LayerComponent), Without<ParallaxCameraComponent>>,
    parallax_speed: Res<ParallaxMoveSpeed>,
) {
    if let Some(mut camera_transform) = camera_query.iter_mut().next() {
        camera_transform.translation.x += parallax_speed.speed * time.delta_seconds();
        for (mut layer_transform, layer) in layer_query.iter_mut() {
            layer_transform.translation.x += parallax_speed.speed * layer.speed.x * time.delta_seconds();
        }
    }
}

pub(crate) fn follow_camera_system(
    query_parallax_container: Query<&ParallaxContainer>,
    camera_query: Query<&GlobalTransform, (With<ParallaxCameraComponent>, With<MainCamera>)>,
    mut layer_query: Query<(&mut Transform, &LayerComponent), Without<ParallaxCameraComponent>>,
) {
    if let Some(camera_transform) = camera_query.iter().next() {
        for parallax_container in &query_parallax_container {
            for layer_entity in &parallax_container.layer_entities {
                if let Ok((mut layer_transform, layer)) = layer_query.get_mut(*layer_entity) {
                    let layer_data = &parallax_container.layer_data[layer.index];
                    let camera_translation = camera_transform.translation();

                    layer_transform.translation.x = camera_translation.x + (layer_data.position.x - camera_translation.x) * layer.speed.x;
                    layer_transform.translation.y = camera_translation.y + (layer_data.position.y - camera_translation.y) * layer.speed.y;
                }
            }
        }
    }
}

/// Update layer positions to keep the effect going indefinitely
fn update_layer_textures_system(
    layer_query: Query<(&LayerComponent, &Children)>,
    mut texture_query: Query<
        (
            &GlobalTransform,
            &mut Transform,
            &LayerTextureComponent,
        ),
        Without<ParallaxCameraComponent>,
    >,
    camera_query: Query<&GlobalTransform, With<ParallaxCameraComponent>>,
    query_window: Query<&Window, With<PrimaryWindow>>
) {
    let window = query_window.single();
    let window_width = window.width();

    if let Some(camera_transform) = camera_query.iter().next() {
        for (layer, children) in layer_query.iter() {
            for &child in children.iter() {
                let (texture_gtransform, mut texture_transform, layer_texture) =
                    texture_query.get_mut(child).unwrap();

                let texture_gtransform = texture_gtransform.compute_transform();

                // Move right-most texture to left side of layer when camera is approaching left-most end
                if camera_transform.translation().x - texture_gtransform.translation.x
                    + ((layer_texture.width * texture_gtransform.scale.x) / 2.0) 
                    < -(window_width * layer.transition_factor)
                {
                    texture_transform.translation.x -= layer_texture.width * layer.texture_count;
                // Move left-most texture to right side of layer when camera is approaching right-most end
                } else if camera_transform.translation().x - texture_gtransform.translation.x
                    - ((layer_texture.width * texture_gtransform.scale.x) / 2.0)
                    > window_width * layer.transition_factor
                {
                    texture_transform.translation.x += layer_texture.width * layer.texture_count;
                }
            }
        }
    }
}
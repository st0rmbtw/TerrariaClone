// Based on https://github.com/Corrosive-Games/bevy-parallax

use std::cmp::max;

use bevy::{prelude::*, window::{PrimaryWindow, WindowResized}, render::view::RenderLayers, transform::TransformSystem};

mod layer;

pub(crate) use layer::*;

use crate::common::{extensions::EntityCommandsExtensions, state::GameState};

pub struct ParallaxPlugin;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub(crate) enum ParallaxSet {
    FollowCamera
}

impl Plugin for ParallaxPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                parallax_container_added,
                update_full_screen_sprites.run_if(not(in_state(GameState::InGame)))
            )
        );
        app.add_systems(PostUpdate, update_layer_textures_system.after(TransformSystem::TransformPropagate));
    }
}

#[derive(Component, Default)]
pub(crate) struct ParallaxContainer {
    /// Data to describe each layer of parallax
    layer_data: Vec<LayerData>,

    render_layer: RenderLayers
}

#[derive(Component)]
struct Processed;

#[derive(Component)]
struct FillScreenHeight;

impl ParallaxContainer {
    pub(crate) fn new(layers: Vec<LayerData>) -> Self {
        Self {
            layer_data: layers,
            render_layer: Default::default(),
        }
    }

    pub(crate) fn with_render_layer(mut self, layer: RenderLayers) -> Self {
        self.render_layer = layer;
        self
    }
}

fn parallax_container_added(
    mut commands: Commands,
    images: Res<Assets<Image>>,
    query_window: Query<&Window, With<PrimaryWindow>>,
    query_parallax_container: Query<(&ParallaxContainer, Entity), Without<Processed>>,
) {
    let window = query_window.single();
    let window_width = window.width();
    let window_height = window.height();

    for (parallax_container, entity) in &query_parallax_container {
        commands.entity(entity)
            .insert(SpatialBundle::default())
            .with_children(|children| {
                // Spawn new layers using layer_data
                for i in 0..parallax_container.layer_data.len() {
                    let layer_data = &parallax_container.layer_data[i];

                    let texture = images.get(&layer_data.image).unwrap();
                    let texture_size = texture.size();

                    let spritesheet_bundle = SpriteBundle {
                        sprite: Sprite {
                            custom_size: layer_data.fill_screen_height.then_some(Vec2::new(texture_size.x, window_height)),
                            anchor: layer_data.anchor.clone(),
                            ..default()
                        },
                        texture: layer_data.image.clone(),
                        ..Default::default()
                    };

                    let x_max_index = match layer_data.speed {
                        LayerSpeed::Horizontal(_) | LayerSpeed::Bidirectional(..) => max(
                            (window_width / (texture_size.x * layer_data.scale / 2.) + 1.0) as i32,
                            1,
                        ),
                        LayerSpeed::Vertical(_) => 0,
                    };

                    let texture_count = 2.0 * x_max_index as f32 + 1.0;

                    children.spawn((
                        Name::new(format!("Parallax Layer ({})", i)),
                        SpatialBundle {
                            transform: Transform {
                                translation: Vec3::new(layer_data.position.x, layer_data.position.y, layer_data.z),
                                scale: Vec3::splat(layer_data.scale),
                                ..default()
                            },
                            ..default()
                        },
                        LayerComponent {
                            speed: match layer_data.speed {
                                LayerSpeed::Horizontal(vx) => Vec2::new(vx, 1.0),
                                LayerSpeed::Vertical(vy) => Vec2::new(1.0, vy),
                                LayerSpeed::Bidirectional(vx, vy) => Vec2::new(vx, vy),
                            },
                        },
                        LayerDataComponent {
                            position: layer_data.position
                        }
                    ))
                    .insert_if(FillScreenHeight, layer_data.fill_screen_height)
                    .with_children(|parent| {
                        for x in -x_max_index..=x_max_index {
                            let mut adjusted_spritesheet_bundle = spritesheet_bundle.clone();
                            adjusted_spritesheet_bundle.transform.translation.x = texture_size.x * x as f32;

                            parent
                                .spawn((
                                    adjusted_spritesheet_bundle,
                                    parallax_container.render_layer,
                                    LayerTextureComponent {
                                        width: texture_size.x,
                                        texture_count,
                                        transition_factor: layer_data.transition_factor
                                    },
                                ));
                        }
                    });
                }
            }
        );

        commands.entity(entity).insert(Processed);
    }
}

#[derive(Component)]
pub(crate) struct ParallaxCameraComponent;

/// Move camera and background layers
pub(crate) fn parallax_animation_system(
    speed: f32,    
) -> impl FnMut(
    Res<Time>,
    Query<&mut Transform, With<ParallaxCameraComponent>>,
    Query<(&mut Transform, &LayerComponent), Without<ParallaxCameraComponent>>
) {
    move |
        time: Res<Time>,
        mut query_camera: Query<&mut Transform, With<ParallaxCameraComponent>>,
        mut query_layer: Query<(&mut Transform, &LayerComponent), Without<ParallaxCameraComponent>>
    | {
        if let Some(mut camera_transform) = query_camera.iter_mut().next() {
            camera_transform.translation.x += speed * time.delta_seconds();
            for (mut layer_transform, layer) in query_layer.iter_mut() {
                layer_transform.translation.x += speed * layer.speed.x * time.delta_seconds();
            }
        }
    }
}

/// Update layer positions to keep the effect going indefinitely
fn update_layer_textures_system(
    mut query_texture: Query<
        (
            &LayerTextureComponent,
            &GlobalTransform,
            &mut Transform,
        ),
        Without<ParallaxCameraComponent>,
    >,
    query_camera: Query<&GlobalTransform, With<ParallaxCameraComponent>>,
    query_window: Query<&Window, With<PrimaryWindow>>
) {
    let Ok(window) = query_window.get_single() else { return; };
    let window_width = window.width();
    
    let Ok(camera_transform) = query_camera.get_single() else { return; };

    let camera_position = camera_transform.translation();

    query_texture.for_each_mut(|(
        layer_texture,
        texture_global_transform,
        mut texture_transform
    )| {
        let texture_translation = texture_global_transform.translation();
        let texture_scale = texture_transform.scale;

        // Move right-most texture to left side of layer when camera is approaching left-most end
        if camera_position.x - texture_translation.x
            + ((layer_texture.width * texture_scale.x) / 2.0) 
            < -(window_width * layer_texture.transition_factor)
        {
            texture_transform.translation.x -= layer_texture.width * layer_texture.texture_count;
        // Move left-most texture to right side of layer when camera is approaching right-most end
        } else if camera_position.x - texture_translation.x
            - ((layer_texture.width * texture_scale.x) / 2.0)
            > window_width * layer_texture.transition_factor
        {
            texture_transform.translation.x += layer_texture.width * layer_texture.texture_count;
        }
    });
}

fn update_full_screen_sprites(
    mut window_resize_events: EventReader<WindowResized>,
    query_layer: Query<&Children, (With<LayerComponent>, With<FillScreenHeight>)>,
    mut query_texture_layer: Query<&mut Sprite, With<LayerTextureComponent>>
) {
    let Some(event) = window_resize_events.iter().last() else { return; };

    query_layer.for_each(|children| {
        for &entity in children.iter() {
            if let Ok(mut sprite) = query_texture_layer.get_mut(entity) {
                if let Some(size) = sprite.custom_size.as_mut() {
                    size.y = event.height;
                }
            }
        }
    });
}
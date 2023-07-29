// Based on https://github.com/Corrosive-Games/bevy-parallax

use std::cmp::max;

use bevy::{prelude::*, window::{PrimaryWindow, WindowResized}, render::view::RenderLayers};

mod layer;

pub(crate) use layer::*;

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
                update_layer_textures_system.after(ParallaxSet::FollowCamera),
            )
        );
        app.add_systems(PreUpdate, update_full_screen_sprites);
    }
}

#[derive(Component, Default)]
pub(crate) struct ParallaxContainer {
    /// Data to describe each layer of parallax
    layer_data: Vec<LayerData>,
    
    processed: bool,

    render_layer: RenderLayers
}

impl ParallaxContainer {
    pub(crate) fn new(layers: Vec<LayerData>) -> Self {
        Self {
            layer_data: layers,
            render_layer: Default::default(),
            processed: false,
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
    mut query_parallax_container: Query<(&mut ParallaxContainer, Entity)>,
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
                    let layer_data = &parallax_container.layer_data[i];

                    let texture = images.get(&layer_data.image).unwrap();

                    let spritesheet_bundle = SpriteBundle {
                        sprite: Sprite {
                            custom_size: layer_data.fill_screen_height.then_some(Vec2::new(texture.size().x, window_height)),
                            anchor: layer_data.anchor.clone(),
                            ..default()
                        },
                        texture: layer_data.image.clone(),
                        ..Default::default()
                    };

                    let x_max_index = match layer_data.speed {
                        LayerSpeed::Horizontal(_) | LayerSpeed::Bidirectional(..) => max(
                            (window_width / (texture.size().x * layer_data.scale / 2.) + 1.0) as i32,
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
                                scale: Vec3::new(layer_data.scale, layer_data.scale, layer_data.scale),
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
                            texture_count,
                            transition_factor: layer_data.transition_factor
                        },
                        LayerDataComponent {
                            fill_screen_height: layer_data.fill_screen_height,
                            position: layer_data.position
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
                            ).insert(parallax_container.render_layer);
                        }
                    });
                }
            }
        );

        parallax_container.processed = true;
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
            &GlobalTransform,
            &mut Transform,
            &LayerTextureComponent,
        ),
        Without<ParallaxCameraComponent>,
    >,
    query_layer: Query<(&LayerComponent, &Children)>,
    query_camera: Query<&GlobalTransform, With<ParallaxCameraComponent>>,
    query_window: Query<&Window, With<PrimaryWindow>>
) {
    let window = query_window.single();
    let window_width = window.width();

    if let Ok(camera_transform) = query_camera.get_single() {
        for (layer, children) in query_layer.iter() {
            for &child in children.iter() {
                let (texture_gtransform, mut texture_transform, layer_texture) = 
                    query_texture.get_mut(child).unwrap();
                
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

pub(super) fn update_full_screen_sprites(
    mut window_resize_events: EventReader<WindowResized>,
    query_layer: Query<(&LayerDataComponent, &Children), With<LayerComponent>>,
    mut query_texture_layer: Query<&mut Sprite, With<LayerTextureComponent>>
) {
    for event in window_resize_events.iter() {
        for (layer_data, children) in &query_layer {
            if !layer_data.fill_screen_height { continue; }

            for &entity in children.iter() {
                if let Ok(mut sprite) = query_texture_layer.get_mut(entity) {
                    if let Some(size) = sprite.custom_size.as_mut() {
                        size.y = event.height;
                    }
                }
            }
        }
    }
}
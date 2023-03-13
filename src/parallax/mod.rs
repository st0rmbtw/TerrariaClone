use std::cmp::max;

use bevy::{prelude::*, window::{WindowResized, PrimaryWindow}};

mod layer;

pub use layer::*;

use crate::plugins::camera::MainCamera;

pub struct ParallaxPlugin {
    pub initial_speed: f32,
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
enum ParallaxSet {
    FollowCamera
}

impl Plugin for ParallaxPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ParallaxMoveSpeed {
            speed: self.initial_speed * 1000.,
        });

        app.add_system(initialize_parallax_system.run_if(resource_added::<ParallaxResource>()));

        app.add_systems(
            (
                update_window_size,
                update_layer_textures_system.after(ParallaxSet::FollowCamera),
                // update_texture_scale
            )
            .chain()
            .distributive_run_if(resource_exists::<ParallaxResource>())
        );
    }
}

#[derive(Resource)]
pub struct ParallaxMoveSpeed {
    pub speed: f32,
}

/// Resource for managing parallax
#[derive(Resource, Debug)]
pub struct ParallaxResource {
    /// Data to describe each layer of parallax
    pub layer_data: Vec<LayerData>,
    /// Parallax layer entities
    pub layer_entities: Vec<Entity>,
    /// Dimensions of window
    pub window_size: Vec2,
}

impl Default for ParallaxResource {
    fn default() -> Self {
        Self {
            layer_data: vec![],
            layer_entities: vec![],
            window_size: Vec2::ZERO,
        }
    }
}

impl ParallaxResource {
    /// Create a new parallax resource
    pub fn new(layer_data: Vec<LayerData>) -> Self {
        ParallaxResource {
            layer_data,
            layer_entities: vec![],
            window_size: Vec2::ZERO,
        }
    }

    /// Delete all layer entities in parallax resource and empty Vec
    pub fn despawn_layers(&mut self, commands: &mut Commands) {
        // Remove all layer entities
        for entity in self.layer_entities.iter() {
            commands.entity(*entity).despawn_recursive();
        }

        // Empty the layer entity vector
        self.layer_entities = vec![];
    }

    /// Create layers from layer data
    pub fn create_layers(
        &mut self,
        commands: &mut Commands,
        images: &Assets<Image>,
    ) {
        // Despawn any existing layers
        self.despawn_layers(commands);

        // Spawn new layers using layer_data
        for (i, layer) in self.layer_data.iter().enumerate() {
            let texture = images.get(&layer.image).unwrap();

            let spritesheet_bundle = SpriteBundle {
                sprite: Sprite {
                    custom_size: if layer.fill_screen_height { Some(Vec2::new(texture.size().x, self.window_size.y)) } else { None },
                    // anchor: layer.anchor.clone(),
                    ..default()
                },
                texture: layer.image.clone(),
                ..Default::default()
            };

            let texture = images.get(&layer.image).unwrap();

            let y_max_index = match layer.speed {
                LayerSpeed::Vertical(_) | LayerSpeed::Bidirectional(..) => max(
                    (self.window_size.y / (texture.size().y * layer.scale / 2.) + 1.0) as i32,
                    1,
                ),
                LayerSpeed::Horizontal(_) => 0,
            };
            let x_max_index = match layer.speed {
                LayerSpeed::Horizontal(_) | LayerSpeed::Bidirectional(..) => max(
                    (self.window_size.x / (texture.size().x * layer.scale / 2.) + 1.0) as i32,
                    1,
                ),
                LayerSpeed::Vertical(_) => 0,
            };
            let texture_count = Vec2::new(
                2.0 * x_max_index as f32 + 1.0,
                2.0 * y_max_index as f32 + 1.0,
            );

            // Spawn parallax layer entity
            let mut entity_commands = commands.spawn_empty();
            entity_commands
                .insert(Name::new(format!("Parallax Layer ({})", i)))
                .insert(SpatialBundle {
                    transform: Transform {
                        translation: Vec3::new(layer.position.x, layer.position.y, layer.z),
                        scale: Vec3::new(layer.scale, layer.scale, 1.0),
                        ..default()
                    },
                    ..default()
                })
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
                });

            // Add layer component to entity
            entity_commands.insert(LayerComponent {
                speed: match layer.speed {
                    LayerSpeed::Horizontal(vx) => Vec2::new(vx, 0.0),
                    LayerSpeed::Vertical(vy) => Vec2::new(0.0, vy),
                    LayerSpeed::Bidirectional(vx, vy) => Vec2::new(vx, vy),
                },
                texture_count: texture_count.x,
                transition_factor: layer.transition_factor,
                index: i
            });

            // Push parallax layer entity to layer_entities
            self.layer_entities.push(entity_commands.id());
        }
    }
}

/// Attach to a single camera to be used with parallax
#[derive(Component)]
pub struct ParallaxCameraComponent;

#[inline(always)]
pub fn move_background_system() -> impl IntoSystemConfig<()> {
    parallax_animation_system
        .run_if(resource_exists::<ParallaxResource>())
        .in_set(ParallaxSet::FollowCamera)
}
    
/// Initialize the parallax resource
fn initialize_parallax_system(
    mut commands: Commands,
    query_window: Query<&Window, With<PrimaryWindow>>,
    images: Res<Assets<Image>>,
    mut parallax_res: ResMut<ParallaxResource>,
) {
    let window = query_window.single();
    parallax_res.window_size = Vec2::new(window.width(), window.height());
    parallax_res.create_layers(&mut commands, &images);
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

pub fn follow_camera_system(
    camera_query: Query<&GlobalTransform, (With<ParallaxCameraComponent>, With<MainCamera>)>,
    mut layer_query: Query<(&mut Transform, &LayerComponent), Without<ParallaxCameraComponent>>,
    res_parallax: Res<ParallaxResource>,
) {
    if let Some(camera_transform) = camera_query.iter().next() {
        for (mut layer_transform, layer) in layer_query.iter_mut() {
            let layer_data = &res_parallax.layer_data[layer.index];
            let camera_translation = camera_transform.translation();

            layer_transform.translation.x = camera_translation.x + (layer_data.position.x - camera_translation.x) * layer.speed.x;
            layer_transform.translation.y = camera_translation.y + (layer_data.position.y - camera_translation.y) * layer.speed.y;
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
    camera_query: Query<(&GlobalTransform, &OrthographicProjection), With<ParallaxCameraComponent>>,
    parallax_res: Res<ParallaxResource>
) {
    if let Some((camera_transform, proj)) = camera_query.iter().next() {
        for (layer, children) in layer_query.iter() {
            for &child in children.iter() {
                let (texture_gtransform, mut texture_transform, layer_texture) =
                    texture_query.get_mut(child).unwrap();

                let texture_gtransform = texture_gtransform.compute_transform();

                // Move right-most texture to left side of layer when camera is approaching left-most end
                if camera_transform.translation().x - texture_gtransform.translation.x
                    + ((layer_texture.width * texture_gtransform.scale.x) / 2.0) 
                    < -(parallax_res.window_size.x * layer.transition_factor)
                {
                    texture_transform.translation.x -= layer_texture.width * layer.texture_count;
                // Move left-most texture to right side of layer when camera is approaching right-most end
                } else if camera_transform.translation().x - texture_gtransform.translation.x
                    - ((layer_texture.width * texture_gtransform.scale.x) / 2.0)
                    > parallax_res.window_size.x * layer.transition_factor
                {
                    texture_transform.translation.x += layer_texture.width * layer.texture_count;
                }
            }
        }
    }
}

fn update_window_size(
    mut resize_events: EventReader<WindowResized>,
    mut res_parallax: ResMut<ParallaxResource>,
    query_children: Query<&Children>,
    mut query_layer: Query<&mut Sprite>,
) {
    for event in resize_events.iter() {
        res_parallax.window_size.x = event.width;
        res_parallax.window_size.y = event.height;

        for (entity, layer_data) in res_parallax.layer_entities.iter().zip(res_parallax.layer_data.iter()) {
            if layer_data.fill_screen_height {
                let children = query_children.get(*entity).unwrap();

                for entity in children.iter() {
                    let mut sprite = query_layer.get_mut(*entity).unwrap();

                    if let Some(size) = &mut sprite.custom_size {
                        size.y = event.height;
                    }
                }
            }
        }
    }
}

fn update_texture_scale(
    res_parallax: Res<ParallaxResource>,
    camera_query: Query<(&Camera, &GlobalTransform, &OrthographicProjection), With<MainCamera>>,
    mut layer_query: Query<(&mut Transform, &LayerComponent), Without<LayerTextureComponent>>,
) {
    let (camera, camera_transform, proj) = camera_query.single();

    for (mut layer_transform, layer) in &mut layer_query {
        let layer_data = &res_parallax.layer_data[layer.index];
        // let orig_pos = camera.world_to_viewport(camera_transform, layer_transform.translation).unwrap();

        layer_transform.scale = Vec3::splat(layer_data.scale * proj.scale);

        // let new_pos = camera.viewport_to_world_2d(camera_transform, orig_pos).unwrap();

        // layer_transform.translation.x = new_pos.x;
        // layer_transform.translation.y = new_pos.y;
    }
}
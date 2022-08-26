use bevy::prelude::*;

mod layer;

pub use layer::*;

use self::layer::LayerComponent;
use iyes_loopless::{prelude::*, condition::ConditionalSystemDescriptor};

pub struct ParallaxPlugin {
    pub initial_speed: f32
}

impl Plugin for ParallaxPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(ParallaxMoveSpeed {
                speed: self.initial_speed
            })
            .add_system(initialize_parallax_system.run_if_resource_added::<ParallaxResource>())
            .add_system(
                update_layer_textures_system
                    .run_if_resource_exists::<ParallaxResource>()
                    .after("follow_camera")
            );
    }
}

pub struct ParallaxMoveSpeed {
    pub speed: f32
}

/// Resource for managing parallax
#[derive(Debug)]
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
        texture_atlasses: &Assets<TextureAtlas>
    ) {
        // Despawn any existing layers
        self.despawn_layers(commands);

        // Spawn new layers using layer_data
        for (i, layer) in self.layer_data.iter().enumerate() {
            let spritesheet_bundle = SpriteSheetBundle {
                texture_atlas: layer.image.clone(),
                ..Default::default()
            };

            let texture = texture_atlasses.get(&layer.image).unwrap();

            // Three textures always spawned
            let mut texture_count = 3.0;

            // Spawn parallax layer entity
            let mut entity_commands = commands.spawn();
            entity_commands
                .insert(Name::new(format!("Parallax Layer ({})", i)))
                .insert_bundle(SpatialBundle {
                    transform: Transform {
                        translation: Vec3::new(layer.position.x, layer.position.y, layer.z),
                        scale: Vec3::new(layer.scale, layer.scale, 1.0),
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    // Spawn center texture
                    parent.spawn_bundle(spritesheet_bundle.clone()).insert(
                        LayerTextureComponent {
                            width: texture.size.x,
                        },
                    );

                    let mut max_x = (texture.size.x / 2.0) * layer.scale;
                    let mut adjusted_spritesheet_bundle = spritesheet_bundle.clone();

                    // Spawn right texture
                    adjusted_spritesheet_bundle.transform.translation.x += texture.size.x;
                    max_x += texture.size.x * layer.scale;
                    parent
                        .spawn_bundle(adjusted_spritesheet_bundle.clone())
                        .insert(LayerTextureComponent {
                            width: texture.size.x,
                        });

                    // Spawn left texture
                    parent
                        .spawn_bundle({
                            let mut bundle = adjusted_spritesheet_bundle.clone();
                            bundle.transform.translation.x *= -1.0;
                            bundle
                        })
                        .insert(LayerTextureComponent {
                            width: texture.size.x,
                        });

                    // Spawn additional textures to make 2 windows length of background textures
                    while max_x < self.window_size.x {
                        adjusted_spritesheet_bundle.transform.translation.x += texture.size.x;
                        max_x += texture.size.x * layer.scale;
                        parent
                            .spawn_bundle(adjusted_spritesheet_bundle.clone())
                            .insert(LayerTextureComponent {
                                width: texture.size.x,
                            });

                        parent
                            .spawn_bundle({
                                let mut bundle = adjusted_spritesheet_bundle.clone();
                                bundle.transform.translation.x *= -1.0;
                                bundle
                            })
                            .insert(LayerTextureComponent {
                                width: texture.size.x,
                            });

                        texture_count += 2.0;
                    }
                });

            // Add layer component to entity
            entity_commands.insert(LayerComponent {
                speed: layer.speed,
                texture_count,
                transition_factor: layer.transition_factor,
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
pub fn move_background_system() -> ConditionalSystemDescriptor {
    follow_camera_system.run_if_resource_exists::<ParallaxResource>().label("follow_camera")
}

/// Initialize the parallax resource
fn initialize_parallax_system(
    mut commands: Commands,
    windows: Res<Windows>,
    texture_atlasses: Res<Assets<TextureAtlas>>,
    mut parallax_res: ResMut<ParallaxResource>,
) {
    let window = windows.get_primary().unwrap();
    parallax_res.window_size = Vec2::new(window.width(), window.height());
    parallax_res.create_layers(&mut commands, &texture_atlasses);
}

/// Move camera and background layers
fn follow_camera_system(
    mut camera_query: Query<&mut Transform, With<ParallaxCameraComponent>>,
    mut layer_query: Query<
        (&mut Transform, &LayerComponent),
        Without<ParallaxCameraComponent>,
    >,
    parallax_speed: Res<ParallaxMoveSpeed>
) {
    if let Some(mut camera_transform) = camera_query.iter_mut().next() {
        camera_transform.translation.x += parallax_speed.speed;
        for (mut layer_transform, layer) in layer_query.iter_mut() {
            layer_transform.translation.x += parallax_speed.speed * layer.speed;
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
            &layer::LayerTextureComponent,
        ),
        Without<ParallaxCameraComponent>,
    >,
    camera_query: Query<(&GlobalTransform, &OrthographicProjection), With<ParallaxCameraComponent>>,
    parallax_resource: Res<ParallaxResource>,
) {
    if let Some((camera_transform, projection)) = camera_query.iter().next() {
        for (layer, children) in layer_query.iter() {
            for &child in children.iter() {
                let (texture_gtransform, mut texture_transform, layer_texture) =
                    texture_query.get_mut(child).unwrap();

                let texture_gtransform = texture_gtransform.compute_transform();

                // Move right-most texture to left side of layer when camera is approaching left-most end
                if camera_transform.translation().x 
                    + (projection.left * projection.scale)
                    - texture_gtransform.translation.x
                    + ((layer_texture.width * texture_gtransform.scale.x) / 2.0)
                    < -(parallax_resource.window_size.x * layer.transition_factor)
                {
                    texture_transform.translation.x -= layer_texture.width * layer.texture_count;
                // Move left-most texture to right side of layer when camera is approaching right-most end
                } else if camera_transform.translation().x
                    + (projection.right * projection.scale)
                    - texture_gtransform.translation.x
                    - ((layer_texture.width * texture_gtransform.scale.x) / 2.0)
                    > parallax_resource.window_size.x * layer.transition_factor
                {
                    texture_transform.translation.x += layer_texture.width * layer.texture_count;
                }
            }
        }
    }
}
use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::{
        camera::RenderTarget,
        mesh::Indices,
        render_resource::{
            AsBindGroup, Extent3d, PrimitiveTopology, ShaderRef, TextureDescriptor,
            TextureDimension, TextureFormat, TextureUsages,
        },
        texture::BevyDefault,
        view::RenderLayers,
    },
    sprite::{Material2d, Material2dPlugin, MaterialMesh2dBundle},
    window::{WindowId, WindowResized}, math::Vec3Swizzles,
};
use iyes_loopless::prelude::IntoConditionalSystem;

use crate::{plugins::{world::{WorldData, LightMap}}, state::GameState};

use super::MainCamera;

pub struct LightingPlugin;

impl Plugin for LightingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(Material2dPlugin::<LightingMaterial>::default())
            .add_system(setup_new_post_processing_cameras.run_if_resource_exists::<WorldData>())
            .add_system(update_image_to_window_size)
            .add_system(update_lighting_material.run_in_state(GameState::InGame));
    }   
}

/// To support window resizing, this fits an image to a windows size.
#[derive(Component)]
struct FitToWindowSize {
    image: Handle<Image>,
    material: Handle<LightingMaterial>,
    window_id: WindowId,
}
#[derive(Component)]
pub struct PostProcessingCamera;

/// Update image size to fit window
fn update_image_to_window_size(
    windows: Res<Windows>,
    mut image_events: EventWriter<AssetEvent<Image>>,
    mut images: ResMut<Assets<Image>>,
    mut post_processing_materials: ResMut<Assets<LightingMaterial>>,
    mut resize_events: EventReader<WindowResized>,
    fit_to_window_size: Query<&FitToWindowSize>,
) {
    for resize_event in resize_events.iter() {
        for fit_to_window in fit_to_window_size.iter() {
            if resize_event.id == fit_to_window.window_id {
                let size = {
                    let window = windows.get(fit_to_window.window_id).expect("PostProcessingCamera is rendering to a window, but this window could not be found");
                    Extent3d {
                        width: window.width() as u32,
                        height: window.height() as u32,
                        ..Default::default()
                    }
                };
                let image = images.get_mut(&fit_to_window.image).expect(
                    "FitToWindowSize is referring to an Image, but this Image could not be found",
                );
                info!("resize to {:?}", size);
                image.resize(size);
                // Hack because of https://github.com/bevyengine/bevy/issues/5595
                image_events.send(AssetEvent::Modified {
                    handle: fit_to_window.image.clone(),
                });
                post_processing_materials.get_mut(&fit_to_window.material);
            }
        }
    }
}

fn update_light_map_texture(light_map: &LightMap, camera_pos: Vec2, proj: &OrthographicProjection, texture: &mut Vec<u8>) {
    let bottom = camera_pos.y + proj.bottom;
    let top = camera_pos.y + proj.top;
    let left = camera_pos.x + proj.left;
    let right = camera_pos.x + proj.right;

    for y in (bottom as usize)..(top as usize) {
        for x in (left as usize)..(right as usize) {
            let mut color = UVec4::new(0, 0, 0, 0);

            if let Some(map_color) = light_map.colors.get((x, y)) {
                color = *map_color;
            }

            let index = x + light_map.width as usize * y;

            texture[index] = (color.x & 0xFF) as u8; // R
            texture[(index) + 1] = (color.y & 0xFF) as u8; // G
            texture[(index) + 2] = (color.z & 0xFF) as u8; // B
            texture[(index) + 3] = 0xFF; // A
        }
    }
}

fn update_lighting_material(
    cameras: Query<(&GlobalTransform, &OrthographicProjection, &Handle<LightingMaterial>), With<MainCamera>>,
    mut images: ResMut<Assets<Image>>,
    mut post_processing_materials: ResMut<Assets<LightingMaterial>>,
    light_map: Res<LightMap>,
) {
    if let Ok((transform, proj, lighting_material_handle)) = cameras.get_single() {
        let camera_position = transform.translation().xy().abs();
        let mut lighting_material = post_processing_materials.get_mut(lighting_material_handle).unwrap();
        
        lighting_material.player_position = camera_position;
        lighting_material.proj = Vec4::new(proj.left, proj.top, proj.bottom, proj.right);

        if light_map.is_changed() {
            let light_map_texture = images.get_mut(&lighting_material.color_map).unwrap();

            update_light_map_texture(&light_map, camera_position, proj, &mut light_map_texture.data);
        }
    }
}

/// sets up post processing for cameras that have had `PostProcessingCamera` added
fn setup_new_post_processing_cameras(
    mut commands: Commands,
    windows: Res<Windows>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut post_processing_materials: ResMut<Assets<LightingMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut cameras: Query<(Entity, &mut Camera, &OrthographicProjection), Added<PostProcessingCamera>>,
    light_map: Res<LightMap>
) {
    for (entity, mut camera, proj) in &mut cameras {
        let original_target = camera.target.clone();

        let mut option_window_id: Option<WindowId> = None;

        // Get the size the camera is rendering to
        let size = match &camera.target {
            RenderTarget::Window(window_id) => {
                let window = windows.get(*window_id).expect(
                    "PostProcessingCamera is rendering to a window, but this window could not be found"
                );
                option_window_id = Some(*window_id);
                Extent3d {
                    width: window.width() as u32,
                    height: window.height() as u32,
                    ..Default::default()
                }
            }
            RenderTarget::Image(handle) => {
                let image = images.get(handle).expect(
                "PostProcessingCamera is rendering to an Image, but this Image could not be found",
                );
                image.texture_descriptor.size
            }
        }; 

        // let mut bytes = Vec::<u8>::with_capacity(light_map.colors.len() * 4);
        let mut bytes = vec![0; light_map.colors.len() * 4];

        for ((row, col), color) in light_map.colors.indexed_iter() {
            let index = ((row * light_map.colors.ncols()) + col) * 4;

            bytes[index] = (color.x & 0xFF) as u8;
            bytes[(index) + 1] = (color.y & 0xFF) as u8;
            bytes[(index) + 2] = (color.z & 0xFF) as u8;
            bytes[(index) + 3] = (color.w & 0xFF) as u8;
        }

        let light_map = Image::new(
            Extent3d {
                width: light_map.colors.ncols() as u32,
                height: light_map.colors.nrows() as u32,
                ..default()
            }, 
            TextureDimension::D2,
            bytes,
            TextureFormat::Rgba8UnormSrgb
        );

        let mut image = Image {
            texture_descriptor: TextureDescriptor {
                label: None,
                size,
                dimension: TextureDimension::D2,
                format: TextureFormat::bevy_default(),
                mip_level_count: 1,
                sample_count: 1,
                usage: TextureUsages::TEXTURE_BINDING
                    | TextureUsages::COPY_DST
                    | TextureUsages::RENDER_ATTACHMENT,
            },
            ..Default::default()
        };

        // fill image.data with zeroes
        image.resize(size);

        let image_handle = images.add(image);
        let light_map_handle = images.add(light_map);

        // This specifies the layer used for the post processing camera, which will be attached to the post processing camera and 2d fullscreen triangle.
        let post_processing_pass_layer =
            RenderLayers::layer((RenderLayers::TOTAL_LAYERS - 1) as u8);

        let half_extents = Vec2::new(size.width as f32 / 2f32, size.height as f32 / 2f32);
        let mut triangle_mesh = Mesh::new(PrimitiveTopology::TriangleList);
        // NOTE: positions are actually not used because the vertex shader maps UV and clip space.
        triangle_mesh.insert_attribute(
            Mesh::ATTRIBUTE_POSITION,
            vec![
                [-half_extents.x, half_extents.y, 0.0],
                [half_extents.x * 3f32, half_extents.y, 0.0],
                [-half_extents.x, half_extents.y * 3f32, 0.0],
            ],
        );
        triangle_mesh.set_indices(Some(Indices::U32(vec![0, 1, 2])));
        triangle_mesh.insert_attribute(
            Mesh::ATTRIBUTE_NORMAL,
            vec![[0.0, 0.0, 1.0], [0.0, 0.0, 1.0], [0.0, 0.0, 1.0]],
        );

        triangle_mesh.insert_attribute(
            Mesh::ATTRIBUTE_UV_0,
            vec![[2.0, 0.0], [0.0, 2.0], [0.0, 0.0]],
        );

        let triangle_handle = meshes.add(triangle_mesh);

        // This material has the texture that has been rendered.
        let material_handle = post_processing_materials.add(LightingMaterial {
            source_image: image_handle.clone(),
            color_map: light_map_handle,
            player_position: Vec2::default(),
            proj: Vec4::new(proj.left, proj.top, proj.bottom, proj.right)
        });

        commands
            .entity(entity)
            // add the handle to the camera so we can access it and change its properties
            .insert(material_handle.clone())
            // also disable show_ui so UI elements don't get rendered twice
            .insert(UiCameraConfig { show_ui: false });
        if let Some(window_id) = option_window_id {
            commands.entity(entity).insert(FitToWindowSize {
                image: image_handle.clone(),
                material: material_handle.clone(),
                window_id,
            });
        }
        camera.target = RenderTarget::Image(image_handle);

        // Post processing 2d fullscreen triangle, with material using the render texture done by the main camera, with a custom shader.
        commands
            .spawn((
                MaterialMesh2dBundle {
                    mesh: triangle_handle.into(),
                    material: material_handle,
                    transform: Transform {
                        translation: Vec3::new(0.0, 0.0, 1.5),
                        ..Default::default()
                    },
                    ..Default::default()
                },
                post_processing_pass_layer
            ));

        // The post-processing pass camera.
        commands
            .spawn((
                Camera2dBundle {
                    camera: Camera {
                        // renders after the first main camera which has default value: 0.
                        priority: camera.priority + 10,
                        // set this new camera to render to where the other camera was rendering
                        target: original_target,
                        
                        ..Default::default()
                    },
                    projection: OrthographicProjection { 
                        scale: 0.9,
                        ..default()
                    },
                    transform: Transform::from_xyz(0., 0., 500.),
                    ..default()
                },
                post_processing_pass_layer
            ));
    }
}

#[derive(AsBindGroup, TypeUuid, Clone)]
#[uuid = "9114bbd2-1bb3-4b5a-a710-8965798db745"]
pub struct LightingMaterial {
    #[texture(0)]
    #[sampler(1)]
    source_image: Handle<Image>,

    #[texture(2)]
    #[sampler(3)]
    color_map: Handle<Image>,

    #[uniform(4)]
    player_position: Vec2,

    #[uniform(5)]
    proj: Vec4
}

impl Material2d for LightingMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/lighting.wgsl".into()
    }
    fn vertex_shader() -> ShaderRef {
        "shaders/screen_vertex.wgsl".into()
    }
}
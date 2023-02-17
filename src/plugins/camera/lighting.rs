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
    window::{WindowId, WindowResized},
};
use bevy_ecs_tilemap::tiles::TilePos;
use iyes_loopless::prelude::IntoConditionalSystem;

use crate::{world_generator::{WORLD_SIZE_X, WORLD_SIZE_Y}, plugins::world::WorldData, CellArrayExtensions};

pub struct LightingPlugin;

impl Plugin for LightingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(Material2dPlugin::<LightingMaterial>::default())
            .add_system(setup_new_post_processing_cameras.run_if_resource_exists::<WorldData>())
            .add_system(update_image_to_window_size);
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

/// sets up post processing for cameras that have had `PostProcessingCamera` added
fn setup_new_post_processing_cameras(
    mut commands: Commands,
    windows: Res<Windows>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut post_processing_materials: ResMut<Assets<LightingMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut cameras: Query<(Entity, &mut Camera), With<PostProcessingCamera>>,
    world_data: Res<WorldData>
) {
    for (entity, mut camera) in &mut cameras {
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

        let tilemap_size = Extent3d {
            width: WORLD_SIZE_X as u32,
            height: WORLD_SIZE_Y as u32,
            ..default()
        };

        let mut colors = Vec::<Vec3>::with_capacity(tilemap_size.width as usize * tilemap_size.height as usize * 4);

        for y in 0..tilemap_size.height {
            for x in 0..tilemap_size.width {
                if let Some(_) = world_data.tiles.get_wall(TilePos::new(x, y)) {
                    if let Some(_) = world_data.tiles.get_tile(TilePos::new(x, y)) {
                        colors.push(Vec3::new(0.2, 0.2, 0.2));
                        continue;
                    }
                }
                colors.push(Vec3::new(1., 1., 1.));
            }
        }

        // for y in 0..tilemap_size.height {
        //     for x in 0..tilemap_size.width {
        //         if let Some(_) = world_data.tiles.get_wall(TilePos::new(x, y)) {
        //             if let Some(_) = world_data.tiles.get_tile(TilePos::new(x, y)) {
        //                 bytes.push(0xff); // R
        //                 bytes.push(0xff); // G
        //                 bytes.push(0xff); // B
        //                 bytes.push(0xff); // A
        //                 continue;
        //             }
        //         }

        //         bytes.push(0x10); // R
        //         bytes.push(0x10); // G
        //         bytes.push(0x10); // B
        //         bytes.push(0xff); // A
        //     }
        // }


        // This is the texture that will be rendered to.
        // let shadow_map = Image::new(
        //     tilemap_size, 
        //     TextureDimension::D2, 
        //     bytes, 
        //     TextureFormat::Rgba8UnormSrgb
        // );

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
        // let shadow_map_handle = images.add(shadow_map);

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
            color_map: colors
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
                    ..Camera2dBundle::default()
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

    #[uniform(4)]
    color_map: Vec<Vec3>
}

impl Material2d for LightingMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/lighting.wgsl".into()
    }
    fn vertex_shader() -> ShaderRef {
        "shaders/screen_vertex.wgsl".into()
    }
}
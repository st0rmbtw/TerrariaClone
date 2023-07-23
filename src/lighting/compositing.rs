use bevy::{
    math::Vec3Swizzles,
    prelude::{*, shape::Quad},
    reflect::{TypeUuid, TypePath},
    render::{
        camera::RenderTarget,
        mesh::{InnerMeshVertexBufferLayout},
        render_resource::{
            AsBindGroup, Extent3d, ShaderRef,
            TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, SpecializedMeshPipelineError, RenderPipelineDescriptor, PrimitiveState,
        },
        texture::{BevyDefault},
        view::RenderLayers,
    },
    sprite::{Material2d, MaterialMesh2dBundle, Material2dKey},
    window::{WindowResized, PrimaryWindow}, core_pipeline::fullscreen_vertex_shader::FULLSCREEN_SHADER_HANDLE, utils::Hashed,
};

use crate::{
    world::light,
    plugins::{
        world::{LightMap, TILE_SIZE},
        camera::{MainCamera, UpdateLightEvent, LightMapCamera}
    },
    world::WorldData
};

use super::pipeline::PipelineTargetsWrapper;


/// To support window resizing, this fits an image to a windows size.
#[derive(Component)]
pub(super) struct FitToWindowSize {
    pub(super) image: Handle<Image>,
}

#[derive(AsBindGroup, TypePath, TypeUuid, Clone)]
#[uuid = "9114bbd2-1bb3-4b5a-a710-8965798db745"]
pub(super) struct PostProcessingMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub(super) source_image: Handle<Image>,

    #[texture(2)]
    #[sampler(3)]
    pub(super) shadow_map_image: Handle<Image>,

    #[texture(4)]
    #[sampler(5)]
    pub(super) light_sources_image: Handle<Image>,

    #[uniform(6)]
    pub(super) player_position: Vec2,

    #[uniform(7)]
    pub(super) scale: f32,

    #[uniform(8)]
    pub(super) world_size: Vec2,

    #[uniform(9)]
    pub(super) enabled: u32
}

impl Material2d for PostProcessingMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/post_processing.wgsl".into()
    }
    fn vertex_shader() -> ShaderRef {
        FULLSCREEN_SHADER_HANDLE.typed().into()
    }

    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        _: &Hashed<InnerMeshVertexBufferLayout>,
        _: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        descriptor.primitive = PrimitiveState::default();
        descriptor.vertex.entry_point = "fullscreen_vertex_shader".into();
        Ok(())
    }
}

#[derive(Component)]
pub(super) struct ShadowMap;

/// Update image size to fit window
pub(super) fn update_image_to_window_size(
    mut images: ResMut<Assets<Image>>,
    mut resize_events: EventReader<WindowResized>,
    fit_to_window_size: Query<&FitToWindowSize>,
) {
    for event in resize_events.iter() {
        if event.width > 0. && event.height > 0. {
            for fit_to_window in fit_to_window_size.iter() {
                let size = {
                    Extent3d {
                        width: event.width as u32,
                        height: event.height as u32,
                        ..Default::default()
                    }
                };
                let image = images.get_mut(&fit_to_window.image).expect(
                    "FitToWindowSize is referring to an Image, but this Image could not be found",
                );
                info!("resize to {:?}", size);
                image.resize(size);
            }
        }
    }
}

pub(super) fn update_lighting_material(
    cameras: Query<
        (
            &GlobalTransform,
            &OrthographicProjection,
            &Handle<PostProcessingMaterial>,
        ),
        (With<MainCamera>, Or<(Changed<GlobalTransform>, Changed<OrthographicProjection>)>)
    >,
    mut post_processing_materials: ResMut<Assets<PostProcessingMaterial>>,
) {
    if let Ok((transform, proj, lighting_material_handle)) = cameras.get_single() {
        let camera_position = transform.translation().xy().abs();
        let mut lighting_material = post_processing_materials
            .get_mut(lighting_material_handle)
            .unwrap();

        lighting_material.player_position = camera_position;
        lighting_material.scale = proj.scale;
    }
}

pub(super) fn update_light_map(
    query_camera: Query<(&Handle<PostProcessingMaterial>, &OrthographicProjection, &GlobalTransform), With<MainCamera>>,
    mut update_light_events: EventReader<UpdateLightEvent>,
    post_processing_materials: Res<Assets<PostProcessingMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut light_map: ResMut<LightMap>,
    world_data: Res<WorldData>
) {
    if let Ok((lighting_material_handle, projection, camera_transform)) = query_camera.get_single() {
        if update_light_events.iter().last().is_some() {
            let lighting_material = post_processing_materials
                .get(lighting_material_handle)
                .unwrap();
            let light_map_texture = images.get_mut(&lighting_material.shadow_map_image).unwrap();
            
            let x_from = ((camera_transform.translation().x + projection.area.min.x) / TILE_SIZE * light::CLUSTER_SIZE as f32) as usize;
            let x_to = ((camera_transform.translation().x + projection.area.max.x) / TILE_SIZE * light::CLUSTER_SIZE as f32) as usize;

            let y_from = ((camera_transform.translation().y + projection.area.max.y) / TILE_SIZE * light::CLUSTER_SIZE as f32).abs() as usize;
            let y_to = ((camera_transform.translation().y + projection.area.min.y) / TILE_SIZE * light::CLUSTER_SIZE as f32).abs() as usize;

            for y in y_from..y_to {
                for x in x_from..x_to {
                    light::propagate_light(x, y, &mut light_map.colors, &world_data);

                    if let Some(color) = light_map.colors.get((y, x)) {
                        let index = ((y * light_map.colors.ncols()) + x) * 4;
                        light_map_texture.data[index]     = *color; // R
                        light_map_texture.data[index + 1] = *color; // G
                        light_map_texture.data[index + 2] = *color; // B
                        light_map_texture.data[index + 3] = 0xFF; // A
                    }
                }
            }

            for y in (y_from..y_to).rev() {
                for x in (x_from..x_to).rev() {
                    light::propagate_light(x, y, &mut light_map.colors, &world_data);

                    if let Some(color) = light_map.colors.get((y, x)) {
                        let index = ((y * light_map.colors.ncols()) + x) * 4;
                        light_map_texture.data[index]     = *color; // R
                        light_map_texture.data[index + 1] = *color; // G
                        light_map_texture.data[index + 2] = *color; // B
                        light_map_texture.data[index + 3] = 0xFF; // A
                    }
                }
            }
        }
    }
}       

pub(super) fn setup_post_processing_camera(
    mut commands: Commands,
    query_windows: Query<&Window, With<PrimaryWindow>>,
    mut query_camera: Query<(Entity, &mut Camera, &OrthographicProjection), Added<LightMapCamera>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut shadow_map_meterials: ResMut<Assets<PostProcessingMaterial>>,
    mut images: ResMut<Assets<Image>>,
    light_map: Res<LightMap>,
    world_data: Res<WorldData>,
    gpu_targets_wrapper: Res<PipelineTargetsWrapper>,
) {
    for (entity, mut camera, proj) in &mut query_camera {
        let original_target = camera.target.clone();

        let window = query_windows.single();

        // Get the size the camera is rendering to
        let size = if let RenderTarget::Window(_) = &camera.target {
            Extent3d {
                width: window.width() as u32,
                height: window.height() as u32,
                ..Default::default()
            }
        } else { 
            panic!("PostProcessingCamera isn't rendering to a camera") 
        };

        let mut bytes = vec![0; light_map.colors.len() * 4];

        for ((row, col), color) in light_map.colors.indexed_iter() {
            let index = ((row * light_map.colors.ncols()) + col) * 4;

            bytes[index]     = *color;
            bytes[index + 1] = *color;
            bytes[index + 2] = *color;
            bytes[index + 3] = 0xFF;
        }

        let shadow_map_image = Image::new(
            Extent3d {
                width: light_map.colors.ncols() as u32,
                height: light_map.colors.nrows() as u32,
                ..default()
            },
            TextureDimension::D2,
            bytes,
            TextureFormat::Rgba8UnormSrgb,
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
                view_formats: &[]
            },
            ..Default::default()
        };

        image.resize(size);

        let image_handle = images.add(image);
        let shadow_map_image_handle = images.add(shadow_map_image);

        // This specifies the layer used for the post processing camera, which will be attached to the post processing camera and 2d fullscreen triangle.
        let post_processing_pass_layer = RenderLayers::layer((RenderLayers::TOTAL_LAYERS - 1) as u8);

        // This material has the texture that has been rendered.
        let material_handle = shadow_map_meterials.add(PostProcessingMaterial {
            player_position: Vec2::default(),
            scale: proj.scale,
            source_image: image_handle.clone(),
            shadow_map_image: shadow_map_image_handle,
            light_sources_image: gpu_targets_wrapper
                .targets
                .as_ref()
                .expect("Targets must be initialized")
                .lighting_target
                .clone(),
            world_size: Vec2::new(world_data.size.width as f32, world_data.size.height as f32),
            enabled: 1
        });

        commands
            .entity(entity)
            // add the handle to the camera so we can access it and change its properties
            .insert(material_handle.clone())
            // also disable show_ui so UI elements don't get rendered twice
            .insert(UiCameraConfig { show_ui: false })
            .insert(FitToWindowSize {
                image: image_handle.clone(),
            });
        
        camera.target = RenderTarget::Image(image_handle);

        commands.spawn((
            ShadowMap,
            MaterialMesh2dBundle {
                mesh: meshes.add(Mesh::from(Quad::new(Vec2::new(1., 1.)))).into(),
                material: material_handle,
                transform: Transform {
                    translation: Vec3::new(0.0, 0.0, 1.5),
                    ..Default::default()
                },
                ..Default::default()
            },
            post_processing_pass_layer,
        ));

        // The post processing pass camera.
        commands.spawn((
            Camera2dBundle {
                camera: Camera {
                    // renders after the first main camera which has default value: 0.
                    order: camera.order + 10,
                    // set this new camera to render to where the other camera was rendering
                    target: original_target,
                    ..default()
                },
                transform: Transform::from_xyz(0., 0., 500.),
                ..default()
            },
            post_processing_pass_layer,
        ));
    }
}

#[cfg(feature = "debug")]
use crate::plugins::debug::DebugConfiguration;

#[cfg(feature = "debug")]
pub(super) fn set_shadow_map_visibility(
    debug_config: Res<DebugConfiguration>,
    query_camera: Query<&Handle<PostProcessingMaterial>, With<MainCamera>>,
    mut post_processing_materials: ResMut<Assets<PostProcessingMaterial>>,
) {
    if debug_config.is_changed() {
        if let Ok(post_processing_material_handle) = query_camera.get_single() {
            if let Some(mut material) = post_processing_materials.get_mut(post_processing_material_handle) {
                material.enabled = if debug_config.shadow_tiles { 1 } else { 0 };
            }
        }
    }
}
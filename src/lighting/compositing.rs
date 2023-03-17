use bevy::{
    math::Vec3Swizzles,
    prelude::{*, shape::Quad},
    reflect::TypeUuid,
    render::{
        camera::RenderTarget,
        mesh::{InnerMeshVertexBufferLayout},
        render_resource::{
            AddressMode, AsBindGroup, Extent3d, SamplerDescriptor, ShaderRef,
            TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, SpecializedMeshPipelineError, RenderPipelineDescriptor, PrimitiveState,
        },
        texture::{BevyDefault, ImageSampler},
        view::RenderLayers,
    },
    sprite::{Material2d, MaterialMesh2dBundle, Material2dKey},
    window::{WindowResized, PrimaryWindow}, core_pipeline::fullscreen_vertex_shader::FULLSCREEN_SHADER_HANDLE, utils::Hashed,
};

use crate::{
    plugins::{
        world::LightMap,
        camera::{MainCamera, UpdateLightEvent}
    },
};

use super::pipeline::PipelineTargetsWrapper;


/// To support window resizing, this fits an image to a windows size.
#[derive(Component)]
pub struct FitToWindowSize {
    image: Handle<Image>,
}

#[derive(AsBindGroup, TypeUuid, Clone)]
#[uuid = "9114bbd2-1bb3-4b5a-a710-8965798db745"]
pub struct PostProcessingMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub source_image: Handle<Image>,

    #[texture(2)]
    #[sampler(3)]
    pub shadow_map_image: Handle<Image>,

    #[texture(4)]
    #[sampler(5)]
    pub light_sources_image: Handle<Image>,

    #[uniform(6)]
    pub player_position: Vec2,

    #[uniform(7)]
    pub scale: f32,
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
pub struct LightMapCamera;

/// Update image size to fit window
pub fn update_image_to_window_size(
    query_windows: Query<&Window, With<PrimaryWindow>>,
    mut images: ResMut<Assets<Image>>,
    mut resize_events: EventReader<WindowResized>,
    fit_to_window_size: Query<&FitToWindowSize>,
) {
    for _ in resize_events.iter() {
        for fit_to_window in fit_to_window_size.iter() {
            let window = query_windows.get_single().expect("No primary window");

            let size = {
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
        }
    }
}

fn update_light_map_texture(
    tile_x: usize,
    tile_y: usize,
    light_map: &LightMap,
    texture_data: &mut [u8],
) {
    for row in -4_i32..=4_i32 {
        for col in -4_i32..=4_i32 {
            let y = ((tile_y as i32) + row) as usize;
            let x = ((tile_x as i32) + col) as usize;

            if let Some(color) = light_map.colors.get((y, x)) {
                let index = ((y * light_map.colors.ncols()) + x) * 4;

                texture_data[index]     = *color; // R
                texture_data[index + 1] = *color; // G
                texture_data[index + 2] = *color; // B
                texture_data[index + 3] = 0xFF; // A
            }
        }
    }
}

pub fn update_lighting_material(
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

pub fn update_light_map(
    cameras: Query<&Handle<PostProcessingMaterial>, With<MainCamera>>,
    mut update_light_events: EventReader<UpdateLightEvent>,
    post_processing_materials: Res<Assets<PostProcessingMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut light_map: ResMut<LightMap>
) {
    if let Ok(lighting_material_handle) = cameras.get_single() {
        if !update_light_events.is_empty() {
            let lighting_material = post_processing_materials
                .get(lighting_material_handle)
                .unwrap();
            let light_map_texture = images.get_mut(&lighting_material.shadow_map_image).unwrap();

            for UpdateLightEvent { tile_pos, color } in update_light_events.iter() {
                let x = tile_pos.x as usize;
                let y = tile_pos.y as usize;
                
                light_map.colors[(y, x)] = *color;
                update_light_map_texture(x, y, &light_map, &mut light_map_texture.data);
            }
        }
    }
}       

pub fn setup_post_processing_camera(
    mut commands: Commands,
    query_windows: Query<&Window, With<PrimaryWindow>>,
    mut query_camera: Query<(Entity, &mut Camera, &OrthographicProjection), Added<LightMapCamera>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut shadow_map_meterials: ResMut<Assets<PostProcessingMaterial>>,
    mut images: ResMut<Assets<Image>>,
    light_map: Res<LightMap>,
    gpu_targets_wrapper: Res<PipelineTargetsWrapper>,
) {
    for (entity, mut camera, proj) in &mut query_camera {
        let original_target = camera.target.clone();

        let window = query_windows.single();

        // Get the size the camera is rendering to
        let size = match &camera.target {
            RenderTarget::Window(_) => {
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
            sampler_descriptor: ImageSampler::Descriptor(SamplerDescriptor {
                address_mode_u: AddressMode::ClampToBorder,
                address_mode_v: AddressMode::ClampToBorder,
                address_mode_w: AddressMode::ClampToBorder,
                ..default()
            }),
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
                projection: OrthographicProjection {
                    scale: 0.9,
                    ..default()
                },
                transform: Transform::from_xyz(0., 0., 500.),
                ..default()
            },
            post_processing_pass_layer,
        ));
    }
}

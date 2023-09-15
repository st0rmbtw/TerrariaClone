use bevy::{
    prelude::{*, shape::Quad},
    render::{render_resource::{
        Extent3d, ShaderRef,
        TextureDimension, AsBindGroup, RenderPipelineDescriptor, SpecializedMeshPipelineError, TextureUsages,
    }, texture::BevyDefault, camera::RenderTarget, view::RenderLayers, mesh::InnerMeshVertexBufferLayout}, reflect::{TypePath, TypeUuid}, sprite::{Material2d, MaterialMesh2dBundle, Material2dKey}, window::{PrimaryWindow, WindowResized}, core_pipeline::fullscreen_vertex_shader::FULLSCREEN_SHADER_HANDLE, utils::Hashed,
};

use crate::plugins::{camera::components::{WorldCamera, MainCamera, BackgroundCamera, InGameBackgroundCamera}, DespawnOnGameExit, cursor::position::CursorPosition};

use super::types::LightSource;

#[derive(AsBindGroup, TypePath, TypeUuid, Clone, Default)]
#[uuid = "9114bbd2-1bb3-4b5a-a710-1235798db745"]
pub(crate) struct LightMapMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub(crate) light_map_image: Handle<Image>,

    #[uniform(2)]
    pub(crate) chunk_pos: UVec2,

    #[uniform(3)]
    pub(crate) world_size: Vec2
}

impl Material2d for LightMapMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/tile_material.wgsl".into()
    }
}

#[derive(AsBindGroup, TypePath, TypeUuid, Clone, Default)]
#[uuid = "d2536358-2824-45c5-9e53-90170edbc9b2"]
pub(crate) struct PostProcessingMaterial {
    #[texture(0)]
    #[sampler(1)]
    pub(crate) background_texture: Handle<Image>,

    #[texture(2)]
    #[sampler(3)]
    pub(crate) ingame_background_texture: Handle<Image>,
    
    #[texture(4)]
    #[sampler(5)]
    pub(crate) world_texture: Handle<Image>,

    #[texture(6)]
    #[sampler(7)]
    pub(crate) main_texture: Handle<Image>,
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
        descriptor.vertex.entry_point = "fullscreen_vertex_shader".into();
        Ok(())
    }
}

#[derive(Component, Deref)]
pub(crate) struct FitToWindowSize(Handle<Image>);

/// Update image size to fit window
pub(super) fn update_image_to_window_size(
    materials: Res<Assets<PostProcessingMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut resize_events: EventReader<WindowResized>,
    mut asset_events: EventWriter<AssetEvent<PostProcessingMaterial>>,
    fit_to_window_size: Query<&FitToWindowSize>,
) {
    if resize_events.is_empty() { return; }

    if let Some(event) = resize_events.iter().last() {
        if event.width > 0. && event.height > 0. {
            for fit_to_window in fit_to_window_size.iter() {
                let size = {
                    Extent3d {
                        width: event.width as u32,
                        height: event.height as u32,
                        ..Default::default()
                    }
                };
                let image = images.get_mut(fit_to_window).expect(
                    "FitToWindowSize is referring to an Image, but this Image could not be found",
                );
                image.resize(size);
            }
        }
    }

    for id in materials.ids() {
        asset_events.send(AssetEvent::Modified { handle: Handle::weak(id) });
    }
}

pub(super) fn setup_post_processing_camera(
    mut commands: Commands,

    mut materials: ResMut<Assets<PostProcessingMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,

    query_window: Query<&Window, With<PrimaryWindow>>,
    mut query_main_camera: Query<&mut Camera, With<MainCamera>>,
    mut query_world_camera: Query<&mut Camera, (With<WorldCamera>, Without<MainCamera>)>,
    mut query_background_camera: Query<&mut Camera, (With<BackgroundCamera>, Without<MainCamera>, Without<WorldCamera>)>,
    mut query_ingame_background_camera: Query<&mut Camera, (With<InGameBackgroundCamera>, Without<BackgroundCamera>, Without<MainCamera>, Without<WorldCamera>)>,
) {
    let Ok(mut main_camera) = query_main_camera.get_single_mut() else { return; };
    let Ok(mut world_camera) = query_world_camera.get_single_mut() else { return; };
    let Ok(mut background_camera) = query_background_camera.get_single_mut() else { return; };
    let Ok(mut ingame_background_camera) = query_ingame_background_camera.get_single_mut() else { return; };

    let window = query_window.single();
    
    let size = Extent3d {
        width: window.width() as u32,
        height: window.height() as u32,
        ..default()
    };

    let mut main_texture = Image::new_fill(
        size,
        TextureDimension::D2,
        &[0, 0, 0, 255],
        BevyDefault::bevy_default()
    );

    let mut world_texture = Image::new_fill(
        size,
        TextureDimension::D2,
        &[0, 0, 0, 255],
        BevyDefault::bevy_default()
    );

    let mut background_texture = Image::new_fill(
        size,
        TextureDimension::D2,
        &[0, 0, 0, 255],
        BevyDefault::bevy_default()
    );

    let mut ingame_background_texture = Image::new_fill(
        size,
        TextureDimension::D2,
        &[0, 0, 0, 255],
        BevyDefault::bevy_default()
    );

    main_texture.texture_descriptor.usage = TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT;
    world_texture.texture_descriptor.usage = TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT;
    background_texture.texture_descriptor.usage = TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT;
    ingame_background_texture.texture_descriptor.usage = TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING | TextureUsages::RENDER_ATTACHMENT;

    let main_texture_handle = images.add(main_texture);
    let world_texture_handle = images.add(world_texture);
    let background_texture_handle = images.add(background_texture);
    let ingame_background_texture_handle = images.add(ingame_background_texture);

    main_camera.target = RenderTarget::Image(main_texture_handle.clone());
    world_camera.target = RenderTarget::Image(world_texture_handle.clone());
    background_camera.target = RenderTarget::Image(background_texture_handle.clone());
    ingame_background_camera.target = RenderTarget::Image(ingame_background_texture_handle.clone());

    commands.spawn((
        DespawnOnGameExit,
        FitToWindowSize(main_texture_handle.clone())
    ));
    commands.spawn((
        DespawnOnGameExit,
        FitToWindowSize(world_texture_handle.clone())
    ));
    commands.spawn((
        DespawnOnGameExit,
        FitToWindowSize(background_texture_handle.clone())
    ));
    commands.spawn((
        DespawnOnGameExit,
        FitToWindowSize(ingame_background_texture_handle.clone())
    ));

    let post_processing_layer = RenderLayers::layer(RenderLayers::TOTAL_LAYERS as u8 - 1);

    commands.spawn((
        DespawnOnGameExit,
        MaterialMesh2dBundle {
            mesh: meshes.add(Quad::new(Vec2::new(1., 1.)).into()).into(),
            material: materials.add(PostProcessingMaterial {
                background_texture: background_texture_handle,
                ingame_background_texture: ingame_background_texture_handle,
                main_texture: main_texture_handle,
                world_texture: world_texture_handle,
            }),
            transform: Transform::from_xyz(0., 0., 1.5),
            ..default()
        },
        post_processing_layer
    ));

    commands.spawn((
        DespawnOnGameExit,
        Camera2dBundle {
            camera: Camera {
                order: 100,
                ..default()
            },
            transform: Transform::from_xyz(0., 0., 500.),
            ..default()
        },
        post_processing_layer
    ));
}

#[derive(Component)]
pub(super) struct MouseLight;

pub(super) fn spawn_mouse_light(
    mut commands: Commands
) {
    commands.spawn((
        SpatialBundle::default(),
        LightSource {
            size: UVec2::splat(2)
        },
        MouseLight
    ));
}

pub(super) fn update_mouse_light(
    cursor_pos: Res<CursorPosition<MainCamera>>,
    mut query_mouse_light: Query<&mut Transform, With<MouseLight>>
) {
    let Ok(mut light_transform) = query_mouse_light.get_single_mut() else { return; };

    light_transform.translation.x = cursor_pos.world.x;
    light_transform.translation.y = cursor_pos.world.y;
}
use bevy::{
    prelude::{*},
    render::{render_resource::{
        Extent3d,
        TextureDimension, TextureUsages,
    }, texture::BevyDefault, camera::RenderTarget, view::RenderLayers}, window::{PrimaryWindow, WindowResized}, core_pipeline::tonemapping::Tonemapping,
};
use rand::{thread_rng, Rng};

use crate::plugins::{camera::components::{WorldCamera, MainCamera, BackgroundCamera, InGameBackgroundCamera}, DespawnOnGameExit, cursor::position::CursorPosition};

use super::{types::LightSource, MainTexture, WorldTexture, BackgroundTexture, InGameBackgroundTexture, PostProcessCamera};

#[derive(Component, Deref)]
pub(crate) struct FitToWindowSize(Handle<Image>);

/// Update image size to fit window
pub(super) fn update_image_to_window_size(
    mut images: ResMut<Assets<Image>>,
    mut resize_events: EventReader<WindowResized>,
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
}

pub(super) fn setup_post_processing_camera(
    mut commands: Commands,
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

    commands.insert_resource(MainTexture(main_texture_handle));
    commands.insert_resource(WorldTexture(world_texture_handle));
    commands.insert_resource(BackgroundTexture(background_texture_handle));
    commands.insert_resource(InGameBackgroundTexture(ingame_background_texture_handle));

    commands.spawn((
        DespawnOnGameExit,
        PostProcessCamera,
        Camera2dBundle {
            camera: Camera {
                order: 100,
                msaa_writeback: false,
                ..default()
            },
            transform: Transform::from_xyz(0., 0., 500.),
            tonemapping: Tonemapping::None,
            ..default()
        },
        RenderLayers::none()
    ));
}

#[derive(Component)]
pub(super) struct MouseLight;

pub(super) fn spawn_mouse_light(
    mut commands: Commands
) {
    commands.spawn((
        DespawnOnGameExit,
        SpatialBundle::default(),
        LightSource {
            size: UVec2::splat(1),
            color: Vec4::from(Color::RED).truncate(),
            intensity: 1.,
            jitter_intensity: 0.2,
        }, 
        MouseLight
    ));
}

pub(super) fn update_mouse_light(
    input: Res<Input<MouseButton>>,
    cursor_pos: Res<CursorPosition<MainCamera>>,
    mut query_mouse_light: Query<(&mut Transform, &mut LightSource), With<MouseLight>>
) {
    let Ok((mut light_transform, mut light_source)) = query_mouse_light.get_single_mut() else { return; };

    let mut rng = thread_rng();

    light_transform.translation.x = cursor_pos.world.x;
    light_transform.translation.y = cursor_pos.world.y;

    if input.just_pressed(MouseButton::Right) {
        light_source.color = Vec4::from(Color::rgb(rng.gen(), rng.gen(), rng.gen())).truncate();
    }
}
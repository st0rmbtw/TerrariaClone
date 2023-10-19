use bevy::{prelude::{Commands, Res, Assets, Mesh, ResMut, UiCameraConfig, Camera2dBundle, default, shape::Quad, Color, Visibility, Camera, Query, Without, With, Input, MouseButton, EventReader, Transform, Vec3, Image, Handle, AssetEvent, EventWriter}, sprite::{ColorMaterial, MaterialMesh2dBundle}, core_pipeline::tonemapping::Tonemapping, input::mouse::{MouseWheel, MouseMotion}, math::Vec3Swizzles, render::render_resource::{TextureDimension, TextureFormat, Extent3d}};

use crate::{world::{WorldData, wall::WallType}, plugins::{DespawnOnGameExit, ui::resources::{Visible, Ui}, camera::components::MainCamera, assets::BackgroundAssets, world::{events::{PlaceTileEvent, BreakTileEvent}, TileType}}, common::math::map_range_usize};

use super::{WorldMapTexture, WORLD_MAP_VIEW_RENDER_LAYER, WorldMapViewCamera, WorldMapView, MapViewStatus};

pub(super) fn setup(
    mut commands: Commands,
    world_map_texture: Res<WorldMapTexture>,
    world_data: Res<WorldData>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>
) {
    commands.spawn((
        WorldMapViewCamera,
        DespawnOnGameExit,
        Camera2dBundle {
            tonemapping: Tonemapping::None,
            ..default()
        },
        UiCameraConfig { show_ui: false },
        WORLD_MAP_VIEW_RENDER_LAYER
    ));

    commands.spawn((
        WorldMapView,
        MaterialMesh2dBundle {
            mesh: meshes.add(Quad::new(world_data.playable_area.size().as_vec2()).into()).into(),
            material: materials.add(ColorMaterial {
                color: Color::WHITE,
                texture: Some(world_map_texture.clone_weak()),
            }),
            visibility: Visibility::Hidden,
            ..default()
        },
        WORLD_MAP_VIEW_RENDER_LAYER
    ));
}

pub(super) fn toggle_world_map_view(
    mut map_view_status: ResMut<MapViewStatus>,
    mut ui_visibility: ResMut<Visible<Ui>>,
    mut query_camera: Query<&mut Camera, (Without<WorldMapViewCamera>, Without<MainCamera>)>,
    mut query_map_view_camera: Query<&mut Camera, With<WorldMapViewCamera>>,
    mut query_map_view: Query<&mut Visibility, With<WorldMapView>>,
) {
    let map_view_opened = !map_view_status.is_opened();
    map_view_status.set_opened(map_view_opened);

    for mut camera in &mut query_camera {
        camera.is_active = !map_view_opened;
    }

    **ui_visibility = !map_view_opened;

    let Ok(mut map_view_camera) = query_map_view_camera.get_single_mut() else { return; };
    let Ok(mut map_view_visibility) = query_map_view.get_single_mut() else { return; };

    map_view_camera.is_active = map_view_opened;
    *map_view_visibility = match map_view_opened {
        true => Visibility::Visible,
        false => Visibility::Hidden,
    }
}

pub(super) fn update_map_view(
    world_data: Res<WorldData>,
    mouse_input: Res<Input<MouseButton>>,
    mut mouse_wheel: EventReader<MouseWheel>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut query_map_view: Query<&mut Transform, With<WorldMapView>>,
) {
    let mut transform = query_map_view.single_mut();
    
    let map_default_size = world_data.playable_area.size().as_vec2();

    for event in mouse_wheel.iter() {
        let old_normalized = transform.translation.xy() / (map_default_size * transform.scale.xy());

        let new_scale = transform.scale + transform.scale * Vec3::splat(event.y / 6.);
        transform.scale = new_scale.clamp(Vec3::splat(0.5), Vec3::splat(20.));

        let new_normalized = transform.translation.xy() / (map_default_size * transform.scale.xy());

        let delta = old_normalized - new_normalized;

        transform.translation.x += map_default_size.x * transform.scale.x * delta.x;
        transform.translation.y += map_default_size.y * transform.scale.y * delta.y;
    }

    if mouse_input.pressed(MouseButton::Left) {
        for event in mouse_motion.iter() {
            transform.translation += Vec3::new(event.delta.x, -event.delta.y, 0.);
        }
    }
}

pub(super) fn clamp_map_view_position(
    world_data: Res<WorldData>,
    mut query_map_view: Query<&mut Transform, With<WorldMapView>>,
) {
    let mut transform = query_map_view.single_mut();

    let map_size = world_data.playable_area.half_size().as_vec2() * transform.scale.xy();

    let clamped_pos = transform.translation.xy().clamp(-map_size, map_size);

    transform.translation.x = clamped_pos.x;
    transform.translation.y = clamped_pos.y;
}

pub(super) fn init_world_map_texture(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    world_data: Res<WorldData>,
    background_assets: Res<BackgroundAssets>
) {
    let sky_image = images.get(&background_assets.background_0).unwrap();
    let sky_image_height = sky_image.texture_descriptor.size.height as usize;
    let sky_image_width = sky_image.texture_descriptor.size.width as usize;

    let mut bytes = vec![0u8; world_data.playable_width() * world_data.playable_height() * 4];

    for y in 0..world_data.playable_height() {
        for x in 0..world_data.playable_width() {
            let world_x = world_data.playable_area.min.x as usize + x;
            let world_y = world_data.playable_area.min.y as usize + y;

            let pos = (world_x, world_y);

            let index = ((y * world_data.playable_width()) + x) * 4;

            if let Some(block) = world_data.get_block(pos) {
                let color = block.color();

                bytes[index] = color[0];
                bytes[index + 1] = color[1];
                bytes[index + 2] = color[2];
                bytes[index + 3] = 255;
            } else if let Some(wall) = world_data.get_wall(pos) {
                let color = wall.color();

                bytes[index] = color[0];
                bytes[index + 1] = color[1];
                bytes[index + 2] = color[2];
                bytes[index + 3] = 255;
            } else {
                if y >= world_data.layer.underground {
                    let color = WallType::Dirt.color();

                    bytes[index] = color[0];
                    bytes[index + 1] = color[1];
                    bytes[index + 2] = color[2];
                    bytes[index + 3] = 255;
                } else {
                    let sky_y = map_range_usize((0, world_data.playable_height()), (0, sky_image_height), y);

                    bytes[index] = sky_image.data[(sky_y * sky_image_width) * 4 + 0];
                    bytes[index + 1] = sky_image.data[(sky_y * sky_image_width) * 4 + 1];
                    bytes[index + 2] = sky_image.data[(sky_y * sky_image_width) * 4 + 2];
                    bytes[index + 3] = 255;
                }
            }
        }
    }

    let image = Image::new(
        Extent3d {
            width: world_data.playable_width() as u32,
            height: world_data.playable_height() as u32,
            ..default()
        },
        TextureDimension::D2,
        bytes,
        TextureFormat::Rgba8UnormSrgb
    );

    commands.insert_resource(WorldMapTexture(images.add(image)));
}

#[inline(always)]
fn set_pixel(image: &mut Image, index: usize, pixel: [u8; 3]) {
    image.data[index] = pixel[0];
    image.data[index + 1] = pixel[1];
    image.data[index + 2] = pixel[2];
    image.data[index + 3] = 255;
}

pub(super) fn update_world_map_texture(
    world_data: Res<WorldData>,
    world_map_texture: Res<WorldMapTexture>,
    mut images: ResMut<Assets<Image>>,
    mut place_tile_events: EventReader<PlaceTileEvent>,
    mut break_tile_events: EventReader<BreakTileEvent>,
    background_assets: Res<BackgroundAssets>,
    mut asset_events: EventWriter<AssetEvent<ColorMaterial>>,
    query_world_map: Query<&Handle<ColorMaterial>, With<WorldMapView>>
) {
    if break_tile_events.is_empty() && place_tile_events.is_empty() { return; }

    let mut image = images.remove(world_map_texture.id()).unwrap();

    for event in place_tile_events.iter() {
        let x = event.tile_pos.x as usize - world_data.playable_area.min.x as usize;
        let y = event.tile_pos.y as usize - world_data.playable_area.min.y as usize;
        let index = ((y * world_data.playable_width()) + x) * 4;

        let color = match event.tile_type {
            TileType::Block(Some(block_type)) => block_type.color(),
            TileType::Wall(Some(wall_type)) => wall_type.color(),
            _ => unreachable!()
        };

        set_pixel(&mut image, index, color);
    }

    for event in break_tile_events.iter() {
        let x = event.tile_pos.x as usize - world_data.playable_area.min.x as usize;
        let y = event.tile_pos.y as usize - world_data.playable_area.min.y as usize;
        let index = ((y * world_data.playable_width()) + x) * 4;

        let color = match event.tile_type {
            TileType::Block(_) => world_data.get_wall_color(event.tile_pos),
            TileType::Wall(_) => world_data.get_block_color(event.tile_pos),
        };

        let color = color
            .unwrap_or_else(|| {
                if y >= world_data.layer.underground {
                    WallType::Dirt.color()
                } else {
                    let sky_image = images.get(&background_assets.background_0).unwrap();
                    let sky_image_height = sky_image.texture_descriptor.size.height as usize;
                    let sky_image_width = sky_image.texture_descriptor.size.width as usize;

                    let sky_y = map_range_usize((0, world_data.playable_height()), (0, sky_image_height), y);

                    let index = (sky_y * sky_image_width) * 4;

                    [sky_image.data[index], sky_image.data[index + 1], sky_image.data[index + 2]]
                }
            });

        set_pixel(&mut image, index, color);
    }

    let _ = images.set(world_map_texture.id(), image);

    let material_handle = query_world_map.single();

    asset_events.send(AssetEvent::Modified { handle: material_handle.clone_weak() });
}

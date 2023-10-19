use bevy::{prelude::{Commands, Res, Assets, Mesh, ResMut, UiCameraConfig, Camera2dBundle, default, shape::Quad, Color, Visibility, Camera, Query, Without, With, Input, MouseButton, EventReader, Transform, Vec3, Image, Handle, AssetEvent, EventWriter, Vec2}, sprite::{ColorMaterial, MaterialMesh2dBundle, SpriteBundle}, core_pipeline::tonemapping::Tonemapping, input::mouse::{MouseWheel, MouseMotion}, math::Vec3Swizzles, render::render_resource::{TextureDimension, TextureFormat, Extent3d}};

use crate::{world::{WorldData, wall::WallType}, plugins::{DespawnOnGameExit, ui::resources::{Visible, Ui}, camera::components::MainCamera, assets::{BackgroundAssets, UiAssets}, world::{events::{PlaceTileEvent, TileRemovedEvent}, TileType}, cursor::components::Hoverable}, common::{math::map_range_usize, components::Bounds}, language::{LocalizedText, keys::UIStringKey}};

use super::{WorldMapTexture, WORLD_MAP_VIEW_RENDER_LAYER, WorldMapViewCamera, WorldMapView, MapViewStatus, SpawnPointIcon};

pub(super) fn setup(
    mut commands: Commands,
    world_map_texture: Res<WorldMapTexture>,
    world_data: Res<WorldData>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    ui_assets: Res<UiAssets>
) {
    let map_size = world_data.playable_area.size().as_vec2();

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
            mesh: meshes.add(Quad::new(map_size).into()).into(),
            material: materials.add(ColorMaterial {
                color: Color::WHITE,
                texture: Some(world_map_texture.clone_weak()),
            }),
            visibility: Visibility::Hidden,
            ..default()
        },
        WORLD_MAP_VIEW_RENDER_LAYER
    ));

    commands.spawn((
        SpawnPointIcon,
        SpriteBundle {
            texture: ui_assets.spawn_point.clone_weak(),
            transform: Transform::from_xyz(0., 0., 10.),
            ..default()
        },
        Bounds::new(22., 24.),
        Hoverable::SimpleText(LocalizedText::from(UIStringKey::SpawnPoint)),
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

pub(super) fn drag_map_view(
    mouse_input: Res<Input<MouseButton>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut query_map_view: Query<&mut Transform, With<WorldMapView>>,
) {
    let mut map_transform = query_map_view.single_mut();

    if mouse_input.pressed(MouseButton::Left) {
        for event in mouse_motion.iter() {
            map_transform.translation += Vec3::new(event.delta.x, -event.delta.y, 0.);
        }
    }
}

pub(super) fn update_map_view(
    world_data: Res<WorldData>,
    mut mouse_wheel: EventReader<MouseWheel>,
    mut query_map_view: Query<&mut Transform, With<WorldMapView>>,
    mut query_spawn_point_icon: Query<&mut Transform, (With<SpawnPointIcon>, Without<WorldMapView>)>,
) {
    let mut map_transform = query_map_view.single_mut();
    let mut spawn_icon_transform = query_spawn_point_icon.single_mut();
    
    let map_default_size = world_data.playable_area.size().as_vec2();

    for event in mouse_wheel.iter() {
        let scale = map_transform.scale.xy();
        let old_normalized = map_transform.translation.xy() / (map_default_size * scale);

        let new_scale = (scale + scale * Vec2::splat(event.y / 6.))
            .clamp(Vec2::splat(0.5), Vec2::splat(20.));

        map_transform.scale.x = new_scale.x;
        map_transform.scale.y = new_scale.y;

        let new_normalized = map_transform.translation.xy() / (map_default_size * new_scale);

        let delta = old_normalized - new_normalized;

        map_transform.translation.x += map_default_size.x * map_transform.scale.x * delta.x;
        map_transform.translation.y += map_default_size.y * map_transform.scale.y * delta.y;
    }

    let map_size = map_default_size * map_transform.scale.xy();

    let spawn_point = Vec2::from(world_data.spawn_point) - world_data.playable_area.min.as_vec2();

    spawn_icon_transform.translation.x = map_transform.translation.x - map_size.x / 2. + spawn_point.x as f32 * map_transform.scale.x;
    spawn_icon_transform.translation.y = map_transform.translation.y + map_size.y / 2. - spawn_point.y as f32 * map_transform.scale.y;
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

#[inline(always)]
fn set_pixel(data: &mut [u8], index: usize, pixel: [u8; 3]) {
    data[index] = pixel[0];
    data[index + 1] = pixel[1];
    data[index + 2] = pixel[2];
    data[index + 3] = 255;
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

            let color = world_data.get_block_color(pos).or(world_data.get_wall_color(pos))
                .unwrap_or_else(|| {
                    if y >= world_data.layer.underground {
                        WallType::Dirt.color()
                    } else {
                        let sky_y = map_range_usize((0, world_data.playable_height()), (0, sky_image_height), y);
                        let index = (sky_y * sky_image_width) * 4;

                        [sky_image.data[index], sky_image.data[index + 1], sky_image.data[index + 2]]
                    }
                });

            set_pixel(&mut bytes, index, color);
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

pub(super) fn update_world_map_texture(
    world_data: Res<WorldData>,
    world_map_texture: Res<WorldMapTexture>,
    mut images: ResMut<Assets<Image>>,
    mut place_tile_events: EventReader<PlaceTileEvent>,
    mut tile_removed_events: EventReader<TileRemovedEvent>,
    background_assets: Res<BackgroundAssets>,
    mut asset_events: EventWriter<AssetEvent<ColorMaterial>>,
    query_world_map: Query<&Handle<ColorMaterial>, With<WorldMapView>>
) {
    if tile_removed_events.is_empty() && place_tile_events.is_empty() { return; }

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

        set_pixel(&mut image.data, index, color);
    }

    for event in tile_removed_events.iter() {
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

        set_pixel(&mut image.data, index, color);
    }

    let _ = images.set(world_map_texture.id(), image);

    let material_handle = query_world_map.single();

    asset_events.send(AssetEvent::Modified { handle: material_handle.clone_weak() });
}
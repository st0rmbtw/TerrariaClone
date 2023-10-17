use bevy::{prelude::{Plugin, App, OnEnter, Commands, Component, Camera2dBundle, Query, Camera, Without, With, Resource, Deref, default, ResMut, Update, IntoSystemConfigs, KeyCode, Assets, Handle, Image, Res, Visibility, apply_deferred, UiCameraConfig, Transform, Input, EventReader, Vec3, resource_equals, MouseButton, Color}, render::{view::RenderLayers, render_resource::{AsBindGroup, ShaderRef, Extent3d, TextureDimension, TextureFormat}}, input::{common_conditions::input_just_pressed, mouse::{MouseWheel, MouseMotion}}, sprite::{Material2d, Material2dPlugin, SpriteBundle, Sprite}, reflect::{TypeUuid, TypePath}, core_pipeline::tonemapping::Tonemapping, math::Vec3Swizzles};

use crate::{common::{state::GameState, math::map_range_usize}, world::{WorldData, wall::WallType}};

use super::{DespawnOnGameExit, InGameSystemSet, camera::components::MainCamera, assets::BackgroundAssets, world::{events::{BreakTileEvent, PlaceTileEvent}, TileType}};

pub(crate) struct WorldMapViewPlugin;
impl Plugin for WorldMapViewPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(Material2dPlugin::<WorldMapViewMaterial>::default());

        app.init_resource::<MapViewStatus>();

        app.add_systems(
            OnEnter(GameState::InGame),
            (init_world_map_texture, apply_deferred, setup).chain()
        );

        app.add_systems(
            Update,
            update_world_map_texture.in_set(InGameSystemSet::Update)
        );

        app.add_systems(
            Update,
            (
                update_map_view,
                clamp_map_view_position
            )
            .chain()
            .in_set(InGameSystemSet::Update)
            .run_if(resource_equals(MapViewStatus::Opened))
        );

        app.add_systems(
            Update,
            toggle_world_map_view
                .in_set(InGameSystemSet::Update)
                .run_if(input_just_pressed(KeyCode::M))
        );
    }
}

const WORLD_MAP_VIEW_RENDER_LAYER: RenderLayers = RenderLayers::layer(15);

#[derive(Component)]
struct WorldMapViewCamera;

#[derive(Component)]
struct WorldMapView;

#[derive(Resource, Deref)]
struct WorldMapTexture(Handle<Image>);

#[derive(Resource, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) enum MapViewStatus {
    Opened,
    #[default]
    Closed
}

impl MapViewStatus {
    fn is_opened(&self) -> bool {
        *self == MapViewStatus::Opened
    }

    fn set_opened(&mut self, opened: bool) {
        match opened {
            true => *self = MapViewStatus::Opened,
            false => *self = MapViewStatus::Closed,
        }
    }
}

#[derive(AsBindGroup, TypeUuid, TypePath, Debug, Clone)]
#[uuid = "1b120582-0216-4a54-95d8-924071b88311"]
struct WorldMapViewMaterial {
    #[texture(0)]
    #[sampler(1)]
    tile_map: Handle<Image>
}

impl Material2d for WorldMapViewMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/map_view.wgsl".into()
    }
}

fn init_world_map_texture(
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

fn update_world_map_texture(
    world_data: Res<WorldData>,
    world_map_texture: Res<WorldMapTexture>,
    mut images: ResMut<Assets<Image>>,
    mut place_tile_events: EventReader<PlaceTileEvent>,
    mut break_tile_events: EventReader<BreakTileEvent>,
    background_assets: Res<BackgroundAssets>
) {
    let mut empty_tile_pos = vec![];
    let mut underground_tile_pos = vec![];

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

        image.data[index] = color[0];
        image.data[index + 1] = color[1];
        image.data[index + 2] = color[2];
        image.data[index + 3] = 255;
    }

    for event in break_tile_events.iter() {
        let x = event.tile_pos.x as usize - world_data.playable_area.min.x as usize;
        let y = event.tile_pos.y as usize - world_data.playable_area.min.y as usize;
        let index = ((y * world_data.playable_width()) + x) * 4;

        let color = if let Some(block) = world_data.get_block(event.tile_pos) {
            block.color()
        } else if let Some(wall) = world_data.get_wall(event.tile_pos) {
            wall.color()
        } else {
            if y >= world_data.layer.underground {
                underground_tile_pos.push(event.tile_pos);
            } else {
                empty_tile_pos.push(event.tile_pos);
            }
            continue;
        };

        image.data[index] = color[0];
        image.data[index + 1] = color[1];
        image.data[index + 2] = color[2];
        image.data[index + 3] = 255;
    }

    let sky_image = images.get(&background_assets.background_0).unwrap();
    let sky_image_height = sky_image.texture_descriptor.size.height as usize;
    let sky_image_width = sky_image.texture_descriptor.size.width as usize;

    for tile_pos in underground_tile_pos {
        let x = tile_pos.x as usize - world_data.playable_area.min.x as usize;
        let y = tile_pos.y as usize - world_data.playable_area.min.y as usize;
        let index = ((y * world_data.playable_width()) + x) * 4;

        let color = WallType::Dirt.color();

        image.data[index] = color[0];
        image.data[index + 1] = color[1];
        image.data[index + 2] = color[2];
        image.data[index + 3] = 255;
    }
    
    for tile_pos in empty_tile_pos {
        let x = tile_pos.x as usize - world_data.playable_area.min.x as usize;
        let y = tile_pos.y as usize - world_data.playable_area.min.y as usize;
        let index = ((y * world_data.playable_width()) + x) * 4;

        let sky_y = map_range_usize((0, world_data.playable_height()), (0, sky_image_height), y);

        image.data[index] = sky_image.data[(sky_y * sky_image_width) * 4 + 0];
        image.data[index + 1] = sky_image.data[(sky_y * sky_image_width) * 4 + 1];
        image.data[index + 2] = sky_image.data[(sky_y * sky_image_width) * 4 + 2];
        image.data[index + 3] = 255;
    }

    let _ = images.set(world_map_texture.id(), image);
}

fn setup(
    mut commands: Commands,
    world_map_texture: Res<WorldMapTexture>,
    world_data: Res<WorldData>
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
        SpriteBundle {
            sprite: Sprite {
                color: Color::WHITE,
                custom_size: Some(world_data.playable_area.size().as_vec2()),
                ..default()
            },
            texture: world_map_texture.clone_weak(),
            visibility: Visibility::Hidden,
            ..default()
        },
        WORLD_MAP_VIEW_RENDER_LAYER
    ));
}

fn toggle_world_map_view(
    mut map_view_status: ResMut<MapViewStatus>,
    mut query_camera: Query<&mut Camera, (Without<WorldMapViewCamera>, Without<MainCamera>)>,
    mut query_main_camera: Query<&mut UiCameraConfig, With<MainCamera>>,
    mut query_map_view_camera: Query<&mut Camera, With<WorldMapViewCamera>>,
    mut query_map_view: Query<&mut Visibility, With<WorldMapView>>,
) {
    let map_view_opened = !map_view_status.is_opened();
    map_view_status.set_opened(map_view_opened);

    for mut camera in &mut query_camera {
        camera.is_active = !map_view_opened;
    }

    if let Ok(mut ui_config) = query_main_camera.get_single_mut() {
        ui_config.show_ui = !map_view_opened;
    }
    let Ok(mut map_view_camera) = query_map_view_camera.get_single_mut() else { return; };
    let Ok(mut map_view_visibility) = query_map_view.get_single_mut() else { return; };

    map_view_camera.is_active = map_view_opened;
    *map_view_visibility = match map_view_opened {
        true => Visibility::Visible,
        false => Visibility::Hidden,
    }
}

fn update_map_view(
    mouse_input: Res<Input<MouseButton>>,
    mut mouse_wheel: EventReader<MouseWheel>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut query_map_view: Query<&mut Transform, With<WorldMapView>>,
) {
    let mut transform = query_map_view.single_mut();

    for event in mouse_wheel.iter() {
        transform.scale = (transform.scale + Vec3::splat(event.y / 5.))
            .clamp(Vec3::splat(0.5), Vec3::splat(20.));
    }

    if mouse_input.pressed(MouseButton::Left) {
        for event in mouse_motion.iter() {
            transform.translation += Vec3::new(event.delta.x, -event.delta.y, 0.);
        }
    }
}

fn clamp_map_view_position(
    mut query_map_view: Query<(&mut Transform, &Sprite), With<WorldMapView>>,
) {
    let (mut transform, sprite) = query_map_view.single_mut();

    let sprite_size = (sprite.custom_size.unwrap() * transform.scale.xy()) / 2.;

    let clamped_pos = transform.translation.xy().clamp(-sprite_size, sprite_size);

    transform.translation.x = clamped_pos.x;
    transform.translation.y = clamped_pos.y;
}
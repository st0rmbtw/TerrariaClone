use bevy::{prelude::{Image, Res, ResMut, Assets, GlobalTransform, OrthographicProjection, With, Query, Deref, UVec2, EventReader, Commands, Transform, Resource, ComputedVisibility, Color}, render::{render_resource::{Extent3d, TextureDimension, TextureUsages, UniformBuffer, StorageBuffer, FilterMode, SamplerDescriptor}, renderer::{RenderQueue, RenderDevice}, Extract, extract_resource::ExtractResource, texture::ImageSampler}, utils::default, math::{URect, Vec3Swizzles}};
use rand::{thread_rng, Rng};

use crate::{world::WorldData, plugins::{camera::components::WorldCamera, world::{constants::TILE_SIZE, WorldSize, events::{PlaceTileEvent, BreakTileEvent}, TileType, time::GameTime}, config::LightSmoothness}, lighting::{LightMapTexture, LIGHTMAP_FORMAT, gpu_types::{GpuLightSourceBuffer, GpuLightSource}, TILES_FORMAT, TileTexture, types::LightSource}};

#[derive(Resource, ExtractResource, Deref, Clone, Copy, Default)]
pub(crate) struct BlurArea(pub(crate) URect);

#[derive(Resource, Clone, Copy, Deref, Default)]
pub(crate) struct LightSourceCount(pub(super) u32);

#[derive(Resource, Default)]
pub(crate) struct LightMapPipelineAssets {
    pub(crate) area_min: UniformBuffer<UVec2>,
    pub(crate) area_max: UniformBuffer<UVec2>,
    pub(crate) ambient_color: UniformBuffer<Color>,
    pub(crate) light_sources: StorageBuffer<GpuLightSourceBuffer>,
}

impl LightMapPipelineAssets {
    pub(crate) fn write_buffer(&mut self, device: &RenderDevice, queue: &RenderQueue) {
        self.area_min.write_buffer(device, queue);
        self.area_max.write_buffer(device, queue);
        self.ambient_color.write_buffer(device, queue);
        self.light_sources.write_buffer(device, queue);
    }
}

pub(crate) fn init_tiles_texture(
    mut commands: Commands,
    world_data: Res<WorldData>,
    mut images: ResMut<Assets<Image>>,
) {
    let mut bytes = vec![1u8; world_data.width() * world_data.height()];

    for y in 0..world_data.height() {
        for x in 0..world_data.width() {
            let block_exists = world_data.solid_block_exists((x, y));
            let wall_exists = world_data.wall_exists((x, y));
            let index = (y * world_data.width()) + x;

            if block_exists {
                bytes[index] = 1;
            } else if wall_exists || y >= world_data.layer.underground {
                bytes[index] = 2;
            } else {
                bytes[index] = 0;
            }
        }
    }

    let mut image = Image::new_fill(
        Extent3d {
            width: world_data.area.width(),
            height: world_data.area.height(),
            ..default()
        },
        TextureDimension::D2,
        &bytes,
        TILES_FORMAT
    );

    image.texture_descriptor.usage = TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING | TextureUsages::STORAGE_BINDING;

    commands.insert_resource(TileTexture(images.add(image)));
}

pub(crate) fn init_light_map_texture(
    mut commands: Commands,
    world_data: Res<WorldData>,
    light_smoothness: Res<LightSmoothness>,
    mut images: ResMut<Assets<Image>>,
) {
    let size = world_data.area.size() * light_smoothness.subdivision();

    let mut texture = Image::new_fill(
        Extent3d {
            width: size.x,
            height: size.y,
            ..Default::default()
        },
        TextureDimension::D2,
        &[0, 0, 0, 255],
        LIGHTMAP_FORMAT,
    );

    texture.texture_descriptor.usage = TextureUsages::COPY_SRC | TextureUsages::TEXTURE_BINDING | TextureUsages::STORAGE_BINDING;

    texture.sampler_descriptor = ImageSampler::Descriptor(SamplerDescriptor {
        mag_filter: FilterMode::Nearest,
        min_filter: FilterMode::Nearest,
        ..default()
    });

    commands.insert_resource(LightMapTexture(images.add(texture)));
}

pub(crate) fn handle_update_tiles_texture_event(
    tile_texture: Res<TileTexture>,
    world_data: Res<WorldData>,
    mut images: ResMut<Assets<Image>>,
    mut place_tile_events: EventReader<PlaceTileEvent>,
    mut break_tile_events: EventReader<BreakTileEvent>
) {
    let image = images.get_mut(&tile_texture.0).unwrap();

    for event in break_tile_events.iter() {
        let x = event.tile_pos.x as usize;
        let y = event.tile_pos.y as usize;

        let index = y * world_data.width() + x;

        match event.tile_type {
            TileType::Block(_) => {
                let wall_exists = world_data.wall_exists(event.tile_pos);

                if wall_exists || y >= world_data.layer.underground {
                    image.data[index] = 2;
                } else {
                    image.data[index] = 0;
                }
            },
            TileType::Wall(_) => {
                let block_exists = world_data.block_exists(event.tile_pos);

                if block_exists {
                    image.data[index] = 1;
                } else {
                    image.data[index] = 0;
                }
            },
        }
    }

    for event in place_tile_events.iter() {
        let x = event.tile_pos.x as usize;
        let y = event.tile_pos.y as usize;

        let index = y * world_data.width() + x;

        match event.tile_type {
            TileType::Block(Some(block_type)) => {
                if !block_type.is_solid() { continue; }
                image.data[index] = 1;
            },
            TileType::Wall(_) => {
                let block_exists = world_data.block_exists(event.tile_pos);

                if !block_exists {
                    image.data[index] = 2;
                }
            },
            _ => unreachable!()
        }
    }
}

pub(crate) fn update_blur_area(
    mut blur_area: ResMut<BlurArea>,
    light_smoothness: Res<LightSmoothness>,
    query_camera: Query<(&GlobalTransform, &OrthographicProjection), With<WorldCamera>>,
) {
    let Ok((camera_transform, projection)) = query_camera.get_single() else { return };

    let camera_position = camera_transform.translation().xy().abs();

    let area = URect::from_corners(
        ((camera_position + projection.area.min) / TILE_SIZE - 16.).as_uvec2() * light_smoothness.subdivision(),
        ((camera_position + projection.area.max) / TILE_SIZE + 16.).as_uvec2() * light_smoothness.subdivision(),
    );

    blur_area.0 = area;
}

pub(crate) fn prepare_lightmap_pipeline_assets(
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut pipeline_assets: ResMut<LightMapPipelineAssets>,
) {
    pipeline_assets.write_buffer(&render_device, &render_queue);
}

pub(crate) fn extract_ambient_color(
    res_game_time: Extract<Option<Res<GameTime>>>,
    mut pipeline_assets: ResMut<LightMapPipelineAssets>,
) {
    if let Some(game_time) = res_game_time.as_ref() {
        pipeline_assets.ambient_color.set(game_time.ambient_color().as_rgba_linear());
    }
}

pub(crate) fn extract_lightmap_pipeline_assets(
    world_size: Extract<Option<Res<WorldSize>>>,
    blur_area: Res<BlurArea>,
    light_smoothness: Res<LightSmoothness>,
    
    mut light_source_count: ResMut<LightSourceCount>,
    mut pipeline_assets: ResMut<LightMapPipelineAssets>,

    query_light_source: Extract<Query<(&Transform, &LightSource, &ComputedVisibility)>>,
) {
    pipeline_assets.area_min.set(blur_area.min);
    pipeline_assets.area_max.set(blur_area.max);

    let Some(world_size) = world_size.as_ref() else { return; };

    let mut rng = thread_rng();
    let light_sources = pipeline_assets.light_sources.get_mut();
    let mut count = 0;

    light_sources.data.clear();

    let world_size = world_size.as_vec2();

    for (transform, light_source, visibility) in &query_light_source {
        if !visibility.is_visible() { continue; }

        let uv = transform.translation.xy().abs() / (world_size * TILE_SIZE);
        let light_pos = (uv * world_size * light_smoothness.subdivision() as f32).as_uvec2();

        let intensity = if light_source.jitter_intensity > 0. {
            light_source.intensity + rng.gen_range(-1.0..1.0) * light_source.jitter_intensity
        } else {
            light_source.intensity
        };

        light_sources.data.push(GpuLightSource {
            pos: light_pos,
            size: light_source.size,
            color: light_source.color * intensity,
        });
        
        count += 1;
    }

    light_source_count.0 = count;
}
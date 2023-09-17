use bevy::{prelude::{Image, Res, ResMut, Assets, GlobalTransform, OrthographicProjection, With, Query, Deref, UVec2, EventReader, DetectChanges, State, Commands, Transform, Resource}, render::{render_resource::{Extent3d, TextureDimension, TextureUsages, UniformBuffer, StorageBuffer}, renderer::{RenderQueue, RenderDevice}, Extract, extract_resource::ExtractResource}, utils::default, math::{URect, Vec3Swizzles}};
use rand::{thread_rng, Rng};

use crate::{world::WorldData, plugins::{camera::components::MainCamera, world::{constants::TILE_SIZE, resources::WorldUndergroundLevel, WorldSize}, config::LightSmoothness}, common::state::GameState};

use super::{pipeline::TILES_FORMAT, UpdateTilesTextureEvent, TileTexture, LightMapTexture, types::LightSource, gpu_types::{GpuLightSource, GpuLightSourceBuffer}};

#[derive(Resource, ExtractResource, Deref, Clone, Default)]
pub(super) struct BlurArea(pub(super) URect);

#[derive(Resource, Default)]
pub(super) struct PipelineAssets {
    pub(super) area_min: UniformBuffer<UVec2>,
    pub(super) area_max: UniformBuffer<UVec2>,
    pub(super) light_sources: StorageBuffer<GpuLightSourceBuffer>,
}

impl PipelineAssets {
    pub fn write_buffer(&mut self, device: &RenderDevice, queue: &RenderQueue) {
        self.area_min.write_buffer(device, queue);
        self.area_max.write_buffer(device, queue);
        self.light_sources.write_buffer(device, queue);
    }
}

#[derive(Resource, Clone, Copy, Deref)]
pub(super) struct LightSourceCount(pub(super) u32);

pub(super) fn init_tiles_texture(
    mut commands: Commands,
    world_data: Res<WorldData>,
    mut images: ResMut<Assets<Image>>,
) {
    let mut bytes = vec![1u8; world_data.size.width * world_data.size.height];

    for y in 0..world_data.size.height {
        for x in 0..world_data.size.width {
            let block_exists = world_data.solid_block_exists((x, y));
            let wall_exists = world_data.wall_exists((x, y));
            let index = (y * world_data.size.width) + x;

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
            width: world_data.size.width as u32,
            height: world_data.size.height as u32,
            ..default()
        },
        TextureDimension::D2,
        &bytes,
        TILES_FORMAT
    );

    image.texture_descriptor.usage = TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING | TextureUsages::STORAGE_BINDING;

    commands.insert_resource(TileTexture(images.add(image)));
}

pub(super) fn handle_update_tiles_texture_event(
    tile_texture: Res<TileTexture>,
    world_data: Res<WorldData>,
    mut images: ResMut<Assets<Image>>,
    mut events: EventReader<UpdateTilesTextureEvent>,
) {
    if events.is_empty() { return; }

    let image = images.get_mut(&tile_texture.0).unwrap();
    for event in events.iter() {
        let block_exists = world_data.solid_block_exists((event.x, event.y));
        let wall_exists = world_data.wall_exists((event.x, event.y));

        let index = event.y * world_data.size.width + event.x;

        if block_exists {
            image.data[index] = 1;
        } else if wall_exists || event.y >= world_data.layer.underground {
            image.data[index] = 2;
        } else {
            image.data[index] = 0;
        }
    }
}

pub(super) fn update_blur_area(
    mut blur_area: ResMut<BlurArea>,
    light_smoothness: Res<LightSmoothness>,
    query_camera: Query<(&GlobalTransform, &OrthographicProjection), With<MainCamera>>,
) {
    let Ok((camera_transform, projection)) = query_camera.get_single() else { return };

    let camera_position = camera_transform.translation().xy().abs();

    let area = URect::from_corners(
        ((camera_position + projection.area.min) / TILE_SIZE - 16.).as_uvec2() * light_smoothness.subdivision(),
        ((camera_position + projection.area.max) / TILE_SIZE + 16.).as_uvec2() * light_smoothness.subdivision(),
    );

    blur_area.0 = area;
}

pub(super) fn prepare_pipeline_assets(
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut pipeline_assets: ResMut<PipelineAssets>,
) {
    pipeline_assets.write_buffer(&render_device, &render_queue);
}

pub(super) fn extract_state(
    mut commands: Commands,
    state: Extract<Res<State<GameState>>>,
) {
    commands.insert_resource(State::new(*state.get()));
}

pub(super) fn extract_light_smoothness(
    mut commands: Commands,
    light_smoothness: Extract<Res<LightSmoothness>>,
) {
    if light_smoothness.is_changed() {
        commands.insert_resource(**light_smoothness);
    }
}

pub(super) fn extract_world_underground_level(
    mut commands: Commands,
    underground_level: Extract<Option<Res<WorldUndergroundLevel>>>,
) {
    let Some(underground_level) = underground_level.as_ref() else { return; };

    if underground_level.is_changed() {
        commands.insert_resource(**underground_level);
    }
}

pub(super) fn extract_pipeline_assets(
    mut commands: Commands,

    blur_area: Extract<Res<BlurArea>>,
    world_size: Extract<Option<Res<WorldSize>>>,
    light_smoothness: Res<LightSmoothness>,
    
    mut pipeline_assets: ResMut<PipelineAssets>,

    query_light_source: Extract<Query<(&Transform, &LightSource)>>,
) {
    pipeline_assets.area_min.set(blur_area.min);
    pipeline_assets.area_max.set(blur_area.max);

    let Some(world_size) = world_size.as_ref() else { return; };

    let mut rng = thread_rng();
    let light_sources = pipeline_assets.light_sources.get_mut();
    let mut count = 0;

    light_sources.data.clear();

    let world_size = world_size.as_vec2();

    for (transform, light_source) in &query_light_source {
        let uv = transform.translation.xy().abs() / (world_size * TILE_SIZE);
        let light_pos = (uv * world_size * light_smoothness.subdivision() as f32).as_uvec2();

        let intensity = light_source.intensity + rng.gen_range(-1f32..1f32) * light_source.jitter_intensity;

        light_sources.data.push(GpuLightSource {
            pos: light_pos,
            size: light_source.size,
            color: light_source.color * intensity,
        });
        
        count += 1;
    }

    commands.insert_resource(LightSourceCount(count));
}

pub(super) fn extract_textures(
    mut commands: Commands,
    tile_texture: Extract<Option<Res<TileTexture>>>,
    lightmap_texture: Extract<Option<Res<LightMapTexture>>>,
) {
    let Some(tile_texture) = tile_texture.as_ref() else { return; };
    let Some(lightmap_texture) = lightmap_texture.as_ref() else { return; };

    if tile_texture.is_changed() {
        commands.insert_resource((**tile_texture).clone());
    }   

    if lightmap_texture.is_changed() {
        commands.insert_resource((**lightmap_texture).clone());
    }
}
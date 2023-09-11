use bevy::{prelude::{Image, Res, ResMut, Assets, GlobalTransform, OrthographicProjection, With, Query, Resource, Deref, UVec2, EventReader}, render::{render_resource::{Extent3d, TextureDimension, TextureUsages, UniformBuffer}, extract_resource::ExtractResource, renderer::{RenderQueue, RenderDevice}, Extract}, utils::default, math::{URect, Vec3Swizzles}};

use crate::{world::WorldData, plugins::{camera::components::MainCamera, world::constants::TILE_SIZE}};

use super::{pipeline::{PipelineTargetsWrapper, TILES_FORMAT}, SUBDIVISION, UpdateTilesTextureEvent, WorldUndergroundLevel};

#[derive(Resource, ExtractResource, Deref, Clone, Default)]
pub(super) struct BlurArea(pub(super) URect);

#[derive(Resource, Default)]
pub(super) struct PipelineAssets {
    pub(super) area_min: UniformBuffer<UVec2>,
    pub(super) area_max: UniformBuffer<UVec2>,
    pub(super) world_underground_level: UniformBuffer<u32>,
}

impl PipelineAssets {
    pub fn write_buffer(&mut self, device: &RenderDevice, queue: &RenderQueue) {
        self.area_min.write_buffer(device, queue);
        self.area_max.write_buffer(device, queue);
        self.world_underground_level.write_buffer(device, queue);
    }
}

pub(super) fn init_tiles_texture(
    res_world_data: Res<WorldData>,
    mut pipeline_targets: ResMut<PipelineTargetsWrapper>,
    mut images: ResMut<Assets<Image>>,
) {
    let mut bytes = vec![1u8; res_world_data.size.width * res_world_data.size.height];

    for y in 0..res_world_data.size.height {
        for x in 0..res_world_data.size.width {
            let block_exists = res_world_data.solid_block_exists((x, y));
            let wall_exists = res_world_data.wall_exists((x, y));
            let index = (y * res_world_data.size.width) + x;

            if block_exists {
                bytes[index] = 1;
            } else if wall_exists {
                bytes[index] = 2;
            } else {
                bytes[index] = 0;
            }
        }
    }

    let mut image = Image::new_fill(
        Extent3d {
            width: res_world_data.size.width as u32,
            height: res_world_data.size.height as u32,
            ..default()
        },
        TextureDimension::D2,
        &bytes,
        TILES_FORMAT
    );

    image.texture_descriptor.usage = TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING | TextureUsages::STORAGE_BINDING;

    pipeline_targets.tiles = Some(images.add(image));
}

pub(super) fn handle_update_tiles_texture_event(
    pipeline_targets: Res<PipelineTargetsWrapper>,
    world_data: Res<WorldData>,
    mut images: ResMut<Assets<Image>>,
    mut events: EventReader<UpdateTilesTextureEvent>,
) {
    if events.is_empty() { return; }

    if let Some(tiles_texture_handle) = &pipeline_targets.tiles {
        let image = images.get_mut(tiles_texture_handle).unwrap();
        for event in events.iter() {
            let block_exists = world_data.solid_block_exists((event.x, event.y));
            let wall_exists = world_data.wall_exists((event.x, event.y));

            let index = event.y * world_data.size.width + event.x;

            if block_exists {
                image.data[index] = 1;
            } else if wall_exists {
                image.data[index] = 2;
            } else {
                image.data[index] = 0;
            }
        }
    }
}

pub(super) fn update_blur_area(
    mut blur_area: ResMut<BlurArea>,
    query_camera: Query<(&GlobalTransform, &OrthographicProjection), With<MainCamera>>,
) {
    let Ok((camera_transform, projection)) = query_camera.get_single() else { return };

    let camera_position = camera_transform.translation().xy().abs();

    let area = URect::from_corners(
        ((camera_position + projection.area.min) / TILE_SIZE - 8.).as_uvec2() * SUBDIVISION,
        ((camera_position + projection.area.max) / TILE_SIZE + 8.).as_uvec2() * SUBDIVISION,
    );

    blur_area.0 = area;
}

pub(super) fn prepare_pipeline_assets(
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut gi_compute_assets: ResMut<PipelineAssets>,
) {
    gi_compute_assets.write_buffer(&render_device, &render_queue);
}

pub(super) fn extract_pipeline_assets(
    world_underground_level: Extract<Res<WorldUndergroundLevel>>,
    blur_area: Extract<Res<BlurArea>>,
    mut pipeline_assets: ResMut<PipelineAssets>,
) {
    *pipeline_assets.area_min.get_mut() = blur_area.min;
    *pipeline_assets.area_max.get_mut() = blur_area.max;
    *pipeline_assets.world_underground_level.get_mut() = world_underground_level.0;
}
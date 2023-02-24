use bevy::prelude::*;
use bevy::render::render_resource::{StorageBuffer, UniformBuffer};
use bevy::render::renderer::{RenderDevice, RenderQueue};
use bevy::render::Extract;
use rand::{thread_rng, Rng};

use crate::plugins::camera::MainCamera;
use super::constants::GI_SCREEN_PROBE_SIZE;
use super::resource::{ComputedTargetSizes, LightPassParams};
use super::types::OmniLightSource2D;
use super::types_gpu::{GpuCameraParams, GpuLightSourceBuffer, GpuOmniLightSource, GpuLightPassParams};

#[rustfmt::skip]
#[derive(Default, Resource)]
pub struct LightPassPipelineAssets {
    pub camera_params:     UniformBuffer<GpuCameraParams>,
    pub light_pass_params: UniformBuffer<GpuLightPassParams>,
    pub light_sources:     StorageBuffer<GpuLightSourceBuffer>,
}

impl LightPassPipelineAssets {
    pub fn write_buffer(&mut self, device: &RenderDevice, queue: &RenderQueue) {
        self.light_sources.write_buffer(device, queue);
        self.camera_params.write_buffer(device, queue);
        self.light_pass_params.write_buffer(device, queue);
    }
}

#[rustfmt::skip]
pub(crate) fn system_prepare_pipeline_assets(
    render_device:         Res<RenderDevice>,
    render_queue:          Res<RenderQueue>,
    mut gi_compute_assets: ResMut<LightPassPipelineAssets>,
) {
    gi_compute_assets.write_buffer(&render_device, &render_queue);
}

#[rustfmt::skip]
pub(crate) fn system_extract_pipeline_assets(
    res_light_pass_params:      Extract<Res<LightPassParams>>,
    res_target_sizes:           Extract<Res<ComputedTargetSizes>>,

    query_lights:               Extract<Query<(&Transform, &OmniLightSource2D, &ComputedVisibility)>>,
    query_camera:               Extract<Query<(&Camera, &GlobalTransform, &OrthographicProjection), With<MainCamera>>>,

    mut gpu_target_sizes:       ResMut<ComputedTargetSizes>,
    mut gpu_pipeline_assets:    ResMut<LightPassPipelineAssets>,
    mut gpu_frame_counter:      Local<i32>,
) {
    *gpu_target_sizes = **res_target_sizes;

    {
        let mut light_sources = gpu_pipeline_assets.light_sources.get_mut();
        let mut rng = thread_rng();
        light_sources.count = 0;
        light_sources.data.clear();
        for (transform, light_source, visibility) in query_lights.iter() {
            if visibility.is_visible() {
                light_sources.count += 1;
                light_sources.data.push(GpuOmniLightSource::new(
                    OmniLightSource2D {
                        intensity: light_source.intensity
                            + rng.gen_range(-1.0..1.0) * light_source.jitter_intensity,
                        ..*light_source
                    },
                    Vec2::new(
                        transform.translation.x
                            + rng.gen_range(-1.0..1.0) * light_source.jitter_translation,
                        transform.translation.y
                            + rng.gen_range(-1.0..1.0) * light_source.jitter_translation,
                    ),
                ));
            }
        }
    }

    {
        if let Ok((camera, camera_global_transform, proj)) = query_camera.get_single() {
            let mut camera_params = gpu_pipeline_assets.camera_params.get_mut();
            let projection = camera.projection_matrix();
            let inverse_projection = projection.inverse();
            let view = camera_global_transform.compute_matrix();
            let inverse_view = view.inverse();

            camera_params.view_proj = projection * inverse_view;
            camera_params.inverse_view_proj = view * inverse_projection;
            camera_params.screen_size = Vec2::new(
                gpu_target_sizes.primary_target_size.x,
                gpu_target_sizes.primary_target_size.y,
            );
            camera_params.screen_size_inv = Vec2::new(
                1.0 / gpu_target_sizes.primary_target_size.x,
                1.0 / gpu_target_sizes.primary_target_size.y,
            );

            let scale = proj.scale;
            camera_params.sdf_scale     = Vec2::splat(scale);
            camera_params.inv_sdf_scale = Vec2::splat(1. / scale);
        } else {
            warn!("Failed to get camera");
        }
    }

    {
        let cols = gpu_target_sizes.primary_target_isize().x as i32 / GI_SCREEN_PROBE_SIZE;
        let rows = gpu_target_sizes.primary_target_isize().y as i32 / GI_SCREEN_PROBE_SIZE;

        let mut gpu_light_pass_params = gpu_pipeline_assets.light_pass_params.get_mut();
        gpu_light_pass_params.frame_counter = *gpu_frame_counter;
        gpu_light_pass_params.probe_size = GI_SCREEN_PROBE_SIZE;
        gpu_light_pass_params.probe_atlas_cols            = cols;
        gpu_light_pass_params.probe_atlas_rows            = rows;
        gpu_light_pass_params.reservoir_size              = res_light_pass_params.reservoir_size;
        gpu_light_pass_params.smooth_kernel_size_h        = res_light_pass_params.smooth_kernel_size.0;
        gpu_light_pass_params.smooth_kernel_size_w        = res_light_pass_params.smooth_kernel_size.1;
        gpu_light_pass_params.direct_light_contrib        = res_light_pass_params.direct_light_contrib;
        gpu_light_pass_params.indirect_light_contrib      = res_light_pass_params.indirect_light_contrib;
        gpu_light_pass_params.indirect_rays_radius_factor = res_light_pass_params.indirect_rays_radius_factor;
        gpu_light_pass_params.indirect_rays_per_sample    = res_light_pass_params.indirect_rays_per_sample;
    }

    *gpu_frame_counter = (*gpu_frame_counter + 1) % (GI_SCREEN_PROBE_SIZE * GI_SCREEN_PROBE_SIZE);
}

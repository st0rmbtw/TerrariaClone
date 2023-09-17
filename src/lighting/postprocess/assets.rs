use bevy::{prelude::{Resource, ResMut, Res, Query, Camera, Transform, With, Vec2}, render::{renderer::{RenderDevice, RenderQueue}, render_resource::UniformBuffer, Extract}};

use crate::{lighting::gpu_types::GpuCameraParams, plugins::{camera::components::WorldCamera, config::Resolution}};

#[derive(Resource, Default)]
pub(crate) struct PostProcessPipelineAssets {
    pub(crate) camera_params: UniformBuffer<GpuCameraParams>,
}

impl PostProcessPipelineAssets {
    pub(crate) fn write_buffer(&mut self, device: &RenderDevice, queue: &RenderQueue) {
        self.camera_params.write_buffer(device, queue);
    }
}

pub(crate) fn prepare_postprocess_pipeline_assets(
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    mut pipeline_assets: ResMut<PostProcessPipelineAssets>,
) {
    pipeline_assets.write_buffer(&render_device, &render_queue);
}

pub(crate) fn extract_postprocess_pipeline_assets(
    query_camera: Extract<Query<(&Camera, &Transform), With<WorldCamera>>>,
    resolution: Extract<Res<Resolution>>,
    mut pipeline_assets: ResMut<PostProcessPipelineAssets>,
) {
    if let Ok((camera, transform)) = query_camera.get_single() {
        let camera_params = pipeline_assets.camera_params.get_mut();
        let inverse_projection = camera.projection_matrix().inverse();
        let view = transform.compute_matrix();

        camera_params.inverse_view_proj = view * inverse_projection;
        camera_params.screen_size = Vec2::new(resolution.width, resolution.height);
        camera_params.screen_size_inv = 1. / camera_params.screen_size;
    }
}
use bevy::{prelude::{Vec2, Vec3, Mat4}, render::render_resource::ShaderType};

use super::types::LightSource;

#[derive(Default, Clone, ShaderType)]
pub(super) struct GpuLightSource {
    pub(super) center: Vec2,
    pub(super) intensity: f32,
    pub(super) color: Vec3,
    pub(super) radius: f32,
}

impl GpuLightSource {
    pub(super) fn new(light: LightSource, center: Vec2) -> Self {
        let color = light.color.as_rgba_f32();
        Self {
            center,
            intensity: light.intensity,
            radius: light.radius,
            color: Vec3::new(color[0], color[1], color[2]),
        }
    }
}

#[derive(Default, Clone, ShaderType)]
pub(super) struct GpuLightSourceBuffer {
    pub(super) count: u32,
    #[size(runtime)]
    pub(super) data:  Vec<GpuLightSource>,
}

#[derive(Default, Clone, ShaderType)]
pub(super) struct GpuCameraParams {
    pub(super) screen_size:       Vec2,
    pub(super) screen_size_inv:   Vec2,
    pub(super) view_proj:         Mat4,
    pub(super) inverse_view_proj: Mat4,
}

#[derive(Clone, ShaderType, Debug)]
pub(super) struct GpuLightPassParams {
    pub(super) frame_counter: i32,
    pub(super) probe_size: i32,
    pub(super) reservoir_size: u32,
}

impl Default for GpuLightPassParams {
    fn default() -> Self {
        Self {
            frame_counter: 0,
            probe_size: 0,
            reservoir_size: 16,
        }
    }
}
use bevy::{prelude::{Vec2, Vec3, Mat4}, render::render_resource::ShaderType};

use super::types::LightSource;

#[derive(Default, Clone, ShaderType)]
pub struct GpuLightSource {
    pub center: Vec2,
    pub intensity: f32,
    pub color: Vec3,
    pub radius: f32,
}

impl GpuLightSource {
    pub fn new(light: LightSource, center: Vec2) -> Self {
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
pub struct GpuLightSourceBuffer {
    pub count: u32,
    #[size(runtime)]
    pub data:  Vec<GpuLightSource>,
}

#[derive(Default, Clone, ShaderType)]
pub struct GpuCameraParams {
    pub screen_size:       Vec2,
    pub screen_size_inv:   Vec2,
    pub view_proj:         Mat4,
    pub inverse_view_proj: Mat4,
    pub sdf_scale:         Vec2,
    pub inv_sdf_scale:     Vec2,
}

#[derive(Clone, ShaderType, Debug)]
pub struct GpuLightPassParams {
    pub frame_counter: i32,
    pub probe_size: i32,
    pub reservoir_size: u32,
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
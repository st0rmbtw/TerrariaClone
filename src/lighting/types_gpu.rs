use bevy::{prelude::{Vec2, Vec3, Mat4}, render::render_resource::ShaderType};

use super::types::OmniLightSource2D;

#[derive(Default, Clone, ShaderType)]
pub struct GpuOmniLightSource {
    pub center:    Vec2,
    pub intensity: f32,
    pub color:     Vec3,
    pub falloff:   Vec3,
}

impl GpuOmniLightSource {
    pub fn new(light: OmniLightSource2D, center: Vec2) -> Self {
        let color = light.color.as_rgba_f32();
        Self {
            center,
            intensity: light.intensity,
            color: Vec3::new(color[0], color[1], color[2]),
            falloff: light.falloff,
        }
    }
}

#[derive(Default, Clone, ShaderType)]
pub struct GpuLightSourceBuffer {
    pub count: u32,
    #[size(runtime)]
    pub data:  Vec<GpuOmniLightSource>,
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
    pub frame_counter:          i32,
    pub probe_size:             i32,
    pub probe_atlas_cols:       i32,
    pub probe_atlas_rows:       i32,

    pub reservoir_size:              u32,
    pub smooth_kernel_size_h:        u32,
    pub smooth_kernel_size_w:        u32,
    pub direct_light_contrib:        f32,
    pub indirect_light_contrib:      f32,
    pub indirect_rays_per_sample:    i32,
    pub indirect_rays_radius_factor: f32,
}

impl Default for GpuLightPassParams {
    fn default() -> Self {
        Self {
            frame_counter: 0,
            probe_size: 0,
            probe_atlas_cols: 0,
            probe_atlas_rows: 0,

            reservoir_size: 16,
            smooth_kernel_size_h: 2,
            smooth_kernel_size_w: 1,
            direct_light_contrib: 0.2,
            indirect_light_contrib: 0.8,

            indirect_rays_per_sample: 64,
            indirect_rays_radius_factor: 3.0,
        }
    }
}
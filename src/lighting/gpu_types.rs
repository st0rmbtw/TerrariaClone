use bevy::{prelude::{UVec2, Vec2, Mat4, Vec3}, render::render_resource::ShaderType};

#[derive(Default, Clone, ShaderType)]
pub(super) struct GpuLightSource {
    pub(super) pos: UVec2,
    pub(super) size: UVec2,
    pub(super) color: Vec3,
}

#[derive(Default, Clone, ShaderType)]
pub(super) struct GpuLightSourceBuffer {
    #[size(runtime)]
    pub data: Vec<GpuLightSource>,
}

#[derive(Default, Clone, ShaderType)]
pub(super) struct GpuCameraParams {
    pub(super) screen_size: Vec2,
    pub(super) screen_size_inv: Vec2,
    pub(super) inverse_view_proj: Mat4,
}
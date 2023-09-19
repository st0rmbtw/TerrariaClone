use bevy::{prelude::{UVec2, Vec2, Mat4, Vec3, Component}, render::{render_resource::ShaderType, extract_component::ExtractComponent}};

#[derive(Default, Clone, ShaderType)]
pub(crate) struct GpuLightSource {
    pub(crate) pos: UVec2,
    pub(crate) size: UVec2,
    pub(crate) color: Vec3,
}

#[derive(Default, Clone, ShaderType)]
pub(crate) struct GpuLightSourceBuffer {
    #[size(runtime)]
    pub(crate) data: Vec<GpuLightSource>,
}

#[derive(Default, Clone, ShaderType, Component, ExtractComponent)]
pub(crate) struct GpuCameraParams {
    pub(crate) screen_size: Vec2,
    pub(crate) screen_size_inv: Vec2,
    pub(crate) inverse_view_proj: Mat4,
}
use bevy::{prelude::UVec2, render::render_resource::ShaderType};

#[derive(Default, Clone, ShaderType)]
pub(super) struct GpuLightSource {
    pub(super) pos: UVec2,
    pub(super) size: UVec2,
}

#[derive(Default, Clone, ShaderType)]
pub(super) struct GpuLightSourceBuffer {
    pub count: u32,
    #[size(runtime)]
    pub data: Vec<GpuLightSource>,
}
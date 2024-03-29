use bevy::prelude::{Component, UVec2, Vec3};

#[derive(Component, Default, Clone)]
pub(crate) struct LightSource {
    pub(crate) size: UVec2,
    pub(crate) color: Vec3,
    pub(crate) intensity: f32,
    pub(crate) jitter_intensity: f32,
}
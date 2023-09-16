use bevy::prelude::{Component, UVec2, Vec3};

#[derive(Component, Default, Clone)]
pub(crate) struct LightSource {
    pub(crate) size: UVec2,
    pub(crate) color: Vec3
}
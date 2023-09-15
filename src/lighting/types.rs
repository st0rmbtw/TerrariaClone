use bevy::prelude::{Component, UVec2};

#[derive(Component, Default, Clone)]
pub(crate) struct LightSource {
    pub(crate) size: UVec2
}
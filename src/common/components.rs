use bevy::prelude::{Component, Vec2};

#[derive(Clone, Copy, Component)]
pub(crate) struct Bounds {
    pub(crate) width: f32,
    pub(crate) height: f32
}

impl Bounds {
    pub(crate) const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    pub(crate) const fn as_vec2(self) -> Vec2 {
        Vec2::new(self.width, self.height)
    }
}
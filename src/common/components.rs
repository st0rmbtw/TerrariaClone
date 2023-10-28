use bevy::prelude::{Component, Vec2};

#[derive(Clone, Copy, Component)]
pub(crate) struct Bounds {
    pub(crate) width: f32,
    pub(crate) height: f32
}

impl Bounds {
    #[inline]
    pub(crate) const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    #[inline]
    pub(crate) const fn as_vec2(self) -> Vec2 {
        Vec2::new(self.width, self.height)
    }
}

impl From<Vec2> for Bounds {
    fn from(value: Vec2) -> Self {
        Self::new(value.x, value.y)
    }
}
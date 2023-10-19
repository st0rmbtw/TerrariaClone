use bevy::prelude::Component;

#[derive(Clone, Copy, Component)]
pub(crate) struct Bounds {
    pub(crate) width: f32,
    pub(crate) height: f32
}

impl Bounds {
    pub(crate) const fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}
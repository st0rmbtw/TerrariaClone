use bevy::prelude::Resource;

#[derive(Resource, Clone, Copy)]
pub(crate) struct Zoom(f32);

impl Zoom {
    pub(crate) fn new(value: f32) -> Self {
        debug_assert!((0.0..=1.0).contains(&value));
        Self(value)
    }

    pub(crate) fn set(&mut self, value: f32) {
        debug_assert!((0.0..=1.0).contains(&value));
        self.0 = value;
    }

    pub(crate) fn get(&self) -> f32 {
        self.0
    }
}
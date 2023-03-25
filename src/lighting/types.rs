use bevy::{prelude::{Color, Component}, reflect::Reflect};

#[derive(Reflect, Component, Clone, Copy, Default)]
pub(crate) struct LightSource {
    pub(crate) intensity: f32,
    pub(crate) color: Color,
    pub(crate) radius: f32,
    pub(crate) jitter_intensity: f32,
    pub(crate) jitter_translation: f32,
}
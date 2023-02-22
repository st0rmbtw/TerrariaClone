use bevy::{prelude::{Vec3, Color, Component}, reflect::Reflect};

#[derive(Reflect, Component, Clone, Copy, Default)]
pub struct OmniLightSource2D {
    pub intensity:          f32,
    pub color:              Color,
    pub falloff:            Vec3,
    pub jitter_intensity:   f32,
    pub jitter_translation: f32,
}
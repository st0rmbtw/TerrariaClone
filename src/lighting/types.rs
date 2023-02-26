use bevy::{prelude::{Color, Component}, reflect::Reflect};

#[derive(Reflect, Component, Clone, Copy, Default)]
pub struct LightSource {
    pub intensity: f32,
    pub color: Color,
    pub radius: f32,
    pub jitter_intensity: f32,
    pub jitter_translation: f32,
}
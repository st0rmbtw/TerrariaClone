use bevy::{prelude::{Component, Bundle, Transform, GlobalTransform, Visibility, ComputedVisibility, Handle}, sprite::{TextureAtlasSprite, TextureAtlas}, render::view::RenderLayers};

use crate::common::components::Velocity;

#[derive(Component)]
pub(crate) struct ParticleData {
    pub(super) spawn_time: f64,
    pub(super) lifetime: f64,
    pub(super) gravity: bool,
    pub(super) size: Option<f32>,
    pub(super) rotation_speed: f32
}

#[derive(Bundle)]
pub(crate) struct ParticleBundle {
    pub(crate) particle_data: ParticleData,
    pub(crate) sprite: TextureAtlasSprite,
    pub(crate) texture_atlas: Handle<TextureAtlas>,
    pub(crate) transform: Transform,
    pub(crate) global_transform: GlobalTransform,
    pub(crate) visibility: Visibility,
    pub(crate) computed_visibility: ComputedVisibility,
    pub(crate) velocity: Velocity,
    pub(crate) render_layer: RenderLayers,
}


use bevy::prelude::Resource;

#[derive(Resource, Clone, Copy, Default)]
pub struct FpsTextVisibility(pub bool);
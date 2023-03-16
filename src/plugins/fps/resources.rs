use bevy::prelude::Resource;

#[derive(Default, Resource, Clone, Copy, PartialEq, Eq)]
pub struct FpsTextVisibility(pub bool);
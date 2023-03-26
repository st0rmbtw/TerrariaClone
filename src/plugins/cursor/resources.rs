use bevy::prelude::{Vec2, Resource};

#[cfg(feature = "debug")]
use bevy::ecs::reflect::ReflectResource;

#[derive(Default, Resource)]
#[cfg_attr(feature = "debug", derive(bevy::reflect::Reflect))]
#[cfg_attr(feature = "debug", reflect(Resource))]
pub(crate) struct CursorPosition {
    pub position: Vec2,
    pub world_position: Vec2,
}
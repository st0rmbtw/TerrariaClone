use bevy::prelude::{Vec2, Deref, DerefMut, Resource};

#[cfg(feature = "debug")]
use bevy::ecs::reflect::ReflectResource;

#[derive(Default, Resource)]
#[cfg_attr(feature = "debug", derive(bevy::reflect::Reflect))]
#[cfg_attr(feature = "debug", reflect(Resource))]
pub struct CursorPosition {
    pub position: Vec2,
    pub world_position: Vec2,
}

#[derive(Default, Deref, DerefMut, Resource)]
pub struct HoveredInfo(pub String);
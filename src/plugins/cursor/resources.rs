use bevy::prelude::{Vec2, Deref, DerefMut, Resource};

#[derive(Default, Resource)]
pub struct CursorPosition {
    pub position: Vec2,
    pub world_position: Vec2,
}

#[derive(Default, Deref, DerefMut, Resource)]
pub struct HoveredInfo(pub String);
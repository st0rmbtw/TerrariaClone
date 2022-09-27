use bevy::prelude::{Vec2, Deref, DerefMut};

#[derive(Default)]
pub struct CursorPosition {
    pub position: Vec2,
    pub world_position: Vec2,
}

#[derive(Default, Deref, DerefMut)]
pub struct HoveredInfo(pub String);
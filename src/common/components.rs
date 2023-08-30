use bevy::prelude::{Component, DerefMut, Deref, Vec2};

#[derive(Component, Clone, Copy, Default, Deref, DerefMut)]
pub(crate) struct Velocity(pub Vec2);
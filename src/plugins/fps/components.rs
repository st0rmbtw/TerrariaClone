use bevy::{prelude::{Component, Deref, DerefMut}, time::Timer};

#[derive(Component)]
pub struct FpsText;

#[derive(Component, Deref, DerefMut)]
pub struct FpsTextTimer(pub Timer);
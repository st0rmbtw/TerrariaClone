use bevy::{prelude::{Deref, DerefMut, Resource, Vec2}, time::Timer};

#[derive(Resource, Default, Clone, Copy)]
pub(super) struct InputAxis {
    pub x: f32,
}

#[derive(Resource, Deref, DerefMut)]
pub(super) struct MovementAnimationTimer(pub Timer);

#[derive(Resource, Default, Clone, Copy)]
pub(super) struct MovementAnimationIndex(pub usize);

#[derive(Resource, Clone, Copy, Default, Deref, DerefMut)]
pub(crate) struct PlayerVelocity(pub Vec2);

#[derive(Resource, Default)]
pub(super) struct PlayerData {
    pub(super) jump: i32,
    pub(super) fall_start: f32,
}

#[derive(Debug, Resource, Clone, Copy, Default)]
pub(super) struct Collisions {
    pub(super) top: bool,
    pub(super) bottom: bool,
    pub(super) left: bool,
    pub(super) right: bool
}
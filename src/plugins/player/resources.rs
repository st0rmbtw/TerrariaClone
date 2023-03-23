use bevy::{prelude::{Deref, DerefMut, Resource, Vec2}, time::Timer};

#[derive(Resource, Default, Clone, Copy)]
pub(super) struct InputAxis {
    pub x: f32,
}

#[derive(Resource, Deref, DerefMut)]
pub(super) struct AnimationTimer(pub Timer);

#[derive(Resource, Default, Clone, Copy)]
pub(super) struct MovementAnimationIndex(pub usize);

#[derive(Resource, Clone, Copy, Default, Deref, DerefMut)]
pub struct PlayerVelocity(pub Vec2);

#[derive(Resource, Default)]
pub struct PlayerData {
    pub jump: i32,
    // The distance of player's fall in tiles
    pub fall_distance: f32,

    pub prev_position: Vec2
}

#[derive(Debug, Resource, Clone, Copy, Default)]
pub(crate) struct Collisions {
    pub top: bool,
    pub bottom: bool,
    pub left: bool,
    pub right: bool
}
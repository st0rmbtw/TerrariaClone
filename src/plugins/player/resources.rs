use bevy::{prelude::{Deref, DerefMut, Resource}, time::Timer};

use crate::world::block::BlockType;

#[derive(Resource, Default, Clone, Copy)]
pub(super) struct InputAxis {
    pub x: f32,
}

#[derive(Resource, Deref, DerefMut)]
pub(super) struct MovementAnimationTimer(pub Timer);

#[derive(Resource, Default, Clone, Copy)]
pub(super) struct MovementAnimationIndex(pub usize);

#[derive(Resource, Default)]
pub(super) struct PlayerData {
    pub(super) jumping: bool,
    pub(super) fall_start: Option<f32>,
    pub(super) ground: Option<BlockType>,
}

#[derive(Debug, Resource, Clone, Copy, Default)]
pub(super) struct Collisions {
    pub(super) top: bool,
    pub(super) bottom: bool,
    pub(super) left: bool,
    pub(super) right: bool
}
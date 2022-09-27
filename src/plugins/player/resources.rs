use bevy::{prelude::{Deref, DerefMut}, time::Timer};

use crate::{util::FRect, Velocity};

#[derive(Default, Clone, Copy)]
pub struct InputAxis {
    pub x: f32,
}

impl InputAxis {
    pub fn is_moving(&self) -> bool {
        self.x != 0.
    }
}

#[derive(Deref, DerefMut)]
pub struct AnimationTimer(pub Timer);

#[derive(Deref, DerefMut)]
pub struct UseItemAnimationTimer(pub Timer);

#[derive(Default, Clone, Copy)]
pub struct MovementAnimationIndex(pub usize);

#[derive(Default, Clone, Copy)]
pub struct UseItemAnimationIndex(pub usize);

#[derive(Clone, Copy, Default, Deref, DerefMut)]
pub struct PlayerVelocity(pub Velocity);

#[derive(Clone, Copy, Default, Deref, DerefMut)]
pub struct PlayerRect(pub FRect);

#[derive(Clone, Copy, Default)]
pub struct PlayerController {
    pub fall_speed: f32,
    pub apex_point: f32,
    pub jump: i32
}

#[derive(Clone, Copy, Default)]
pub struct Collisions {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool
}
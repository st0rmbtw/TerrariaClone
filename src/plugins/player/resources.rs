use bevy::{prelude::{Deref, DerefMut, Resource}, time::Timer};

use crate::Velocity;

#[derive(Resource, Default, Clone, Copy)]
pub struct InputAxis {
    pub x: f32,
}

impl InputAxis {
    pub fn is_moving(&self) -> bool {
        self.x != 0.
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct AnimationTimer(pub Timer);

#[derive(Resource, Deref, DerefMut)]
pub struct UseItemAnimationTimer(pub Timer);

#[derive(Resource, Default, Clone, Copy)]
pub struct MovementAnimationIndex(pub usize);

#[derive(Resource, Default, Clone, Copy)]
pub struct UseItemAnimationIndex(pub usize);

#[derive(Resource, Clone, Copy, Default, Deref, DerefMut)]
pub struct PlayerVelocity(pub Velocity);

#[derive(Resource, Clone, Copy, Default)]
pub struct PlayerController {
    pub fall_speed: f32,
    pub apex_point: f32,
    pub jump: i32
}

#[derive(Resource, Clone, Copy, Default)]
pub struct Collisions {
    pub top: bool,
    pub bottom: bool,
    pub left: bool,
    pub right: bool
}

impl Collisions {
    pub fn none(&self) -> bool {
        !self.top && !self.bottom && !self.left && !self.right
    }

    pub fn x(&self) -> bool {
        self.left || self.right
    }

    pub fn y(&self) -> bool {
        self.top || self.bottom
    }
}
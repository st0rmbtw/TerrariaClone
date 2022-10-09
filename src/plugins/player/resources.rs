use bevy::{prelude::{Deref, DerefMut}, time::Timer};

use crate::{Velocity, util::FRect};

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

#[derive(Clone, Copy, Default)]
pub struct PlayerController {
    pub fall_speed: f32,
    pub apex_point: f32,
    pub jump: i32
}

#[derive(Clone, Copy, Default)]
pub struct Collisions {
    pub top: bool,
    pub bottom: Option<FRect>,
    pub left: Option<FRect>,
    pub right: bool
}

impl Collisions {
    pub fn none(&self) -> bool {
        !self.top && !self.bottom.is_some() && !self.left.is_some() && !self.right
    }

    pub fn x(&self) -> bool {
        self.left.is_some() || self.right
    }

    pub fn y(&self) -> bool {
        self.top || self.bottom.is_some()
    }
}
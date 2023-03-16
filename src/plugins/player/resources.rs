use bevy::{prelude::{Deref, DerefMut, Resource, Vec2}, time::Timer};

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
pub struct PlayerVelocity(pub Vec2);

#[derive(Resource, Clone, Copy, Default)]
pub struct PlayerData {
    pub jump: i32,
    // The distance of player's fall in tiles
    pub fall_distance: f32,

    pub prev_position: Vec2
}

#[derive(Debug, Resource, Clone, Copy, Default)]
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
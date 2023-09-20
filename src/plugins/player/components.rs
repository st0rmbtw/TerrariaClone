use bevy::{prelude::{Name, SpatialBundle, Transform, Component, Bundle}, utils::default};

use crate::{common::{state::MovementState, rect::FRect, components::{Velocity, EntityRect}}, PLAYER_LAYER};

use super::{InputAxis, WALKING_ANIMATION_MAX_INDEX, PLAYER_HEIGHT, PLAYER_WIDTH};

#[cfg(feature = "debug")]
use bevy::prelude::{ReflectComponent, Reflect};

#[derive(Component, Default)]
pub(crate) struct Player;

#[derive(Default, PartialEq, Eq, Clone, Copy, Component)]
#[cfg_attr(feature = "debug", derive(Reflect))]
#[cfg_attr(feature = "debug", reflect(Component))]
#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::InspectorOptions))]
pub(crate) enum FaceDirection {
    Left,
    #[default]
    Right,
}

impl FaceDirection {
    #[inline]
    pub fn is_left(&self) -> bool {
        *self == FaceDirection::Left
    }
}

impl From<InputAxis> for Option<FaceDirection> {
    fn from(axis: InputAxis) -> Self {
        match axis.x {
            x if x > 0. => Some(FaceDirection::Right),
            x if x < 0. => Some(FaceDirection::Left),
            _ => None
        }
    }
}

impl From<&FaceDirection> for f32 {
    fn from(direction: &FaceDirection) -> Self {
        match direction {
            FaceDirection::Left => -1.,
            FaceDirection::Right => 1.,
        }
    }
}

pub(super) trait AnimationData {
    fn index(&self) -> usize;
}

#[derive(Component, Clone, Copy)]
pub(super) struct WalkingAnimationData {
    pub offset: usize,
    pub count: usize,
}

impl Default for WalkingAnimationData {
    fn default() -> Self {
        WalkingAnimationData {
            offset: 0,
            count: WALKING_ANIMATION_MAX_INDEX,
        }
    }
}

#[derive(Component, Clone, Copy, Default)]
pub(super) struct IdleAnimationData(pub usize);

#[derive(Component, Clone, Copy, Default)]
pub(super) struct FlyingAnimationData(pub usize);

impl AnimationData for IdleAnimationData {
    fn index(&self) -> usize { self.0 }
}

impl AnimationData for FlyingAnimationData {
    fn index(&self) -> usize { self.0 }
}

#[derive(Bundle, Default)]
pub(super) struct MovementAnimationBundle {
    pub(super) walking: WalkingAnimationData,
    pub(super) idle: IdleAnimationData,
    pub(super) flying: FlyingAnimationData
}

#[derive(Bundle)]
pub(super) struct PlayerBundle {
    pub(super) player: Player,
    pub(super) name: Name,
    pub(super) movement_state: MovementState,
    pub(super) face_direction: FaceDirection,
    pub(super) velocity: Velocity,
    pub(super) rect: EntityRect,
    pub(super) spatial: SpatialBundle
}

impl PlayerBundle {
    pub(crate) fn new(x: f32, y: f32) -> Self {
        Self {
            spatial: SpatialBundle::from_transform(Transform::from_xyz(x, y, PLAYER_LAYER)),
            rect: EntityRect(FRect::new_center(x, y, PLAYER_WIDTH, PLAYER_HEIGHT)),
            ..default()
        }
    }
}

impl Default for PlayerBundle {
    fn default() -> Self {
        Self { 
            name: Name::new("Player"),
            player: Default::default(),
            movement_state: Default::default(),
            face_direction: Default::default(),
            spatial: Default::default(),
            velocity: Default::default(),
            rect: Default::default()
        }
    }
}
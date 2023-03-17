use bevy::{prelude::{Component, Entity, Bundle, Resource, Name, SpatialBundle}, utils::default};

use crate::common::state::MovementState;

use super::{InputAxis, WALKING_ANIMATION_MAX_INDEX};

#[derive(Component, Default)]
pub struct Player;

#[derive(Default, PartialEq, Eq, Clone, Copy, Component)]
#[cfg_attr(feature = "debug", derive(bevy_inspector_egui::InspectorOptions))]
pub enum FaceDirection {
    LEFT,
    #[default]
    RIGHT,
}

impl From<&InputAxis> for Option<FaceDirection> {
    fn from(axis: &InputAxis) -> Self {
        match axis.x {
            x if x > 0. => Some(FaceDirection::RIGHT),
            x if x < 0. => Some(FaceDirection::LEFT),
            _ => None
        }
    }
}

impl From<FaceDirection> for f32 {
    fn from(direction: FaceDirection) -> Self {
        match direction {
            FaceDirection::LEFT => -1.,
            FaceDirection::RIGHT => 1.,
        }
    }
}

impl FaceDirection {
    #[inline]
    pub fn is_left(&self) -> bool {
        *self == FaceDirection::LEFT
    }
}

#[derive(Resource, Component, PartialEq, Clone, Copy)]
pub(super) struct UseItemAnimation(pub bool);

#[derive(Component)]
pub(super) struct ChangeFlip;

#[derive(Component)]
pub(super) struct PlayerBodySprite;

#[derive(Component)]
pub(super) struct UsedItem;

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

#[derive(Component)]
pub struct PlayerParticleEffects {
    pub walking: Entity,
}


#[derive(Component, Clone, Copy, Default)]
pub(super) struct IdleAnimationData(pub usize);

#[derive(Component, Clone, Copy, Default)]
pub(super) struct FlyingAnimationData(pub usize);

#[derive(Component, Clone, Copy, Default)]
pub(super) struct UseItemAnimationData(pub usize);

impl AnimationData for IdleAnimationData {
    fn index(&self) -> usize { self.0 }
}

impl AnimationData for FlyingAnimationData {
    fn index(&self) -> usize { self.0 }
}

#[derive(Bundle, Default)]
pub(super) struct MovementAnimationBundle {
    pub walking: WalkingAnimationData,
    pub idle: IdleAnimationData,
    pub flying: FlyingAnimationData
}

#[derive(Bundle)]
pub(super) struct PlayerBundle {
    pub(super) player: Player,
    pub(super) name: Name,
    pub(super) movement_state: MovementState,
    pub(super) face_direction: FaceDirection,
    #[bundle]
    pub(super) spatial: SpatialBundle
}

impl Default for PlayerBundle {
    fn default() -> Self {
        Self { 
            name: Name::new("Player"),
            player: default(),
            movement_state: default(),
            face_direction: default(),
            spatial: default()
        }
    }
}
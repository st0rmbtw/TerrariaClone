use bevy::{prelude::{Name, SpatialBundle, Transform, Deref, DerefMut, Component, Bundle}, utils::default};

use crate::{common::{state::MovementState, rect::FRect}, PLAYER_LAYER};

use super::{InputAxis, WALKING_ANIMATION_MAX_INDEX, PLAYER_HEIGHT, PLAYER_WIDTH};

#[derive(Component, Default)]
pub(crate) struct Player;

#[derive(Default, PartialEq, Eq, Clone, Copy, Component)]
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

#[derive(Component, Deref, DerefMut, Default)]
pub(crate) struct PlayerRect(pub(crate) FRect);

#[derive(Bundle)]
pub(super) struct PlayerBundle {
    pub(super) player: Player,
    pub(super) name: Name,
    pub(super) movement_state: MovementState,
    pub(super) face_direction: FaceDirection,
    pub(super) player_rect: PlayerRect,
    pub(super) spatial: SpatialBundle
}

impl PlayerBundle {
    pub(crate) fn new(x: f32, y: f32) -> Self {
        Self {
            spatial: SpatialBundle::from_transform(Transform::from_xyz(x, y, PLAYER_LAYER)),
            player_rect: PlayerRect(FRect::new_center(x, y, PLAYER_WIDTH, PLAYER_HEIGHT)),
            ..default()
        }
    }
}

impl Default for PlayerBundle {
    fn default() -> Self {
        Self { 
            name: Name::new("Player"),
            player: default(),
            movement_state: default(),
            face_direction: default(),
            spatial: default(),
            player_rect: default()
        }
    }
}
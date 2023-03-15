use bevy::{prelude::{Component, Entity, Bundle, Resource, Name, SpatialBundle, KeyCode, MouseButton, Transform}, utils::default};
use leafwing_input_manager::{InputManagerBundle, prelude::{ActionState, InputMap}};

use crate::state::MovementState;

use super::{InputAxis, WALKING_ANIMATION_MAX_INDEX, PlayerAction};

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
pub struct UseItemAnimation(pub bool);

#[derive(Component)]
pub struct ChangeFlip;

#[derive(Component)]
pub struct PlayerBodySprite;

#[derive(Component)]
pub struct UsedItem;

pub trait AnimationData {
    fn index(&self) -> usize;
}

#[derive(Component, Clone, Copy)]
pub struct WalkingAnimationData {
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
pub struct IdleAnimationData(pub usize);

#[derive(Component, Clone, Copy, Default)]
pub struct FlyingAnimationData(pub usize);

#[derive(Component, Clone, Copy, Default)]
pub struct UseItemAnimationData(pub usize);

impl AnimationData for IdleAnimationData {
    fn index(&self) -> usize { self.0 }
}

impl AnimationData for FlyingAnimationData {
    fn index(&self) -> usize { self.0 }
}

#[derive(Bundle, Default)]
pub struct MovementAnimationBundle {
    pub walking: WalkingAnimationData,
    pub idle: IdleAnimationData,
    pub flying: FlyingAnimationData
}

#[derive(Bundle, Default)]
pub struct PlayerBundle {
    player: Player,
    name: Name,
    movement_state: MovementState,
    face_direction: FaceDirection,
    #[bundle]
    input_manager: InputManagerBundle<PlayerAction>,
    #[bundle]
    spatial: SpatialBundle
}

impl PlayerBundle {
    pub fn new(transform: Transform) -> Self {
        Self {
            name: Name::new("Player"),
            input_manager: InputManagerBundle::<PlayerAction> {
                action_state: ActionState::default(),
                input_map: InputMap::default()
                    .insert(KeyCode::A, PlayerAction::RunLeft)
                    .insert(KeyCode::D, PlayerAction::RunRight)
                    .insert(KeyCode::Space, PlayerAction::Jump)
                    .insert(MouseButton::Left, PlayerAction::UseItem)
                    .build()
            },
            spatial: SpatialBundle {
                transform,
                ..default()
            },
            ..default()
        }
    }
}
use bevy::{prelude::{Query, With, Component, Quat}, sprite::TextureAtlasSprite};

use crate::common::{state::MovementState};

use super::{Player, AnimationData, PlayerBodySprite, FaceDirection};

pub(super) fn simple_animation<C: AnimationData + Component>(
    mut query: Query<
        (&mut TextureAtlasSprite, &C),
        With<PlayerBodySprite>,
    >,
) {
    query.for_each_mut(|(mut sprite, anim_data)| {
        sprite.index = anim_data.index();
    });
}

pub(super) fn is_walking(
    player_query: Query<&MovementState, With<Player>>,
) -> bool {
    if let Ok(state) = player_query.get_single() {
        if *state == MovementState::Walking {
            return true;
        }
    }

    false
}

pub(super) fn is_idle(
    player_query: Query<&MovementState, With<Player>>,
) -> bool {
    if let Ok(state) = player_query.get_single() {
        if *state == MovementState::Idle {
            return true;
        }
    }

    false
}

pub(super) fn is_flying(
    player_query: Query<&MovementState, With<Player>>,
) -> bool {
    if let Ok(state) = player_query.get_single() {
        if *state == MovementState::Flying {
            return true;
        }
    }

    false
}

pub(super) fn get_rotation_by_direction(direction: FaceDirection) -> Quat {
    let start_rotation = match direction {
        // Assumed that sprite is flipped by y and its anchor is TopLeft
        FaceDirection::Left => 1.2,
        // Assumed that sprite is not flipped and its anchor is BottomLet
        FaceDirection::Right => 2.,
    };

    Quat::from_rotation_z(start_rotation)
}
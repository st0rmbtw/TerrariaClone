use bevy::{prelude::{Query, With, Component, Vec2}, sprite::TextureAtlasSprite};

use crate::{state::MovementState, util::FRect};

use super::{Player, AnimationData, PlayerBodySprite, PLAYER_WIDTH, PLAYER_HEIGHT};

pub fn simple_animation<C: AnimationData + Component>(
    mut query: Query<
        (&mut TextureAtlasSprite, &C),
        With<PlayerBodySprite>,
    >,
) {
    query.for_each_mut(|(mut sprite, anim_data)| {
        sprite.index = anim_data.index();
    });
}

pub fn is_walking(
    player_query: Query<&MovementState, With<Player>>,
) -> bool {
    if let Ok(state) = player_query.get_single() {
        if *state == MovementState::WALKING {
            return true;
        }
    }

    false
}

pub fn is_idle(
    player_query: Query<&MovementState, With<Player>>,
) -> bool {
    if let Ok(state) = player_query.get_single() {
        if *state == MovementState::IDLE {
            return true;
        }
    }

    false
}

pub fn is_flying(
    player_query: Query<&MovementState, With<Player>>,
) -> bool {
    if let Ok(state) = player_query.get_single() {
        if *state == MovementState::FLYING {
            return true;
        }
    }

    false
}

pub fn get_player_rect(position: Vec2, x_multiplier: f32) -> FRect {
    FRect {
        left: position.x - (PLAYER_WIDTH * x_multiplier) / 2.,
        right: position.x + (PLAYER_WIDTH * x_multiplier) / 2.,
        top: position.y - PLAYER_HEIGHT / 2.,
        bottom: position.y + PLAYER_HEIGHT / 2.,
    }
}

pub fn round(number: f32, multiple: f32) -> f32 {
    let mut result = number.abs() + multiple / 2.;
    result -= result % multiple;
    result *= number.signum();

    result
}
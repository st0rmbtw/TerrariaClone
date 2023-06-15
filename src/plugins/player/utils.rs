use bevy::{prelude::{Query, With, Component, Res}, sprite::TextureAtlasSprite};

use crate::{common::state::MovementState, plugins::inventory::{UseItemAnimationData, SwingAnimation}};

use super::{Player, AnimationData, PlayerSpriteBody};

pub(super) fn simple_animation<C: AnimationData + Component>(
    swing_animation: Res<SwingAnimation>,
    mut query: Query<(&mut TextureAtlasSprite, &C, Option<&UseItemAnimationData>), With<PlayerSpriteBody>>,
) {
    query.for_each_mut(|(mut sprite, anim_data, use_item_animation)| {
        if use_item_animation.is_none() || !**swing_animation {
            sprite.index = anim_data.index();
        }
    });
}

pub(super) fn is_walking(
    query_player: Query<&MovementState, With<Player>>,
) -> bool {
    if let Ok(state) = query_player.get_single() {
        if *state == MovementState::Walking {
            return true;
        }
    }

    false
}

pub(super) fn is_idle(
    query_player: Query<&MovementState, With<Player>>,
) -> bool {
    if let Ok(state) = query_player.get_single() {
        if *state == MovementState::Idle {
            return true;
        }
    }

    false
}

pub(super) fn is_flying(
    query_player: Query<&MovementState, With<Player>>,
) -> bool {
    if let Ok(state) = query_player.get_single() {
        if *state == MovementState::Flying {
            return true;
        }
    }

    false
}

pub(super) fn get_fall_distance(position: f32, fall_start: Option<f32>) -> f32 {
    fall_start.map(|fs| (position - fs).abs()).unwrap_or(0.)
}
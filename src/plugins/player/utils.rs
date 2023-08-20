use bevy::{prelude::{Query, With, Component, Res}, sprite::TextureAtlasSprite};

use crate::plugins::inventory::{UseItemAnimationData, SwingAnimation};

use super::{AnimationData, PlayerSpriteBody};

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

#[inline]
pub(super) fn get_fall_distance(position: f32, fall_start: Option<f32>) -> f32 {
    fall_start.map(|fs| (position - fs).abs()).unwrap_or(0.)
}
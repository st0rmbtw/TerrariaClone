use bevy::{prelude::{Query, With, Component, Vec2}, sprite::TextureAtlasSprite};
use bevy_ecs_tilemap::tiles::TilePos;

use crate::{state::MovementState, util::FRect, plugins::world::{TILE_SIZE, WorldData}};

use super::{Player, AnimationData, PlayerBodySprite, PLAYER_SPRITE_WIDTH, PLAYER_SPRITE_HEIGHT, Collisions};

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
        left: position.x - (PLAYER_SPRITE_WIDTH * x_multiplier) / 2.,
        right: position.x + (PLAYER_SPRITE_WIDTH * x_multiplier) / 2.,
        bottom: position.y - PLAYER_SPRITE_HEIGHT / 2.,
        top: position.y + PLAYER_SPRITE_HEIGHT / 2.
    }
}

pub fn round(number: f32, multiple: f32) -> f32 {
    let mut result = number.abs() + multiple / 2.;
    result -= result % multiple;
    result *= number.signum();

    result
}

pub fn get_collisions(
    position: Vec2,
    world_data: &WorldData
) -> Collisions {
    let player_rect = get_player_rect(position, 0.65);
    
    let mut col_left: Option<FRect> = None;
    let mut col_right = false;
    let mut col_down: Option<FRect> = None;
    let mut col_top = false;
    
    let left = (player_rect.left / TILE_SIZE).floor() as u32;
    let right = (player_rect.right / TILE_SIZE).ceil() as u32;
    let top = (player_rect.top / TILE_SIZE).ceil();
    let bottom = ((player_rect.bottom) / TILE_SIZE).floor();
    let utop = top.abs() as u32;
    let ubottom = bottom.abs() as u32;

    for x in left..(right + 2) {
        if col_down.is_some() {
            break;
        }

        if world_data.tile_exists(TilePos { x, y: ubottom }) {
            let rect = FRect {
                left: x as f32 * TILE_SIZE - TILE_SIZE / 2.,
                right: x as f32 * TILE_SIZE + TILE_SIZE / 2.,
                bottom: ubottom as f32 * TILE_SIZE + TILE_SIZE / 2.,
                top: ubottom as f32 * TILE_SIZE - TILE_SIZE / 2.
            };

            if rect.intersect(player_rect) {
                col_down = Some(rect);
            }
        }
    }

    for x in left..(right + 2) {
        if col_top {
            break;
        }

        if world_data.tile_exists(TilePos { x, y: utop }) {
            let rect = FRect {
                left: x as f32 * TILE_SIZE - TILE_SIZE / 2.,
                right: x as f32 * TILE_SIZE + TILE_SIZE / 2.,
                bottom: utop as f32 * TILE_SIZE + TILE_SIZE / 2.,
                top: utop as f32 * TILE_SIZE - TILE_SIZE / 2.
            };

            if rect.intersect(player_rect) {
                col_top = true;
            }
        }
    }

    for y in utop..ubottom {
        if col_left.is_some() {
            break;
        }

        if world_data.tile_exists(TilePos { x: left, y }) {
            let rect = FRect {
                left: left as f32 * TILE_SIZE - TILE_SIZE / 2.,
                right: left as f32 * TILE_SIZE + TILE_SIZE / 2.,
                bottom: y as f32 * TILE_SIZE - TILE_SIZE / 2.,
                top: y as f32 * TILE_SIZE + TILE_SIZE / 2.
            };

            if rect.intersect(player_rect) {
                col_left = Some(rect);
            }
        }
    }

    for y in utop..ubottom {
        if col_right {
            break;
        }

        if world_data.tile_exists(TilePos { x: right, y }) {
            let rect = FRect {
                left: right as f32 * TILE_SIZE - TILE_SIZE / 2.,
                right: right as f32 * TILE_SIZE + TILE_SIZE / 2.,
                bottom: y as f32 * TILE_SIZE - TILE_SIZE / 2.,
                top: y as f32 * TILE_SIZE + TILE_SIZE / 2.
            };

            if rect.intersect(player_rect) {
                col_right = true;
            }
        }
    }

    Collisions { 
        top: col_top, 
        bottom: col_down, 
        left: col_left, 
        right: col_right 
    }
}
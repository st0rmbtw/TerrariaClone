use bevy::{prelude::{Query, With, Component, Vec2}, sprite::TextureAtlasSprite};
use ndarray::ArrayView2;

use crate::{state::MovementState, util::FRect, world_generator::Cell, plugins::world::TILE_SIZE};

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

pub fn get_player_rect(position: Vec2) -> FRect {
    FRect {
        left: position.x - PLAYER_SPRITE_WIDTH / 2.,
        right: position.x + PLAYER_SPRITE_WIDTH / 2.,
        bottom: position.y - PLAYER_SPRITE_HEIGHT / 2.,
        top: position.y + PLAYER_SPRITE_HEIGHT / 2.
    }
}

pub fn get_collisions(
    player_position: Vec2,
    tiles: &ArrayView2<Cell>,
) -> Collisions {
    let bottom = (((player_position.y - PLAYER_SPRITE_HEIGHT / 2.) - 8.) / TILE_SIZE).abs() as usize;
    let top = (((player_position.y + PLAYER_SPRITE_HEIGHT / 2.) + 8.) / TILE_SIZE);

    let utop = top.abs() as usize;

    let mut col_left = false;
    let mut col_right = false;
    let mut col_down = false;
    let mut col_top = false;

    let left = (((player_position.x - (PLAYER_SPRITE_WIDTH * 0.5) / 2.)) / TILE_SIZE).round() as usize;
    let right = (((player_position.x + (PLAYER_SPRITE_WIDTH * 0.5) / 2.)) / TILE_SIZE).round() as usize;

    for x in left..(right + 1) {
        if col_down {
            break;
        }

        if tiles.get((bottom, x)).and_then(|cell| cell.tile).is_some() {
            col_down = true;
        }
    }
    
    for x in left..(right + 1) {
        if col_top {
            break;
        }

        if top == 0. || tiles.get((utop, x)).and_then(|cell| cell.tile).is_some() {
            col_top = true;
        }
    }

    let left = (((player_position.x - (PLAYER_SPRITE_WIDTH * 0.5) / 2.) + 8.) / TILE_SIZE) as usize;

    for y in utop..bottom {
        if col_left {
            break;
        }

        if tiles.get((y, left)).and_then(|cell| cell.tile).is_some() {
            col_left = true;
        }
    }

    let right = (((player_position.x + (PLAYER_SPRITE_WIDTH * 0.5) / 2.) - 6.) / TILE_SIZE) as usize;

    for y in utop..bottom {
        if col_right {
            break;
        }

        if tiles.get((y, right)).and_then(|cell| cell.tile).is_some() {
            col_right = true;
        }
    }

    Collisions { 
        up: col_top, 
        down: col_down, 
        left: col_left, 
        right: col_right 
    }
}
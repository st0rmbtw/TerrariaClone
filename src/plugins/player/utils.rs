use bevy::{prelude::{Query, With, Component, Vec2}, sprite::TextureAtlasSprite};
use bevy_ecs_tilemap::tiles::TilePos;

use crate::{state::MovementState, util::FRect, plugins::world::{TILE_SIZE, WorldData}, world_generator::WORLD_SIZE_Y};

use super::{Player, AnimationData, PlayerBodySprite, PLAYER_WIDTH, PLAYER_HEIGHT, Collisions};

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

// Returns new player velocity and collisions struct 
pub fn get_collisions(
    position: Vec2,
    velocity: Vec2,
    world_data: &WorldData
) -> (Vec2, Collisions) {
    let left = ((position.x - PLAYER_WIDTH / 2.) / TILE_SIZE) - 1.;
    let right = ((position.x + PLAYER_WIDTH / 2.) / TILE_SIZE) + 2.;
    let mut bottom = ((position.y + PLAYER_HEIGHT / 2.) / TILE_SIZE) + 2.;
    let mut top = ((position.y - PLAYER_HEIGHT / 2.) / TILE_SIZE) - 1.;

    if top < 0. {
        top = 0.;
    }

    if bottom > WORLD_SIZE_Y as f32 {
        bottom = WORLD_SIZE_Y as f32;
    }

    let uleft = left as u32;
    let uright = right as u32;
    let utop = top as u32;
    let ubottom = bottom as u32;

    let mut result = velocity;
    let next_position = position - velocity;

    let mut num5 = u32::MAX;
    let mut num6 = u32::MAX;
    let mut num7 = u32::MAX;
    let mut num8 = u32::MAX;

    let mut num9 = (bottom + 3.) * TILE_SIZE;

    let mut collisions = Collisions::default();

    for x in uleft..uright {
        for y in utop..ubottom {
            if world_data.tile_exists(TilePos { x, y }) {
                let tile_pos = Vec2::new(x as f32 * TILE_SIZE, y as f32 * TILE_SIZE);
                
                if (next_position.x + PLAYER_WIDTH / 2.) > (tile_pos.x - TILE_SIZE / 2.) && (next_position.x - PLAYER_WIDTH / 2.) < (tile_pos.x + TILE_SIZE / 2.) && (next_position.y + PLAYER_HEIGHT / 2.) > (tile_pos.y - TILE_SIZE / 2.) && (next_position.y - PLAYER_HEIGHT / 2.) < (tile_pos.y + TILE_SIZE / 2.) {
                    if position.y + PLAYER_HEIGHT / 2. <= tile_pos.y - TILE_SIZE / 2. {
                        collisions.bottom = true;
                        dbg!("D");
                        if num9 > tile_pos.y {
                            num7 = x;
                            num8 = y;
                            if num7 != num5 {
                                result.y = dbg!((tile_pos.y - TILE_SIZE / 2.) - (position.y + PLAYER_HEIGHT / 2.));
                                num9 = tile_pos.y;
                            }
                        }
                    } else {
                        if position.x + PLAYER_WIDTH / 2. <= tile_pos.x - TILE_SIZE / 2. {
                            dbg!("C");
                            num5 = x;
                            num6 = y;
                            if num6 != num8 {
                                result.x = tile_pos.x - TILE_SIZE - (position.x + PLAYER_WIDTH / 2.);
                            }
                            if num7 == num5 {
                                result.y = velocity.y;
                            }
                        } else {
                            if position.x - PLAYER_WIDTH / 2. >= tile_pos.x + TILE_SIZE / 2. {
                                dbg!("B");
                                num5 = x;
                                num6 = y;
                                if num6 != num8 {
                                    result.x = tile_pos.x + TILE_SIZE - (position.x - PLAYER_WIDTH / 2.);
                                }
                                if num7 == num5 {
                                    result.y = velocity.y;
                                }
                            } else {
                                // if position.y >= tile_pos.y + TILE_SIZE / 2. {
                                //     collisions.top = true;
                                //     dbg!("A");
                                //     num7 = x;
                                //     num8 = y;
                                //     result.y = tile_pos.y + TILE_SIZE / 2. - position.y + 0.01;
                                //     if num8 == num6 {
                                //         result.x = velocity.x;
                                //     }
                                // }
                            }
                        }
                    }
                }
            }
        }
    }
        
    (result, collisions)
}
use bevy::{prelude::*, math::Vec3Swizzles};
use bevy_hanabi::prelude::*;

use crate::{
    plugins::{
        world::TILE_SIZE,
        inventory::{UsedItem, SwingAnimation},
    },
    common::{math::{move_towards, map_range_usize}, state::MovementState, rect::FRect}, world::WorldData,
};

use super::*;

#[cfg(feature = "debug")]
use bevy_prototype_debug_lines::DebugLines;

pub(super) fn horizontal_movement(
    axis: Res<InputAxis>,
    mut velocity: ResMut<PlayerVelocity>
) {
    if axis.x > 0. {
        if velocity.x < 0. {
            velocity.x *= 0.9;
        }
        velocity.x += ACCELERATION;
        velocity.x = velocity.x.clamp(-MAX_RUN_SPEED, MAX_RUN_SPEED);
    } else if axis.x < 0. {
        if velocity.x > 0. {
            velocity.x *= 0.9;
        }
        velocity.x -= ACCELERATION;
        velocity.x = velocity.x.clamp(-MAX_RUN_SPEED, MAX_RUN_SPEED);
    } else {
        velocity.x = move_towards(velocity.x, 0., SLOWDOWN);
    } 
}

pub(super) fn update_jump(
    input: Res<Input<KeyCode>>,
    collisions: Res<Collisions>,
    mut velocity: ResMut<PlayerVelocity>,
    mut player_data: ResMut<PlayerData>,
) {
    if input.pressed(KeyCode::Space) && collisions.bottom {
        player_data.jump = JUMP_HEIGHT;
        velocity.y = JUMP_SPEED;
    }

    if input.pressed(KeyCode::Space) {
        if player_data.jump > 0 {
            if velocity.y == 0. {
                player_data.jump = 0;
            } else {
                velocity.y = JUMP_SPEED;

                player_data.jump -= 1;
            }
        }
    } else {
        player_data.jump = 0;
    }
}

pub(super) fn gravity(
    collisions: Res<Collisions>,
    mut player_data: ResMut<PlayerData>,
    mut velocity: ResMut<PlayerVelocity>,
    query_player: Query<&Transform, With<Player>>
) {
    if !collisions.bottom {
        if velocity.y <= 0. && player_data.fall_start == 0. {
            let transform = query_player.single();
            player_data.fall_start = transform.translation.y;
        }

        velocity.y -= GRAVITY;
    }

    velocity.y = velocity.y.max(MAX_FALL_SPEED);
}

pub(super) fn detect_collisions(
    world_data: Res<WorldData>,
    mut collisions: ResMut<Collisions>,
    mut velocity: ResMut<PlayerVelocity>,
    mut player_data: ResMut<PlayerData>,
    mut player_query: Query<&mut Transform, With<Player>>,
    #[cfg(feature = "debug")]
    mut debug_lines: ResMut<DebugLines>,
    #[cfg(feature = "debug")]
    debug_config: Res<DebugConfiguration>,
) {
    let mut transform = player_query.single_mut();

    let position = transform.translation.xy();
    let next_position = transform.translation.xy() + **velocity;

    let player_rect = FRect::new_center(position.x, position.y, PLAYER_WIDTH, PLAYER_HEIGHT);
    let next_player_rect = FRect::new_center(next_position.x, next_position.y, PLAYER_WIDTH, PLAYER_HEIGHT);

    let left = ((position.x - PLAYER_HALF_WIDTH) / TILE_SIZE) - 1.;
    let right = ((position.x + PLAYER_HALF_WIDTH) / TILE_SIZE) + 2.;
    let mut top = ((position.y.abs() - PLAYER_HALF_HEIGHT) / TILE_SIZE) - 1.;
    let mut bottom = ((position.y.abs() + PLAYER_HALF_HEIGHT) / TILE_SIZE) + 2.;

    bottom = bottom.clamp(0., world_data.size.height as f32);
    top = top.max(0.);

    let left_u32 = left as u32;
    let right_u32 = right as u32;
    let top_u32 = top as u32;
    let bottom_u32 = bottom as u32;

    let mut new_collisions = Collisions::default();

    let mut move_player_up = false;

    for x in left_u32..right_u32 {
        for y in top_u32..bottom_u32 {
            if world_data.solid_block_exists((x, y)) {
                let tile_rect = FRect::new_center(
                    x as f32 * TILE_SIZE,
                    -(y as f32 * TILE_SIZE),
                    TILE_SIZE,
                    TILE_SIZE
                );

                if next_player_rect.intersects(&tile_rect) {
                    let delta_x = player_rect.centerx - tile_rect.centerx;
                    let delta_y = if player_rect.centery < tile_rect.centery {
                        player_rect.top().abs() + (tile_rect.top() + tile_rect.height / 2.)
                    } else {
                        player_rect.bottom().abs() + (tile_rect.bottom() - tile_rect.height / 2.)
                    };

                    if delta_x.abs() > delta_y.abs() {
                        // Check if there is a space of 3 blocks to move the player up
                        let is_enough_space = world_data.solid_block_not_exists((x, y - 1))
                            && world_data.solid_block_not_exists((x, y - 2))
                            && world_data.solid_block_not_exists((x, y - 3));

                        // Check if the tile is on the same level as player's legs
                        let is_bottom_tile = tile_rect.top() <= player_rect.bottom() + TILE_SIZE
                            && tile_rect.top() > player_rect.bottom();

                        if is_enough_space && is_bottom_tile {
                            move_player_up = true;
                            transform.translation.y = tile_rect.top() + player_rect.height / 2.;
                            velocity.y = 0.;
                            continue;
                        }

                        if delta_x > 0. {
                            velocity.x = 0.;
                            new_collisions.left = true;

                            // If the player's left side is more to the left than the tile's right side then move the player right.
                            transform.translation.x = tile_rect.right + player_rect.width / 2.;

                            #[cfg(feature = "debug")]
                            if debug_config.show_collisions {
                                tile_rect.draw_right_side(&mut debug_lines, 0.1, Color::BLUE);
                            }
                        } else {
                            velocity.x = 0.;
                            new_collisions.right = true;

                            // If the player's right side is more to the right than the tile's left side then move the player left.
                            transform.translation.x = tile_rect.left - player_rect.width / 2.;

                            #[cfg(feature = "debug")]
                            if debug_config.show_collisions {
                                tile_rect.draw_left_side(&mut debug_lines, 0.1, Color::GREEN);
                            }
                        }
                    } else {
                        // Checking for collisions again with an offset to workaround the bug when the player stuck in a wall.
                        if FRect::new_bounds_h(next_player_rect.left + 2.0, next_player_rect.top(), PLAYER_WIDTH - 4.0, PLAYER_HEIGHT).intersects(&tile_rect) {
                            if delta_y > 0. {
                                velocity.y = 0.;
                                new_collisions.top = true;

                                // If the player's top side is higher than the tile's bottom side then move the player down.
                                if player_rect.top() > tile_rect.bottom() {
                                    velocity.y = tile_rect.bottom() - player_rect.top();
                                }

                                #[cfg(feature = "debug")]
                                if debug_config.show_collisions {
                                    tile_rect.draw_bottom_side(&mut debug_lines, 0.1, Color::YELLOW);
                                }
                            } else {
                                if !new_collisions.bottom && !move_player_up {
                                    // If the player's bottom side is lower than the tile's top side then move the player up
                                    if player_rect.bottom() < tile_rect.top() {
                                        velocity.y = tile_rect.top() - player_rect.bottom();
                                    } else {
                                        transform.translation.y = tile_rect.top() + player_rect.height / 2.;
                                        velocity.y = 0.;
                                    }
                                }

                                new_collisions.bottom = true;

                                if player_data.fall_start != 0. {
                                    let fall_distance = ((position.y.abs() + player_data.fall_start) / TILE_SIZE).ceil();
                                    if fall_distance > 0. {
                                        debug!(fall_distance);
                                    }
                                }
                                player_data.fall_start = 0.;

                                #[cfg(feature = "debug")]
                                if debug_config.show_collisions {
                                    tile_rect.draw_top_side(&mut debug_lines, 0.1, Color::RED);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    *collisions = new_collisions;
}

#[allow(non_upper_case_globals)]
pub(super) fn move_player(
    world_data: Res<WorldData>,
    velocity: Res<PlayerVelocity>,
    mut player_query: Query<&mut Transform, With<Player>>,
    mut player_data: ResMut<PlayerData>
) {
    let mut transform = player_query.single_mut();

    const min_x: f32 = PLAYER_WIDTH * 0.75 / 2. - TILE_SIZE / 2.;
    let min_y: f32 = -(world_data.size.height as f32) * TILE_SIZE + PLAYER_HALF_HEIGHT;

    let max_x = world_data.size.width as f32 * TILE_SIZE - PLAYER_WIDTH * 0.75 / 2. - TILE_SIZE / 2.;
    const max_y: f32 = -PLAYER_HALF_HEIGHT;

    let new_position = (transform.translation.xy() + velocity.0).clamp(Vec2::new(min_x, min_y), Vec2::new(max_x, max_y));
    player_data.prev_position = transform.translation.xy();

    transform.translation.x = new_position.x;
    transform.translation.y = new_position.y;
}

pub(super) fn spawn_particles(
    player: Query<(&MovementState, &PlayerParticleEffects), With<Player>>,
    mut effects: Query<&mut ParticleEffect>,
    collisions: Res<Collisions>
) {
    let (movement_state, particle_effects) = player.single();
    let mut effect = effects.get_mut(particle_effects.walking).unwrap();

    effect
        .maybe_spawner()
        .unwrap()   
        .set_active(*movement_state == MovementState::Walking && collisions.bottom);
}

pub(super) fn update_movement_state(
    collisions: Res<Collisions>,
    player_data: Res<PlayerData>,
    velocity: Res<PlayerVelocity>,
    mut query: Query<&mut MovementState, With<Player>>,
) {
    let mut movement_state = query.single_mut();
    *movement_state = match velocity.0 {
        _ if !collisions.bottom || player_data.jump > 0 => MovementState::Flying,
        Vec2 { x, .. } if x != 0. => MovementState::Walking,
        _ => MovementState::Idle
    };
}

pub(super) fn update_face_direction(axis: Res<InputAxis>, mut query: Query<&mut FaceDirection>) {
    let mut direction = query.single_mut();
    let axis: &InputAxis = &axis;

    if let Some(new_direction) = axis.into() {
        if *direction != new_direction {
            *direction = new_direction;
        }
    }
}

pub(super) fn update_input_axis(input: Res<Input<KeyCode>>, mut axis: ResMut<InputAxis>) {
    let left = input.pressed(KeyCode::A);
    let right = input.pressed(KeyCode::D);

    let x = -(left as i8) + right as i8;

    axis.x = x as f32;
}

pub(super) fn update_movement_animation_timer_duration(
    velocity: Res<PlayerVelocity>,
    mut timer: ResMut<AnimationTimer>,
) {
    if velocity.x != 0. {
        let time = 100. / velocity.x.abs();

        timer.set_duration(Duration::from_millis(time.max(1.) as u64));
    }
}

pub(super) fn update_movement_animation_index(
    time: Res<Time>,
    mut timer: ResMut<AnimationTimer>,
    mut index: ResMut<MovementAnimationIndex>,
) {
    if timer.tick(time.delta()).just_finished() {
        index.0 = (index.0 + 1) % WALKING_ANIMATION_MAX_INDEX;
    }
}

pub(super) fn flip_player(
    player_query: Query<&FaceDirection, (With<Player>, Changed<FaceDirection>)>,
    mut sprite_query: Query<&mut TextureAtlasSprite, With<ChangeFlip>>,
) {
    let direction = player_query.get_single();

    if let Ok(direction) = direction {
        sprite_query.for_each_mut(|mut sprite| {
            sprite.flip_x = direction.is_left();
        });
    }
}

pub(super) fn flip_using_item(
    player_query: Query<&FaceDirection, (With<Player>, Changed<FaceDirection>)>,
    mut sprite_query: Query<&mut Sprite, With<UsedItem>>,
) {
    let direction = player_query.get_single();

    if let Ok(direction) = direction {
        let mut sprite = sprite_query.single_mut();

        match direction {
            FaceDirection::Left => {
                sprite.flip_x = true;
                sprite.anchor = Anchor::BottomRight;
            },
            FaceDirection::Right => {
                sprite.flip_x = false;
                sprite.anchor = Anchor::BottomLeft;
            },
        }
    }
}

pub(super) fn walking_animation(
    swing_animation: Res<SwingAnimation>,
    index: Res<MovementAnimationIndex>,
    mut query: Query<(&mut TextureAtlasSprite, &WalkingAnimationData, Option<&UseItemAnimationData>), With<PlayerBodySprite>>,
) {
    query.for_each_mut(|(mut sprite, anim_data, use_item_animation)| {
        if use_item_animation.is_none() || !**swing_animation {
            let walking_anim_offset = anim_data.offset;
            let walking_anim_count = anim_data.count;

            sprite.index = walking_anim_offset + map_range_usize(
                (0, WALKING_ANIMATION_MAX_INDEX),
                (0, walking_anim_count),
                index.0,
            );
        }
    });
}

#[cfg(feature = "debug")]
pub(super) fn current_speed(
    velocity: Res<PlayerVelocity>,
    mut debug_config: ResMut<DebugConfiguration>
) {
    // https://terraria.fandom.com/wiki/Stopwatch
    let factor = (60. * 3600.) / 42240.;

    let velocity_x = velocity.x.abs() * factor;
    let velocity_y = velocity.y.abs() * factor;

    debug_config.player_speed.x = velocity_x;
    debug_config.player_speed.y = velocity_y;
}

#[cfg(feature = "debug")]
pub(super) fn draw_hitbox(
    query_player: Query<&Transform, With<Player>>,
    mut debug_lines: ResMut<DebugLines>,
) {
    let transform = query_player.single();

    let left = transform.translation.x - PLAYER_HALF_WIDTH;
    let right = transform.translation.x + PLAYER_HALF_WIDTH;

    let top = transform.translation.y - PLAYER_HALF_HEIGHT;
    let bottom = transform.translation.y + PLAYER_HALF_HEIGHT;

    debug_lines.line_colored(
        Vec3::new(left, top, 10.0),
        Vec3::new(right, top, 10.0),
        0.,
        Color::RED
    );

    debug_lines.line_colored(
        Vec3::new(left, bottom, 10.0),
        Vec3::new(right, bottom, 10.0),
        0.,
        Color::RED
    );

    debug_lines.line_colored(
        Vec3::new(left, top, 10.0),
        Vec3::new(left, bottom, 10.0),
        0.,
        Color::RED
    );

    debug_lines.line_colored(
        Vec3::new(right, top, 10.0),
        Vec3::new(right, bottom, 10.0),
        0.,
        Color::RED
    );
}

#[cfg(feature = "debug")]
use crate::plugins::cursor::CursorPosition;

#[cfg(feature = "debug")]
pub(super) fn teleport_player(
    cursor_position: Res<CursorPosition>,
    mut query_player: Query<&mut Transform, With<Player>>,
) {
    if let Ok(mut transform) = query_player.get_single_mut() {
        transform.translation.x = cursor_position.world_position.x;
        transform.translation.y = cursor_position.world_position.y;
    }
}
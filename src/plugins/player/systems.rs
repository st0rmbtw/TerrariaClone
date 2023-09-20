use bevy::{prelude::*, sprite::Anchor};

use crate::{
    plugins::{
        world::constants::TILE_SIZE,
        inventory::{ItemInHand, SwingAnimation},
    },
    common::{math::{move_towards, map_range_usize}, state::MovementState, rect::FRect, components::Velocity}, world::WorldData,
};

use super::{*, utils::get_fall_distance};

pub(super) fn horizontal_movement(
    axis: Res<InputAxis>,
    mut query_player: Query<&mut Velocity, With<Player>>,
) {
    let Ok(mut velocity) = query_player.get_single_mut() else { return; };

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
    mut player_data: ResMut<PlayerData>,
    mut query_player: Query<&mut Velocity, With<Player>>,
    mut jump: Local<i32>
) {
    let Ok(mut velocity) = query_player.get_single_mut() else { return; };
    
    // TODO: Call just_pressed instead when https://github.com/bevyengine/bevy/issues/6183 is fixed
    if input.pressed(KeyCode::Space) && collisions.bottom {
        *jump = JUMP_HEIGHT;
        velocity.y = JUMP_SPEED;
        player_data.jumping = true;
    }

    if input.pressed(KeyCode::Space) {
        if *jump > 0 {
            if velocity.y == 0. {
                *jump = 0;
            } else {
                velocity.y = JUMP_SPEED;

                *jump -= 1;
            }
        }
    } else {
        *jump = 0;
    }
}

pub(super) fn gravity(
    collisions: Res<Collisions>,
    mut player_data: ResMut<PlayerData>,
    mut query_player: Query<(&mut Velocity, &PlayerRect), With<Player>>
) {
    let Ok((mut velocity, rect)) = query_player.get_single_mut() else { return; };

    const DIRECTION: f32 = -1.0;

    if !collisions.bottom {
        if velocity.y <= 0. && player_data.fall_start.is_none() {
            player_data.fall_start = Some(rect.bottom());
        }

        velocity.y += GRAVITY * DIRECTION;
    }

    if velocity.y.abs() > MAX_FALL_SPEED.abs() {
        velocity.y = MAX_FALL_SPEED * DIRECTION;
    }
}

pub(super) fn detect_collisions(
    world_data: Res<WorldData>,
    mut collisions: ResMut<Collisions>,
    mut player_data: ResMut<PlayerData>,
    mut query_player: Query<(&mut PlayerPosition, &mut Velocity, &FaceDirection, &PlayerRect), With<Player>>,
    #[cfg(feature = "debug")]
    mut gizmos: Gizmos,
    #[cfg(feature = "debug")]
    debug_config: Res<DebugConfiguration>,
) {
    let Ok((
        mut player_pos, mut velocity, face_direction, PlayerRect(player_rect)
    )) = query_player.get_single_mut() else { return; };

    let position = player_pos.0;
    let next_position = player_pos.0 + velocity.0;

    let next_player_rect = FRect::new_center(next_position.x, next_position.y, PLAYER_WIDTH, PLAYER_HEIGHT);

    let left = ((position.x - PLAYER_HALF_WIDTH) / TILE_SIZE) - 1.;
    let right = ((position.x + PLAYER_HALF_WIDTH) / TILE_SIZE) + 2.;
    let mut top = ((position.y.abs() - PLAYER_HALF_HEIGHT) / TILE_SIZE) - 1.;
    let bottom = ((position.y.abs() + PLAYER_HALF_HEIGHT) / TILE_SIZE) + 2.;

    top = top.max(0.);

    let left_u32 = left as u32;
    let right_u32 = right as u32;
    let top_u32 = top as u32;
    let bottom_u32 = bottom as u32;

    let mut new_collisions = Collisions::default();

    'outer: for x in left_u32..right_u32 {
        for y in top_u32..bottom_u32 {
            if y >= world_data.size.height as u32 || world_data.solid_block_exists((x, y)) {
                let tile_rect = FRect::new_center(
                    x as f32 * TILE_SIZE,
                    -(y as f32 * TILE_SIZE),
                    TILE_SIZE,
                    TILE_SIZE
                );

                if next_player_rect.intersects(&tile_rect) {
                    let delta_x = tile_rect.centerx - player_rect.centerx;
                    let delta_y = if player_rect.centery < tile_rect.centery {
                        player_rect.top().abs() + (tile_rect.top() + tile_rect.height / 2.)
                    } else {
                        player_rect.bottom().abs() + (tile_rect.bottom() - tile_rect.height / 2.)
                    };

                    if delta_x.abs() > delta_y.abs() {
                        // Check if there is a space of 3 blocks to move the player up
                        let is_enough_space = !world_data.solid_block_exists((x, y - 1))
                            && !world_data.solid_block_exists((x, y - 2))
                            && !world_data.solid_block_exists((x, y - 3));

                        // Check if the tile is on the same level as player's legs
                        let is_bottom_tile = tile_rect.top() <= player_rect.bottom() + TILE_SIZE
                            && tile_rect.top() > player_rect.bottom();

                        if is_enough_space && is_bottom_tile && f32::from(face_direction) == delta_x.signum() {
                            new_collisions.bottom = true;
                            velocity.y = (tile_rect.top() - player_rect.bottom()) * 0.2;
                            break 'outer;
                        }

                        if delta_x < 0. {
                            velocity.x = 0.;
                            new_collisions.left = true;

                            // If the player's left side is more to the left than the tile's right side then move the player right.
                            if next_player_rect.left <= tile_rect.right {
                                player_pos.x = tile_rect.right + player_rect.width / 2.;
                            }

                            #[cfg(feature = "debug")]
                            if debug_config.show_collisions {
                                tile_rect.draw_right_side(&mut gizmos, Color::BLUE);
                            }
                        } else {
                            velocity.x = 0.;
                            new_collisions.right = true;

                            // If the player's right side is more to the right than the tile's left side then move the player left.
                            if next_player_rect.right >= tile_rect.left {
                                player_pos.x = tile_rect.left - player_rect.width / 2.;
                            }

                            #[cfg(feature = "debug")]
                            if debug_config.show_collisions {
                                tile_rect.draw_left_side(&mut gizmos, Color::GREEN);
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
                                    tile_rect.draw_bottom_side(&mut gizmos, Color::YELLOW);
                                }
                            } else {
                                new_collisions.bottom = true;
                                player_data.jumping = false;
                                
                                // If the player's bottom side is lower than the tile's top side then move the player up
                                if player_rect.bottom() >= tile_rect.top() {
                                    player_pos.y = tile_rect.top() + player_rect.height / 2.;
                                    velocity.y = 0.;
                                }

                                let _fall_distance = (get_fall_distance(player_rect.bottom(), player_data.fall_start) / TILE_SIZE).ceil();

                                player_data.fall_start = None;

                                #[cfg(feature = "debug")]
                                if debug_config.show_collisions {
                                    tile_rect.draw_top_side(&mut gizmos, Color::RED);
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
pub(super) fn update_player_position(
    world_data: Res<WorldData>,
    mut query_player: Query<(&mut PlayerPosition, &Velocity), With<Player>>,
) {
    let Ok((mut player_position, velocity)) = query_player.get_single_mut() else { return; };

    const min_x: f32 = PLAYER_HALF_WIDTH - TILE_SIZE / 2.;
    let min_y: f32 = -(world_data.size.height as f32) * TILE_SIZE;

    let max_x = world_data.size.width as f32 * TILE_SIZE - PLAYER_HALF_WIDTH - TILE_SIZE / 2.;
    const max_y: f32 = -PLAYER_HALF_HEIGHT - TILE_SIZE / 2.;

    let new_position = (player_position.0 + velocity.0).floor();

    player_position.0 = new_position.clamp(vec2(min_x, min_y), vec2(max_x, max_y));
}

#[allow(non_upper_case_globals)]
pub(super) fn move_player(
    mut query_player: Query<(&mut Transform, &PlayerPosition), With<Player>>,
) {
    let Ok((mut transform, player_position)) = query_player.get_single_mut() else { return; };

    transform.set_if_neq(transform.with_translation(Vec3::new(player_position.x, player_position.y, transform.translation.z)));
}

pub(super) fn update_player_rect(
    mut query_player: Query<(&PlayerPosition, &mut PlayerRect), With<Player>>,
) {
    let Ok((player_position, mut player_rect)) = query_player.get_single_mut() else { return; };

    *player_rect = PlayerRect(FRect::new_center(player_position.x, player_position.y, PLAYER_WIDTH, PLAYER_HEIGHT));
}

pub(super) fn update_movement_state(
    collisions: Res<Collisions>,
    player_data: Res<PlayerData>,
    mut query: Query<(&PlayerRect, &Velocity, &mut MovementState), With<Player>>,
) {
    let Ok((player_rect, velocity, mut movement_state)) = query.get_single_mut() else { return; };

    let fall_distance = get_fall_distance(player_rect.bottom(), player_data.fall_start);

    *movement_state = match velocity.0 {
        _ if (!collisions.bottom && fall_distance > TILE_SIZE) || player_data.jumping => MovementState::Flying,
        Vec2 { x, .. } if x != 0. => MovementState::Walking,
        _ => MovementState::Idle
    };
}

pub(super) fn update_face_direction(axis: Res<InputAxis>, mut query: Query<&mut FaceDirection>) {
    let mut direction = query.single_mut();

    if let Some(new_direction) = (*axis).into() {
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

pub(super) fn update_movement_animation_timer(
    query_player: Query<&Velocity, With<Player>>,
    mut timer: ResMut<MovementAnimationTimer>,
) {
    let Ok(velocity) = query_player.get_single() else { return; };

    if velocity.x != 0. {
        let time = 100. / velocity.x.abs();

        timer.set_duration(Duration::from_millis(time.max(1.) as u64));
    }
}

pub(super) fn update_movement_animation_index(
    time: Res<Time>,
    mut timer: ResMut<MovementAnimationTimer>,
    mut index: ResMut<MovementAnimationIndex>,
) {
    if timer.tick(time.delta()).just_finished() {
        index.0 = (index.0 + 1) % WALKING_ANIMATION_MAX_INDEX;
    }
}

pub(super) fn flip_player(
    query_player: Query<&FaceDirection, (With<Player>, Changed<FaceDirection>)>,
    mut query_sprite: Query<&mut TextureAtlasSprite, With<ChangeFlip>>,
) {
    let direction = query_player.get_single();

    if let Ok(direction) = direction {
        query_sprite.for_each_mut(|mut sprite| {
            sprite.flip_x = direction.is_left();
        });
    }
}

pub(super) fn flip_using_item(
    query_player: Query<&FaceDirection, (With<Player>, Changed<FaceDirection>)>,
    mut query_sprite: Query<&mut Sprite, With<ItemInHand>>,
) {
    let direction = query_player.get_single();

    if let Ok(direction) = direction {
        let mut sprite = query_sprite.single_mut();

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
    mut query: Query<(&mut TextureAtlasSprite, &WalkingAnimationData, Option<&UseItemAnimationData>), With<PlayerSpriteBody>>,
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
    mut debug_config: ResMut<DebugConfiguration>,
    query_player: Query<&Velocity, With<Player>>,
) {
    let Ok(velocity) = query_player.get_single() else { return; };

    // https://terraria.fandom.com/wiki/Stopwatch
    const FACTOR: f32 = (60. * 3600.) / 42240.;

    debug_config.player_speed = velocity.abs() * FACTOR;
}

#[cfg(feature = "debug")]
pub(super) fn draw_hitbox(
    query_player: Query<&Transform, With<Player>>,
    mut gizmos: Gizmos
) {
    let transform = query_player.single();
    let player_pos = transform.translation.truncate();

    gizmos.rect_2d(player_pos, 0., Vec2::new(PLAYER_WIDTH, PLAYER_HEIGHT), Color::RED);
}

#[cfg(feature = "debug")]
use crate::plugins::{cursor::position::CursorPosition, camera::components::MainCamera};

#[cfg(feature = "debug")]
pub(super) fn teleport_player(
    cursor_position: Res<CursorPosition<MainCamera>>,
    mut query_player: Query<(&mut PlayerPosition, &mut Velocity), With<Player>>,
) {
    if let Ok((mut player_position, mut velocity)) = query_player.get_single_mut() {
        player_position.x = cursor_position.world.x;
        player_position.y = cursor_position.world.y;
        velocity.0 = Vec2::ZERO;
    }
}
use std::f32::consts::PI;

use bevy::{prelude::*, sprite::Anchor};
use rand::{thread_rng, Rng};

use crate::{
    plugins::{
        world::{constants::TILE_SIZE, WORLD_RENDER_LAYER},
        inventory::{ItemInHand, SwingAnimation}, particles::{ParticleCommandsExt, Particle, PARTICLE_SIZE, ParticleBuilder}, entity::components::{EntityRect, Velocity},
    },
    common::{math::{move_towards, map_range_usize}, state::MovementState, rect::FRect, helpers::{random_point_cone, random_point_circle}}, world::WorldData,
};

#[cfg(feature = "debug")]
use bevy_inspector_egui::bevy_egui::EguiContexts;

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
    } else if axis.x < 0. {
        if velocity.x > 0. {
            velocity.x *= 0.9;
        }
        velocity.x -= ACCELERATION;
    } else {
        velocity.x = move_towards(velocity.x, 0., SLOWDOWN);
    }
    velocity.x = velocity.x.clamp(-MAX_WALK_SPEED, MAX_WALK_SPEED);
}

pub(super) fn update_jump(
    input: Res<Input<KeyCode>>,
    collisions: Res<Collisions>,
    mut player_data: ResMut<PlayerData>,
    mut query_player: Query<&mut Velocity, With<Player>>,
    mut jump: Local<i32>,
    #[cfg(feature = "debug")] mut egui: EguiContexts
) {
    #[cfg(feature = "debug")]
    let ctx = egui.ctx_mut();

    #[cfg(feature = "debug")]
    if ctx.wants_keyboard_input() { return; }

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
    mut query_player: Query<(&mut Velocity, &EntityRect), With<Player>>
) {
    let Ok((mut velocity, position)) = query_player.get_single_mut() else { return; };

    const DIRECTION: f32 = -1.0;

    if !collisions.bottom {
        if velocity.y <= 0. && player_data.fall_start.is_none() {
            player_data.fall_start = Some(position.bottom());
        }
    }

    velocity.y += GRAVITY * DIRECTION;

    if velocity.y > MAX_FALL_SPEED {
        velocity.y = MAX_FALL_SPEED;
    } else if velocity.y < -MAX_FALL_SPEED {
        velocity.y = -MAX_FALL_SPEED;
    }
}

pub(super) fn detect_collisions(
    world_data: Res<WorldData>,
    mut collisions: ResMut<Collisions>,
    mut player_data: ResMut<PlayerData>,
    mut query_player: Query<(&mut EntityRect, &mut Velocity, &FaceDirection), With<Player>>,
    #[cfg(feature = "debug")]
    mut gizmos: Gizmos,
    #[cfg(feature = "debug")]
    debug_config: Res<DebugConfiguration>,
) {
    let Ok((
        mut player_rect, mut velocity, face_direction
    )) = query_player.get_single_mut() else { return; };

    let pos = player_rect.center();
    let next_position = pos + velocity.0;

    let next_rect = FRect::new_center(next_position.x, next_position.y, player_rect.width(), player_rect.height());

    let left = (player_rect.left() / TILE_SIZE) - 1.;
    let right = (player_rect.right() / TILE_SIZE) + 2.;
    let mut top = (player_rect.top().abs() / TILE_SIZE) - 1.;
    let bottom = (player_rect.bottom().abs() / TILE_SIZE) + 2.;

    top = top.max(0.);

    let left_u32 = left as u32;
    let right_u32 = right as u32;
    let top_u32 = top as u32;
    let bottom_u32 = bottom as u32;

    let mut new_collisions = Collisions::default();
    player_data.ground = None;

    'outer: for x in left_u32..right_u32 {
        for y in top_u32..bottom_u32 {
            if y >= world_data.playable_area.max.y || world_data.solid_block_exists((x, y)) {
                let tile_rect = FRect::new_center(
                    x as f32 * TILE_SIZE + TILE_SIZE / 2.,
                    -(y as f32 * TILE_SIZE + TILE_SIZE / 2.),
                    TILE_SIZE,
                    TILE_SIZE
                );

                if next_rect.intersects(&tile_rect) {
                    let delta_x = tile_rect.centerx - player_rect.centerx;
                    let delta_y = if player_rect.centery < tile_rect.centery {
                        player_rect.top().abs() + (tile_rect.top() + tile_rect.height() / 2.)
                    } else {
                        player_rect.bottom().abs() + (tile_rect.bottom() - tile_rect.height() / 2.)
                    };

                    if delta_x.abs() > delta_y.abs() {
                        // Step up
                        {
                            // Check if there is a space of 3 blocks to move the player up
                            let is_enough_space = !world_data.solid_block_exists((x, y - 1))
                                && !world_data.solid_block_exists((x, y - 2))
                                && !world_data.solid_block_exists((x, y - 3));

                            // Check if the tile is on the same level as player's legs
                            let is_bottom_tile = tile_rect.top() <= player_rect.bottom() + TILE_SIZE
                                && tile_rect.top() > player_rect.bottom();

                            let direction = f32::from(face_direction);

                            if is_enough_space && is_bottom_tile && direction == delta_x.signum() && velocity.x.abs() > 0. {
                                new_collisions.bottom = true;
                                velocity.y = 1.;
                                velocity.x = velocity.x.abs().max(0.5) * direction;
                                break 'outer;
                            }
                        }

                        if delta_x < 0. {
                            new_collisions.left = true;

                            // If the player's left side is more to the left than the tile's right side then move the player right.
                            if player_rect.left() <= tile_rect.right() {
                                velocity.x = 0.;
                                player_rect.centerx += tile_rect.right() - player_rect.left();
                            }

                            #[cfg(feature = "debug")]
                            if debug_config.show_collisions {
                                tile_rect.draw_right_side(&mut gizmos, Color::BLUE);
                            }
                        } else {
                            new_collisions.right = true;

                            // If the player's right side is more to the right than the tile's left side then move the player left.
                            if player_rect.right() >= tile_rect.left() {
                                velocity.x = 0.;
                                player_rect.centerx += tile_rect.left() - player_rect.right();
                            }

                            #[cfg(feature = "debug")]
                            if debug_config.show_collisions {
                                tile_rect.draw_left_side(&mut gizmos, Color::GREEN);
                            }
                        }
                    } else {
                        // Checking for collisions again with an offset to workaround the bug when the player stuck in a wall.
                        if FRect::new(next_rect.left() + 2.0, next_rect.top(), PLAYER_WIDTH - 4.0, PLAYER_HEIGHT).intersects(&tile_rect) {
                            if delta_y > 0. {
                                new_collisions.top = true;

                                // If the player's top side is higher than the tile's bottom side then move the player down.
                                if player_rect.top() >= tile_rect.bottom() {
                                    velocity.y = 0.;
                                    player_rect.centery += tile_rect.bottom() - player_rect.top();
                                }

                                #[cfg(feature = "debug")]
                                if debug_config.show_collisions {
                                    tile_rect.draw_bottom_side(&mut gizmos, Color::YELLOW);
                                }
                            } else {
                                new_collisions.bottom = true;
                                player_data.jumping = false;
                                player_data.ground = world_data.get_block((x, y)).map(|b| b.block_type);
                                
                                // If the player's bottom side is lower than the tile's top side then move the player up
                                if player_rect.bottom() <= tile_rect.top() {
                                    velocity.y = 0.;
                                    player_rect.centery += tile_rect.top() - player_rect.bottom();
                                }

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

pub(super) fn update_movement_state(
    collisions: Res<Collisions>,
    player_data: Res<PlayerData>,
    mut query: Query<(&EntityRect, &Velocity, &mut MovementState), With<Player>>,
) {
    let Ok((player_rect, velocity, mut movement_state)) = query.get_single_mut() else { return; };

    let fall_distance = get_fall_distance(player_rect.bottom(), player_data.fall_start);

    *movement_state = match velocity.0 {
        _ if (!collisions.bottom && fall_distance > TILE_SIZE) || player_data.jumping => MovementState::Flying,
        Vec2 { x, .. } if x != 0. => MovementState::Walking,
        _ => MovementState::Idle
    };
}

pub(super) fn spawn_particles_on_walk(
    mut commands: Commands,
    player_data: Res<PlayerData>,
    query_player: Query<(&MovementState, &FaceDirection, &Velocity, &EntityRect), With<Player>>,
) {
    let (movement_state, face_direction, velocity, rect) = query_player.single();

    if *movement_state != MovementState::Walking { return; }

    let Some(ground_block) = player_data.ground else { return; };
    if !ground_block.dusty() { return; }
    
    let particle = Particle::get_by_block(ground_block).unwrap();
    
    let direction = match face_direction {
        FaceDirection::Left => vec2(1., 0.),
        FaceDirection::Right => vec2(-1., 0.),
    };

    let mut rng = thread_rng();

    for _ in 0..(velocity.x.abs().floor() as u32) {
        let size = rng.gen_range(0f32..=1f32) * PARTICLE_SIZE;
        let position = vec2(rect.centerx, rect.bottom());

        let point = random_point_cone(direction, 90., 50.);
        let velocity = point.normalize() * 0.5;

        commands.spawn_particle(
            ParticleBuilder::new(particle, position, velocity, 0.3)
                .with_size(size)
                .with_rotation(PI / 12.)
                .with_render_layer(WORLD_RENDER_LAYER)
        );
    }
}

pub(super) fn spawn_particles_grounded(
    mut commands: Commands,
    collisions: Res<Collisions>,
    player_data: Res<PlayerData>,
    query_player: Query<&EntityRect, With<Player>>,
    mut prev_grounded: Local<bool>
) {
    let rect = query_player.single();

    let Some(ground_block) = player_data.ground else {
        *prev_grounded = collisions.bottom;
        return;
    };

    if !ground_block.dusty() {
        *prev_grounded = collisions.bottom;
        return;
    }

    let particle = Particle::get_by_block(ground_block).unwrap();

    let fall_distance = get_fall_distance(rect.bottom(), player_data.fall_start);

    if !*prev_grounded && collisions.bottom && fall_distance > TILE_SIZE * 1.5 {
        let center = vec2(rect.centerx, rect.bottom());

        let mut rng = thread_rng();

        for _ in 0..10 {
            let size = rng.gen_range(0f32..=1f32) * PARTICLE_SIZE;
            let point = random_point_circle(1., 0.5) * PLAYER_HALF_WIDTH;
            let position = center + point;
            let velocity = vec2(point.normalize().x, 0.5);

            commands.spawn_particle(
                ParticleBuilder::new(particle, position, velocity, 0.3)
                    .with_size(size)
                    .with_render_layer(WORLD_RENDER_LAYER)
                    .with_rotation(PI / 12.)
            );
        }
    }

    *prev_grounded = collisions.bottom;
}

pub(super) fn update_face_direction(axis: Res<InputAxis>, mut query: Query<&mut FaceDirection>) {
    let mut direction = query.single_mut();

    if let Some(new_direction) = (*axis).into() {
        direction.set_if_neq(new_direction);
    }
}

pub(super) fn update_input_axis(
    input: Res<Input<KeyCode>>,
    mut axis: ResMut<InputAxis>,
    #[cfg(feature = "debug")] mut egui: EguiContexts,
    #[cfg(feature = "debug")] debug_config: Res<DebugConfiguration>
) {
    axis.x = 0.;

    #[cfg(feature = "debug")]
    let ctx = egui.ctx_mut();

    #[cfg(feature = "debug")]
    if ctx.wants_keyboard_input() || debug_config.free_camera { return; }
    
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
    let Ok(direction) = query_player.get_single() else { return; };

    query_sprite.for_each_mut(|mut sprite| {
        sprite.flip_x = direction.is_left();
    });
}

pub(super) fn flip_using_item(
    query_player: Query<&FaceDirection, (With<Player>, Changed<FaceDirection>)>,
    mut query_sprite: Query<&mut Sprite, With<ItemInHand>>,
) {
    let Ok(direction) = query_player.get_single() else { return; };
    
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

pub(super) fn walking_animation(
    swing_animation: Res<SwingAnimation>,
    index: Res<MovementAnimationIndex>,
    mut query: Query<(&mut TextureAtlasSprite, &WalkingAnimationData, Option<&UseItemAnimationData>), With<PlayerSpriteBody>>,
) {
    query.for_each_mut(|(mut sprite, anim_data, use_item_animation)| {
        if use_item_animation.is_none() || !**swing_animation {
            sprite.index = anim_data.offset + map_range_usize(
                (0, WALKING_ANIMATION_MAX_INDEX),
                (0, anim_data.count),
                index.0,
            );
        }
    });
}

pub(super) fn reset_fallstart(
    collisions: Res<Collisions>,
    mut player_data: ResMut<PlayerData>
) {
    if collisions.bottom {
        player_data.fall_start = None;
    }
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
    query_player: Query<&EntityRect, With<Player>>,
    mut gizmos: Gizmos
) {
    let player_rect = query_player.single();
    let player_pos = player_rect.center();

    gizmos.rect_2d(player_pos, 0., vec2(PLAYER_WIDTH, PLAYER_HEIGHT), Color::RED);
}

#[cfg(feature = "debug")]
use crate::plugins::{cursor::position::CursorPosition, camera::components::MainCamera};

#[cfg(feature = "debug")]
pub(super) fn teleport_player(
    cursor_position: Res<CursorPosition<MainCamera>>,
    mut query_player: Query<(&mut EntityRect, &mut Velocity), With<Player>>,
) {
    let Ok((mut player_rect, mut velocity)) = query_player.get_single_mut() else { return; };
    
    player_rect.centerx = cursor_position.world.x;
    player_rect.centery = cursor_position.world.y;
    velocity.0 = Vec2::ZERO;
}
use bevy::{prelude::*, math::Vec3Swizzles};
use bevy_hanabi::prelude::*;
use leafwing_input_manager::prelude::ActionState;
use rand::seq::SliceRandom;

use crate::{
    state::MovementState,
    plugins::{
        world::{WorldData, TILE_SIZE}, 
        assets::{ItemAssets, SoundAssets}, 
        inventory::{SelectedItem, Inventory},
    }, 
    util::{move_towards, map_range_usize, self}, 
    items::{get_animation_points, ItemStack, Item}
};

use super::*;

#[cfg(feature = "debug")]
use bevy_prototype_debug_lines::DebugLines;

pub fn horizontal_movement(
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

pub fn update_jump(
    query: Query<&ActionState<PlayerAction>, With<Player>>,
    collisions: Res<Collisions>,
    mut velocity: ResMut<PlayerVelocity>,
    mut player_data: ResMut<PlayerData>,
) {
    let input = query.single();

    if input.pressed(PlayerAction::Jump) && collisions.bottom {
        player_data.jump = JUMP_HEIGHT;
        velocity.y = JUMP_SPEED;
    }

    if input.pressed(PlayerAction::Jump) {
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

pub fn gravity(
    time: Res<Time>,
    collisions: Res<Collisions>,
    mut player_data: ResMut<PlayerData>,
    mut velocity: ResMut<PlayerVelocity>,
) {
    if !collisions.bottom {
        player_data.fall_distance += GRAVITY / (time.delta_seconds() * 16.);
        velocity.y -= GRAVITY;

        velocity.y = velocity.y.max(MAX_FALL_SPEED);
    }
}

pub fn detect_collisions(
    time: Res<Time>,
    world_data: Res<WorldData>,
    mut collisions: ResMut<Collisions>,
    mut velocity: ResMut<PlayerVelocity>,
    mut player_data: ResMut<PlayerData>,
    player_query: Query<&Transform, With<Player>>,
) {
    let transform = player_query.single();

    let position = (transform.translation.xy()).abs();
    let next_position = (transform.translation.xy() + velocity.0).abs();

    let left = ((position.x - PLAYER_HALF_WIDTH) / TILE_SIZE) - 1.;
    let right = ((position.x + PLAYER_HALF_WIDTH) / TILE_SIZE) + 2.;
    let mut bottom = ((position.y + PLAYER_HALF_HEIGHT) / TILE_SIZE) + 3.;
    let mut top = ((position.y - PLAYER_HALF_HEIGHT) / TILE_SIZE) - 1.;

    bottom = bottom.clamp(0., world_data.size.height as f32);
    top = top.max(0.);

    let left_u32 = left as u32;
    let right_u32 = right as u32;
    let top_u32 = top as u32;
    let bottom_u32 = bottom as u32;

    let mut new_collisions = Collisions::default();

    let mut yx: i32;
    let mut xy: i32;
    let mut yy: i32 = -1;
    let mut xx: i32 = -1;

    let mut a = (bottom + 3.) * TILE_SIZE;

    for x in left_u32..right_u32 {
        for y in top_u32..bottom_u32 {
            if world_data.solid_block_exists((x, y)) {
                let tile_pos = Vec2::new(x as f32 * TILE_SIZE, y as f32 * TILE_SIZE);

                if (next_position.x + PLAYER_HALF_WIDTH) > (tile_pos.x - TILE_SIZE / 2.) && (next_position.x - PLAYER_HALF_WIDTH) < (tile_pos.x + TILE_SIZE / 2.) && (next_position.y + PLAYER_HALF_HEIGHT) > (tile_pos.y - TILE_SIZE / 2.) && (next_position.y - PLAYER_HALF_HEIGHT) < (tile_pos.y + TILE_SIZE / 2.) {
                    if position.y + PLAYER_HALF_HEIGHT <= tile_pos.y - TILE_SIZE / 2. {
                        new_collisions.bottom = true;

                        let fall_distance = (player_data.fall_distance / 16.).round();

                        if fall_distance > 0. {
                            debug!(
                                fall_distance = fall_distance
                            );
                        }
                        
                        player_data.fall_distance = 0.;

                        if a > tile_pos.y {
                            yx = x as i32;
                            yy = y as i32;
                            if yx != xx {
                                // velocity.y = ((tile_pos.y - TILE_SIZE / 2.) - (position.y + PLAYER_HALF_HEIGHT));
                                velocity.y = 0.;
                                a = tile_pos.y;
                            }
                        }
                    } else if position.x + PLAYER_HALF_WIDTH <= tile_pos.x - TILE_SIZE / 2. {
                        new_collisions.right = true;
                        velocity.x = 0.;
                        xx = x as i32;
                        xy = y as i32;
                        if xy != yy {
                            velocity.x = (tile_pos.x - TILE_SIZE / 2.) - (position.x + PLAYER_HALF_WIDTH);
                        }
                    } else if position.x - PLAYER_HALF_WIDTH >= tile_pos.x + TILE_SIZE / 2. {
                        new_collisions.left = true;
                        velocity.x = 0.;
                        xx = x as i32;
                        xy = y as i32;
                        if xy != yy {
                            velocity.x = (tile_pos.x + TILE_SIZE / 2.) - (position.x - PLAYER_HALF_WIDTH);
                        }
                    } else if position.y >= tile_pos.y + TILE_SIZE / 2. {
                        collisions.top = true;
                        yx = x as i32;
                        yy = y as i32;
                        velocity.y = ((tile_pos.y + TILE_SIZE / 2.) - (position.y - PLAYER_HALF_HEIGHT)) * time.delta_seconds();
                    }
                }
            }
        }
    }

    *collisions = new_collisions;
}

#[allow(non_upper_case_globals)]
pub fn move_player(
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
    
    transform.translation.x = new_position.x;
    transform.translation.y = new_position.y;

    player_data.prev_position = new_position;
}

pub fn interpolate_player_transform(
    time: Res<Time>,
    player_data: Res<PlayerData>,
    fixed_time: Res<FixedTime>,
    mut player_query: Query<&mut Transform, With<Player>>,
) {
    let mut transform = player_query.single_mut();

    let fixed_elapsed = fixed_time.accumulated().as_secs_f32();
    let fixed_delta_time = fixed_time.period.as_secs_f32();
    
    let interpolation = (time.elapsed_seconds() - fixed_elapsed) / fixed_delta_time;

    let interpolated = player_data.prev_position.lerp(transform.translation.xy(), interpolation);
    
    transform.translation.x = interpolated.x;
    transform.translation.y = interpolated.y;
}

pub fn spawn_particles(
    player: Query<(&MovementState, &FaceDirection, &PlayerParticleEffects), With<Player>>,
    mut effects: Query<(&mut ParticleEffect, &mut Transform)>,
) {
    for (movement_state, face_direction, particle_effects) in &player {
        let (mut effect, mut effect_transform) = effects.get_mut(particle_effects.walking).unwrap();

        effect_transform.translation = match face_direction {
            FaceDirection::LEFT => Vec3::new(0., -PLAYER_HALF_HEIGHT, 0.),
            FaceDirection::RIGHT => Vec3::new(0., -PLAYER_HALF_HEIGHT, 0.),
        };

        effect
            .maybe_spawner()
            .unwrap()   
            .set_active(*movement_state == MovementState::Walking);
    }
}

pub fn update_movement_state(
    player_data: Res<PlayerData>,
    velocity: Res<PlayerVelocity>,
    mut query: Query<&mut MovementState, With<Player>>,
) {
    let mut movement_state = query.single_mut();

    *movement_state = match velocity.0 {
        _ if (player_data.fall_distance.round() / 16.) > 1. || player_data.jump > 0 => MovementState::Flying,
        Vec2 { x, .. } if x != 0. => MovementState::Walking,
        _ => MovementState::Idle
    };
}

pub fn update_face_direction(axis: Res<InputAxis>, mut query: Query<&mut FaceDirection>) {
    let mut direction = query.single_mut();
    let axis: &InputAxis = &axis;

    if let Some(new_direction) = axis.into() {
        if *direction != new_direction {
            *direction = new_direction;
        }
    }
}

pub fn update_axis(query: Query<&ActionState<PlayerAction>, With<Player>>, mut axis: ResMut<InputAxis>) {
    let input = query.single();

    let left = input.pressed(PlayerAction::RunLeft);
    let right = input.pressed(PlayerAction::RunRight);

    let x = -(left as i8) + right as i8;

    axis.x = x as f32;
}

pub fn update_movement_animation_timer_duration(
    velocity: Res<PlayerVelocity>,
    mut timer: ResMut<AnimationTimer>,
) {
    if velocity.x != 0. {
        let time = 100. / velocity.x.abs();

        timer.set_duration(Duration::from_millis(time.max(1.) as u64));
    }
}

pub fn update_movement_animation_index(
    time: Res<Time>,
    mut timer: ResMut<AnimationTimer>,
    mut index: ResMut<MovementAnimationIndex>,
) {
    if timer.tick(time.delta()).just_finished() {
        index.0 = (index.0 + 1) % WALKING_ANIMATION_MAX_INDEX;
    }
}

pub fn flip_player(
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

pub fn walking_animation(
    index: Res<MovementAnimationIndex>,
    mut query: Query<
        (&mut TextureAtlasSprite, &WalkingAnimationData),
        With<PlayerBodySprite>,
    >,
) {
    query.for_each_mut(|(mut sprite, anim_data)| {
        let walking_anim_offset = anim_data.offset;
        let walking_anim_count = anim_data.count;

        sprite.index = walking_anim_offset + map_range_usize(
            (0, WALKING_ANIMATION_MAX_INDEX),
            (0, walking_anim_count),
            index.0,
        );
    });
}

pub fn player_using_item(
    query: Query<&ActionState<PlayerAction>, With<Player>>,
    selected_item: Res<SelectedItem>,
    mut anim: ResMut<UseItemAnimation>,
) {
    let input = query.single();

    let using_item = input.pressed(PlayerAction::UseItem) && selected_item.is_some();

    if using_item {
        anim.0 = true;
    }
}

pub fn set_using_item_visibility(
    anim: Res<UseItemAnimation>,
    mut using_item_query: Query<&mut Visibility, With<UsedItem>>,
) {
    if let Ok(visibility) = using_item_query.get_single_mut() {
        util::set_visibility(visibility, anim.0);
    }
}

pub fn set_using_item_image(
    item_assets: Res<ItemAssets>,
    selected_item: Res<SelectedItem>,
    mut using_item_query: Query<&mut Handle<Image>, With<UsedItem>>,
) {
    let mut image = using_item_query.single_mut();

    if let Some(item_stack) = selected_item.0 {
        *image = item_assets.get_by_item(item_stack.item);
    }
}

pub fn set_using_item_position(
    index: Res<UseItemAnimationIndex>,
    selected_item: Res<SelectedItem>,
    mut using_item_query: Query<&mut Transform, With<UsedItem>>,
    player_query: Query<&FaceDirection, With<Player>>,
) {
    let mut transform = using_item_query.single_mut();
    let direction = player_query.single();

    if selected_item.is_some() {
        let position = get_animation_points()[index.0];

        transform.translation.x = position.x * f32::from(*direction);
        transform.translation.y = position.y;
    }
}

pub fn set_using_item_rotation_on_player_direction_change(
    player_query: Query<&FaceDirection, (With<Player>, Changed<FaceDirection>)>,
    mut using_item_query: Query<&mut Transform, With<UsedItem>>,
) {
    let player_query_result = player_query.get_single();
    let using_item_query_result = using_item_query.get_single_mut();

    if let Ok(mut transform) = using_item_query_result {
        if let Ok(direction) = player_query_result {
            transform.rotation = get_rotation_by_direction(*direction);
        }
    }
}

pub fn set_using_item_rotation(
    time: Res<Time>,
    index: Res<UseItemAnimationIndex>,
    selected_item: Res<SelectedItem>,
    mut using_item_query: Query<&mut Transform, With<UsedItem>>,
    player_query: Query<&FaceDirection, With<Player>>,
) {
    const ROTATION_STEP: f32 = -11.;

    let direction = player_query.single();
    let mut transform = using_item_query.single_mut();

    if selected_item.is_some() {
        let position = get_animation_points()[index.0];
        let direction_f = f32::from(*direction);

        transform.rotate_around(
            position.extend(0.15),
            Quat::from_rotation_z(ROTATION_STEP * direction_f * time.delta_seconds()),
        );

        if index.0 == 0 && index.is_changed() {
            transform.rotation = get_rotation_by_direction(*direction);
        }
    }
}

pub fn update_use_item_animation_index(
    time: Res<Time>,
    mut index: ResMut<UseItemAnimationIndex>,
    mut timer: ResMut<UseItemAnimationTimer>,
    mut anim: ResMut<UseItemAnimation>,
    inventory: Res<Inventory>,
    sound_assets: Res<SoundAssets>,
    audio: Res<Audio>
) {
    if timer.tick(time.delta()).just_finished() {
        if index.0 == 0 {
            if let Some(ItemStack { item: Item::Tool(_), .. }) = inventory.selected_item() {
                let sound = sound_assets.swing.choose(&mut rand::thread_rng()).unwrap();
                audio.play(sound.clone_weak());
            }
        }

        index.0 = (index.0 + 1) % USE_ITEM_ANIMATION_FRAMES_COUNT;
    }

    if index.is_changed() && index.0 == 0 {
        anim.0 = false;
    }
}

pub fn use_item_animation(
    index: Res<UseItemAnimationIndex>,
    mut query: Query<(&mut TextureAtlasSprite, &UseItemAnimationData), With<PlayerBodySprite>>,
) {
    query.for_each_mut(|(mut sprite, anim_data)| {
        sprite.index = anim_data.0 + index.0;
    });
}

pub fn current_speed(
    velocity: Res<PlayerVelocity>
) {
    // https://terraria.fandom.com/wiki/Stopwatch
    let factor = (60. * 3600.) / 42240.;

    let velocity_x = velocity.x.abs() as f64 * factor;
    let velocity_y = velocity.y.abs() as f64 * factor;

    if velocity_x > 0. {
        debug!(
            horizontal = velocity_x,
        );
    }

    if velocity_y > 0. {
        debug!(
            vertical = velocity_y,
        );
    }
}

pub fn draw_hitbox(
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
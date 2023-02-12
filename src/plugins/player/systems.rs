use bevy::{prelude::*, math::Vec3Swizzles, diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin}};
use bevy_ecs_tilemap::tiles::TilePos;
use bevy_hanabi::prelude::*;

use crate::{
    state::MovementState,
    plugins::{
        world::{WorldData, TILE_SIZE}, 
        assets::ItemAssets, 
        inventory::SelectedItem,
    }, 
    world_generator::{WORLD_SIZE_X, WORLD_SIZE_Y}, 
    util::{move_towards, map_range}, 
    items::get_animation_points, CellArrayExtensions
};

use super::*;

#[cfg(feature = "debug_movement")]
pub fn debug_horizontal_movement(
    axis: Res<InputAxis>,
    mut velocity: ResMut<PlayerVelocity>
) {
    velocity.x = axis.x * 10.;
}

#[cfg(feature = "debug_movement")]
pub fn debug_vertical_movement(
    input: Res<Input<KeyCode>>,
    mut velocity: ResMut<PlayerVelocity>
) {
    let up = input.pressed(KeyCode::W);
    let down = input.pressed(KeyCode::S);

    let y = -(down as i8) + up as i8;

    velocity.y = y as f32 * 10.;
}

pub fn horizontal_movement(
    time: Res<Time>,
    axis: Res<InputAxis>,
    mut velocity: ResMut<PlayerVelocity>
) {
    if axis.is_moving() {
        let max_speed = MAX_RUN_SPEED * time.delta_seconds();

        velocity.x += axis.x * ACCELERATION * time.delta_seconds();
        velocity.x = velocity.x.clamp(-max_speed, max_speed);
    } else {
        velocity.x = move_towards(velocity.x, 0., SLOWDOWN * time.delta_seconds());
    } 
}

pub fn update_jump(
    time: Res<Time>,
    input: Res<Input<KeyCode>>,
    collisions: Res<Collisions>,
    mut velocity: ResMut<PlayerVelocity>,
    mut player_controller: ResMut<PlayerController>,
) {
    if input.pressed(KeyCode::Space) && collisions.bottom {
        player_controller.jump = JUMP_HEIGHT;
        velocity.y = JUMP_SPEED;
    }

    if input.pressed(KeyCode::Space) {
        if player_controller.jump > 0 {
            if velocity.y == 0. {
                player_controller.jump = 0;
            } else {
                velocity.y = JUMP_SPEED;

                player_controller.jump -= 1;
            }
        }
    } else {
        player_controller.jump = 0;
    }
}

pub fn update(
    time: Res<Time>,
    mut player_query: Query<&mut Transform, With<Player>>,
    world_data: Res<WorldData>,
    mut velocity: ResMut<PlayerVelocity>,
    mut collisions: ResMut<Collisions>,
    mut controller: ResMut<PlayerController>
) {
    const PLAYER_HALF_WIDTH: f32 = PLAYER_WIDTH / 2.;
    const PLAYER_HALF_HEIGHT: f32 = PLAYER_HEIGHT / 2.;
    const MIN: f32 = PLAYER_WIDTH * 0.75 / 2. - TILE_SIZE / 2.;
    const MAX: f32 = WORLD_SIZE_X as f32 * TILE_SIZE - PLAYER_WIDTH * 0.75 / 2. - TILE_SIZE / 2.;

    let mut transform = player_query.single_mut();

    if !collisions.bottom {
        controller.fall_distance += GRAVITY * time.delta_seconds();
        velocity.y -= GRAVITY * time.delta_seconds();

        let max_fall_speed = MAX_FALL_SPEED * time.delta_seconds();

        velocity.y = velocity.y.max(max_fall_speed);
    }

    let position = (transform.translation.xy()).abs();

    let player_rect = crate::Rect::new(
        transform.translation.x, transform.translation.y, velocity.x, velocity.y, PLAYER_WIDTH, PLAYER_HEIGHT
    );

    let left = ((position.x - PLAYER_HALF_WIDTH) / TILE_SIZE) - 1.;
    let right = ((position.x + PLAYER_HALF_WIDTH) / TILE_SIZE) + 2.;
    let mut bottom = ((position.y + PLAYER_HALF_HEIGHT) / TILE_SIZE) + 3.;
    let mut top = ((position.y - PLAYER_HALF_HEIGHT) / TILE_SIZE) - 1.;

    bottom = bottom.clamp(0., WORLD_SIZE_Y as f32);
    top = top.max(0.);

    let uleft = left as u32;
    let uright = right as u32;
    let utop = top as u32;
    let ubottom = bottom as u32;

    let mut new_collisions = Collisions::default();

    for x in uleft..uright {
        for y in utop..ubottom {
            if world_data.tiles.tile_exists(TilePos::new(x, y)) {
                let tile_pos = Vec2::new(x as f32 * TILE_SIZE, y as f32 * TILE_SIZE);

                let tile_rect = crate::Rect::new(tile_pos.x, -tile_pos.y, 0., 0., TILE_SIZE, TILE_SIZE);

                let (a, cols) = player_rect.swept_aabb(tile_rect);

                println!("{}", cols.bottom);
                
                new_collisions = cols;

                if cols.y() {
                    velocity.y = velocity.y * a;
                }

                if cols.x() {
                    velocity.x = velocity.x * a;
                }

                // if (next_position.x + PLAYER_WIDTH / 2.) > (tile_pos.x - TILE_SIZE / 2.) && (next_position.x - PLAYER_WIDTH / 2.) < (tile_pos.x + TILE_SIZE / 2.) && (next_position.y + PLAYER_HEIGHT / 2.) > (tile_pos.y - TILE_SIZE / 2.) && (next_position.y - PLAYER_HEIGHT / 2.) < (tile_pos.y + TILE_SIZE / 2.) {
                //     if position.y + PLAYER_HEIGHT / 2. <= tile_pos.y - TILE_SIZE / 2. {
                //         new_collisions.bottom = true;

                //         velocity.y = 0.;

                //         if controller.fall_distance_in_tiles() > 0. {
                //             println!("------------------------------");
                //             println!("Fall distance: {} px, {} tiles", controller.fall_distance.round(), controller.fall_distance_in_tiles());
                //             println!("------------------------------");
                //         }
                        
                //         controller.fall_distance = 0.;

                //         if num9 > tile_pos.y {
                //             num7 = x as i32;
                //             num8 = y as i32;
                //             if num7 != num5 {
                //                 velocity.y = ((tile_pos.y - TILE_SIZE / 2.) - (position.y + PLAYER_HEIGHT / 2.)) * time.delta_seconds();
                //                 num9 = tile_pos.y;
                //             }
                //         }
                //     } else {
                //         if position.x + PLAYER_WIDTH / 2. <= tile_pos.x - TILE_SIZE / 2. {
                //             num5 = x as i32;
                //             num6 = y as i32;
                //             if num6 != num8 {
                //                 velocity.x = ((tile_pos.x - TILE_SIZE / 2.) - (position.x + PLAYER_WIDTH / 2.)) * time.delta_seconds();
                //             }
                //             if num7 == num5 {
                //                 velocity.y = velocity.y;
                //             }
                //         } else {
                //             if position.x - PLAYER_WIDTH / 2. >= tile_pos.x + TILE_SIZE / 2. {
                //                 num5 = x as i32;
                //                 num6 = y as i32;
                //                 if num6 != num8 {
                //                     velocity.x = ((tile_pos.x + TILE_SIZE / 2.) - (position.x - PLAYER_WIDTH / 2.)) * time.delta_seconds();
                //                 }
                //                 if num7 == num5 {
                //                     velocity.y = velocity.y;
                //                 }
                //             } else {
                //                 if position.y >= tile_pos.y + TILE_SIZE / 2. {
                //                     collisions.top = true;
                //                     num7 = x as i32;
                //                     num8 = y as i32;
                //                     velocity.y = ((tile_pos.y + TILE_SIZE / 2.) - (position.y - PLAYER_HEIGHT / 2.) + 0.01) * time.delta_seconds();
                //                     if num8 == num6 {
                //                         velocity.x = velocity.x;
                //                     }
                //                 }
                //             }
                //         }
                //     }
                // }
            }
        }
    }

    // println!("{}", new_collisions.bottom);

    *collisions = new_collisions;

    let raw = transform.translation.xy() + velocity.0;

    transform.translation.x = raw.x.clamp(MIN, MAX);
    transform.translation.y = raw.y.clamp(-(WORLD_SIZE_Y as f32) * TILE_SIZE + PLAYER_HALF_HEIGHT, -PLAYER_HALF_HEIGHT);
}

pub fn spawn_particles(
    player: Query<(&MovementState, &FaceDirection, &PlayerParticleEffects), With<Player>>,
    mut effects: Query<(&mut ParticleEffect, &mut Transform)>,
) {
    for (movement_state, face_direction, particle_effects) in &player {
        let (mut effect, mut effect_transform) = effects.get_mut(particle_effects.walking).unwrap();

        effect_transform.translation = match face_direction {
            FaceDirection::LEFT => Vec3::new(0., -PLAYER_HEIGHT / 2., 0.),
            FaceDirection::RIGHT => Vec3::new(0., -PLAYER_HEIGHT / 2., 0.),
        };

        effect
            .maybe_spawner()
            .unwrap()   
            .set_active(*movement_state == MovementState::Walking);
    }
}

pub fn update_movement_state(
    player_controller: Res<PlayerController>,
    velocity: Res<PlayerVelocity>,
    mut query: Query<&mut MovementState, With<Player>>,
) {
    let mut movement_state = query.single_mut();

    *movement_state = match velocity.0 {
        Vec2 { x, y } if x != 0. && y == 0. => MovementState::Walking,
        _ if player_controller.fall_distance_in_tiles() > 1. || player_controller.jump > 0 => MovementState::Flying,
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

pub fn update_axis(input: Res<Input<KeyCode>>, mut axis: ResMut<InputAxis>) {
    let left = input.pressed(KeyCode::A);
    let right = input.pressed(KeyCode::D);

    let x = -(left as i8) + right as i8;

    axis.x = x as f32;
}

pub fn update_movement_animation_timer_duration(
    velocity: Res<PlayerVelocity>,
    mut timer: ResMut<AnimationTimer>,
) {
    if velocity.x != 0. {
        let time = 50. / velocity.x.abs();

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

        sprite.index = walking_anim_offset + map_range(
            (0, WALKING_ANIMATION_MAX_INDEX),
            (0, walking_anim_count),
            index.0,
        );
    });
}

pub fn player_using_item(
    input: Res<Input<MouseButton>>,
    selected_item: Res<SelectedItem>,
    mut anim: ResMut<UseItemAnimation>,
) {
    let using_item = input.pressed(MouseButton::Left) && selected_item.is_some();

    if using_item {
        anim.0 = true;
    }
}

pub fn set_using_item_visibility(
    anim: Res<UseItemAnimation>,
    mut using_item_query: Query<&mut Visibility, With<UsedItem>>,
) {
    let mut visibility = using_item_query.single_mut();
    visibility.is_visible = anim.0;
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

    if let Some(item_stack) = selected_item.0 {
        let position = get_animation_points(item_stack.item)[index.0];

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
        let item_type = selected_item.unwrap().item;
        let direction_f = f32::from(*direction);

        let position = get_animation_points(item_type)[index.0];

        if index.0 == 0 && index.is_changed() {
            transform.rotation = get_rotation_by_direction(*direction);
        }

        transform.rotate_around(
            position.extend(0.15),
            Quat::from_rotation_z(ROTATION_STEP * direction_f * time.delta_seconds()),
        );
    }
}

pub fn update_use_item_animation_index(
    time: Res<Time>,
    mut index: ResMut<UseItemAnimationIndex>,
    mut timer: ResMut<UseItemAnimationTimer>,
    mut anim: ResMut<UseItemAnimation>,
) {
    if timer.tick(time.delta()).just_finished() {
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
    diagnostics: Res<Diagnostics>,
    velocity: Res<PlayerVelocity>
) {
    let diagnostic = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS).unwrap();

    if let Some(fps) = diagnostic.value() {
        let velocity_x = velocity.x.abs();
        let velocity_y = velocity.y.abs();

        let factor = (fps * 3600.) / 42240.;

        if velocity_x > 0. {
            println!("Horizontal speed: {:.1} mph", velocity.x.abs() as f64 * factor);
        }

        if velocity_y > 0. {
            println!("Vertical speed: {:.1} mph", velocity.y.abs() as f64 * factor);
        }
    }
}

// TODO: Debug function, remove in feature
#[cfg(feature = "debug")]
pub fn set_sprite_index(
    input: Res<Input<KeyCode>>,
    mut query: Query<(&mut TextureAtlasSprite, &WalkingAnimationData), With<PlayerBodySprite>>,
) {
    query.for_each_mut(|(mut sprite, animation_data)| {
        let anim_offset = animation_data.offset;

        let mut new_sprite_index = sprite.index;

        if input.just_pressed(KeyCode::J) {
            new_sprite_index = sprite.index.checked_sub(1).unwrap_or(0);
        }

        if input.just_pressed(KeyCode::L) {
            new_sprite_index = sprite.index + 1;
        }

        new_sprite_index = new_sprite_index.checked_sub(anim_offset).unwrap_or(0);

        sprite.index = anim_offset + (new_sprite_index % WALKING_ANIMATION_MAX_INDEX);
    });
}
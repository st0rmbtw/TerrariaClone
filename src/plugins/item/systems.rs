use bevy::{prelude::{Query, With, Res, Commands, Entity, Transform, Changed, Without, ResMut, Local, FixedTime, Vec3, Vec2}, ecs::query::Has};

use crate::{common::rect::FRect, plugins::{item::{GRAVITY, MAX_VERTICAL_SPEED}, world::constants::TILE_SIZE, cursor::components::Hoverable, inventory::Inventory, player::Player, audio::{AudioCommandsExt, SoundType}, entity::components::{EntityRect, Velocity}}, world::WorldData};

use super::{STACK_RANGE, item_hoverable_text, GRAB_RANGE, MAX_HORIZONTAL_SPEED};
use super::components::*;

pub(super) fn gravity(
    mut query: Query<&mut Velocity, (With<DroppedItem>, Without<Following>)>
) {
    for mut velocity in &mut query {
        const DIRECTION: f32 = -1.0;

        velocity.y += GRAVITY * DIRECTION;

        if velocity.y > MAX_VERTICAL_SPEED {
            velocity.y = MAX_VERTICAL_SPEED;
        } else if velocity.y < -MAX_VERTICAL_SPEED {
            velocity.y = -MAX_VERTICAL_SPEED;
        }
    }
}

pub(super) fn air_resistance(
    mut query: Query<&mut Velocity, (With<DroppedItem>, Without<Following>)>
) {
    for mut velocity in &mut query {
        velocity.x *= 0.95;

        if velocity.x < 0.1 && velocity.x > -0.1 {
            velocity.x = 0.;
        }

        velocity.x = velocity.x.clamp(-MAX_HORIZONTAL_SPEED, MAX_HORIZONTAL_SPEED);
    }
}

pub(super) fn rotate_item(
    mut query: Query<(&mut Transform, &Velocity), With<DroppedItem>>
) {
    for (mut transform, velocity) in &mut query {
        let direction = Vec2::new(velocity.x / MAX_HORIZONTAL_SPEED, 1.).normalize();
        transform.look_to(direction.extend(-10.), Vec3::Z);
    }
}

pub(super) fn detect_collisions(
    world_data: Res<WorldData>,
    mut query: Query<(&mut Velocity, &mut EntityRect), (With<DroppedItem>, Without<Following>)>
) {
    for (mut velocity, mut item_rect) in &mut query {
        let pos = item_rect.center();
        let next_position = pos + velocity.0;

        let next_rect = FRect::new_center(next_position.x, next_position.y, item_rect.width(), item_rect.height());

        let left = (item_rect.left() / TILE_SIZE) - 1.;
        let right = (item_rect.right() / TILE_SIZE) + 2.;
        let mut top = (item_rect.top().abs() / TILE_SIZE) - 1.;
        let bottom = (item_rect.bottom().abs() / TILE_SIZE) + 2.;

        top = top.max(0.);

        let left_u32 = left as u32;
        let right_u32 = right as u32;
        let top_u32 = top as u32;
        let bottom_u32 = bottom as u32;

        for x in left_u32..right_u32 {
            for y in top_u32..bottom_u32 {
                if y >= world_data.area.height() || world_data.solid_block_exists((x, y)) {
                    let tile_rect = FRect::new_center(
                        x as f32 * TILE_SIZE + TILE_SIZE / 2.,
                        -(y as f32 * TILE_SIZE + TILE_SIZE / 2.),
                        TILE_SIZE,
                        TILE_SIZE
                    );

                    if next_rect.intersects(&tile_rect) {
                        let delta_x = tile_rect.centerx - item_rect.centerx;
                        let delta_y = tile_rect.centery - item_rect.centery;

                        if delta_x.abs() > delta_y.abs() {
                            if delta_x < 0. {
                                // If the item's left side is more to the left than the tile's right side then move the item right.
                                if item_rect.left() <= tile_rect.right() {
                                    velocity.x = 0.;
                                    item_rect.centerx = tile_rect.right() + item_rect.half_width();
                                }
                            } else {
                                // If the item's right side is more to the right than the tile's left side then move the item left.
                                if item_rect.right() >= tile_rect.left() {
                                    velocity.x = 0.;
                                    item_rect.centerx = tile_rect.left() - item_rect.half_width();
                                }
                            }
                        } else {
                            // Checking for collisions again with an offset to workaround the bug when the item stuck in a wall.
                            if FRect::new(next_rect.left() + 2.0, next_rect.top(), item_rect.width() - 4.0, item_rect.height()).intersects(&tile_rect) {
                                if delta_y > 0. {
                                    // If the item's top side is higher than the tile's bottom side then move the item down.
                                    if item_rect.top() >= tile_rect.bottom() {
                                        velocity.y = 0.;
                                        item_rect.centery = tile_rect.bottom() - item_rect.half_height();
                                    }
                                } else {
                                    // If the item's bottom side is lower than the tile's top side then move the item up
                                    if item_rect.bottom() <= tile_rect.top() {
                                        velocity.y = 0.;
                                        item_rect.centery = tile_rect.top() + item_rect.half_height();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

pub(super) fn stack_items(
    mut commands: Commands,
    mut query: Query<(Entity, &mut EntityRect, &mut Velocity, &mut DroppedItem), Without<Following>>,
    mut stacked: Local<Vec<Entity>>
) {
    stacked.clear();

    let mut combinations = query.iter_combinations_mut();

    while let Some([
        (entity, mut rect, mut velocity, mut item),
        (other_entity, other_rect, other_velocity, mut other_item),
    ]) = combinations.fetch_next() {
        if stacked.contains(&entity) || stacked.contains(&other_entity) { continue; }

        let item_stack = item.item_stack;
        let item_max_stack = item_stack.item.max_stack();
        let other_item_stack = other_item.item_stack;

        if item_stack.item != other_item_stack.item { continue; }
        if item_stack.stack >= item_max_stack { continue; }

        let stack_rect = FRect::new_center(rect.centerx, rect.centery, STACK_RANGE, STACK_RANGE);
        if !stack_rect.intersects(&other_rect) { continue; }

        if item_stack.stack + other_item_stack.stack > item_max_stack {
            let a = item_max_stack - item_stack.stack;
            item.item_stack.stack += a;
            other_item.item_stack.stack -= a;
        } else {
            item.item_stack.stack += other_item_stack.stack;
            other_item.item_stack.stack = 0;
        }

        velocity.0 = (velocity.0 + other_velocity.0) / 2.;
        rect.centerx = (rect.centerx + other_rect.centerx) / 2.;
        rect.centery = (rect.centery + other_rect.centery) / 2.;

        if other_item.item_stack.stack == 0 {
            commands.entity(other_entity).despawn();
        }

        stacked.push(other_entity);
    }
}

pub(super) fn update_item_hoverable_info(
    mut query: Query<(&mut Hoverable, &DroppedItem), Changed<DroppedItem>>
) {
    for (mut hoverable, dropped_item) in &mut query {
        *hoverable = Hoverable::SimpleText(item_hoverable_text(dropped_item.item_stack));
    }
}

pub(super) fn follow_player(
    time: Res<FixedTime>,
    mut commands: Commands,
    mut inventory: ResMut<Inventory>,
    query_player: Query<&EntityRect, With<Player>>,
    mut query_items: Query<(Entity, &EntityRect, &DroppedItem, &mut Velocity, Has<Following>, Option<&mut GrabTimer>)>
) {
    let player_rect = query_player.single();

    for (entity, item_rect, dropped_item, mut velocity, is_following, grab_timer_opt) in &mut query_items {
        if let Some(mut grab_timer) = grab_timer_opt {
            if !grab_timer.tick(time.period).finished() { continue; }
        }

        let player_grab_rect = FRect::new_center(player_rect.centerx, player_rect.centery, GRAB_RANGE, GRAB_RANGE);

        if !player_grab_rect.intersects(item_rect) || !inventory.can_be_added(dropped_item.item_stack) {
            if is_following {
                commands.entity(entity).remove::<Following>();
            }
            continue;
        }

        let distance = (player_rect.center() - item_rect.center()).length();

        if distance <= 16. {
            inventory.add_item_stack(dropped_item.item_stack);
            commands.entity(entity).despawn();
            commands.play_sound(SoundType::ItemGrab);
            continue;
        }

        let direction = (player_rect.center() - item_rect.center()).normalize_or_zero();

        velocity.0 = direction * 4.;

        if !is_following {
            commands.entity(entity).insert(Following);
        }
    }
}
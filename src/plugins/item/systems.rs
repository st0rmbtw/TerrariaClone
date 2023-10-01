use bevy::{prelude::{Query, With, Res, Commands, Entity, Transform, Changed, DetectChangesMut, Without, ResMut, Local, FixedTime}, math::vec2, ecs::query::Has};

use crate::{common::{components::{Velocity, EntityRect}, math::move_towards, rect::FRect}, plugins::{item::{GRAVITY, MAX_VERTICAL_SPEED}, world::constants::TILE_SIZE, cursor::components::Hoverable, inventory::Inventory, player::Player}, world::WorldData};

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
        velocity.x = move_towards(velocity.x, 0., 0.1);

        velocity.x = velocity.x.clamp(-MAX_HORIZONTAL_SPEED, MAX_HORIZONTAL_SPEED);
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
                if y >= world_data.size.height as u32 || world_data.solid_block_exists((x, y)) {
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

pub(super) fn update_item_rect(
    world_data: Res<WorldData>,
    mut query: Query<(&Velocity, &mut EntityRect), With<DroppedItem>>
) {
    for (velocity, mut item_rect) in &mut query {
        let min_x: f32 = item_rect.half_width() - TILE_SIZE / 2.;
        let min_y: f32 = -(world_data.size.height as f32) * TILE_SIZE;

        let max_x = world_data.size.width as f32 * TILE_SIZE - item_rect.half_width() - TILE_SIZE / 2.;
        let max_y: f32 = -item_rect.half_height() - TILE_SIZE / 2.;

        let new_position = (item_rect.center() + velocity.0)
            .floor()
            .clamp(vec2(min_x, min_y), vec2(max_x, max_y));

        item_rect.centerx = new_position.x;
        item_rect.centery = new_position.y;
    }
}

pub(super) fn find_stackable_items(
    mut commands: Commands,
    query: Query<(Entity, &EntityRect, &DroppedItem), Without<Stacking>>,
    mut stacked: Local<Vec<Entity>>
) {
    stacked.clear();

    let mut combinations = query.iter_combinations();

    while let Some([
        (entity, rect, item),
        (other_entity, other_rect, other_item),
    ]) = combinations.fetch_next() {
        if stacked.contains(&entity) { continue; }

        let item_stack = item.item_stack;
        let other_item_stack = other_item.item_stack;

        if item_stack.item != other_item_stack.item { continue; }
        if item_stack.stack + other_item_stack.stack > item_stack.item.max_stack() { continue; }

        let distance = rect.center().distance(other_rect.center());

        if distance > STACK_RANGE {
            commands.entity(other_entity).remove::<Following>();
            commands.entity(other_entity).remove::<Stacking>();
            continue;
        }

        commands.entity(other_entity)
            .insert(Following)
            .insert(Stacking { with: entity });

        stacked.push(other_entity);

        commands.entity(entity).insert(Following);
    }
}

pub(super) fn update_stackable_items(
    mut query_stacked: Query<(&mut Velocity, &EntityRect, &Stacking), With<DroppedItem>>,
    mut query_items: Query<(&mut Velocity, &EntityRect), (With<DroppedItem>, Without<Stacking>)>,
) {
    for (mut velocity, rect, stacked) in &mut query_stacked {
        let Ok((mut other_velocity, other_rect)) = query_items.get_mut(stacked.with) else { continue; };

        velocity.0 = (other_rect.center() - rect.center()).normalize_or_zero() * 2.;
        other_velocity.0 = (rect.center() - other_rect.center()).normalize_or_zero() * 2.;
    }
}

pub(super) fn delete_stacked(
    mut commands: Commands,
    query_stacked: Query<(Entity, &EntityRect, &Stacking, &DroppedItem)>,
    mut query_items: Query<(Entity, &EntityRect, &mut DroppedItem), Without<Stacking>>,
) {
    for (entity, rect, stacked, other_dropped_item) in &query_stacked {
        let Ok((other_entity, other_rect, mut dropped_item)) = query_items.get_mut(stacked.with) else { continue; };

        let distance = rect.center().distance(other_rect.center());

        if distance > 8. { continue; }

        dropped_item.item_stack.stack += other_dropped_item.item_stack.stack;

        commands.entity(entity).despawn();
        commands.entity(other_entity).remove::<Following>();
    }
}

pub(super) fn move_item(
    mut query: Query<(&mut Transform, &EntityRect), With<DroppedItem>>
) {
    for (mut transform, entity_rect) in &mut query {
        transform.set_if_neq(transform.with_translation(entity_rect.center().extend(transform.translation.z)));
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
    mut query_items: Query<(Entity, &EntityRect, &DroppedItem, &mut Velocity, Has<Following>, Option<&mut GrabTimer>), Without<Stacking>>
) {
    let player_rect = query_player.single();

    for (entity, item_rect, dropped_item, mut velocity, is_following, grab_timer_opt) in &mut query_items {
        if let Some(mut grab_timer) = grab_timer_opt {
            if !grab_timer.tick(time.period).finished() { continue; }
        }
        
        let distance = (player_rect.center() - item_rect.center()).length();

        if distance > GRAB_RANGE || !inventory.can_be_added(dropped_item.item_stack) {
            if is_following {
                commands.entity(entity).remove::<Following>();
            }
            continue;
        }

        if distance <= 16. {
            inventory.add_item_stack(dropped_item.item_stack);
            commands.entity(entity).despawn();
            continue;
        }

        velocity.0 = (player_rect.center() - item_rect.center()).normalize_or_zero() * 4.;

        if !is_following {
            commands.entity(entity).insert(Following);
        }
    }
}
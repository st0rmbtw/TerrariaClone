use std::time::Duration;

use bevy::{prelude::{ResMut, EventReader, KeyCode, Input, Res, With, Query, Visibility, Handle, Image, MouseButton, EventWriter, DetectChanges, Local, Transform, Quat, Commands, Vec2}, input::mouse::MouseWheel, sprite::TextureAtlasSprite, time::{Timer, TimerMode}};

#[cfg(feature = "debug")]
use bevy_inspector_egui::bevy_egui::EguiContexts;

use crate::{plugins::{ui::ingame::inventory::SLOT_COUNT_IN_ROW, assets::ItemAssets, cursor::position::CursorPosition, world::{events::{DigBlockEvent, PlaceBlockEvent, SeedEvent, BreakBlockEvent}, constants::TILE_SIZE}, player::{FaceDirection, Player, PlayerSpriteBody}, audio::{SoundType, AudioCommandsExt}, camera::components::MainCamera, item::ItemCommandsExt, entity::components::{EntityRect, Velocity}}, common::{helpers::{self, tile_to_world_pos}, rect::FRect}, items::Item, world::{WorldData, block::BlockType}};

use super::{Inventory, SelectedItem, util::keycode_to_digit, SwingItemCooldown, ItemInHand, UseItemAnimationIndex, PlayerUsingItem, UseItemAnimationData, SwingItemCooldownMax, ITEM_ROTATION, SwingAnimation, ITEM_ANIMATION_POINTS};

pub(super) fn select_inventory_cell(
    mut commands: Commands,
    input: Res<Input<KeyCode>>,
    mut inventory: ResMut<Inventory>,
    #[cfg(feature = "debug")] mut egui: EguiContexts
) {
    #[cfg(feature = "debug")]
    let ctx = egui.ctx_mut();

    #[cfg(feature = "debug")]
    if ctx.wants_keyboard_input() { return; }

    let digit = input
        .get_just_pressed()
        .find_map(keycode_to_digit);

    if digit.is_some_and(|i| inventory.select_item(i)) {
        commands.play_sound(SoundType::MenuTick);
    }
}

pub(super) fn scroll_select_inventory_item(
    mut commands: Commands,
    mut inventory: ResMut<Inventory>, 
    mut mouse_wheel: EventReader<MouseWheel>,
    #[cfg(feature = "debug")] mut egui: EguiContexts
) {
    #[cfg(feature = "debug")]
    let ctx = egui.ctx_mut();

    #[cfg(feature = "debug")]
    if ctx.wants_pointer_input() { return; }

    for event in mouse_wheel.iter() {
        let selected_item_index = inventory.selected_slot as f32;
        let hotbar_length = SLOT_COUNT_IN_ROW as f32;
        let next_index = selected_item_index - event.y.signum();
        let new_index = ((next_index % hotbar_length) + hotbar_length) % hotbar_length;

        inventory.select_item(new_index as usize);

        commands.play_sound(SoundType::MenuTick);
    }
}

pub(super) fn drop_item_stack(
    mut commands: Commands,
    input: Res<Input<KeyCode>>,
    mut inventory: ResMut<Inventory>,
    mut swing_animation: ResMut<SwingAnimation>,
    mut swing_cooldown: ResMut<SwingItemCooldown>,
    mut swing_cooldown_max: ResMut<SwingItemCooldownMax>,
    query_player: Query<(&Velocity, &EntityRect, &FaceDirection), With<Player>>,
    mut query_using_item: Query<&mut Handle<Image>, With<ItemInHand>>,
) {
    if input.just_pressed(KeyCode::T) {
        let Ok((velocity, player_rect, face_direction)) = query_player.get_single() else { return; };

        let selected_slot = inventory.selected_slot();

        if let Some(item_stack) = inventory.drop_item(selected_slot) {
            let mut using_item_image = query_using_item.single_mut();
            swing_animation.0 = true;
            swing_cooldown.0 = 10;
            swing_cooldown_max.0 = 10;
            *using_item_image = Handle::default();

            commands.spawn_dropped_item(
                player_rect.center(),
                Vec2::new(f32::from(face_direction) * 4. + velocity.x, 2.),
                item_stack,
                Some(Timer::new(Duration::from_secs_f32(1.5), TimerMode::Once))
            );
        }
    }
}

pub(super) fn set_selected_item(
    inventory: Res<Inventory>,
    mut selected_item: ResMut<SelectedItem>
) {
    selected_item.0 = inventory.selected_item();
}

pub(super) fn use_item(
    mut inventory: ResMut<Inventory>,
    using_item: Res<PlayerUsingItem>,
    cursor_position: Res<CursorPosition<MainCamera>>,
    world_data: Res<WorldData>,
    #[cfg(feature = "debug")]
    debug_config: Res<crate::plugins::debug::DebugConfiguration>,
    query_player: Query<&EntityRect, With<Player>>,
    mut dig_block_events: EventWriter<DigBlockEvent>,
    mut break_block_events: EventWriter<BreakBlockEvent>,
    mut place_block_events: EventWriter<PlaceBlockEvent>,
    mut seed_events: EventWriter<SeedEvent>,
    mut use_cooldown: Local<u32>,
) {
    let player_rect = query_player.single();

    #[cfg(feature = "debug")]
    let instant_break = debug_config.instant_break;
    #[cfg(not(feature = "debug"))]
    let instant_break = false;

    if *use_cooldown > 0 && !instant_break {
        *use_cooldown -= 1;
        return;
    }

    if **using_item {
        let selected_item_index = inventory.selected_slot();

        if let Some(item_stack) = inventory.selected_item() {
            let tile_pos = helpers::get_tile_pos_from_world_coords(world_data.size, cursor_position.world);

            match item_stack.item {
                Item::Tool(tool) => {
                    *use_cooldown = tool.use_cooldown();
                    if !world_data.get_block(tile_pos).is_some_and(|b| b.check_required_tool(tool)) {
                        return;
                    }
                    
                    // Don't break a block if there is a non solid block above it
                    if tile_pos.y > 0 {
                        if world_data.solid_block_exists(tile_pos) && world_data.get_block((tile_pos.x, tile_pos.y - 1)).is_some_and(|b| !b.is_solid()) {
                            return;
                        }
                    }
                    
                    if instant_break {
                        break_block_events.send(BreakBlockEvent { tile_pos });    
                    } else {
                        dig_block_events.send(DigBlockEvent { tile_pos, tool });
                    }
                },
                Item::Block(item_block) => {
                    if world_data.block_exists(tile_pos) { return; }

                    // Forbid placing a block inside the player 
                    {
                        let Vec2 { x, y } = tile_to_world_pos(tile_pos);
                        let tile_rect = FRect::new_center(x, y, TILE_SIZE, TILE_SIZE);
                        if player_rect.intersects(&tile_rect) { return; }
                    }

                    let block_type = BlockType::from(item_block);

                    place_block_events.send(PlaceBlockEvent { tile_pos, block_type });
                    inventory.consume_item(selected_item_index);
                },
                Item::Seed(seed) => {
                    if !world_data.block_exists_with_type(tile_pos, BlockType::Dirt) { return; }

                    seed_events.send(SeedEvent { tile_pos, seed });
                    inventory.consume_item(selected_item_index);
                }
            }
        }
    }
}

pub(super) fn update_swing_cooldown(
    mut swing_cooldown: ResMut<SwingItemCooldown>
) {
    if **swing_cooldown > 0 {
        **swing_cooldown -= 1;
    }
}

pub(super) fn stop_swing_animation(
    swing_cooldown: Res<SwingItemCooldown>,
    using_item: Res<PlayerUsingItem>,
    mut swing_animation: ResMut<SwingAnimation>,
) {
    if **swing_cooldown == 0 && !using_item.0 {
        **swing_animation = false;
    }
}

pub(super) fn start_swing_animation(
    selected_item: Res<SelectedItem>,
    using_item: Res<PlayerUsingItem>,
    item_assets: Res<ItemAssets>,
    mut swing_animation: ResMut<SwingAnimation>,
    mut swing_cooldown: ResMut<SwingItemCooldown>,
    mut swing_cooldown_max: ResMut<SwingItemCooldownMax>,
    mut query_using_item: Query<&mut Handle<Image>, With<ItemInHand>>,
) {
    if using_item.is_changed() && using_item.0 {
        let Some(selected_item) = **selected_item else { return; };
        
        let mut image = query_using_item.single_mut();
        *image = item_assets.get_by_item(selected_item.item);

        if **swing_cooldown == 0 {
            **swing_cooldown = selected_item.item.swing_cooldown();
            **swing_cooldown_max = selected_item.item.swing_cooldown();
        }

        **swing_animation = true;
    }
}

pub(super) fn reset_swing_animation(mut index: ResMut<UseItemAnimationIndex>) {
    **index = 2;
}

pub(super) fn set_using_item_visibility(visible: bool) -> impl FnMut(Res<SwingAnimation>, Query<&mut Visibility, With<ItemInHand>>) {
    move |swing_animation: Res<SwingAnimation>, mut query_using_item: Query<&mut Visibility, With<ItemInHand>>| {
        if swing_animation.is_changed() && **swing_animation == visible {
            if let Ok(visibility) = query_using_item.get_single_mut() {
                helpers::set_visibility(visibility, visible);
            }
        }
    }
}

pub(super) fn set_using_item_position(
    index: Res<UseItemAnimationIndex>,
    mut query_using_item: Query<&mut Transform, With<ItemInHand>>,
    query_player: Query<&FaceDirection, With<Player>>,
) {
    let mut transform = query_using_item.single_mut();
    let face_direction = query_player.single();
    let direction = f32::from(face_direction);

    let position = ITEM_ANIMATION_POINTS[**index];

    transform.translation.x = position.x * direction;
    transform.translation.y = position.y;
}

pub(super) fn set_using_item_rotation(
    swing_cooldown: Res<SwingItemCooldown>,
    swing_cooldown_max: Res<SwingItemCooldownMax>,
    mut query_using_item: Query<&mut Transform, With<ItemInHand>>,
    query_player: Query<&FaceDirection, With<Player>>,
) {
    let face_direction = query_player.single();
    let mut transform = query_using_item.single_mut();

    let direction = f32::from(face_direction);

    // 0..1
    let rotation = (**swing_cooldown as f32) / (**swing_cooldown_max as f32);
    // -1..1
    let rotation = rotation * 2.0 - 1.;

    transform.rotation = Quat::from_rotation_z(rotation * direction * ITEM_ROTATION + direction * 0.5);
}

pub(super) fn update_use_item_animation_index(
    mut index: ResMut<UseItemAnimationIndex>,
    swing_cooldown: Res<SwingItemCooldown>,
    swing_cooldown_max: Res<SwingItemCooldownMax>,
) {
    if (**swing_cooldown as f32) < (**swing_cooldown_max as f32) * 0.333 {
        **index = 2;
    } else if (**swing_cooldown as f32) < (**swing_cooldown_max as f32) * 0.666 {
        **index = 1;
    } else {
        **index = 0;
    }
}

pub(super) fn update_sprite_index(
    index: Res<UseItemAnimationIndex>,
    mut query: Query<(&mut TextureAtlasSprite, &UseItemAnimationData), With<PlayerSpriteBody>>,
) {
    query.for_each_mut(|(mut sprite, anim_data)| {
        sprite.index = anim_data.0 + **index;
    });
}

pub(super) fn play_swing_sound(
    mut commands: Commands,
    selected_item: Res<SelectedItem>,
    swing_cooldown: Res<SwingItemCooldown>,
    swing_cooldown_max: Res<SwingItemCooldownMax>,
) {
    if **swing_cooldown == **swing_cooldown_max {
        if let Some(Item::Tool(tool)) = selected_item.map(|i| i.item) {
            commands.play_sound(SoundType::PlayerToolSwing(tool));
        }
    }
}

pub(super) fn update_player_using_item(
    input: Res<Input<MouseButton>>,
    selected_item: Res<SelectedItem>,
    mut using_item: ResMut<PlayerUsingItem>,
    #[cfg(feature = "debug")] mut egui: EguiContexts
) {
    #[cfg(feature = "debug")]
    let ctx = egui.ctx_mut();

    #[cfg(feature = "debug")]
    if ctx.is_pointer_over_area() || ctx.wants_pointer_input() { return; }

    let pressed = input.pressed(MouseButton::Left) || input.just_pressed(MouseButton::Left);
    
    **using_item = pressed && selected_item.is_some();
}
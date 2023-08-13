use std::borrow::Cow;

use autodefault::autodefault;
use bevy::{prelude::{ResMut, EventReader, KeyCode, Input, Res, Name, With, Query, Changed, Commands, Entity, Visibility, ChildBuilder, Handle, Image, ImageBundle, BuildChildren, NodeBundle, TextBundle, Color, MouseButton, EventWriter, DetectChanges, Local, Transform, Quat, DetectChangesMut}, input::mouse::MouseWheel, ui::{Style, AlignSelf, UiImage, UiRect, JustifyContent, AlignItems, FocusPolicy, FlexDirection, Val, PositionType, AlignContent, Interaction, BackgroundColor, ZIndex}, text::{Text, TextStyle, TextAlignment}, sprite::TextureAtlasSprite, utils::default};

use crate::{plugins::{ui::ExtraUiVisibility, assets::{ItemAssets, UiAssets, FontAssets}, cursor::{components::Hoverable, resources::CursorPosition}, world::events::{DigBlockEvent, PlaceBlockEvent, SeedEvent, BreakBlockEvent}, player::{FaceDirection, Player, PlayerSpriteBody}, audio::{PlaySoundEvent, SoundType}}, common::{extensions::EntityCommandsExtensions, helpers, IsVisible}, language::LanguageContent, items::{Item, get_animation_points}, world::WorldData};

use super::{Inventory, SelectedItem, SelectedItemNameMarker, InventoryCellItemImage, InventoryCellIndex, InventoryItemAmount, InventoryUi, HotbarCellMarker, INVENTORY_CELL_SIZE_SELECTED, INVENTORY_CELL_SIZE, CELL_COUNT_IN_ROW, INVENTORY_ROWS, HotbarUi, util::keycode_to_digit, SwingItemCooldown, ItemInHand, UseItemAnimationIndex, PlayerUsingItem, UseItemAnimationData, SwingItemCooldownMax, ITEM_ROTATION, SwingAnimation};

#[autodefault]
pub(crate) fn spawn_inventory_ui(
    commands: &mut Commands,
    ui_assets: &UiAssets,
    fonts: &FontAssets,
    language_content: &LanguageContent
) -> Entity {
    commands
        .spawn((
            Name::new("Inventory Container"),
            NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    margin: UiRect {
                        left: Val::Px(20.),
                        top: Val::Px(5.),
                    }
                },
            }
        ))
        .with_children(|children| {
            // region: Selected Item Name

            children
                .spawn((
                    Name::new("Selected Item Name"),
                    SelectedItemNameMarker,
                    TextBundle {
                        style: Style {
                            margin: UiRect {
                                ..UiRect::horizontal(Val::Px(10.))
                            },
                            align_self: AlignSelf::Center,
                        },
                        text: Text::from_section(
                            language_content.ui.items.clone(),
                            TextStyle {
                                font: fonts.andy_bold.clone_weak(),
                                font_size: 20.,
                                color: Color::WHITE,
                            },
                        )
                        .with_alignment(TextAlignment::Center),
                    }
                ));

            // endregion

            // region: Hotbar

            children
                .spawn((
                    Name::new("Hotbar"),
                    HotbarUi,
                    NodeBundle {
                        style: Style {
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                        }
                    }
                ))
                .with_children(|children| {
                    for i in 0..CELL_COUNT_IN_ROW {
                        spawn_inventory_cell(
                            children,
                            format!("Hotbar Cell #{i}"),
                            ui_assets.inventory_background.clone_weak(),
                            true,
                            i,
                            fonts,
                        );
                    }
                });

            // endregion

            // region: Inventory
            children
                .spawn((
                    Name::new("Inventory"),
                    InventoryUi,
                    NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            margin: UiRect {
                                top: Val::Px(2.),
                                ..default()
                            }
                        },
                        visibility: Visibility::Hidden,
                    }
                ))
                .with_children(|children| {
                    for j in 0..INVENTORY_ROWS {
                        children.spawn((
                            Name::new(format!("Inventory Row #{}", j)),
                            NodeBundle {
                                style: Style {
                                    margin: UiRect::vertical(Val::Px(2.)),
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::Center,
                                },
                            }
                        )).with_children(|children| {
                            for i in 0..CELL_COUNT_IN_ROW {
                                let index = ((j * CELL_COUNT_IN_ROW) + i) + CELL_COUNT_IN_ROW;

                                spawn_inventory_cell(
                                    children,
                                    format!("Inventory Cell #{}", index),
                                    ui_assets.inventory_background.clone_weak(),
                                    false,
                                    index,
                                    fonts,
                                );
                            }
                        });
                    }
                });
            // endregion
        })
        .id()
}

fn spawn_inventory_cell(
    children: &mut ChildBuilder<'_, '_, '_>,
    name: impl Into<Cow<'static, str>>,
    cell_background: Handle<Image>,
    hotbar_cell: bool,
    index: usize,
    fonts: &FontAssets,
) {
    children
        .spawn((
            Hoverable::None,
            Name::new(name),
            InventoryCellIndex(index),
            Interaction::default(),
            ImageBundle {
                style: Style {
                    margin: UiRect::horizontal(Val::Px(2.)),
                    width: Val::Px(INVENTORY_CELL_SIZE),
                    height: Val::Px(INVENTORY_CELL_SIZE),
                    align_self: AlignSelf::Center,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    padding: UiRect::all(Val::Px(8.)),
                    ..default()
                },
                image: cell_background.into(),
                background_color: BackgroundColor(Color::rgba(1., 1., 1., 0.8)),
                ..default()
            }
        ))
        .insert_if(HotbarCellMarker, hotbar_cell)
        .with_children(|c| {
            c.spawn((
                InventoryCellIndex(index),
                InventoryCellItemImage::default(),
                ImageBundle {
                    focus_policy: FocusPolicy::Pass,
                    z_index: ZIndex::Global(2),
                    ..default()
                }
            ));

            if hotbar_cell {
                c.spawn(NodeBundle {
                    style: Style {
                        padding: UiRect::all(Val::Px(5.)),
                        width: Val::Percent(100.),
                        height: Val::Percent(100.),
                        position_type: PositionType::Absolute,
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::FlexStart,
                        align_content: AlignContent::FlexStart,
                        ..default()
                    },
                    focus_policy: FocusPolicy::Pass,
                    z_index: ZIndex::Global(3),
                    ..default()
                }).with_children(|c| {
                    // Hotbar cell index
                    c.spawn(TextBundle {
                        style: Style {
                            align_self: AlignSelf::FlexEnd,
                            ..default()
                        },
                        focus_policy: FocusPolicy::Pass,
                        text: Text::from_section(
                            ((index + 1) % CELL_COUNT_IN_ROW).to_string(),
                            TextStyle {
                                font: fonts.andy_bold.clone_weak(),
                                font_size: 16.,
                                color: Color::WHITE,
                            },
                        ),
                        z_index: ZIndex::Global(4),
                        ..default()
                    });

                    // Item stack
                    c.spawn((
                        InventoryCellIndex(index),
                        InventoryItemAmount::default(),
                        TextBundle {
                            style: Style {
                                align_self: AlignSelf::Center,
                                ..default()
                            },
                            focus_policy: FocusPolicy::Pass,
                            text: Text::from_section(
                                "",
                                TextStyle {
                                    font: fonts.andy_regular.clone_weak(),
                                    font_size: 16.,
                                    color: Color::WHITE,
                                },
                            ),
                            z_index: ZIndex::Global(4),
                            ..default()
                        }
                    ));
                });
            }
        });
}

pub(super) fn update_selected_cell_size(
    inventory: Res<Inventory>,
    visibility: Res<ExtraUiVisibility>,
    mut hotbar_cells: Query<(&InventoryCellIndex, &mut Style), With<HotbarCellMarker>>,
) {
    for (cell_index, mut style) in hotbar_cells.iter_mut() {
        let selected = cell_index.0 == inventory.selected_slot;
        match selected {
            true if !visibility.is_visible() => {
                style.width = Val::Px(INVENTORY_CELL_SIZE_SELECTED);
                style.height = Val::Px(INVENTORY_CELL_SIZE_SELECTED);
            },
            _ => {
                style.width = Val::Px(INVENTORY_CELL_SIZE);
                style.height = Val::Px(INVENTORY_CELL_SIZE);
            },
        };
    }
}

pub(super) fn update_selected_cell_image(
    inventory: Res<Inventory>,
    mut hotbar_cells: Query<(&InventoryCellIndex, &mut UiImage), With<HotbarCellMarker>>,
    ui_assets: Res<UiAssets>,
) {
    for (cell_index, mut image) in hotbar_cells.iter_mut() {
        let selected = cell_index.0 == inventory.selected_slot;
        image.texture = if selected {
            ui_assets.selected_inventory_background.clone_weak()
        } else {
            ui_assets.inventory_background.clone_weak()
        }
    }
}

pub(super) fn select_inventory_cell(
    input: Res<Input<KeyCode>>,
    mut inventory: ResMut<Inventory>, 
    mut play_sound: EventWriter<PlaySoundEvent>
) {
    let digit = input
        .get_just_pressed()
        .find_map(keycode_to_digit);

    if digit.is_some_and(|i| inventory.select_item(i)) {
        play_sound.send(PlaySoundEvent(SoundType::MenuTick));
    }
}

pub(super) fn scroll_select_inventory_item(
    mut inventory: ResMut<Inventory>, 
    mut mouse_wheel: EventReader<MouseWheel>,
    mut play_sound: EventWriter<PlaySoundEvent>
) {
    for event in mouse_wheel.iter() {
        let selected_item_index = inventory.selected_slot as f32;
        let hotbar_length = CELL_COUNT_IN_ROW as f32;
        let next_index = selected_item_index - event.y.signum();
        let new_index = ((next_index % hotbar_length) + hotbar_length) % hotbar_length;

        inventory.select_item(new_index as usize);

        play_sound.send(PlaySoundEvent(SoundType::MenuTick));
    }
}

pub(super) fn set_selected_item(inventory: Res<Inventory>, mut selected_item: ResMut<SelectedItem>) {
    selected_item.0 = inventory.selected_item();
}

pub(super) fn update_hoverable(
    mut hotbar_cells: Query<(&mut Hoverable, &InventoryCellIndex), With<HotbarCellMarker>>,
    inventory: Res<Inventory>,
    language_content: Res<LanguageContent>
) {
    for (mut hoverable, cell_index) in &mut hotbar_cells {
        if let Some(item) = inventory.get_item(cell_index.0) {
            let name = if item.stack > 1 {
                language_content.item_name(item.item)
            } else {
                format!("{} ({})", language_content.item_name(item.item), item.stack)
            };

            *hoverable = Hoverable::SimpleText(name);
        } else {
            *hoverable = Hoverable::None;
        }
    }
}

pub(super) fn update_selected_item_name_alignment(
    mut query_selected_item_name: Query<&mut Style, With<SelectedItemNameMarker>>,
    visibility: Res<ExtraUiVisibility>
) {
    if visibility.is_changed() {
        let mut style = query_selected_item_name.single_mut();
        style.align_self = if visibility.is_visible() {
            AlignSelf::FlexStart
        } else {
            AlignSelf::Center
        };
    }
}

pub(super) fn update_selected_item_name_text(
    mut query_selected_item_name: Query<&mut Text, With<SelectedItemNameMarker>>,
    current_item: Res<SelectedItem>,
    visibility: Res<ExtraUiVisibility>,
    language_content: Res<LanguageContent>
) {
    if current_item.is_changed() || visibility.is_changed() {
        let mut text = query_selected_item_name.single_mut();

        text.sections[0].value = if visibility.is_visible() {
            language_content.ui.inventory.clone()
        } else {
            current_item.0
                .map(|item_stack| language_content.item_name(item_stack.item))
                .unwrap_or(language_content.ui.items.clone())
        }
    }
}

pub(super) fn update_cell(
    inventory: Res<Inventory>,
    mut item_images: Query<(&mut InventoryCellItemImage, &InventoryCellIndex)>,
    item_assets: Res<ItemAssets>,
) {
    for (mut cell_image, cell_index) in &mut item_images {
        cell_image.0 = inventory
            .get_item(cell_index.0)
            .map(|item_stack| item_assets.get_by_item(item_stack.item))
            .unwrap_or_default();
    }
}

pub(super) fn update_cell_image(
    mut query: Query<(&mut UiImage, &InventoryCellItemImage), Changed<InventoryCellItemImage>>,
) {
    for (mut image, item_image) in &mut query {
        image.texture = item_image.0.clone();
    }
}

pub(super) fn update_item_amount(
    inventory: Res<Inventory>,
    mut query: Query<(&mut InventoryItemAmount, &InventoryCellIndex)>,
) {
    for (mut item_stack, cell_index) in &mut query {
        let stack = inventory.items.get(cell_index.0)
            .and_then(|item| *item)
            .map(|item_stack| item_stack.stack)
            .unwrap_or(0);

        item_stack.0 = stack;
    }
}

pub(super) fn update_item_amount_text(
    mut query: Query<(&mut Text, &mut Visibility, &InventoryItemAmount), Changed<InventoryItemAmount>>,
) {
    for (mut text, mut visibility, item_stack) in &mut query {
        helpers::set_visibility(&mut visibility, item_stack.0 > 1);
        if item_stack.0 > 1 {
            text.sections[0].value = item_stack.0.to_string();
        }
    }
}

pub(super) fn use_item(
    using_item: Res<PlayerUsingItem>,
    cursor: Res<CursorPosition>,
    world_data: Res<WorldData>,
    #[cfg(feature = "debug")]
    debug_config: Res<crate::plugins::debug::DebugConfiguration>,
    mut inventory: ResMut<Inventory>,
    mut dig_block_events: EventWriter<DigBlockEvent>,
    mut break_block_events: EventWriter<BreakBlockEvent>,
    mut place_block_events: EventWriter<PlaceBlockEvent>,
    mut seed_events: EventWriter<SeedEvent>,
    mut use_cooldown: Local<u32>,
) {
    #[cfg(feature = "debug")]
    let instant_break = debug_config.instant_break;
    #[cfg(not(feature = "debug"))]
    let instant_break = false;

    if *use_cooldown > 0 && !instant_break {
        *use_cooldown -= 1;
        return;
    }

    if **using_item {
        let selected_item_index = inventory.selected_slot;

        if let Some(item_stack) = inventory.selected_item() {
            let tile_pos = helpers::get_tile_pos_from_world_coords(cursor.world_position);

            match item_stack.item {
                Item::Tool(tool) => {
                    *use_cooldown = tool.use_cooldown();
                    if !world_data.get_block(tile_pos).is_some_and(|b| b.check_required_tool(tool)) {
                        return;
                    }
                    
                    if instant_break {
                        break_block_events.send(BreakBlockEvent { tile_pos });    
                    } else {
                        dig_block_events.send(DigBlockEvent { tile_pos, tool });
                    }
                },
                Item::Block(block) => {
                    if !world_data.block_exists(tile_pos) {
                        place_block_events.send(PlaceBlockEvent { tile_pos, block });
                        inventory.consume_item(selected_item_index);
                    }
                },
                Item::Seed(seed) => {
                    seed_events.send(SeedEvent { tile_pos, seed });
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
    mut using_item: ResMut<PlayerUsingItem>,
    mut swing_animation: ResMut<SwingAnimation>,
) {
    if **swing_cooldown == 0 {
        **swing_animation = false;
        **using_item = false;
    }
}

pub(super) fn set_using_item_image(
    item_assets: Res<ItemAssets>,
    selected_item: Res<SelectedItem>,
    mut query_using_item: Query<&mut Handle<Image>, With<ItemInHand>>,
) {
    let mut image = query_using_item.single_mut();
    if let Some(item_stack) = **selected_item {
        *image = item_assets.get_by_item(item_stack.item);
    }
}

pub(super) fn set_using_item_visibility(visible: bool) -> impl FnMut(Res<SwingAnimation>, Query<&mut Visibility, With<ItemInHand>>) {
    move |swing_animation: Res<SwingAnimation>, mut query_using_item: Query<&mut Visibility, With<ItemInHand>>| {
        if swing_animation.is_changed() && **swing_animation == visible {
            if let Ok(mut visibility) = query_using_item.get_single_mut() {
                helpers::set_visibility(&mut visibility, visible);
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

    let position = get_animation_points()[**index];

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
    selected_item: Res<SelectedItem>,
    swing_cooldown: Res<SwingItemCooldown>,
    swing_cooldown_max: Res<SwingItemCooldownMax>,
    mut play_sound: EventWriter<PlaySoundEvent>
) {
    if **swing_cooldown == **swing_cooldown_max {
        if let Some(Item::Tool(tool)) = selected_item.map(|i| i.item) {
            play_sound.send(PlaySoundEvent(SoundType::PlayerToolSwing(tool)));
        }
    }
}

pub(super) fn update_player_using_item(
    input: Res<Input<MouseButton>>,
    selected_item: Res<SelectedItem>,
    mut using_item: ResMut<PlayerUsingItem>,
    mut swing_animation: ResMut<SwingAnimation>,
    mut swing_cooldown: ResMut<SwingItemCooldown>,
    mut swing_cooldown_max: ResMut<SwingItemCooldownMax>
) {
    **using_item = if input.pressed(MouseButton::Left) || input.just_pressed(MouseButton::Left) {
        if let Some(selected_item) = **selected_item {
            if !**swing_animation {
                **swing_cooldown = selected_item.item.swing_cooldown();
                **swing_cooldown_max = selected_item.item.swing_cooldown();
            }

            **swing_animation = true;

            true
        } else {
            false
        }
    } else {
        false
    }
}

pub(super) fn trigger_inventory_changed(
    mut inventory: ResMut<Inventory>
) {
    inventory.set_changed()
}
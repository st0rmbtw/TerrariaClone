use std::borrow::Cow;

use autodefault::autodefault;
use bevy::{prelude::{ResMut, EventReader, KeyCode, Input, Res, Name, With, Query, Changed, Commands, Entity, Visibility, ChildBuilder, Handle, Image, ImageBundle, BuildChildren, NodeBundle, TextBundle, Color, MouseButton, EventWriter, Audio, DetectChanges, Local, Transform, Quat}, input::mouse::MouseWheel, ui::{Style, AlignSelf, UiImage, UiRect, JustifyContent, AlignItems, FocusPolicy, FlexDirection, Val, Size, PositionType, AlignContent, Interaction, BackgroundColor, ZIndex}, text::{Text, TextStyle, TextAlignment}, sprite::TextureAtlasSprite};
use rand::seq::SliceRandom;

use crate::{plugins::{ui::{ToggleExtraUiEvent, ExtraUiVisibility}, assets::{ItemAssets, UiAssets, FontAssets, SoundAssets}, cursor::{Hoverable, CursorPosition, UpdateHoverableInfoEvent}, world::{DigBlockEvent, PlaceBlockEvent, WorldData, SeedEvent}, player::{FaceDirection, Player, PlayerBodySprite}}, common::{extensions::EntityCommandsExtensions, helpers}, language::LanguageContent, items::{Item, get_animation_points, ItemStack}, DebugConfiguration};

use super::{Inventory, HOTBAR_LENGTH, SelectedItem, SelectedItemNameMarker, InventoryCellItemImage, InventoryCellIndex, InventoryItemAmount, InventoryUi, HotbarCellMarker, INVENTORY_CELL_SIZE_SELECTED, INVENTORY_CELL_SIZE, CELL_COUNT_IN_ROW, INVENTORY_ROWS, HotbarUi, util::keycode_to_digit, SwingItemCooldown, UsedItem, UseItemAnimationIndex, PlayerUsingItem, UseItemAnimationData, SwingItemCooldownMax, ITEM_ROTATION, SwingAnimation};

#[autodefault]
pub(crate) fn spawn_inventory_ui(
    commands: &mut Commands,
    ui_assets: &UiAssets,
    fonts: &FontAssets,
    language_content: &LanguageContent
) -> Entity {
    commands
        .spawn(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                margin: UiRect {
                    left: Val::Px(20.),
                    top: Val::Px(5.),
                }
            },
        })
        .insert(Name::new("Inventory Container"))
        .with_children(|children| {
            // region: Selected Item Name

            children
                .spawn(TextBundle {
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
                })
                .insert(Name::new("Selected Item Name"))
                .insert(SelectedItemNameMarker);

            // endregion

            // region: Hotbar

            children
                .spawn(NodeBundle {
                    style: Style {
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                    }
                })
                .insert(Name::new("Hotbar"))
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
                })
                .insert(HotbarUi::default());

            // endregion

            // region: Inventory
            children
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                    },
                    visibility: Visibility::Hidden,
                })
                .with_children(|children| {
                    for j in 0..INVENTORY_ROWS {
                        children
                            .spawn(NodeBundle {
                                style: Style {
                                    margin: UiRect::vertical(Val::Px(2.)),
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::Center,
                                },
                            })
                            .insert(Name::new(format!("Inventory Row #{}", j)))
                            .with_children(|children| {
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
                })
                .insert(Name::new("Inventory"))
                .insert(InventoryUi::default());
            // endregion
        })
        .id()
}

#[autodefault(except(InventoryCell))]
fn spawn_inventory_cell(
    children: &mut ChildBuilder<'_, '_, '_>,
    name: impl Into<Cow<'static, str>>,
    cell_background: Handle<Image>,
    hotbar_cell: bool,
    index: usize,
    fonts: &FontAssets,
) {
    children
        .spawn(ImageBundle {
            style: Style {
                margin: UiRect::horizontal(Val::Px(2.)),
                size: INVENTORY_CELL_SIZE,
                align_self: AlignSelf::Center,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
            },
            image: cell_background.into(),
            background_color: BackgroundColor(Color::rgba(1., 1., 1., 0.8))
        })
        .with_children(|c| {
            c.spawn(ImageBundle {
                focus_policy: FocusPolicy::Pass,
                style: Style {
                    margin: UiRect::all(Val::Px(8.)),
                },
                z_index: ZIndex::Global(2)
            })
            .insert(InventoryCellIndex(index))
            .insert(InventoryCellItemImage::default());

            if hotbar_cell {
                c.spawn(NodeBundle {
                    style: Style {
                        padding: UiRect::all(Val::Px(5.)),
                        size: Size::new(Val::Percent(100.), Val::Percent(100.)),
                        position_type: PositionType::Absolute,
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::FlexStart,
                        align_content: AlignContent::FlexStart,
                    },
                    focus_policy: FocusPolicy::Pass,
                    z_index: ZIndex::Global(3)
                }).with_children(|c| {
                    // Hotbar cell index
                    c.spawn(TextBundle {
                        style: Style {
                            align_self: AlignSelf::FlexEnd,
                        },
                        focus_policy: FocusPolicy::Pass,
                        text: Text::from_section(
                            ((index + 1) % HOTBAR_LENGTH).to_string(),
                            TextStyle {
                                font: fonts.andy_bold.clone_weak(),
                                font_size: 16.,
                                color: Color::WHITE,
                            },
                        ),
                        z_index: ZIndex::Global(4)
                    });

                    // Item stack
                    c.spawn(TextBundle {
                        style: Style {
                            align_self: AlignSelf::Center,
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
                        z_index: ZIndex::Global(4)
                    })
                    .insert(InventoryCellIndex(index))
                    .insert(InventoryItemAmount::default());
                });
            }
        })
        .insert(Name::new(name))
        .insert(InventoryCellIndex(index))
        .insert_if(HotbarCellMarker, hotbar_cell)
        .insert(Interaction::default());
}

pub(super) fn update_inventory_visibility(
    mut query: Query<&mut Visibility, With<InventoryUi>>,
    mut events: EventReader<ToggleExtraUiEvent>,
) {
    for event in events.iter() {
        for visibility in &mut query {
            helpers::set_visibility(visibility, event.0);
        }
    }
}

pub(super) fn update_selected_cell_size(
    inventory: Res<Inventory>,
    mut hotbar_cells: Query<(&InventoryCellIndex, &mut Style), With<HotbarCellMarker>>,
    visibility: Res<ExtraUiVisibility>,
) {
    if inventory.is_changed() {
        for (cell_index, mut style) in hotbar_cells.iter_mut() {
            let selected = cell_index.0 == inventory.selected_slot;
            style.size = match selected {
                true if !visibility.0 => INVENTORY_CELL_SIZE_SELECTED,
                _ => INVENTORY_CELL_SIZE,
            };
        }
    }
}

pub(super) fn update_selected_cell_image(
    inventory: Res<Inventory>,
    mut hotbar_cells: Query<(&InventoryCellIndex, &mut UiImage), With<HotbarCellMarker>>,
    ui_assets: Res<UiAssets>,
) {
    if inventory.is_changed() {
        for (cell_index, mut image) in hotbar_cells.iter_mut() {
            let selected = cell_index.0 == inventory.selected_slot;
            image.texture = if selected {
                ui_assets.selected_inventory_background.clone_weak()
            } else {
                ui_assets.inventory_background.clone_weak()
            }
        }
    }
}

pub(super) fn select_inventory_cell(
    mut inventory: ResMut<Inventory>, 
    input: Res<Input<KeyCode>>,
    sounds: Res<SoundAssets>,
    audio: Res<Audio>
) {
    let digit = input
        .get_just_pressed()
        .find_map(keycode_to_digit);

    if let Some(index) = digit {
        if inventory.select_item(index) {
            audio.play(sounds.menu_tick.clone_weak());
        }
    }
}

pub(super) fn scroll_select_inventory_item(
    mut inventory: ResMut<Inventory>, 
    mut events: EventReader<MouseWheel>,
    sounds: Res<SoundAssets>,
    audio: Res<Audio>
) {
    for event in events.iter() {
        let selected_item_index = inventory.selected_slot as f32;
        let hotbar_length = HOTBAR_LENGTH as f32;
        let next_index = selected_item_index - event.y.signum();
        let new_index = ((next_index % hotbar_length) + hotbar_length) % hotbar_length;

        inventory.select_item(new_index as usize);

        audio.play(sounds.menu_tick.clone_weak());
    }
}

pub(super) fn set_selected_item(inventory: Res<Inventory>, mut selected_item: ResMut<SelectedItem>) {
    if inventory.is_changed() {
        selected_item.0 = inventory.selected_item();
    }
}

pub(super) fn update_selected_item_name_alignment(
    mut selected_item_name_query: Query<&mut Style, With<SelectedItemNameMarker>>,
    mut events: EventReader<ToggleExtraUiEvent>,
) {
    for event in events.iter() {
        let mut style = selected_item_name_query.single_mut();
        style.align_self = if event.0 {
            AlignSelf::FlexStart
        } else {
            AlignSelf::Center
        };
    }
}

pub(super) fn update_selected_item_name_text(
    mut selected_item_name_query: Query<&mut Text, With<SelectedItemNameMarker>>,
    current_item: Res<SelectedItem>,
    extra_ui_visibility: Res<ExtraUiVisibility>,
    language_content: Res<LanguageContent>
) {
    if current_item.is_changed() || extra_ui_visibility.is_changed() {
        let mut text = selected_item_name_query.single_mut();

        text.sections[0].value = if extra_ui_visibility.0 {
            language_content.ui.inventory.clone()
        } else {
            let name = current_item.0
                .map(|item_stack| item_stack.item);

            name
                .map(|item| language_content.name(item))
                .unwrap_or(language_content.ui.items.clone())
        }
    }
}

pub(super) fn update_cell(
    inventory: Res<Inventory>,
    mut item_images: Query<(&mut InventoryCellItemImage, &InventoryCellIndex)>,
    item_assets: Res<ItemAssets>,
) {
    if inventory.is_changed() {
        for (mut cell_image, cell_index) in &mut item_images {
            cell_image.0 = inventory
                .get_item(cell_index.0)
                .map(|item_stack| item_assets.get_by_item(item_stack.item))
                .unwrap_or_default();
        }
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
    if inventory.is_changed() {
        for (mut item_stack, cell_index) in &mut query {
            let stack = inventory.items.get(cell_index.0)
                .and_then(|item| *item)
                .map(|item_stack| item_stack.stack)
                .unwrap_or(0);

            item_stack.0 = stack;
        }
    }
}

pub(super) fn update_item_amount_text(
    mut query: Query<(&mut Text, &mut Visibility, &InventoryItemAmount), Changed<InventoryItemAmount>>,
) {
    for (mut text, mut visibility, item_stack) in &mut query {
        if item_stack.0 > 1 {
            text.sections[0].value = item_stack.0.to_string();  
            *visibility = Visibility::Inherited;
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}

pub(super) fn inventory_cell_background_hover(
    query: Query<(&Interaction, &InventoryCellIndex), Changed<Interaction>>,
    inventory: Res<Inventory>,
    language_content: Res<LanguageContent>,
    mut update_hoverable_info_events: EventWriter<UpdateHoverableInfoEvent>,
) {
    for (interaction, cell_index) in &query {
        if let Some(item_stack) = inventory.get_item(cell_index.0) {
            match interaction {
                Interaction::None => {
                    update_hoverable_info_events.send(UpdateHoverableInfoEvent(Hoverable::None));
                },
                _ => {
                    let mut name = language_content.name(item_stack.item).to_owned();
                    
                    if item_stack.stack > 1 {
                        name.push_str(&format!(" ({})", item_stack.stack));
                    }

                    update_hoverable_info_events.send(UpdateHoverableInfoEvent(
                        Hoverable::SimpleText(name.to_string())
                    ));
                }
            }
        }
    }
}

pub(super) fn use_item(
    using_item: Res<PlayerUsingItem>,
    cursor: Res<CursorPosition>,
    debug_config: Res<DebugConfiguration>,
    mut inventory: ResMut<Inventory>,
    mut dig_block_events: EventWriter<DigBlockEvent>,
    mut place_block_events: EventWriter<PlaceBlockEvent>,
    mut seed_events: EventWriter<SeedEvent>,
    mut use_cooldown: Local<u32>,
    world_data: Res<WorldData>
) {
    if *use_cooldown > 0 && !debug_config.instant_break {
        *use_cooldown -= 1;
        return;
    }

    if **using_item {
        let selected_item_index = inventory.selected_slot;

        if let Some(item_stack) = inventory.selected_item() {
            let tile_pos = helpers::get_tile_coords(cursor.world_position);

            match item_stack.item {
                Item::Tool(tool) => {
                    if world_data.block_exists(tile_pos) {
                        dig_block_events.send(DigBlockEvent { tile_pos, tool });
                    }
                    *use_cooldown = tool.use_cooldown();
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
    mut using_item_query: Query<&mut Handle<Image>, With<UsedItem>>,
) {
    if selected_item.is_changed() {
        let mut image = using_item_query.single_mut();
        if let Some(item_stack) = **selected_item {
            *image = item_assets.get_by_item(item_stack.item);
        }
    }
}

pub(super) fn set_using_item_visibility(visible: bool) -> impl FnMut(Res<SwingAnimation>, Query<&mut Visibility, With<UsedItem>>) {
    move |swing_animation: Res<SwingAnimation>, mut using_item_query: Query<&mut Visibility, With<UsedItem>>| {
        if swing_animation.is_changed() && **swing_animation == visible {
            if let Ok(visibility) = using_item_query.get_single_mut() {
                helpers::set_visibility(visibility, visible);
            }
        }
    }
}

pub(super) fn set_using_item_position(
    index: Res<UseItemAnimationIndex>,
    mut query_using_item: Query<&mut Transform, With<UsedItem>>,
    query_player: Query<&FaceDirection, With<Player>>,
) {
    let mut transform = query_using_item.single_mut();
    let direction = query_player.single();

    let position = get_animation_points()[**index];

    transform.translation.x = position.x * f32::from(*direction);
    transform.translation.y = position.y;
}

pub(super) fn set_using_item_rotation(
    swing_cooldown: Res<SwingItemCooldown>,
    swing_cooldown_max: Res<SwingItemCooldownMax>,
    mut query_using_item: Query<&mut Transform, With<UsedItem>>,
    query_player: Query<&FaceDirection, With<Player>>,
) {
    let direction = query_player.single();
    let mut transform = query_using_item.single_mut();

    let direction_f = f32::from(*direction);

    // 0..1
    let rotation = (**swing_cooldown as f32) / (**swing_cooldown_max as f32);
    // -1..1
    let rotation = rotation * 2.0 - 1.;

    let rotation = Quat::from_rotation_z(rotation * direction_f * ITEM_ROTATION + direction_f * 0.3);

    transform.rotation = rotation;
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
    mut query: Query<(&mut TextureAtlasSprite, &UseItemAnimationData), With<PlayerBodySprite>>,
) {
    query.for_each_mut(|(mut sprite, anim_data)| {
        sprite.index = anim_data.0 + **index;
    });
}

pub(super) fn play_swing_sound(
    selected_item: Res<SelectedItem>,
    sound_assets: Res<SoundAssets>,
    audio: Res<Audio>,
    swing_cooldown: Res<SwingItemCooldown>,
    swing_cooldown_max: Res<SwingItemCooldownMax>
) {
    if **swing_cooldown == **swing_cooldown_max {
        if let Some(ItemStack { item: Item::Tool(_), .. }) = **selected_item {
            let sound = sound_assets.swing.choose(&mut rand::thread_rng()).unwrap();
            audio.play(sound.clone_weak());
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
        } else{
            false
        }
    } else {
        false
    }
}
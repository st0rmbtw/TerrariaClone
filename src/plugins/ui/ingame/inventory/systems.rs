use autodefault::autodefault;
use bevy::{prelude::{Commands, Name, NodeBundle, BuildChildren, TextBundle, Color, Entity, ImageBundle, default, ChildBuilder, Handle, Image, Visibility, With, Res, Query, DetectChanges, Changed, ResMut, DetectChangesMut, Without}, ui::{Style, FlexDirection, UiRect, Val, AlignSelf, AlignItems, JustifyContent, Interaction, BackgroundColor, ZIndex, FocusPolicy, AlignContent, PositionType, UiImage, widget::UiImageSize}, text::{Text, TextStyle, TextAlignment}};

use crate::{plugins::{assets::{UiAssets, FontAssets, ItemAssets}, cursor::components::Hoverable, inventory::{Inventory, SelectedItem, Slot}, ui::{InventoryUiVisibility, components::PreviousInteraction}, audio::{AudioCommandsExt, SoundType}}, language::{keys::{LanguageStringKey, UIStringKey, ItemStringKey}, LocalizedText, args}, common::{extensions::EntityCommandsExtensions, BoolValue, helpers}};

use super::{components::*, INVENTORY_ROWS, SLOT_COUNT_IN_ROW, HOTBAR_SLOT_SIZE, INVENTORY_SLOT_SIZE, HOTBAR_SLOT_SIZE_SELECTED};

#[autodefault]
pub(crate) fn spawn_inventory_ui(
    commands: &mut Commands,
    ui_assets: &UiAssets,
    fonts: &FontAssets,
) -> Entity {
    commands
        .spawn((
            Name::new("InventoryContainer"),
            InventoryUiContainer,
            NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::FlexStart,
                },
            }
        ))
        .with_children(|children| {
            children
                .spawn((
                    Name::new("SelectedItemName"),
                    SelectedItemName,
                    TextBundle {
                        style: Style {
                            margin: UiRect {
                                ..UiRect::horizontal(Val::Px(10.))
                            },
                            align_self: AlignSelf::Center,
                        },
                        text: Text::from_section(
                            String::new(),
                            TextStyle {
                                font: fonts.andy_bold.clone_weak(),
                                font_size: 24.,
                                color: Color::WHITE,
                            },
                        )
                        .with_alignment(TextAlignment::Center),
                    },
                    LocalizedText::from(UIStringKey::Items),
                ));

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
                    for i in 0..SLOT_COUNT_IN_ROW {
                        spawn_inventory_slot(
                            children,
                            ui_assets.inventory_background.clone_weak(),
                            true,
                            i,
                            fonts,
                        );
                    }
                });

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
                    // Starting from 1 because hotbar takes the first row
                    for j in 1..=INVENTORY_ROWS {
                        children.spawn((
                            Name::new(format!("Row #{}", j)),
                            NodeBundle {
                                style: Style {
                                    margin: UiRect::vertical(Val::Px(2.)),
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::Center,
                                },
                            }
                        )).with_children(|children| {
                            for i in 0..SLOT_COUNT_IN_ROW {
                                let index = (j * SLOT_COUNT_IN_ROW) + i;

                                spawn_inventory_slot(
                                    children,
                                    ui_assets.inventory_background.clone_weak(),
                                    false,
                                    index,
                                    fonts,
                                );
                            }
                        });
                    }
                });
        })
        .id()
}

fn spawn_inventory_slot(
    children: &mut ChildBuilder<'_, '_, '_>,
    slot_background: Handle<Image>,
    hotbar_slot: bool,
    index: usize,
    fonts: &FontAssets,
) {
    let size = if hotbar_slot { HOTBAR_SLOT_SIZE } else { INVENTORY_SLOT_SIZE };

    children
        .spawn((
            Hoverable::None,
            Name::new(format!("Slot #{}", index)),
            SlotIndex(index),
            Interaction::default(),
            PreviousInteraction::default(),
            InventorySlot,
            ImageBundle {
                style: Style {
                    margin: UiRect::horizontal(Val::Px(2.)),
                    width: Val::Px(size),
                    height: Val::Px(size),
                    align_self: AlignSelf::Center,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    padding: UiRect::all(Val::Px(8.)),
                    ..default()
                },
                image: slot_background.into(),
                background_color: BackgroundColor(Color::WHITE.with_a(0.8)),
                ..default()
            }
        ))
        .insert_if(HotbarSlot, hotbar_slot)
        .with_children(|c| {
            c.spawn((
                SlotIndex(index),
                SlotItemImage::default(),
                ImageBundle {
                    focus_policy: FocusPolicy::Pass,
                    z_index: ZIndex::Global(2),
                    ..default()
                }
            ));
            
            c.spawn(NodeBundle {
                style: Style {
                    padding: UiRect::axes(Val::Px(5.), Val::Px(2.5)),
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    position_type: PositionType::Absolute,
                    flex_direction: FlexDirection::Column,
                    justify_content: if hotbar_slot { JustifyContent::SpaceBetween } else { JustifyContent::End },
                    align_items: AlignItems::FlexStart,
                    align_content: AlignContent::FlexStart,
                    ..default()
                },
                focus_policy: FocusPolicy::Pass,
                z_index: ZIndex::Global(3),
                ..default()
            }).with_children(|c| {
                if hotbar_slot {
                    // Hotbar slot index
                    c.spawn((
                        SlotIndex(index),
                        HotbarSlotIndex,
                        TextBundle {
                            style: Style {
                                align_self: AlignSelf::FlexStart,
                                ..default()
                            },
                            focus_policy: FocusPolicy::Pass,
                            text: Text::from_section(
                                ((index + 1) % SLOT_COUNT_IN_ROW).to_string(),
                                TextStyle {
                                    font: fonts.andy_bold.clone_weak(),
                                    font_size: 16.,
                                    color: Color::rgb(0.9, 0.9, 0.9),
                                },
                            ),
                            z_index: ZIndex::Global(4),
                            ..default()
                        }
                    ));
                }

                // Item stack
                c.spawn((
                    SlotIndex(index),
                    ItemAmount::default(),
                    TextBundle {
                        style: Style {
                            align_self: AlignSelf::Center,
                            ..default()
                        },
                        focus_policy: FocusPolicy::Pass,
                        text: Text::from_section(
                            String::new(),
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
        });
}

pub(super) fn update_slot_size(
    inventory: Res<Inventory>,
    visibility: Res<InventoryUiVisibility>,
    mut hotbar_slots: Query<(&SlotIndex, &mut Style), (With<HotbarSlot>, Without<SlotItemImage>)>,
    mut item_images: Query<(&SlotIndex, &UiImageSize, &mut Style), (Without<HotbarSlot>, With<SlotItemImage>)>
) {
    for (slot_index, mut style) in &mut hotbar_slots {
        let selected = slot_index.0 == inventory.selected_slot;
        if visibility.value() {
            style.width = Val::Px(INVENTORY_SLOT_SIZE);
            style.height = Val::Px(INVENTORY_SLOT_SIZE);
        } else if selected {
            style.width = Val::Px(HOTBAR_SLOT_SIZE_SELECTED);
            style.height = Val::Px(HOTBAR_SLOT_SIZE_SELECTED);
        } else {
            style.width = Val::Px(HOTBAR_SLOT_SIZE);
            style.height = Val::Px(HOTBAR_SLOT_SIZE);
        }
    }

    for (slot_index, image_size, mut style) in &mut item_images {
        let selected = slot_index.0 == inventory.selected_slot;
        let image_size = image_size.size();

        if visibility.value() {
            style.width = Val::Px(image_size.x * 0.95);
            style.height = Val::Px(image_size.y * 0.95);
        } else if selected {
            style.width = Val::Px(image_size.x);
            style.height = Val::Px(image_size.y);
        } else {
            style.width = Val::Px(image_size.x * 0.9);
            style.height = Val::Px(image_size.y * 0.9);
        }
    }
}

pub(super) fn update_slot_index_text(
    inventory: Res<Inventory>,
    visibility: Res<InventoryUiVisibility>,
    mut hotbar_slots: Query<(&SlotIndex, &mut Text), With<HotbarSlotIndex>>,
) {
    for (slot_index, mut text) in &mut hotbar_slots {
        let (color, font_size) = if visibility.value() {
            if inventory.selected_slot == slot_index.0 && !inventory.item_exists(Slot::MouseItem) {
                (Color::WHITE, 18.)
            } else {
                (Color::rgb(0.8, 0.8, 0.8), 16.)
            }
        } else {
            (Color::WHITE, 16.)
        };

        text.sections[0].style.color = color;
        text.sections[0].style.font_size = font_size;
    }
}

pub(super) fn update_slot_background_image(
    inventory: Res<Inventory>,
    ui_assets: Res<UiAssets>,
    visibility: Res<InventoryUiVisibility>,
    mut hotbar_slots: Query<(&SlotIndex, &mut UiImage), With<HotbarSlot>>,
) {
    for (slot_index, mut image) in &mut hotbar_slots {
        let selected = slot_index.0 == inventory.selected_slot;
        let texture = if selected && !visibility.value() {
            &ui_assets.selected_inventory_background
        } else {
            &ui_assets.inventory_background
        };

        image.texture = texture.clone_weak();
    }
}

pub(super) fn update_hoverable(
    inventory: Res<Inventory>,
    mut hotbar_slots: Query<(&SlotIndex, &mut Hoverable), With<InventorySlot>>
) {
    for (slot_index, mut hoverable) in &mut hotbar_slots {
        if let Some(item) = inventory.get_item(Slot::Index(slot_index.0)) {
            let item_key = LanguageStringKey::Items(ItemStringKey::get_by_item(item.item));

            let name = if item.stack > 1 {
                LocalizedText::new(item_key, "{} ({})", args![item.stack])
            } else {
                LocalizedText::from(item_key)
            };

            *hoverable = Hoverable::SimpleText(name);
        } else {
            *hoverable = Hoverable::None;
        }
    }
}

pub(super) fn update_selected_item_name_alignment(
    visibility: Res<InventoryUiVisibility>,
    mut query_selected_item_name: Query<&mut Style, With<SelectedItemName>>
) {
    if visibility.is_changed() {
        let mut style = query_selected_item_name.single_mut();
        style.align_self = if visibility.value() {
            AlignSelf::FlexStart
        } else {
            AlignSelf::Center
        };
    }
}

pub(super) fn update_selected_item_name_text(
    current_item: Res<SelectedItem>,
    visibility: Res<InventoryUiVisibility>,
    mut query_selected_item_name: Query<&mut LocalizedText, With<SelectedItemName>>
) {
    if !current_item.is_changed() && !visibility.is_changed() { return; } 

    let mut localized_text = query_selected_item_name.single_mut();

    localized_text.key = if visibility.value() {
        UIStringKey::Inventory.into()
    } else {
        current_item.0.as_ref()
            .map(|item_stack| item_stack.item)
            .map(ItemStringKey::get_by_item)
            .map(LanguageStringKey::Items)
            .unwrap_or(UIStringKey::Items.into())
    }
}

pub(super) fn update_slot_item_image(
    inventory: Res<Inventory>,
    item_assets: Res<ItemAssets>,
    mut item_images: Query<(&SlotIndex, &mut SlotItemImage)>,
) {
    for (slot_index, mut slot_image) in &mut item_images {
        let image = inventory
            .get_item(Slot::Index(slot_index.0))
            .map(|item_stack| item_assets.get_by_item(item_stack.item))
            .unwrap_or_default();

        slot_image.set_if_neq(SlotItemImage(image));
    }
}

pub(super) fn update_slot_image(
    mut inventory: ResMut<Inventory>,
    mut query: Query<(&mut UiImage, &SlotItemImage), Changed<SlotItemImage>>,
) {
    for (mut image, item_image) in &mut query {
        image.texture = item_image.0.clone_weak();
    }

    // For some reason ui image updates only after a second time
    if !query.is_empty() {
        inventory.set_changed();
    }
}

pub(super) fn update_item_amount(
    inventory: Res<Inventory>,
    mut query: Query<(&SlotIndex, &mut ItemAmount)>,
) {
    for (slot_index, mut item_stack) in &mut query {
        let stack = inventory.slots.get(slot_index.0)
            .and_then(|item| *item)
            .map(|item_stack| item_stack.stack)
            .unwrap_or(0);

        item_stack.0 = stack;
    }
}

pub(super) fn update_item_amount_text(
    mut query: Query<(&mut Text, &mut Visibility, &ItemAmount), Changed<ItemAmount>>,
) {
    for (mut text, visibility, item_stack) in &mut query {
        helpers::set_visibility(visibility, item_stack.0 > 1);
        if item_stack.0 > 1 {
            text.sections[0].value = item_stack.0.to_string();
        }
    }
}

pub(super) fn take_item(
    mut commands: Commands,
    mut inventory: ResMut<Inventory>,
    query_slot: Query<(&Interaction, &PreviousInteraction, &SlotIndex)>
) {
    if inventory.is_changed() { return; }

    if !inventory.item_exists(Slot::MouseItem) {
        for (interaction, previous_interaction, index) in &query_slot {
            if *interaction == Interaction::Pressed && previous_interaction.0 != Interaction::Pressed {
                if let Some(item_stack) = inventory.remove_item(Slot::Index(index.0)) {
                    inventory.set_item(Slot::MouseItem, item_stack);
                    commands.play_sound(SoundType::ItemGrab);
                }
            }
        }
    }
}

pub(super) fn put_item(
    mut commands: Commands,
    mut inventory: ResMut<Inventory>,
    query_slot: Query<(&Interaction, &PreviousInteraction, &SlotIndex)>
) {
    if inventory.is_changed() { return; }

    if let Some(mouse_item) = inventory.get_item(Slot::MouseItem) {
        for (interaction, previous_interaction, index) in &query_slot {
            if *interaction == Interaction::Pressed && previous_interaction.0 != Interaction::Pressed {
                if let Some(item) = inventory.get_item(Slot::Index(index.0)) {
                    inventory.set_item(Slot::MouseItem, item);
                } else {
                    inventory.remove_item(Slot::MouseItem);
                }
                
                inventory.set_item(Slot::Index(index.0), mouse_item);
                commands.play_sound(SoundType::ItemGrab);
            }
        }
    }
}

pub(super) fn trigger_inventory_changed(mut inventory: ResMut<Inventory>) {
    inventory.set_changed()
}
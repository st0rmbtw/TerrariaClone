use autodefault::autodefault;
use bevy::{prelude::{Commands, Name, NodeBundle, BuildChildren, TextBundle, Color, Entity, ImageBundle, default, ChildBuilder, Handle, Image, Visibility, With, Res, Query, DetectChanges, Changed, ResMut, DetectChangesMut, Without}, ui::{Style, FlexDirection, UiRect, Val, AlignSelf, AlignItems, JustifyContent, Interaction, BackgroundColor, ZIndex, FocusPolicy, AlignContent, PositionType, UiImage, widget::UiImageSize}, text::{Text, TextStyle, TextAlignment}};

use crate::{plugins::{assets::{UiAssets, FontAssets, ItemAssets}, cursor::components::Hoverable, inventory::{Inventory, SelectedItem}, ui::InventoryUiVisibility}, language::LanguageContent, common::{extensions::EntityCommandsExtensions, BoolValue, helpers}};

use super::{components::*, INVENTORY_ROWS, CELL_COUNT_IN_ROW, HOTBAR_CELL_SIZE, INVENTORY_CELL_SIZE, HOTBAR_CELL_SIZE_SELECTED};

#[autodefault]
pub(crate) fn spawn_inventory_ui(
    commands: &mut Commands,
    ui_assets: &UiAssets,
    fonts: &FontAssets,
    language_content: &LanguageContent
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
                            &language_content.ui.items,
                            TextStyle {
                                font: fonts.andy_bold.clone_weak(),
                                font_size: 24.,
                                color: Color::WHITE,
                            },
                        )
                        .with_alignment(TextAlignment::Center),
                    }
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
                    for i in 0..CELL_COUNT_IN_ROW {
                        spawn_inventory_cell(
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
                            for i in 0..CELL_COUNT_IN_ROW {
                                let index = (j * CELL_COUNT_IN_ROW) + i;

                                spawn_inventory_cell(
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

fn spawn_inventory_cell(
    children: &mut ChildBuilder<'_, '_, '_>,
    cell_background: Handle<Image>,
    hotbar_cell: bool,
    index: usize,
    fonts: &FontAssets,
) {
    let width = if hotbar_cell { HOTBAR_CELL_SIZE } else { INVENTORY_CELL_SIZE };
    let height = if hotbar_cell { HOTBAR_CELL_SIZE } else { INVENTORY_CELL_SIZE };

    children
        .spawn((
            Hoverable::None,
            Name::new(format!("Cell #{}", index)),
            CellIndex(index),
            Interaction::default(),
            ImageBundle {
                style: Style {
                    margin: UiRect::horizontal(Val::Px(2.)),
                    width: Val::Px(width),
                    height: Val::Px(height),
                    align_self: AlignSelf::Center,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    padding: UiRect::all(Val::Px(8.)),
                    ..default()
                },
                image: cell_background.into(),
                background_color: BackgroundColor(Color::WHITE.with_a(0.8)),
                ..default()
            }
        ))
        .insert_if(HotbarCell, hotbar_cell)
        .with_children(|c| {
            c.spawn((
                CellIndex(index),
                CellItemImage::default(),
                ImageBundle {
                    focus_policy: FocusPolicy::Pass,
                    z_index: ZIndex::Global(2),
                    ..default()
                }
            ));

            if hotbar_cell {
                c.spawn(NodeBundle {
                    style: Style {
                        padding: UiRect::axes(Val::Px(5.), Val::Px(2.5)),
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
                    c.spawn((
                        CellIndex(index),
                        HotbarCellIndex,
                        TextBundle {
                            style: Style {
                                align_self: AlignSelf::FlexStart,
                                ..default()
                            },
                            focus_policy: FocusPolicy::Pass,
                            text: Text::from_section(
                                ((index + 1) % CELL_COUNT_IN_ROW).to_string(),
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

                    // Item stack
                    c.spawn((
                        CellIndex(index),
                        ItemAmount::default(),
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

pub(super) fn update_cell_size(
    inventory: Res<Inventory>,
    visibility: Res<InventoryUiVisibility>,
    mut hotbar_cells: Query<(&CellIndex, &mut Style), (With<HotbarCell>, Without<CellItemImage>)>,
    mut item_images: Query<(&CellIndex, &UiImageSize, &mut Style), (Without<HotbarCell>, With<CellItemImage>)>
) {
    for (cell_index, mut style) in &mut hotbar_cells {
        let selected = cell_index.0 == inventory.selected_slot;
        if visibility.value() {
            style.width = Val::Px(INVENTORY_CELL_SIZE);
            style.height = Val::Px(INVENTORY_CELL_SIZE);
        } else if selected {
            style.width = Val::Px(HOTBAR_CELL_SIZE_SELECTED);
            style.height = Val::Px(HOTBAR_CELL_SIZE_SELECTED);
        } else {
            style.width = Val::Px(HOTBAR_CELL_SIZE);
            style.height = Val::Px(HOTBAR_CELL_SIZE);
        }
    }

    for (cell_index, image_size, mut style) in &mut item_images {
        let selected = cell_index.0 == inventory.selected_slot;
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

pub(super) fn update_cell_index_text(
    inventory: Res<Inventory>,
    visibility: Res<InventoryUiVisibility>,
    mut hotbar_cells: Query<(&CellIndex, &mut Text), With<HotbarCellIndex>>,
) {
    for (cell_index, mut text) in &mut hotbar_cells {
        let (color, font_size) = if visibility.value() {
            if inventory.selected_slot == cell_index.0 {
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

pub(super) fn update_cell_background_image(
    inventory: Res<Inventory>,
    ui_assets: Res<UiAssets>,
    visibility: Res<InventoryUiVisibility>,
    mut hotbar_cells: Query<(&CellIndex, &mut UiImage), With<HotbarCell>>,
) {
    for (cell_index, mut image) in &mut hotbar_cells {
        let selected = cell_index.0 == inventory.selected_slot;
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
    language_content: Res<LanguageContent>,
    mut hotbar_cells: Query<(&CellIndex, &mut Hoverable), With<HotbarCell>>
) {
    for (cell_index, mut hoverable) in &mut hotbar_cells {
        if let Some(item) = inventory.get_item(cell_index.0) {
            let name: String = if item.stack > 1 {
                format!("{} ({})", language_content.item_name(item.item), item.stack)
            } else {
                language_content.item_name(item.item).clone()
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
    language_content: Res<LanguageContent>,
    mut query_selected_item_name: Query<&mut Text, With<SelectedItemName>>
) {
    if current_item.is_changed() || visibility.is_changed() {
        let mut text = query_selected_item_name.single_mut();

        text.sections[0].value = if visibility.value() {
            language_content.ui.inventory.clone()
        } else {
            current_item.0
                .map(|item_stack| language_content.item_name(item_stack.item))
                .unwrap_or(&language_content.ui.items)
                .clone()
        }
    }
}

pub(super) fn update_cell(
    inventory: Res<Inventory>,
    item_assets: Res<ItemAssets>,
    mut item_images: Query<(&CellIndex, &mut CellItemImage)>,
) {
    for (cell_index, mut cell_image) in &mut item_images {
        cell_image.0 = inventory
            .get_item(cell_index.0)
            .map(|item_stack| item_assets.get_by_item(item_stack.item))
            .unwrap_or_default();
    }
}

pub(super) fn update_cell_item_image(
    mut query: Query<(&mut UiImage, &CellItemImage), Changed<CellItemImage>>,
) {
    for (mut image, item_image) in &mut query {
        image.texture = item_image.0.clone();
    }
}

pub(super) fn update_item_amount(
    inventory: Res<Inventory>,
    mut query: Query<(&CellIndex, &mut ItemAmount)>,
) {
    for (cell_index, mut item_stack) in &mut query {
        let stack = inventory.items.get(cell_index.0)
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

pub(super) fn trigger_inventory_changed(mut inventory: ResMut<Inventory>) {
    inventory.set_changed()
}
use std::borrow::Cow;

use autodefault::autodefault;
use bevy::{prelude::{Commands, Name, NodeBundle, BuildChildren, TextBundle, Color, Entity, ImageBundle, default, ChildBuilder, Handle, Image, Visibility, With, Res, Query, DetectChanges, Changed, ResMut, DetectChangesMut}, ui::{Style, FlexDirection, UiRect, Val, AlignSelf, AlignItems, JustifyContent, Interaction, BackgroundColor, ZIndex, FocusPolicy, AlignContent, PositionType, UiImage}, text::{Text, TextStyle, TextAlignment}};

use crate::{plugins::{assets::{UiAssets, FontAssets, ItemAssets}, cursor::components::Hoverable, inventory::{Inventory, SelectedItem}, ui::ExtraUiVisibility}, language::LanguageContent, common::{extensions::EntityCommandsExtensions, IsVisible, helpers}};

use super::{components::*, INVENTORY_ROWS, CELL_COUNT_IN_ROW, INVENTORY_CELL_SIZE, INVENTORY_CELL_SIZE_SELECTED};

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
        let texture = if selected { &ui_assets.selected_inventory_background } else { &ui_assets.inventory_background };

        image.texture = texture.clone_weak();
    }
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
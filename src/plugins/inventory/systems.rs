use std::borrow::Cow;

use autodefault::autodefault;
use bevy::{prelude::{ResMut, EventReader, KeyCode, Input, Res, Name, With, Query, Changed, Commands, Entity, Visibility, ChildBuilder, Handle, Image, ImageBundle, BuildChildren, NodeBundle, TextBundle, Color}, input::mouse::MouseWheel, ui::{Style, AlignSelf, UiImage, UiRect, JustifyContent, AlignItems, FocusPolicy, FlexDirection, Val, Size, PositionType, AlignContent, Interaction}, text::{Text, TextStyle, TextAlignment}};

use crate::{plugins::{ui::{ToggleExtraUiEvent, ExtraUiVisibility}, assets::{ItemAssets, UiAssets, FontAssets}, cursor::HoveredInfo}, TRANSPARENT, util::{EntityCommandsExtensions, RectExtensions}};

use super::{Inventory, HOTBAR_LENGTH, SelectedItem, SelectedItemNameMarker, INVENTORY_STRING, ITEMS_STRING, InventoryCellItemImage, InventoryCellIndex, InventoryItemAmount, InventoryUi, HotbarCellMarker, INVENTORY_CELL_SIZE_SELECTED, INVENTORY_CELL_SIZE, KEYCODE_TO_DIGIT, CELL_COUNT_IN_ROW, INVENTORY_ROWS_COUNT, HotbarUi};

#[autodefault]
pub fn spawn_inventory_ui(
    commands: &mut Commands,
    ui_assets: &UiAssets,
    fonts: &FontAssets,
) -> Entity {
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                align_items: AlignItems::Center,
                align_content: AlignContent::Center,
                flex_direction: FlexDirection::ColumnReverse,
                margin: UiRect {
                    left: Val::Px(20.),
                    top: Val::Px(5.),
                },
            },
            color: TRANSPARENT.into(),
        })
        .insert(Name::new("Inventory Container"))
        .with_children(|children| {
            // region: Selected Item Name

            children
                .spawn_bundle(TextBundle {
                    style: Style {
                        margin: UiRect {
                            ..UiRect::horizontal(10.)
                        },
                        align_self: AlignSelf::Center,
                    },
                    text: Text::from_section(
                        "".to_string(),
                        TextStyle {
                            font: fonts.andy_bold.clone(),
                            font_size: 20.,
                            color: Color::WHITE,
                        },
                    )
                    .with_alignment(TextAlignment::CENTER),
                })
                .insert(Name::new("Selected Item Name"))
                .insert(SelectedItemNameMarker);

            // endregion

            // region: Hotbar

            children
                .spawn_bundle(NodeBundle {
                    style: Style {
                        align_items: AlignItems::Center,
                    },
                    color: TRANSPARENT.into(),
                })
                .insert(Name::new("Hotbar"))
                .with_children(|children| {
                    for i in 0..CELL_COUNT_IN_ROW {
                        spawn_inventory_cell(
                            children,
                            format!("Hotbar Cell #{}", i),
                            ui_assets.inventory_back.clone(),
                            true,
                            i,
                            &fonts,
                        );
                    }
                })
                .insert(HotbarUi::default());

            // endregion

            // region: Inventory
            children
                .spawn_bundle(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::ColumnReverse,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                    },
                    visibility: Visibility { is_visible: false },
                    color: TRANSPARENT.into(),
                })
                .with_children(|children| {
                    for j in 0..INVENTORY_ROWS_COUNT {
                        children
                            .spawn_bundle(NodeBundle {
                                style: Style {
                                    margin: UiRect::vertical(2.),
                                    align_items: AlignItems::Center,
                                    justify_content: JustifyContent::Center,
                                },
                                color: TRANSPARENT.into(),
                            })
                            .insert(Name::new(format!("Inventory Row #{}", j)))
                            .with_children(|children| {
                                for i in 0..CELL_COUNT_IN_ROW {
                                    // +CELL_COUNT_IN_ROW because hotbar takes first CELL_COUNT_IN_ROW cells
                                    let index = ((j * CELL_COUNT_IN_ROW) + i) + CELL_COUNT_IN_ROW;

                                    spawn_inventory_cell(
                                        children,
                                        format!("Inventory Cell #{}", index),
                                        ui_assets.inventory_back.clone(),
                                        false,
                                        index,
                                        &fonts,
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

pub fn update_inventory_visibility(
    mut query: Query<&mut Visibility, With<InventoryUi>>,
    mut events: EventReader<ToggleExtraUiEvent>,
) {
    for event in events.iter() {
        for mut visibility in &mut query {
            visibility.is_visible = event.0;
        }
    }
}

pub fn update_selected_cell_size(
    inventory: Res<Inventory>,
    mut hotbar_cells: Query<(&InventoryCellIndex, &mut Style), With<HotbarCellMarker>>,
    visibility: Res<ExtraUiVisibility>,
) {
    for (cell_index, mut style) in hotbar_cells.iter_mut() {
        let selected = cell_index.0 == inventory.selected_slot;

        style.size = match selected {
            true if !visibility.0 => INVENTORY_CELL_SIZE_SELECTED,
            _ => INVENTORY_CELL_SIZE,
        };
    }
}

pub fn update_selected_cell_image(
    inventory: Res<Inventory>,
    mut hotbar_cells: Query<(&InventoryCellIndex, &mut UiImage), With<HotbarCellMarker>>,
    ui_assets: Res<UiAssets>,
) {
    for (cell_index, mut image) in hotbar_cells.iter_mut() {
        let selected = cell_index.0 == inventory.selected_slot;

        image.0 = if selected {
            ui_assets.selected_inventory_back.clone()
        } else {
            ui_assets.inventory_back.clone()
        }
    }
}

#[autodefault(except(InventoryCell))]
pub fn spawn_inventory_cell(
    children: &mut ChildBuilder<'_, '_, '_>,
    name: impl Into<Cow<'static, str>>,
    cell_background: Handle<Image>,
    hotbar_cell: bool,
    index: usize,
    fonts: &FontAssets,
) {
    let mut background_image = ImageBundle {
        style: Style {
            margin: UiRect::horizontal(2.),
            size: INVENTORY_CELL_SIZE,
            align_self: AlignSelf::Center,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center
        },
        image: cell_background.into(),
    };

    background_image.color = (*background_image.color.0.set_a(0.8)).into();

    children
        .spawn_bundle(background_image)
        .with_children(|c| {
            c.spawn_bundle(ImageBundle {
                focus_policy: FocusPolicy::Pass,
                style: Style {
                    flex_direction: FlexDirection::Column,
                    margin: UiRect::all(Val::Px(8.)),
                },
            })
            .insert(InventoryCellIndex(index))
            .insert(InventoryCellItemImage::default());

            if hotbar_cell {
                c.spawn_bundle(NodeBundle {
                    style: Style {
                        padding: UiRect::all(Val::Px(5.)),
                        size: Size::new(Val::Percent(100.), Val::Percent(100.)),
                        position_type: PositionType::Absolute,
                        flex_direction: FlexDirection::ColumnReverse,
                        justify_content: JustifyContent::SpaceBetween,
                        align_items: AlignItems::FlexStart,
                        align_content: AlignContent::FlexStart,
                    },
                    color: TRANSPARENT.into(),
                    focus_policy: FocusPolicy::Pass,
                })
                .with_children(|c| {
                    // Hotbar cell index
                    c.spawn_bundle(TextBundle {
                        focus_policy: FocusPolicy::Pass,
                        text: Text::from_section(
                            ((index + 1) % HOTBAR_LENGTH).to_string(),
                            TextStyle {
                                font: fonts.andy_bold.clone(),
                                font_size: 16.,
                                color: Color::WHITE,
                            },
                        ),
                    });

                    // Item stack
                    c.spawn_bundle(TextBundle {
                        style: Style {
                            align_self: AlignSelf::Center,
                        },
                        focus_policy: FocusPolicy::Pass,
                        text: Text::from_section(
                            "",
                            TextStyle {
                                font: fonts.andy_regular.clone(),
                                font_size: 16.,
                                color: Color::WHITE,
                            },
                        ),
                    })
                    .insert(InventoryCellIndex(index))
                    .insert(InventoryItemAmount::default());
                });
            }
        })
        .insert(Name::new(name))
        .insert(InventoryCellIndex(index))
        .insert_if(HotbarCellMarker, || hotbar_cell)
        .insert(Interaction::default());
}

pub fn select_item(mut inventory: ResMut<Inventory>, input: Res<Input<KeyCode>>) {
    let digit = input
        .get_just_pressed()
        .find_map(|k| KEYCODE_TO_DIGIT.get(k));

    if let Some(index) = digit {
        inventory.select_item(*index);
    }
}

pub fn scroll_select_item(mut inventory: ResMut<Inventory>, mut events: EventReader<MouseWheel>) {
    for event in events.iter() {
        let selected_item_index = inventory.selected_slot as f32;
        let hotbar_length = HOTBAR_LENGTH as f32;
        let new_index = (
            ((selected_item_index + event.y.signum() * -1.) % hotbar_length) + hotbar_length
        ) % hotbar_length;

        inventory.select_item(new_index as usize);
    }
}

pub fn set_selected_item(inventory: Res<Inventory>, mut selected_item: ResMut<SelectedItem>) {
    if inventory.is_changed() {
        selected_item.0 = inventory.selected_item();
    }
}

pub fn update_selected_item_name_alignment(
    mut selected_item_name_query: Query<&mut Style, With<SelectedItemNameMarker>>,
    mut events: EventReader<ToggleExtraUiEvent>,
) {
    let mut style = selected_item_name_query.single_mut();

    for event in events.iter() {
        style.align_self = if event.0 {
            AlignSelf::FlexStart
        } else {
            AlignSelf::Center
        }
    }
}

pub fn update_selected_item_name_text(
    mut selected_item_name_query: Query<&mut Text, With<SelectedItemNameMarker>>,
    current_item: Res<SelectedItem>,
    extra_ui_visibility: Res<ExtraUiVisibility>,
) {
    if current_item.is_changed() || extra_ui_visibility.is_changed() {
        let mut text = selected_item_name_query.single_mut();

        text.sections[0].value = if extra_ui_visibility.0 {
            INVENTORY_STRING.to_string()
        } else {
            let name = current_item
                .0
                .map(|item_stack| item_stack.item);

            name
                .map(|item| item.name().to_string())
                .unwrap_or(ITEMS_STRING.to_string())
        }
    }
}

pub fn update_cell(
    inventory: Res<Inventory>,
    mut item_images: Query<(&mut InventoryCellItemImage, &InventoryCellIndex)>,
    item_assets: Res<ItemAssets>,
) {
    if inventory.is_changed() {
        for (mut cell_image, cell_index) in &mut item_images {
            cell_image.0 = inventory
                .get_item(cell_index.0)
                .map(|item_stack| item_assets.get_by_item(item_stack.item))
                .unwrap_or(item_assets.no_item());
        }
    }
}

pub fn update_cell_image(
    mut item_images: Query<
        (&mut UiImage, &InventoryCellItemImage),
        Changed<InventoryCellItemImage>,
    >,
) {
    for (mut image, item_image) in &mut item_images {
        image.0 = item_image.0.clone();
    }
}

pub fn update_item_amount(
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

pub fn update_item_amount_text(
    mut query: Query<(&mut Text, &mut Visibility, &InventoryItemAmount), Changed<InventoryItemAmount>>,
) {
    for (mut text, mut visiblity, item_stack) in &mut query {
        if item_stack.0 > 1 {
            text.sections[0].value = item_stack.0.to_string();
            visiblity.is_visible = true;
        } else {
            visiblity.is_visible = false;
        }
    }
}

pub fn inventory_cell_background_hover(
    query: Query<(&Interaction, &InventoryCellIndex), Changed<Interaction>>,
    inventory: Res<Inventory>,
    mut info: ResMut<HoveredInfo>,
) {
    for (interaction, cell_index) in &query {
        if let Some(item_stack) = inventory.get_item(cell_index.0) {
            info.0 = match interaction {
                Interaction::None => "".to_string(),
                _ => {
                    let mut name = item_stack.item.name().to_owned();
                    
                    if item_stack.stack > 1 {
                        name.push_str(&format!(" ({})", item_stack.stack.to_string()));
                    }
        
                    name.to_string()
                }
            }
        }
    }
}
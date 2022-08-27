use std::{collections::HashMap, borrow::Cow};

use bevy::{prelude::{Plugin, App, Commands, Res, NodeBundle, default, Color, ImageBundle, Component, KeyCode, Query, ParallelSystemDescriptorCoercion, Changed, With, TextBundle, Image, Handle, Visibility, ResMut, DerefMut, Deref, ExclusiveSystemDescriptorCoercion, Entity}, ui::{AlignItems, Style, Val, FlexDirection, AlignContent, UiRect, Size, AlignSelf, UiImage, Interaction, FocusPolicy, JustifyContent, PositionType}, hierarchy::{BuildChildren, ChildBuilder}, input::Input, core::Name, text::{Text, TextAlignment, TextStyle}};
use bevy_inspector_egui::Inspectable;
use iyes_loopless::{prelude::{AppLooplessStateExt, ConditionSet}, condition::IntoConditionalExclusiveSystem};
use smallvec::SmallVec;

use crate::{item::{Item, ITEM_COPPER_PICKAXE, ITEM_DATA, ItemId, ItemData}, util::{RectExtensions, EntityCommandsExtensions}, TRANSPARENT, state::GameState};

use super::{UiAssets, FontAssets, ItemAssets, HoveredInfo};

pub const SPAWN_PLAYER_UI_LABEL: &str = "spawn_player_ui";

// 5 is a total count of inventory rows. -1 because the hotbar is a first row
const INVENTORY_ROWS_COUNT: usize = 5 - 1;

// region: Inventory cell size
const INVENTORY_CELL_SIZE_F: f32 = 40.;
const INVENTORY_CELL_SIZE_BIGGER_F: f32 = INVENTORY_CELL_SIZE_F * 1.3;

const INVENTORY_CELL_SIZE_VAL: Val = Val::Px(INVENTORY_CELL_SIZE_F);
const INVENTORY_CELL_SIZE_BIGGER_VAL: Val = Val::Px(INVENTORY_CELL_SIZE_BIGGER_F);
// endregion

const CELL_COUNT_IN_ROW: usize = 10;

const ITEMS_STRING: &str = "Items";
const INVENTORY_STRING: &str = "Inventory";

lazy_static! {
    static ref KEYCODE_TO_DIGIT: HashMap<KeyCode, usize> = HashMap::from([
        (KeyCode::Key1, 0),
        (KeyCode::Key2, 1),
        (KeyCode::Key3, 2),
        (KeyCode::Key4, 3),
        (KeyCode::Key5, 4),
        (KeyCode::Key6, 5),
        (KeyCode::Key7, 6),
        (KeyCode::Key8, 7),
        (KeyCode::Key9, 8),
        (KeyCode::Key0, 9)
    ]);
}

// region: Plugin
pub struct PlayerInventoryPlugin;

impl Plugin for PlayerInventoryPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<SelectedItem>()
            .insert_resource({
                let mut inventory = Inventory::default();
                inventory.items.insert(0, Some(ITEM_COPPER_PICKAXE));

                inventory
            })

            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::InGame)
                    .with_system(change_inventory_visibility)
                    .with_system(set_selected_item)
                    .with_system(select_hotbar_cell)
                    .with_system(update_inventory_visibility)
                    .with_system(update_selected_cell)
                    .with_system(update_selected_item_name_style)
                    .with_system(update_selected_item_name_text)
                    .with_system(update_cell)
                    .with_system(update_cell_image)
                    .with_system(inventory_cell_background_hover)
                    .with_system(update_item_stack)
                    .with_system(update_item_stack_text)
                    .into()
            );
    }
}

// endregion

// region: Structs

#[derive(Component, Default)]
pub struct Inventory {
    pub items: SmallVec::<[Option<Item>; 50]>
}

impl Inventory {
    fn get_item(&self, index: usize) -> Option<Item> {
        self.items.iter().nth(index).and_then(|a| *a)
    }
}

#[derive(Component, Default, Inspectable)]
struct InventoryUi {
    showing: bool
}

#[derive(Component, Default)]
struct HotbarUi {
    selected_cell: usize
}

#[derive(Component)]
struct HotbarCellMarker;

#[derive(Component)]
struct SelectedItemNameMarker;

#[derive(Component)]
struct InventoryCell {
    index: usize
}

#[derive(Component, Default)]
struct InventoryCellItemImage(Handle<Image>);

#[derive(Component, Default)]
struct InventoryItemStack(u16);

#[derive(Component, Default, Deref, DerefMut)]
pub struct SelectedItem(pub Option<Item>);

// endregion


fn get_item_data_by_id<'a>(id: &ItemId) -> &'a ItemData {
    ITEM_DATA.get(id).expect("Item not found")
}


pub fn spawn_inventory_ui(
    commands: &mut Commands,
    ui_assets: &UiAssets,
    fonts: &FontAssets,
) -> Entity {
    commands.spawn_bundle(NodeBundle {
        style: Style {
            align_items: AlignItems::Center,
            align_content: AlignContent::Center,
            flex_direction: FlexDirection::ColumnReverse,
            margin: UiRect { 
                left: Val::Px(20.),
                top: Val::Px(5.),
                ..default()
            },
            ..default()
        },
        color: TRANSPARENT.into(),
        ..default()
    })
    .insert(Name::new("Inventory Container"))
    .with_children(|children| {
        // region: Selected Item Name

        children.spawn_bundle(TextBundle {
            style: Style {
                margin: UiRect {
                    ..UiRect::horizontal(10.)
                },
                align_self: AlignSelf::Center,
                ..default()
            },
            text: Text::from_section(
                "".to_string(), 
                TextStyle {
                    font: fonts.andy_bold.clone(),
                    font_size: 20.,
                    color: Color::WHITE,
                }
            ).with_alignment(TextAlignment::CENTER),
            ..default()
        })
        .insert(Name::new("Selected Item Name"))
        .insert(SelectedItemNameMarker);

        // endregion

        // region: Hotbar

        children.spawn_bundle(NodeBundle {
            style: Style {
                align_items: AlignItems::Center,
                ..default()
            },
            color: TRANSPARENT.into(),
            ..default()
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
                    &fonts
                );
            }
        })
        .insert(HotbarUi::default());

        // endregion

        // region: Inventory
        children.spawn_bundle(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::ColumnReverse,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            color: TRANSPARENT.into(),
            ..default()
        })
        .with_children(|children| {
            for j in 0..INVENTORY_ROWS_COUNT {
                children.spawn_bundle(NodeBundle {
                    style: Style {
                        margin: UiRect::vertical(2.),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    color: TRANSPARENT.into(),
                    ..default()
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
                            &fonts
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

fn change_inventory_visibility(
    input: Res<Input<KeyCode>>,
    mut query: Query<&mut InventoryUi>
) {
    if input.just_pressed(KeyCode::Escape) {
        let mut inventory = query.single_mut();
        inventory.showing = !inventory.showing;
    }
}

fn update_inventory_visibility(
    mut query: Query<(&InventoryUi, &mut Visibility), Changed<InventoryUi>>
) {
    for (inventory_ui, mut visibility) in &mut query {
        visibility.is_visible = inventory_ui.showing;
    }
}

fn update_selected_cell(
    mut hotbar_cells: Query<(&InventoryCell, &mut Style, &mut UiImage), With<HotbarCellMarker>>,
    hotbar_query: Query<&HotbarUi>,
    inventory_query: Query<&InventoryUi>,
    ui_assets: Res<UiAssets>
) {
    let inventory = inventory_query.single();
    let hotbar = hotbar_query.single();

    for (cell, mut style, mut image) in hotbar_cells.iter_mut() {
        let selected = cell.index == hotbar.selected_cell;
        if selected {
            style.size = if inventory.showing {
                Size::new(INVENTORY_CELL_SIZE_VAL, INVENTORY_CELL_SIZE_VAL)
            } else {
                Size::new(INVENTORY_CELL_SIZE_BIGGER_VAL, INVENTORY_CELL_SIZE_BIGGER_VAL)
            };

            image.0 = ui_assets.selected_inventory_back.clone();
        } else {
            style.size = Size::new(INVENTORY_CELL_SIZE_VAL, INVENTORY_CELL_SIZE_VAL);

            image.0 = ui_assets.inventory_back.clone();
        }
    }
}

fn spawn_inventory_cell(
    children: &mut ChildBuilder<'_, '_, '_>, 
    name: impl Into<Cow<'static, str>>, 
    cell_background: Handle<Image>,
    hotbar_cell: bool,
    index: usize,
    fonts: &FontAssets
) {
    let mut background_image = ImageBundle {
        style: Style {
            margin: UiRect::horizontal(2.),
            size: Size { 
                width: INVENTORY_CELL_SIZE_VAL, 
                height: INVENTORY_CELL_SIZE_VAL 
            },
            align_self: AlignSelf::Center,
            ..default()
        },
        image: cell_background.into(),
        ..default()
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
                    ..default()
                },
                ..default()
            })
            .insert(InventoryCell {
                index
            })
            .insert(InventoryCellItemImage::default())
            .insert(Interaction::default());

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
                        ..default()
                    },
                    color: TRANSPARENT.into(),
                    focus_policy: FocusPolicy::Pass,
                    ..default()
                }).with_children(|c| {

                    // Hotbar cell index
                    c.spawn_bundle(TextBundle {
                        focus_policy: FocusPolicy::Pass,
                        text: Text::from_section(((index + 1) % 10).to_string(), TextStyle { 
                            font: fonts.andy_bold.clone(),
                            font_size: 16.,
                            color: Color::WHITE
                        }),
                        ..default()
                    });

                    // Item stack
                    c.spawn_bundle(TextBundle {
                        style: Style {
                            align_self: AlignSelf::Center,
                            ..default()
                        },
                        focus_policy: FocusPolicy::Pass,
                        text: Text::from_section("", TextStyle { 
                            font: fonts.andy_regular.clone(),
                            font_size: 16.,
                            color: Color::WHITE
                        }),
                        ..default()
                    })
                    .insert(InventoryCell {
                        index
                    })
                    .insert(InventoryItemStack::default());
                });
            }

        })
        .insert(Name::new(name))
        .insert(InventoryCell { index })
        .insert_if(HotbarCellMarker, || { hotbar_cell })
        .insert(Interaction::default());
}

fn select_hotbar_cell(
    input: Res<Input<KeyCode>>,
    mut query: Query<&mut HotbarUi>
) {
    let digit = input
        .get_just_pressed()
        .find_map(|k| KEYCODE_TO_DIGIT.get(k));

    if let Some(digit) = digit {
        let mut hotbar = query.single_mut();
        hotbar.selected_cell = *digit;
    }
}

fn set_selected_item(
    inventory: Res<Inventory>,
    mut selected_item: ResMut<SelectedItem>,
    hotbar_query: Query<&HotbarUi, Changed<HotbarUi>>,
) {
    if let Ok(hotbar) = hotbar_query.get_single() {
        selected_item.0 = inventory
            .get_item(hotbar.selected_cell)
            .map(|item| item);
    }
}

fn update_selected_item_name_style(
    inventory_query: Query<&InventoryUi, Changed<InventoryUi>>,
    mut selected_item_name_query: Query<&mut Style, With<SelectedItemNameMarker>>,
) {
    let mut style = selected_item_name_query.single_mut();
    let inventory_query_result = inventory_query.get_single();

    if let Ok(inventory) = inventory_query_result {
        style.align_self = if inventory.showing {
            AlignSelf::FlexStart
        } else {
            AlignSelf::Center
        }
    }
}

fn update_selected_item_name_text(
    inventory_query: Query<&InventoryUi>,
    mut selected_item_name_query: Query<&mut Text, With<SelectedItemNameMarker>>,
    current_item: Res<SelectedItem>
) {
    let mut text = selected_item_name_query.single_mut();
    let inventory_query_result = inventory_query.get_single();

    if let Ok(inventory) = inventory_query_result {
        text.sections[0].value = if inventory.showing {
            INVENTORY_STRING.to_string()
        } else {
            let name = current_item.0.map(|item| get_item_data_by_id(&item.id).name);

            name.map(|name| name.to_string()).unwrap_or(ITEMS_STRING.to_string())
        }
    }
}

fn update_cell(
    inventory: Res<Inventory>,
    mut item_images: Query<(&mut InventoryCellItemImage, &InventoryCell)>,
    item_assets: Res<ItemAssets>,
) {
    if inventory.is_changed() {
        for (mut cell_image, cell) in &mut item_images {
            cell_image.0 = inventory
                .get_item(cell.index)
                .map(|item| item_assets.get_by_id(item.id))
                .unwrap_or(item_assets.no_item());
        }
    }
}

fn update_cell_image(
    mut item_images: Query<(&mut UiImage, &InventoryCellItemImage), Changed<InventoryCellItemImage>>,
) {
    for (mut image, item_image) in &mut item_images {
        image.0 = item_image.0.clone();
    }
}

fn update_item_stack(
    inventory: Res<Inventory>,
    mut query: Query<(&mut InventoryItemStack, &InventoryCell)>
) {
    if inventory.is_changed() {
        for (mut item_stack, cell) in &mut query {
            if let Some(item) = inventory.items.get(cell.index).and_then(|item| *item) {
                item_stack.0 = item.stack;
            }
        }
    }
}

fn update_item_stack_text(
    mut query: Query<(&mut Text, &InventoryItemStack), Changed<InventoryItemStack>>
) {
    for (mut text, item_stack) in &mut query {
        if item_stack.0 > 1 {
            text.sections[0].value = item_stack.0.to_string();
        }
    }
}

fn inventory_cell_background_hover(
    query: Query<(&Interaction, &InventoryCell), Changed<Interaction>>,
    inventory: Res<Inventory>,
    mut info: ResMut<HoveredInfo>
) {
    for (interaction, cell) in &query {
        if let Some(item) = inventory.get_item(cell.index) {
            let name = if *interaction != Interaction::None {
                get_item_data_by_id(&item.id).name
            } else {
                ""
            };

            info.0 = name.to_string();
        }
    }
}
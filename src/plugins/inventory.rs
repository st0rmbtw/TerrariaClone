use std::{collections::HashMap, borrow::Cow};

use bevy::{prelude::{Plugin, App, Commands, Res, NodeBundle, default, Color, ImageBundle, Component, KeyCode, Query, ParallelSystemDescriptorCoercion, Changed, With, TextBundle, Image, Handle, Visibility, ResMut, Vec2, Children, SystemSet}, ui::{AlignItems, Style, Val, FlexDirection, AlignContent, UiRect, Size, AlignSelf, UiImage, Interaction, Node}, hierarchy::{BuildChildren, ChildBuilder}, input::Input, core::Name, text::{Text, TextAlignment, TextStyle}};
use bevy_inspector_egui::Inspectable;
use smallvec::SmallVec;

use crate::{item::{Item, ITEM_COPPER_PICKAXE}, util::{RectExtensions, EntityCommandsExtensions}, TRANSPARENT, state::GameState};

use super::{UiAssets, FontAssets, ItemAssets, HoveredInfo};

pub const SPAWN_PLAYER_UI_LABEL: &str = "spawn_player_ui";

// 5 is a total count of inventory rows. -1 because the hotbar is a first row
const INVENTORY_ROWS_COUNT: usize = 5 - 1;

const INVENTORY_CELL_SIZE_F: f32 = 42.;
const INVENTORY_CELL_SIZE_VAL: Val = Val::Px(INVENTORY_CELL_SIZE_F);
const INVENTORY_CELL_SIZE_BIGGER_VAL: Val = Val::Px(INVENTORY_CELL_SIZE_F * 1.3);

const CELL_COUNT_IN_ROW: usize = 10;

const DEFAULT_CURRENT_ITEM_NAME: &str = "Items";
const CURRENT_ITEM_NAME_INVENTORY: &str = "Inventory";

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
            .insert_resource(SelectedItemName::default())
            .insert_resource({
                let mut inventory = Inventory::default();
                inventory.items.insert(0, Some(&ITEM_COPPER_PICKAXE));

                inventory
            })

            .add_system_set(
                SystemSet::on_enter(GameState::InGame)
                    .with_system(spawn_inventory_ui.label(SPAWN_PLAYER_UI_LABEL))
            )

            .add_system_set(
                SystemSet::on_update(GameState::InGame)
                    .with_system(change_visibility)
                    .with_system(select_hotbar_cell)
                    .with_system(update_inventory_visibility)
                    .with_system(update_selected_cell)
                    .with_system(update_selected_item_name)
                    .with_system(update_cell)
                    .with_system(update_cell_image)
                    .with_system(set_selected_item_name)
                    .with_system(inventory_cell_background_hover)
            );
    }
}

// endregion

// region: Structs

#[derive(Component, Default)]
pub struct Inventory<'a> {
    pub items: SmallVec::<[Option<&'a Item>; 50]>
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

#[derive(Default)]
struct SelectedItemName {
    name: Option<String>
}

#[derive(Component)]
struct SelectedItemNameMarker;

#[derive(Component)]
struct InventoryCell {
    index: usize
}

#[derive(Component)]
struct InventoryCellItemImage {
    index: usize,
    item_image: Handle<Image>
}

// endregion

fn spawn_inventory_ui(
    mut commands: Commands,
    ui_assets: Res<UiAssets>,
    fonts: Res<FontAssets>
) {
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
                    top: Val::Px(2.),
                    ..UiRect::horizontal(10.)
                },
                align_self: AlignSelf::Center,
                ..default()
            },
            text: Text::from_section(
                "".to_string(), 
                TextStyle {
                    font: fonts.andy_regular.clone(),
                    font_size: 24.,
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
                    UiRect::horizontal(2.),
                    format!("Hotbar Cell #{}", i),
                    ui_assets.inventory_back.clone(),
                    true,
                    i
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
                            UiRect::horizontal(2.),
                            format!("Inventory Cell #{}", index),
                            ui_assets.inventory_back.clone(),
                            false,
                            index
                        );
                    }
                });
            }
        })
        .insert(Name::new("Inventory"))
        .insert(InventoryUi::default());
        // endregion
    });
}

fn change_visibility(
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
    mut hotbar_cells: Query<(&mut Style, &mut UiImage), With<HotbarCellMarker>>,
    hotbars: Query<&HotbarUi>,
    inventories: Query<&InventoryUi>,
    ui_assets: Res<UiAssets>
) {
    let inventory = inventories.single();

    let hotbar = hotbars.single();

    for (i, (mut style, mut image)) in hotbar_cells.iter_mut().enumerate() {
        let selected = i == hotbar.selected_cell;
        if selected {
            image.0 = ui_assets.selected_inventory_back.clone();

            if !inventory.showing {
                style.size = Size::new(INVENTORY_CELL_SIZE_BIGGER_VAL, INVENTORY_CELL_SIZE_BIGGER_VAL);
            } else {
                style.size = Size::new(INVENTORY_CELL_SIZE_VAL, INVENTORY_CELL_SIZE_VAL);
            }

        } else {
            style.size = Size::new(INVENTORY_CELL_SIZE_VAL, INVENTORY_CELL_SIZE_VAL);

            image.0 = ui_assets.inventory_back.clone();
        }
    }
}

fn spawn_inventory_cell(
    children: &mut ChildBuilder<'_, '_, '_>, 
    margin: UiRect<Val>, 
    name: impl Into<Cow<'static, str>>, 
    cell_background: Handle<Image>,
    hotbar_cell: bool,
    index: usize
) {
    let mut background_image = ImageBundle {
        node: Node {
            size: Vec2::new(INVENTORY_CELL_SIZE_F, INVENTORY_CELL_SIZE_F)
        },
        style: Style {
            margin,
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
                style: Style {
                    flex_direction: FlexDirection::Column,
                    margin: UiRect::all(Val::Px(10.)),
                    ..default()
                },
                ..default()
            }).insert(InventoryCellItemImage {
                index,
                item_image: Handle::default()
            }).insert(Interaction::default());
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

fn set_selected_item_name(
    hotbars: Query<&HotbarUi, Changed<HotbarUi>>,
    inventory: Res<Inventory<'static>>,
    mut selected_item_name: ResMut<SelectedItemName>
) {
    if let Ok(hotbar) = hotbars.get_single() {
        selected_item_name.name = if let Some(Some(item)) = inventory.items.iter().nth(hotbar.selected_cell) {
            Some(item.name.clone())
        } else {
            None
        }
    }
}

fn update_selected_item_name(
    inventories: Query<&InventoryUi>,
    mut query: Query<(&mut Text, &mut Style), With<SelectedItemNameMarker>>,
    current_item_name: Res<SelectedItemName>
) {
    let (mut text, mut style) = query.single_mut();

    let inventory = inventories.single();

    if inventory.showing {
        text.sections[0].value = CURRENT_ITEM_NAME_INVENTORY.to_string();
        style.align_self = AlignSelf::FlexStart;
    } else {
        style.align_self = AlignSelf::Center;
        text.sections[0].value = if let Some(name) = &current_item_name.name {
            name.clone()
        } else {
            // TODO: Cyrillic symbols are showed incorrectly
            DEFAULT_CURRENT_ITEM_NAME.to_string()
        }
    }
}

fn update_cell(
    inventory: Res<Inventory<'static>>,
    mut item_images: Query<&mut InventoryCellItemImage>,
    item_assets: Res<ItemAssets>,
) {
    for mut cell_image in &mut item_images {
        cell_image.item_image = if let Some(Some(item)) = inventory.items.iter().nth(cell_image.index) {
            item_assets.get_by_id(item.id)
        } else {
            item_assets.no_item()
        }
    }
}

fn update_cell_image(
    mut item_images: Query<(&mut UiImage, &InventoryCellItemImage), Changed<InventoryCellItemImage>>,
) {
    for (mut image, item_image) in &mut item_images {
        image.0 = item_image.item_image.clone();
    }
}

fn inventory_cell_background_hover(
    interactions: Query<&Interaction>,
    query: Query<(&Interaction, &InventoryCell, &Children), Changed<Interaction>>,
    inventory: Res<Inventory<'static>>,
    mut info: ResMut<HoveredInfo>
) {
    for (interaction, cell, children) in &query {
        if let Some(Some(item)) = inventory.items.iter().nth(cell.index) {
            let any_children_hovered = children.iter()
                .filter_map(|c| interactions.get(*c).ok())
                .any(|i| matches!(i, Interaction::Hovered));

            if *interaction == Interaction::Hovered || any_children_hovered {
                info.0 = item.name.clone();
            } else {
                info.0 = "".to_string();
            }
        }
    }
}
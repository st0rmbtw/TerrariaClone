use std::{collections::HashMap, borrow::Cow};

use bevy::{prelude::{Plugin, App, Commands, Res, NodeBundle, default, Color, ImageBundle, Component, KeyCode, Query, ParallelSystemDescriptorCoercion, Changed, With, TextBundle, Image, Handle, Transform, Visibility, GlobalTransform}, ui::{AlignItems, Style, Val, FlexDirection, UiColor, AlignContent, UiRect, Size, JustifyContent, AlignSelf, UiImage}, hierarchy::{BuildChildren, ChildBuilder, Children}, input::Input, core::Name, text::{Text, TextAlignment, TextStyle, Font}};
// use bevy_inspector_egui::Inspectable;
use smallvec::SmallVec;

use crate::{item::Item, util::RectExtensions, TRANSPARENT};

use super::{UiAssets, FontAssets, ItemAssets};

pub const SPAWN_PLAYER_UI_LABEL: &str = "spawn_player_ui";

// 5 is a total count of inventory rows. -1 because the hotbar is a first row
const INVENTORY_ROWS_COUNT: i32 = 5 - 1;

const INVENTORY_CELL_SIZE: f32 = 42.;
const INVENTORY_CELL_SIZE_VAL: Val = Val::Px(INVENTORY_CELL_SIZE);
const INVENTORY_CELL_SIZE_BIGGER_VAL: Val = Val::Px(INVENTORY_CELL_SIZE * 1.3);

const CELL_COUNT_IN_ROW: i32 = 10;


lazy_static! {
    static ref KEYCODE_TO_DIGIT: HashMap<KeyCode, i32> = HashMap::from([
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
            .add_startup_system(spawn_inventory_ui.label(SPAWN_PLAYER_UI_LABEL))
            .add_system(change_visibility)
            .add_system(select_hotbar_cell)
            .add_system(update_inventory_visibility)
            .add_system(update_selected_cell)
            .add_system(update_current_item_name);
    }
}

// endregion

// region: Structs

#[derive(Component, Default)]
pub struct Inventory {
    items: SmallVec::<[Option<Item>; 50]>
}

#[derive(Component, Default, /* Inspectable */)]
struct InventoryUi {
    showing: bool
}

#[derive(Component, Default)]
struct HotbarUi {
    selected_cell: i32
}

#[derive(Component, Default)]
struct CurrentItemName {
    name: Option<String>
}

// endregion

fn spawn_inventory_ui(
    mut commands: Commands,
    ui_assets: Res<UiAssets>,
    fonts: Res<FontAssets>,
    item_assets: Res<ItemAssets>
) {
    commands.spawn_bundle(NodeBundle {
        style: Style {
            align_items: AlignItems::Center,
            align_content: AlignContent::Center,
            flex_direction: FlexDirection::ColumnReverse,
            margin: UiRect { 
                left: Val::Px(21.),
                ..default()
            },
            ..default()
        },
        color: TRANSPARENT.into(),
        ..default()
    })
    .insert(Name::new("Inventory Ui"))
    .with_children(|children| {
        // region: Current Item Name

        children.spawn_bundle(TextBundle {
            style: Style {
                margin: UiRect {
                    top: Val::Px(2.),
                    ..UiRect::horizontal(Val::Px(10.))
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
        }).insert(CurrentItemName::default());

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
                    UiRect::horizontal(Val::Px(2.)),
                    format!("Hotbar Cell #{}", i),
                    ui_assets.inventory_back.clone(),
                    item_assets.copper_pickaxe.clone(),
                    fonts.andy_regular.clone()
                )
            }
        })
        .insert(HotbarUi::default());

        // endregion

        // region: Inventory
        children.spawn_bundle(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..default()
            },
            color: TRANSPARENT.into(),
            ..default()
        })
        .with_children(|children| {
            for _ in 0..INVENTORY_ROWS_COUNT {
                children.spawn_bundle(NodeBundle {
                    style: Style {
                        margin: UiRect::vertical(Val::Px(2.)),
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    color: TRANSPARENT.into(),
                    ..default()
                }).with_children(|children| {
                    for i in 0..CELL_COUNT_IN_ROW {
                        spawn_inventory_cell(
                            children, 
                            UiRect::horizontal(Val::Px(2.)),
                            format!("Inventory Cell #{}", i),
                            ui_assets.inventory_back.clone(),
                            item_assets.copper_pickaxe.clone(),
                            fonts.andy_regular.clone()
                        )
                    }
                });
            }
        })
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
    for (inventory_ui, mut visibility) in query.iter_mut() {
        visibility.is_visible = inventory_ui.showing;
    }
}

fn update_selected_cell(
    mut cells: Query<(&mut Style, &mut UiImage)>,
    hotbars: Query<&HotbarUi>,
    hotbar_children: Query<&Children, With<HotbarUi>>,
    inventories: Query<&InventoryUi>,
    ui_assets: Res<UiAssets>
) {
    let inventory = inventories.single();

    let hotbar = hotbars.single();
    let children = hotbar_children.single();

    for (i, child) in children.iter().enumerate() {
        if let Ok((mut style, mut image)) = cells.get_mut(*child) {
            let selected = (i as i32) == hotbar.selected_cell;

            if selected {

                image.0 = ui_assets.inventory_back14.clone();

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
}

fn spawn_inventory_cell(
    children: &mut ChildBuilder<'_, '_, '_>, 
    margin: UiRect<Val>, 
    name: impl Into<Cow<'static, str>>, 
    cell_background: Handle<Image>,
    item_image: Handle<Image>,
    font: Handle<Font>
) {
    let mut background_image = ImageBundle {
        style: Style {
            size: Size::new(INVENTORY_CELL_SIZE_VAL, INVENTORY_CELL_SIZE_VAL),
            margin,
            align_self: AlignSelf::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        image: cell_background.into(),
        ..default()
    };

    background_image.color = (*background_image.color.0.set_a(0.8)).into();

    children
        .spawn_bundle(background_image)
        .with_children(|c| {
            c.spawn_bundle(NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.), Val::Auto),
                    align_self: AlignSelf::Center,
                    justify_content: JustifyContent::Center,
                    align_content: AlignContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                color: TRANSPARENT.into(),
                ..default()
            }).with_children(|c| {
                c.spawn_bundle(TextBundle {
                    style: Style {
                        justify_content: JustifyContent::Center,
                        align_content: AlignContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    text: Text::from_section(
                        "1", 
                        TextStyle { 
                            font,
                            font_size: 20., 
                            color: Color::WHITE.into()
                        },
                    ).with_alignment(TextAlignment::TOP_LEFT),
                    ..default()
                }).with_children(|c| {
                    c.spawn_bundle(ImageBundle {
                        style: Style {
                            align_self: AlignSelf::Center,
                            margin: UiRect::all(Val::Px(10.)),
                            ..default()
                        },
                        image: item_image.into(),
                        ..default()
                    });
                });
            });
        })
        .insert(Name::new(name));
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

fn update_current_item_name(
    inventories: Query<&InventoryUi>,
    mut query: Query<(&mut Text, &mut Style, &CurrentItemName)>,
) {
    let (mut text, mut style, current_item_name) = query.single_mut();

    let inventory = inventories.single();

    if inventory.showing {
        text.sections[0].value = "Inventory".to_string();
        style.align_self = AlignSelf::FlexStart;
    } else {
        style.align_self = AlignSelf::Center;
        text.sections[0].value = if let Some(name) = &current_item_name.name {
            name.clone()
        } else {
            // TODO: Cyrillic symbols dont show correctly
            "Items".to_string()
        }
    }
}
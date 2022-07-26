use std::{collections::HashMap, borrow::Cow};

use bevy::{prelude::{Plugin, App, Commands, Res, NodeBundle, default, Color, ImageBundle, Component, KeyCode, Query, Visibility, ParallelSystemDescriptorCoercion, Changed, With, Entity}, ui::{AlignItems, JustifyContent, Style, Val, FlexDirection, UiImage, Display, UiColor}, math::{Rect, Size}, hierarchy::{BuildChildren, ChildBuilder, Children}, input::Input, core::Name};
use bevy_inspector_egui::Inspectable;

use crate::{item::{Item, ITEM_WOODEN_PICKAXE}, TRANSPARENT, RectExtensions};

use super::UiAssets;

pub const SPAWN_PLAYER_UI_LABEL: &str = "spawn_player_ui";

// 5 is a total count of inventory rows. -1 because the hotbar is a first row
const INVENTORY_ROWS_COUNT: i32 = 5 - 1;

const INVENTORY_CELL_SIZE: f32 = 36.;
const CELL_COUNT_IN_ROW: i32 = 10;

lazy_static! {
    static ref KEYCODE_TO_DIGIT: HashMap<KeyCode, i8> = HashMap::from([
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

pub struct PlayerInventoryPlugin;

impl Plugin for PlayerInventoryPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_startup_system(spawn_inventory_ui.label(SPAWN_PLAYER_UI_LABEL))
            .add_system(show_on_esc)
            .add_system(select_hotbar_cell)
            .add_system(update_inventory_visibility)
            .add_system(update_selected_cell);
    }
}

struct Cell {
    index: i8,
    item: Option<&'static Item>,
    keycode: KeyCode
}

pub struct Hotbar {
    selected_cell: i8,
    cells: [Cell; 9]
}

impl Default for Hotbar {
    fn default() -> Self {
        Self { 
            selected_cell: 0, 
            cells: [
                Cell {
                    index: 0,
                    item: Some(&ITEM_WOODEN_PICKAXE),
                    keycode: KeyCode::Key1
                },
                Cell {
                    index: 1,
                    item: None,
                    keycode: KeyCode::Key2
                },
                Cell {
                    index: 2,
                    item: None,
                    keycode: KeyCode::Key3
                },
                Cell {
                    index: 3,
                    item: None,
                    keycode: KeyCode::Key4
                },
                Cell {
                    index: 4,
                    item: None,
                    keycode: KeyCode::Key5
                },
                Cell {
                    index: 5,
                    item: None,
                    keycode: KeyCode::Key6
                },
                Cell {
                    index: 6,
                    item: None,
                    keycode: KeyCode::Key7
                },
                Cell {
                    index: 7,
                    item: None,
                    keycode: KeyCode::Key8
                },
                Cell {
                    index: 8,
                    item: None,
                    keycode: KeyCode::Key9
                },
            ]
        }
    }
}

#[derive(Component, Default)]
pub struct Inventory {
    hotbar: Hotbar
}

#[derive(Component, Inspectable)]
struct InventoryUi {
    showing: bool
}

#[derive(Component)]
struct HotbarUi;

fn show_on_esc(
    input: Res<Input<KeyCode>>,
    mut query: Query<&mut InventoryUi>
) {
    if input.just_pressed(KeyCode::Escape) {
        let mut inventory = query.single_mut();
        inventory.showing = !inventory.showing;
    }
}

fn spawn_inventory_ui(
    mut commands: Commands,
    ui_assets: Res<UiAssets>
) {
    commands.spawn_bundle(NodeBundle {
        style: Style {
            align_items: AlignItems::FlexEnd,
            justify_content: JustifyContent::FlexStart,
            flex_direction: FlexDirection::ColumnReverse,
            margin: Rect { 
                top: Val::Px(10.),
                left: Val::Px(10.),
                ..default()
            },
            ..default()
        },
        color: TRANSPARENT.into(),
        ..default()
    })
    .insert(Name::new("Inventory"))
    .with_children(|children| {
        // region: Hotbar

        children.spawn_bundle(NodeBundle {
            color: TRANSPARENT.into(),
            ..default()
        })
        .insert(Name::new("Hotbar"))
        .with_children(|children| {
            for i in 0..CELL_COUNT_IN_ROW {
                spawn_inventory_cell(
                    children, 
                    ui_assets.iner_panel_background.clone().into(),
                    Rect::horizontal(Val::Px(2.)),
                    None,
                    format!("Hotbar Cell #{}", i)
                )
            }
        })
        .insert(HotbarUi);

        // endregion

        // region: Inventory
        children.spawn_bundle(NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                margin: Rect::top(Val::Px(2.)),
                ..default()
            },
            color: TRANSPARENT.into(),
            ..default()
        })
        .with_children(|children| {
            for _ in 0..INVENTORY_ROWS_COUNT {
                children.spawn_bundle(NodeBundle {
                    color: TRANSPARENT.into(),
                    ..default()
                }).with_children(|children| {
                    for i in 0..CELL_COUNT_IN_ROW {
                        spawn_inventory_cell(
                            children, 
                            ui_assets.iner_panel_background.clone().into(),
                            Rect::all(Val::Px(2.)),
                            Some(Color::GRAY),
                            format!("Inventory Cell #{}", i)
                        )
                    }
                });
            }
        })
        .insert(InventoryUi {
            showing: false
        });
        // endregion
    });
}

fn update_inventory_visibility(
    mut query: Query<(&InventoryUi, &mut Style), Changed<InventoryUi>>
) {
    for (inventory_ui, mut style) in query.iter_mut() {
        style.display = if inventory_ui.showing { Display::Flex } else { Display::None };
    }
}

fn update_selected_cell(
    mut cells: Query<(Entity, &mut UiColor)>,
    inventories: Query<&Inventory, Changed<Inventory>>,
    hotbar_children: Query<&Children, With<HotbarUi>>,
) {
    for inventory in inventories.iter() {
        let children = hotbar_children.single();

        for (i, child) in children.iter().enumerate() {
            if let Ok((_, mut color)) = cells.get_mut(*child) {
                if i as i8 == inventory.hotbar.selected_cell {
                    *color = Color::YELLOW.into();
                } else {
                    *color = Color::default().into()
                }
            }
        }
    }
}

fn spawn_inventory_cell(children: &mut ChildBuilder<'_, '_, '_>, image: UiImage, margin: Rect<Val>, color: Option<Color>, name: impl Into<Cow<'static, str>>) {
    children.spawn_bundle(ImageBundle {
        style: Style {
            size: Size::new(Val::Px(INVENTORY_CELL_SIZE), Val::Px(INVENTORY_CELL_SIZE)),
            margin,
            ..default()
        },
        image,
        color: color.unwrap_or_default().into(),
        ..default()
    })
    .insert(Name::new(name));
}

fn select_hotbar_cell(
    input: Res<Input<KeyCode>>,
    mut query: Query<&mut Inventory>
) {
    let digit = input
        .get_just_pressed()
        .find_map(|k| KEYCODE_TO_DIGIT.get(k));

    if let Some(digit) = digit {
        let mut inventory = query.single_mut();
        inventory.hotbar.selected_cell = *digit;
    }
}
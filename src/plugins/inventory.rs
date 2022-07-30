use std::{collections::HashMap, borrow::Cow};

use bevy::{prelude::{Plugin, App, Commands, Res, NodeBundle, default, Color, ImageBundle, Component, KeyCode, Query, ParallelSystemDescriptorCoercion, Changed, With}, ui::{AlignItems, Style, Val, FlexDirection, UiImage, Display, UiColor}, math::{Rect, Size}, hierarchy::{BuildChildren, ChildBuilder, Children}, input::Input, core::Name};
use bevy_inspector_egui::Inspectable;
use smallvec::SmallVec;

use crate::{item::Item, util::RectExtensions, TRANSPARENT};

use super::UiAssets;

pub const SPAWN_PLAYER_UI_LABEL: &str = "spawn_player_ui";

// 5 is a total count of inventory rows. -1 because the hotbar is a first row
const INVENTORY_ROWS_COUNT: i32 = 5 - 1;

const INVENTORY_CELL_SIZE: f32 = 32.5;
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

pub struct PlayerInventoryPlugin;

impl Plugin for PlayerInventoryPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_startup_system(spawn_inventory_ui.label(SPAWN_PLAYER_UI_LABEL))
            .add_system(change_visibility)
            .add_system(select_hotbar_cell)
            .add_system(update_inventory_visibility)
            .add_system(update_selected_cell);
    }
}

#[derive(Component, Default)]
pub struct Inventory {
    items: SmallVec::<[Option<Item>; 50]>
}

#[derive(Component, Inspectable)]
struct InventoryUi {
    showing: bool
}

#[derive(Component, Default)]
struct HotbarUi {
    selected_cell: i32
}

fn spawn_inventory_ui(
    mut commands: Commands,
    ui_assets: Res<UiAssets>
) {
    commands.spawn_bundle(NodeBundle {
        style: Style {
            align_items: AlignItems::FlexEnd,
            flex_direction: FlexDirection::ColumnReverse,
            margin: Rect { 
                top: Val::Px(15.),
                left: Val::Px(15.),
                ..default()
            },
            ..default()
        },
        color: TRANSPARENT.into(),
        ..default()
    })
    .insert(Name::new("Inventory Ui"))
    .with_children(|children| {
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
                    ui_assets.iner_panel_background.clone().into(),
                    Rect::horizontal(Val::Px(2.)),
                    Some(Color::YELLOW),
                    format!("Hotbar Cell #{}", i)
                )
            }
        })
        .insert(HotbarUi::default());

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
    mut query: Query<(&InventoryUi, &mut Style), Changed<InventoryUi>>
) {
    for (inventory_ui, mut style) in query.iter_mut() {
        style.display = if inventory_ui.showing { Display::Flex } else { Display::None };
    }
}

fn update_selected_cell(
    mut cells: Query<(&mut UiColor, &mut Style)>,
    hotbars: Query<&HotbarUi, Changed<HotbarUi>>,
    hotbar_children: Query<&Children, With<HotbarUi>>,
) {
    if let Ok(hotbar) = hotbars.get_single() {
        let children = hotbar_children.single();

        for (i, child) in children.iter().enumerate() {
            if let Ok((mut color, mut style)) = cells.get_mut(*child) {
                let selected = (i as i32) == hotbar.selected_cell;

                if selected {

                    *color = Color::YELLOW.into();
                    style.size = Size::new(INVENTORY_CELL_SIZE_BIGGER_VAL, INVENTORY_CELL_SIZE_BIGGER_VAL);

                } else {

                    *color = Color::default().into();
                    style.size = Size::new(INVENTORY_CELL_SIZE_VAL, INVENTORY_CELL_SIZE_VAL);

                }
            }
        }
    }
}

fn spawn_inventory_cell(children: &mut ChildBuilder<'_, '_, '_>, image: UiImage, margin: Rect<Val>, color: Option<Color>, name: impl Into<Cow<'static, str>>) {
    children.spawn_bundle(ImageBundle {
        style: Style {
            size: Size::new(INVENTORY_CELL_SIZE_VAL, INVENTORY_CELL_SIZE_VAL),
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
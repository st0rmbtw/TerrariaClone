use std::{collections::HashMap, borrow::Cow};

use autodefault::autodefault;
use bevy::{
    prelude::*, 
    ui::{AlignItems, Style, Val, FlexDirection, AlignContent, UiRect, Size, AlignSelf, UiImage, Interaction, FocusPolicy, JustifyContent, PositionType}, 
    hierarchy::{BuildChildren, ChildBuilder}, 
    input::{Input, mouse::MouseWheel}, 
    core::Name, 
    text::{Text, TextAlignment, TextStyle}
};
use bevy_inspector_egui::Inspectable;
use iyes_loopless::prelude::ConditionSet;
use smallvec::SmallVec;

use crate::{item::{Item, ITEM_DATA, ItemId, ItemData, Items}, util::{RectExtensions, EntityCommandsExtensions}, TRANSPARENT, state::GameState};

use super::{UiAssets, FontAssets, ItemAssets, HoveredInfo, ToggleExtraUiEvent, ExtraUiVisibility, UiVisibility};

pub const SPAWN_PLAYER_UI_LABEL: &str = "spawn_player_ui";

const ITEMS_STRING: &str = "Items";
const INVENTORY_STRING: &str = "Inventory";

// 5 is a total count of inventory rows. -1 because the hotbar is a first row
const INVENTORY_ROWS_COUNT: usize = 5 - 1;

// region: Inventory cell size
const INVENTORY_CELL_SIZE_F: f32 = 40.;
const INVENTORY_CELL_SIZE_BIGGER_F: f32 = INVENTORY_CELL_SIZE_F * 1.3;

const INVENTORY_CELL_SIZE_VAL: Val = Val::Px(INVENTORY_CELL_SIZE_F);
const INVENTORY_CELL_SIZE_BIGGER_VAL: Val = Val::Px(INVENTORY_CELL_SIZE_BIGGER_F);
// endregion

const CELL_COUNT_IN_ROW: usize = 10;

const HOTBAR_LENGTH: usize = 10;

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
                inventory.items.insert(0, Some(Items::COPPER_PICKAXE));

                inventory
            })

            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::InGame)
                    .with_system(scroll_select_item)
                    .with_system(select_item)
                    .with_system(set_selected_item)
                    .into()
            )
            
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::InGame)
                    .run_if_resource_equals(UiVisibility::default())
                    .with_system(update_inventory_visibility)
                    .with_system(update_selected_cell_size)
                    .with_system(update_selected_cell_image)
                    .with_system(update_selected_item_name_alignment)
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
    pub items: SmallVec::<[Option<Item>; 50]>,
    pub selected_item_index: usize
}

impl Inventory {
    fn get_item(&self, index: usize) -> Option<Item> {
        self.items.iter().nth(index).and_then(|a| *a)
    }

    fn select_item(&mut self, index: usize) {
        assert!(index <= 9);
        self.selected_item_index = index;
    }

    fn selected_item(&self) -> Option<Item> {
        self.get_item(self.selected_item_index)
    }
}

#[derive(Component, Default, Inspectable)]
struct InventoryUi;

#[derive(Component, Default)]
struct HotbarUi;
#[derive(Component)]
struct HotbarCellMarker;

#[derive(Component)]
struct SelectedItemNameMarker;

#[derive(Component)]
struct InventoryCellIndex(usize);

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

#[autodefault]
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
                top: Val::Px(5.)
            }
        },
        color: TRANSPARENT.into()
    })
    .insert(Name::new("Inventory Container"))
    .with_children(|children| {
        // region: Selected Item Name

        children.spawn_bundle(TextBundle {
            style: Style {
                margin: UiRect {
                    ..UiRect::horizontal(10.)
                },
                align_self: AlignSelf::Center
            },
            text: Text::from_section(
                "".to_string(), 
                TextStyle {
                    font: fonts.andy_bold.clone(),
                    font_size: 20.,
                    color: Color::WHITE,
                }
            ).with_alignment(TextAlignment::CENTER)
        })
        .insert(Name::new("Selected Item Name"))
        .insert(SelectedItemNameMarker);

        // endregion

        // region: Hotbar

        children.spawn_bundle(NodeBundle {
            style: Style {
                align_items: AlignItems::Center
            },
            color: TRANSPARENT.into()
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
                justify_content: JustifyContent::Center
            },
            visibility: Visibility {
                is_visible: false
            },
            color: TRANSPARENT.into()
        })
        .with_children(|children| {
            for j in 0..INVENTORY_ROWS_COUNT {
                children.spawn_bundle(NodeBundle {
                    style: Style {
                        margin: UiRect::vertical(2.),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center
                    },
                    color: TRANSPARENT.into()
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

fn update_inventory_visibility(
    mut query: Query<&mut Visibility, With<InventoryUi>>,
    mut events: EventReader<ToggleExtraUiEvent>
) {
    for event in events.iter() {
        for mut visibility in &mut query {
            visibility.is_visible = event.0;
        }
    }
}

fn update_selected_cell_size(
    inventory: Res<Inventory>,
    mut hotbar_cells: Query<(&InventoryCellIndex, &mut Style), With<HotbarCellMarker>>,
    visibility: Res<ExtraUiVisibility>
) {
    for (cell_index, mut style) in hotbar_cells.iter_mut() {
        let selected = cell_index.0 == inventory.selected_item_index;

        style.size = match selected {
            true if !visibility.0 => Size::new(INVENTORY_CELL_SIZE_BIGGER_VAL, INVENTORY_CELL_SIZE_BIGGER_VAL),
            _ => Size::new(INVENTORY_CELL_SIZE_VAL, INVENTORY_CELL_SIZE_VAL)
        };
    }
}

fn update_selected_cell_image(
    inventory: Res<Inventory>,
    mut hotbar_cells: Query<(&InventoryCellIndex, &mut UiImage), With<HotbarCellMarker>>,
    ui_assets: Res<UiAssets>
) {
    for (cell_index, mut image) in hotbar_cells.iter_mut() {
        let selected = cell_index.0 == inventory.selected_item_index;
        
        image.0 = if selected {
            ui_assets.selected_inventory_back.clone()
        } else {
            ui_assets.inventory_back.clone()
        }
    }
}

#[autodefault(except(InventoryCell))]
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
            align_self: AlignSelf::Center
        },
        image: cell_background.into()
    };

    background_image.color = (*background_image.color.0.set_a(0.8)).into();

    children
        .spawn_bundle(background_image)
        .with_children(|c| {
            c.spawn_bundle(ImageBundle {
                focus_policy: FocusPolicy::Pass,
                style: Style {
                    flex_direction: FlexDirection::Column,
                    margin: UiRect::all(Val::Px(8.))
                }
            })
            .insert(InventoryCellIndex(index))
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
                        align_content: AlignContent::FlexStart
                    },
                    color: TRANSPARENT.into(),
                    focus_policy: FocusPolicy::Pass
                }).with_children(|c| {

                    // Hotbar cell index
                    c.spawn_bundle(TextBundle {
                        focus_policy: FocusPolicy::Pass,
                        text: Text::from_section(
                            ((index + 1) % HOTBAR_LENGTH).to_string(), 
                            TextStyle { 
                                font: fonts.andy_bold.clone(),
                                font_size: 16.,
                                color: Color::WHITE
                            }
                        )
                    });

                    // Item stack
                    c.spawn_bundle(TextBundle {
                        style: Style {
                            align_self: AlignSelf::Center
                        },
                        focus_policy: FocusPolicy::Pass,
                        text: Text::from_section("", TextStyle { 
                            font: fonts.andy_regular.clone(),
                            font_size: 16.,
                            color: Color::WHITE
                        })
                    })
                    .insert(InventoryCellIndex(index))
                    .insert(InventoryItemStack::default());
                });
            }

        })
        .insert(Name::new(name))
        .insert(InventoryCellIndex(index))
        .insert_if(HotbarCellMarker, || { hotbar_cell })
        .insert(Interaction::default());
}

fn select_item(
    mut inventory: ResMut<Inventory>,
    input: Res<Input<KeyCode>>,
) {
    let digit = input
        .get_just_pressed()
        .find_map(|k| KEYCODE_TO_DIGIT.get(k));

    if let Some(index) = digit {
        inventory.select_item(*index);
    }
}

fn scroll_select_item(
    mut inventory: ResMut<Inventory>,
    mut events: EventReader<MouseWheel>
) {
    for event in events.iter() {
        let selected_item_index = inventory.selected_item_index as f32;
        let hotbar_length = HOTBAR_LENGTH as f32;
        let new_index = (((selected_item_index + event.y.signum()) % hotbar_length) + hotbar_length) % hotbar_length;

        inventory.select_item(new_index as usize);
    }
}

fn set_selected_item(
    inventory: Res<Inventory>,
    mut selected_item: ResMut<SelectedItem>,
) {
    if inventory.is_changed() {
        selected_item.0 = inventory.selected_item();
    }
}

fn update_selected_item_name_alignment(
    mut selected_item_name_query: Query<&mut Style, With<SelectedItemNameMarker>>,
    mut events: EventReader<ToggleExtraUiEvent>
) {
    let mut style = selected_item_name_query.single_mut();

    for event in events.iter() {
        style.align_self = if event.0 { AlignSelf::FlexStart } else { AlignSelf::Center }
    }
}

fn update_selected_item_name_text(
    mut selected_item_name_query: Query<&mut Text, With<SelectedItemNameMarker>>,
    current_item: Res<SelectedItem>,
    extra_ui_visibility: Res<ExtraUiVisibility>
) {
    if current_item.is_changed() || extra_ui_visibility.is_changed() {
        let mut text = selected_item_name_query.single_mut();

        text.sections[0].value = if extra_ui_visibility.0 {
            INVENTORY_STRING.to_string()
        } else {
            let name = current_item.0.map(|item| get_item_data_by_id(&item.id).name);

            name.map(|name| name.to_string()).unwrap_or(ITEMS_STRING.to_string())
        }
    }
}

fn update_cell(
    inventory: Res<Inventory>,
    mut item_images: Query<(&mut InventoryCellItemImage, &InventoryCellIndex)>,
    item_assets: Res<ItemAssets>,
) {
    if inventory.is_changed() {
        for (mut cell_image, cell_index) in &mut item_images {
            cell_image.0 = inventory
                .get_item(cell_index.0)
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
    mut query: Query<(&mut InventoryItemStack, &InventoryCellIndex)>
) {
    if inventory.is_changed() {
        for (mut item_stack, cell_index) in &mut query {
            if let Some(item) = inventory.items.get(cell_index.0).and_then(|item| *item) {
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
    query: Query<(&Interaction, &InventoryCellIndex), Changed<Interaction>>,
    inventory: Res<Inventory>,
    mut info: ResMut<HoveredInfo>
) {
    for (interaction, cell_index) in &query {
        if let Some(item) = inventory.get_item(cell_index.0) {
            let name = if *interaction != Interaction::None {
                get_item_data_by_id(&item.id).name
            } else {
                ""
            };

            info.0 = name.to_string();
        }
    }
}
mod components;
mod resources;
mod systems;

use bevy::{ui::{Val, Size}, prelude::{KeyCode, Plugin, App}, utils::HashMap};
pub use components::*;
use iyes_loopless::prelude::{ConditionSet, AppLooplessFixedTimestepExt, IntoConditionalSystem};
pub use resources::*;
pub use systems::*;

use crate::{state::GameState, items::Items};

use super::ui::UiVisibility;

pub const SPAWN_PLAYER_UI_LABEL: &str = "spawn_player_ui";

// 5 is a total count of inventory rows. -1 because the hotbar is a first row
const INVENTORY_ROWS_COUNT: usize = 5 - 1;

// region: Inventory cell size
const INVENTORY_CELL_SIZE_F: f32 = 40.;
const INVENTORY_CELL_SIZE_BIGGER_F: f32 = INVENTORY_CELL_SIZE_F * 1.3;

const INVENTORY_CELL_SIZE: Size = Size {
    width: Val::Px(INVENTORY_CELL_SIZE_F),
    height: Val::Px(INVENTORY_CELL_SIZE_F),
};

const INVENTORY_CELL_SIZE_SELECTED: Size = Size {
    width: Val::Px(INVENTORY_CELL_SIZE_BIGGER_F),
    height: Val::Px(INVENTORY_CELL_SIZE_BIGGER_F),
};
// endregion

pub(self) const CELL_COUNT_IN_ROW: usize = 10;

const HOTBAR_LENGTH: usize = 10;

pub struct PlayerInventoryPlugin;

impl Plugin for PlayerInventoryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedItem>()
            .insert_resource({
                let mut inventory = Inventory::default();
                inventory.add_item(Items::COPPER_PICKAXE);
                inventory.add_item(Items::DIRT_BLOCK.with_stack(999));
                inventory.add_item(Items::STONE_BLOCK.with_stack(999));

                inventory
            })
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(GameState::InGame)
                    .with_system(scroll_select_inventory_item)
                    .with_system(select_inventory_cell)
                    .with_system(set_selected_item)
                    .into(),
            )
            .add_fixed_timestep_system("fixed_update", 0, use_item.run_in_state(GameState::InGame))
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
                    .with_system(update_item_amount)
                    .with_system(update_item_amount_text)
                    .into(),
            );
    }
}

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
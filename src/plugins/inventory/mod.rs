mod components;
mod resources;
mod systems;
mod util;

use bevy::prelude::{Plugin, App, IntoSystemConfigs, Res, in_state, Update, FixedUpdate, resource_changed, resource_equals};
pub(crate) use components::*;
pub(crate) use resources::*;
pub(crate) use systems::*;

use crate::{common::state::GameState, items::Items};

use super::ui::UiVisibility;

// 5 is the total amount of inventory rows. -1 because the hotbar takes the first row
const INVENTORY_ROWS: usize = 5 - 1;

const INVENTORY_CELL_SIZE: f32 = 40.;
const INVENTORY_CELL_SIZE_SELECTED: f32 = INVENTORY_CELL_SIZE * 1.3;

const CELL_COUNT_IN_ROW: usize = 10;

const ITEM_ROTATION: f32 = 1.7;

pub struct PlayerInventoryPlugin;
impl Plugin for PlayerInventoryPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedItem>();
        app.init_resource::<SwingItemCooldown>();
        app.init_resource::<SwingItemCooldownMax>();

        app.insert_resource(UseItemAnimationIndex::default());
        app.insert_resource(PlayerUsingItem(false));
        app.insert_resource(SwingAnimation(false));
        app.insert_resource({
                let mut inventory = Inventory::default();
                inventory.add_item(Items::COPPER_PICKAXE);
                inventory.add_item(Items::COPPER_AXE);
                inventory.add_item(Items::DIRT_BLOCK.with_max_stack());
                inventory.add_item(Items::STONE_BLOCK.with_max_stack());
                inventory.add_item(Items::GRASS_SEEDS.with_max_stack());

                inventory
            });

        app.add_systems(Update, scroll_select_inventory_item.run_if(in_state(GameState::InGame)));
        app.add_systems(Update, select_inventory_cell.run_if(in_state(GameState::InGame)));
        app.add_systems(Update, set_selected_item.run_if(in_state(GameState::InGame)));

        app.add_systems(FixedUpdate, use_item.run_if(in_state(GameState::InGame)));

        app.add_systems(
            Update,
            (
                update_player_using_item,
                set_using_item_image.run_if(resource_changed::<SelectedItem>()),
                set_using_item_visibility(false),
            )
            .run_if(in_state(GameState::InGame))
        );

        app.add_systems(
            FixedUpdate,
            (
                play_swing_sound,
                update_swing_cooldown,
                (
                    update_use_item_animation_index,
                    set_using_item_position,
                    set_using_item_rotation,
                    set_using_item_visibility(true),
                    update_sprite_index,
                ).chain(),
                stop_swing_animation
            )
            .chain()
            .run_if(in_state(GameState::InGame))
            .run_if(|res: Res<SwingAnimation>| **res == true)
        );

        app.add_systems(
            Update,
            (
                on_extra_ui_visibility_toggle,
                update_selected_cell_size,
                update_selected_cell_image,
                update_selected_item_name_alignment,
                update_selected_item_name_text,
                update_hoverable,
                (
                    update_cell,
                    update_cell_image,
                ).chain(),
                (
                    update_item_amount,
                    update_item_amount_text,
                ).chain()
            )
            .run_if(in_state(GameState::InGame))
            .run_if(resource_equals(UiVisibility::VISIBLE))
        );
    }
}
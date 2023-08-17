mod components;
mod resources;
mod systems;
mod util;

use bevy::prelude::{Plugin, App, IntoSystemConfigs, Update, FixedUpdate, OnExit, Condition, Commands, resource_exists_and_changed, resource_added, resource_exists_and_equals, resource_changed};
pub(crate) use components::*;
pub(crate) use resources::*;
pub(crate) use systems::*;

use crate::{common::{state::GameState, systems::set_visibility}, items::{ItemStack, Tool, Axe, Pickaxe, Seed}, world::block::BlockType, InGameSystemSet};

use super::ui::ExtraUiVisibility;

// 5 is the total amount of inventory rows. -1 because the hotbar takes the first row
const INVENTORY_ROWS: usize = 5 - 1;

const INVENTORY_CELL_SIZE: f32 = 40.;
const INVENTORY_CELL_SIZE_SELECTED: f32 = INVENTORY_CELL_SIZE * 1.3;

const CELL_COUNT_IN_ROW: usize = 10;

const ITEM_ROTATION: f32 = 1.7;

pub struct PlayerInventoryPlugin;
impl Plugin for PlayerInventoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnExit(GameState::WorldLoading), setup);

        app.add_systems(
            FixedUpdate,
            (
                use_item,
                stop_swing_animation
            )
            .in_set(InGameSystemSet::FixedUpdate)
        );

        app.add_systems(
            FixedUpdate,
            (
                play_swing_sound,
                update_swing_cooldown,
                update_use_item_animation_index,
                update_sprite_index,
                set_using_item_position,
                set_using_item_rotation,
                set_using_item_visibility(true),
                reset_swing_animation,
            )
            .chain()
            .in_set(InGameSystemSet::FixedUpdate)
            .run_if(resource_exists_and_equals(SwingAnimation(true)))
        );

        app.add_systems(
            Update,
            (
                update_player_using_item,
                set_using_item_image.run_if(resource_exists_and_changed::<SelectedItem>()),
                set_using_item_visibility(false)
            )
            .in_set(InGameSystemSet::Update)
        );

        app.add_systems(
            Update,
            (
                scroll_select_inventory_item,
                select_inventory_cell,
                set_selected_item.run_if(resource_exists_and_changed::<Inventory>()),
            )
            .in_set(InGameSystemSet::Update)
        );

        app.add_systems(
            Update,
            (
                set_visibility::<InventoryUi, ExtraUiVisibility>,
                trigger_inventory_changed.run_if(resource_changed::<ExtraUiVisibility>()),
                update_selected_item_name_alignment,
                update_selected_item_name_text,
                (
                    update_selected_cell_size,
                    update_selected_cell_image,
                    update_hoverable
                )
                .distributive_run_if(
                    resource_exists_and_changed::<Inventory>().or_else(resource_added::<Inventory>())
                ),
                (update_cell, update_cell_image).chain(),
                (update_item_amount, update_item_amount_text).chain(),
            )
            .in_set(InGameSystemSet::Update)
        );
    }
}

fn setup(mut commands: Commands) {
    commands.init_resource::<SelectedItem>();
    commands.init_resource::<SwingItemCooldown>();
    commands.init_resource::<SwingItemCooldownMax>();
    commands.insert_resource(UseItemAnimationIndex::default());
    commands.insert_resource(PlayerUsingItem(false));
    commands.insert_resource(SwingAnimation(false));
    
    let mut inventory = Inventory::default();
    inventory.add_item(ItemStack::new_tool(Tool::Pickaxe(Pickaxe::CopperPickaxe)));
    inventory.add_item(ItemStack::new_tool(Tool::Axe(Axe::CopperAxe)));
    inventory.add_item(ItemStack::new_block(BlockType::Dirt).with_max_stack());
    inventory.add_item(ItemStack::new_block(BlockType::Stone).with_max_stack());
    inventory.add_item(ItemStack::new_seed(Seed::Grass).with_max_stack());

    commands.insert_resource(inventory);
}
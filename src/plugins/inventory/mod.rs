mod components;
mod resources;
mod systems;
mod util;

use bevy::prelude::{Plugin, App, IntoSystemConfigs, Update, FixedUpdate, OnExit, Commands, resource_exists_and_changed, resource_exists_and_equals};
pub(crate) use components::*;
pub(crate) use resources::*;

use crate::{common::state::GameState, items::{ItemStack, Tool, Axe, Pickaxe, Seed}, world::block::BlockType};

use super::InGameSystemSet;

const ITEM_ROTATION: f32 = 1.7;

pub struct PlayerInventoryPlugin;
impl Plugin for PlayerInventoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnExit(GameState::WorldLoading), setup);

        app.add_systems(
            FixedUpdate,
            (
                systems::use_item,
                systems::stop_swing_animation
            )
            .in_set(InGameSystemSet::FixedUpdate)
        );

        app.add_systems(
            FixedUpdate,
            (
                systems::play_swing_sound,
                systems::update_swing_cooldown,
                systems::update_use_item_animation_index,
                systems::update_sprite_index,
                systems::set_using_item_position,
                systems::set_using_item_rotation,
                systems::set_using_item_visibility(true),
                systems::reset_swing_animation,
            )
            .chain()
            .in_set(InGameSystemSet::FixedUpdate)
            .run_if(resource_exists_and_equals(SwingAnimation(true)))
        );

        app.add_systems(
            Update,
            (
                (
                    systems::update_player_using_item,
                    systems::start_swing_animation
                ).chain(),
                systems::set_using_item_image.run_if(resource_exists_and_changed::<SelectedItem>()),
                systems::set_using_item_visibility(false)
            )
            .in_set(InGameSystemSet::Update)
        );

        app.add_systems(
            Update,
            (
                systems::scroll_select_inventory_item,
                systems::select_inventory_cell,
                systems::set_selected_item.run_if(resource_exists_and_changed::<Inventory>()),
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
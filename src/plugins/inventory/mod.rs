mod components;
mod resources;
mod systems;
mod util;

use bevy::{prelude::{Plugin, App, IntoSystemConfigs, Update, FixedUpdate, OnExit, Commands, resource_exists_and_changed, resource_exists_and_equals, Vec2, not}, math::vec2};
pub(crate) use components::*;
pub(crate) use resources::*;

use crate::{common::{state::GameState, conditions::mouse_over_ui}, items::{ItemStack, ItemTool, Axe, Pickaxe, ItemSeed, ItemBlock}};

use super::{InGameSystemSet, ui::InventoryUiVisibility};

const ITEM_ROTATION: f32 = 1.7;

const ITEM_ANIMATION_POINTS: [Vec2; 3] = [vec2(-7.5, 11.0), vec2(6.0, 7.5), vec2(7.0, -4.0)];

pub struct PlayerInventoryPlugin;
impl Plugin for PlayerInventoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnExit(GameState::WorldLoading), setup);
        app.add_systems(OnExit(GameState::InGame), cleanup);

        app.add_systems(
            FixedUpdate,
            systems::use_item.in_set(InGameSystemSet::FixedUpdate)
        );

        app.add_systems(
            FixedUpdate,
            (
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
                .run_if(resource_exists_and_equals(SwingAnimation(true))),
                
                systems::stop_swing_animation
            )
            .chain()
            .in_set(InGameSystemSet::FixedUpdate)
        );

        app.add_systems(
            Update,
            (
                (
                    systems::update_player_using_item
                        .run_if(not(mouse_over_ui)),
                    systems::start_swing_animation
                ).chain(),
                (
                    systems::scroll_select_inventory_item
                        .run_if(resource_exists_and_equals(InventoryUiVisibility::HIDDEN)),
                    systems::select_inventory_cell,
                    systems::set_selected_item.run_if(resource_exists_and_changed::<Inventory>())
                )
                .chain(),
                systems::set_using_item_image.run_if(resource_exists_and_changed::<SelectedItem>()),
                systems::set_using_item_visibility(false),

                systems::drop_item_stack,
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
    inventory.add_item_stack(ItemStack::new_tool(ItemTool::Pickaxe(Pickaxe::CopperPickaxe)));
    inventory.add_item_stack(ItemStack::new_tool(ItemTool::Axe(Axe::CopperAxe)));
    inventory.add_item_stack(ItemStack::new_block(ItemBlock::Dirt).with_max_stack());
    inventory.add_item_stack(ItemStack::new_block(ItemBlock::Stone).with_max_stack());
    inventory.add_item_stack(ItemStack::new_block(ItemBlock::Wood).with_max_stack());
    inventory.add_item_stack(ItemStack::new_seed(ItemSeed::Grass).with_max_stack());

    commands.insert_resource(inventory);
}

fn cleanup(mut commands: Commands) {
    commands.remove_resource::<SelectedItem>();
    commands.remove_resource::<SwingItemCooldown>();
    commands.remove_resource::<SwingItemCooldownMax>();
    commands.remove_resource::<UseItemAnimationIndex>();
    commands.remove_resource::<PlayerUsingItem>();
    commands.remove_resource::<SwingAnimation>();
    commands.remove_resource::<Inventory>();
}
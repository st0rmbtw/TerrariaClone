mod components;
mod resources;
mod systems;
mod util;

use bevy::{ui::{Val, Size}, prelude::{Plugin, App, OnUpdate, IntoSystemConfigs, IntoSystemAppConfig, CoreSchedule, IntoSystemConfig, Res, in_state, IntoSystemAppConfigs, SystemSet}};
pub(crate) use components::*;
pub(crate) use resources::*;
pub(crate) use systems::*;

use crate::{common::state::GameState, items::Items};

use super::ui::UiVisibility;

// 5 is the total amount of inventory rows. -1 because the hotbar is a first row
const INVENTORY_ROWS: usize = 5 - 1;

// region: Inventory cell size
const INVENTORY_CELL_SIZE_F: f32 = 40.;
const INVENTORY_CELL_SIZE_BIGGER_F: f32 = INVENTORY_CELL_SIZE_F * 1.3;

const INVENTORY_CELL_SIZE: Size = Size::all(Val::Px(INVENTORY_CELL_SIZE_F));
const INVENTORY_CELL_SIZE_SELECTED: Size = Size::all(Val::Px(INVENTORY_CELL_SIZE_BIGGER_F));
// endregion

pub(self) const CELL_COUNT_IN_ROW: usize = 10;

const HOTBAR_LENGTH: usize = 10;

const ITEM_ROTATION: f32 = 1.7;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
enum UseItemAnimationSet {
    UpdateSwingCooldown,
    PlayAnimation,
    SetCooldown
}

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

        app.add_system(scroll_select_inventory_item.in_set(OnUpdate(GameState::InGame)));
        app.add_system(select_inventory_cell.in_set(OnUpdate(GameState::InGame)));
        app.add_system(set_selected_item.in_set(OnUpdate(GameState::InGame)));

        app.add_system(
            use_item
                .in_schedule(CoreSchedule::FixedUpdate)
                .run_if(in_state(GameState::InGame))
        );

        app.add_systems(
            (
                update_player_using_item,
                set_using_item_image,
                set_using_item_visibility(false),
            )
            .in_set(OnUpdate(GameState::InGame))
        );

        app.add_system(
            play_swing_sound
                .run_if(in_state(GameState::InGame))
                .run_if(|res: Res<SwingAnimation>| **res == true)
                .in_schedule(CoreSchedule::FixedUpdate)
        );

        app.add_system(
            update_swing_cooldown
                .run_if(in_state(GameState::InGame))
                .in_set(UseItemAnimationSet::UpdateSwingCooldown)
                .in_schedule(CoreSchedule::FixedUpdate)
                .after(play_swing_sound)
        );

        app.add_systems(
            (
                update_use_item_animation_index,
                set_using_item_position,
                set_using_item_rotation,
                set_using_item_visibility(true),
                update_sprite_index,
            )
            .chain()
            .distributive_run_if(|res: Res<SwingAnimation>| **res == true)
            .in_set(UseItemAnimationSet::PlayAnimation)
            .after(UseItemAnimationSet::UpdateSwingCooldown)
            .in_schedule(CoreSchedule::FixedUpdate)
        );

        app.add_system(
            stop_swing_animation
                .run_if(|res: Res<SwingAnimation>| **res == true)
                .run_if(in_state(GameState::InGame))
                .in_schedule(CoreSchedule::FixedUpdate)
                .in_set(UseItemAnimationSet::SetCooldown)
                .after(UseItemAnimationSet::PlayAnimation)
        );

        app.add_systems(
            (
                update_inventory_visibility,
                update_selected_cell_size,
                update_selected_cell_image,
                update_selected_item_name_alignment,
                update_selected_item_name_text,
                update_cell,
                update_cell_image,
                inventory_cell_background_hover,
                update_item_amount,
                update_item_amount_text,
            )
            .chain()
            .in_set(OnUpdate(GameState::InGame))
            .distributive_run_if(|res: Res<UiVisibility>| *res == UiVisibility::default())
        );
    }
}
pub(in crate::plugins::ui) mod systems;
mod components;

use bevy::prelude::{Plugin, App, IntoSystemConfigs, resource_exists_and_changed, resource_added, Update, Condition, resource_exists_and_equals};

use crate::{common::systems::{set_visibility, set_visibility_negated}, plugins::{inventory::Inventory, InGameSystemSet, ui::SettingsMenuVisibility}};

use crate::plugins::ui::InventoryUiVisibility;

const INVENTORY_ROWS: usize = 5 - 1;

const HOTBAR_SLOT_SIZE: f32 = 40.;
const INVENTORY_SLOT_SIZE: f32 = HOTBAR_SLOT_SIZE * 1.1;
const HOTBAR_SLOT_SIZE_SELECTED: f32 = HOTBAR_SLOT_SIZE * 1.3;

pub(crate) const SLOT_COUNT_IN_ROW: usize = 10;

pub(in crate::plugins::ui) struct InventoryUiPlugin;
impl Plugin for InventoryUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                set_visibility::<components::InventoryUi, InventoryUiVisibility>,
                set_visibility_negated::<components::InventoryUiContainer, SettingsMenuVisibility>,
                systems::trigger_inventory_changed.run_if(resource_exists_and_changed::<InventoryUiVisibility>()),
                systems::update_selected_item_name_alignment,
                systems::update_selected_item_name_text,
                (
                    systems::update_slot_size,
                    systems::update_slot_background_image,
                    systems::update_hoverable,
                    systems::update_slot_index_text,

                    (
                        systems::update_slot_item_image,
                        systems::update_slot_image
                    ).chain(),
                    (
                        systems::update_item_amount,
                        systems::update_item_amount_text,
                    ).chain()
                )
                .distributive_run_if(
                    resource_exists_and_changed::<Inventory>().or_else(resource_added::<Inventory>())
                ),

                (
                    systems::take_item,
                    systems::put_item,
                )
                .run_if(resource_exists_and_equals(InventoryUiVisibility::VISIBLE))
            )
            .in_set(InGameSystemSet::Update)
        );
    }
}

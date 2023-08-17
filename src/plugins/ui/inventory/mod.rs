pub(super) mod systems;
mod components;

use bevy::prelude::{Plugin, App, resource_changed, IntoSystemConfigs, resource_exists_and_changed, resource_added, Update, Condition};

use crate::{common::systems::set_visibility, plugins::inventory::Inventory, InGameSystemSet};

use super::ExtraUiVisibility;

// 5 is the total amount of inventory rows. -1 because the hotbar takes the first row
const INVENTORY_ROWS: usize = 5 - 1;

const INVENTORY_CELL_SIZE: f32 = 40.;
const INVENTORY_CELL_SIZE_SELECTED: f32 = INVENTORY_CELL_SIZE * 1.3;

pub(crate) const CELL_COUNT_IN_ROW: usize = 10;

pub(super) struct InventoryUiPlugin;
impl Plugin for InventoryUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                set_visibility::<components::InventoryUi, ExtraUiVisibility>,
                systems::trigger_inventory_changed.run_if(resource_changed::<ExtraUiVisibility>()),
                systems::update_selected_item_name_alignment,
                systems::update_selected_item_name_text,
                (
                    systems::update_selected_cell_size,
                    systems::update_selected_cell_image,
                    systems::update_hoverable
                )
                .distributive_run_if(
                    resource_exists_and_changed::<Inventory>().or_else(resource_added::<Inventory>())
                ),
                (
                    (
                        systems::update_cell,
                        systems::update_cell_image
                    ).chain(),
                    (
                        systems::update_item_amount,
                        systems::update_item_amount_text
                    ).chain(),
                )
                .run_if(resource_exists_and_changed::<Inventory>())
            )
            .in_set(InGameSystemSet::Update)
        );
    }
}

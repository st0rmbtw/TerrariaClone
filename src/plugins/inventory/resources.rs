use bevy::{prelude::{Resource, Deref, DerefMut, ReflectResource}, reflect::Reflect};

use crate::{items::ItemStack, plugins::ui::inventory::CELL_COUNT_IN_ROW};

#[derive(Resource, Default, Deref, DerefMut)]
pub(crate) struct SelectedItem(pub Option<ItemStack>);

#[derive(Resource, Default, Deref, DerefMut)]
pub(super) struct SwingItemCooldown(pub u32);

#[derive(Resource, Default, Deref, DerefMut)]
pub(super) struct SwingItemCooldownMax(pub u32);

#[derive(Resource, PartialEq, Clone, Copy, Deref, DerefMut)]
pub(super) struct PlayerUsingItem(pub bool);

#[derive(Resource, PartialEq, Clone, Copy, Deref, DerefMut)]
pub(crate) struct SwingAnimation(pub bool);

#[derive(Resource, Default, Clone, Copy, Deref, DerefMut, Reflect)]
#[reflect(Resource)]
pub(crate) struct UseItemAnimationIndex(usize);

#[derive(Resource)]
pub(crate) struct Inventory {
    pub(crate) items: [Option<ItemStack>; 50],
    pub(crate) selected_slot: usize,
}

impl Default for Inventory {
    fn default() -> Self {
        Self { items: [None; 50], selected_slot: 0 }
    }
}

impl Inventory {
    pub fn get_item(&self, slot: usize) -> Option<ItemStack> {
        self.items.iter().nth(slot).and_then(|a| *a)
    }

    pub fn get_item_mut(&mut self, slot: usize) -> Option<&mut ItemStack> {
        self.items.iter_mut().nth(slot).and_then(|a| a.as_mut())
    }

    pub fn remove_item(&mut self, slot: usize) {
        self.items[slot] = None;
    }

    /// Returns `true` if the `slot` is less than [`CELL_COUNT_IN_ROW`] and is not the same as the selected_slot
    pub fn select_item(&mut self, slot: usize) -> bool {
        if slot < CELL_COUNT_IN_ROW && slot != self.selected_slot {
            self.selected_slot = slot;
            return true;
        }

        false
    }

    pub fn selected_item(&self) -> Option<ItemStack> {
        self.get_item(self.selected_slot)
    }

    pub fn consume_item(&mut self, slot: usize) {
        let item_option = self.get_item_mut(slot);
        if let Some(item) = item_option {
            if item.stack > 1 {
                item.stack -= 1;
            } else {
                self.remove_item(slot);
            }
        }
    }

    pub fn add_item(&mut self, item: ItemStack) {
        for inv_item_option in self.items.iter_mut() {
            match inv_item_option {
                Some(inv_item) if inv_item.item == item.item => {
                    let new_stack = inv_item.stack + item.stack;

                    if new_stack < inv_item.item.max_stack() {
                        inv_item.stack += new_stack;
                        break;
                    }
                },
                None => {
                    *inv_item_option = Some(item);
                    break;
                },
                _ => ()
            }
        }
    }
}

use bevy::{prelude::{Resource, Deref, DerefMut, ReflectResource}, reflect::Reflect};

use crate::{items::{ItemStack, Stack}, plugins::ui::ingame::inventory::SLOT_COUNT_IN_ROW};

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
    pub(crate) slots: [Option<ItemStack>; 50],
    pub(crate) selected_slot: usize,
}

impl Default for Inventory {
    fn default() -> Self {
        Self { slots: [None; 50], selected_slot: 0 }
    }
}

impl Inventory {
    pub fn get_item(&self, slot: usize) -> Option<ItemStack> {
        debug_assert!((0..50).contains(&slot));
        self.slots[slot]
    }

    pub fn get_item_mut(&mut self, slot: usize) -> Option<&mut ItemStack> {
        debug_assert!((0..50).contains(&slot));
        self.slots[slot].as_mut()
    }

    pub fn remove_item(&mut self, slot: usize) {
        debug_assert!((0..50).contains(&slot));
        self.slots[slot] = None;
    }

    /// Returns `true` if the `slot` is less than [`CELL_COUNT_IN_ROW`] and is not the same as the selected_slot
    pub fn select_item(&mut self, slot: usize) -> bool {
        if slot < SLOT_COUNT_IN_ROW && slot != self.selected_slot {
            self.selected_slot = slot;
            return true;
        }

        false
    }

    #[inline(always)]
    pub fn selected_item(&self) -> Option<ItemStack> {
        self.get_item(self.selected_slot)
    }

    #[inline(always)]
    pub fn consume_item(&mut self, slot: usize) {
        self.consume_item_impl(slot, 1);
    }

    fn consume_item_impl(&mut self, slot: usize, stack: Stack) -> Option<ItemStack> {
        let item = self.get_item_mut(slot)?;
        let item_copy = *item;

        if item.stack > stack {
            item.stack -= stack;
        } else {
            self.remove_item(slot);
        }

        Some(item_copy.with_stack(stack))
    }

    // Returns the amount of items added to the inventory
    pub fn add_item_stack(&mut self, new_item: ItemStack) -> u16 {
        let mut remaining = new_item.stack;

        for inv_item_option in self.slots.iter_mut() {
            match inv_item_option {
                Some(inv_item) if inv_item.item == new_item.item && inv_item.stack < inv_item.item.max_stack() => {
                    let new_stack = inv_item.stack + remaining;

                    if new_stack <= inv_item.item.max_stack() {
                        inv_item.stack = new_stack;
                        return new_item.stack;
                    } else {
                        inv_item.stack += remaining % inv_item.item.max_stack();
                        remaining -= remaining % inv_item.item.max_stack();
                    }
                },
                None => {
                    *inv_item_option = Some(new_item);
                    return new_item.stack;
                },
                _ => continue
            }
        }

        0
    }

    pub fn can_be_added(&self, new_item: ItemStack) -> bool {
        let mut remaining = new_item.stack;

        for item_option in self.slots.iter() {
            let Some(item_stack) = item_option else { return true; };

            if remaining == 0 { return true; }

            if new_item.item != item_stack.item { continue; }
            if item_stack.stack == item_stack.item.max_stack() { continue; }

            let new_stack = item_stack.stack + remaining;

            if new_stack <= item_stack.item.max_stack() {
                return true;
            } else {
                remaining -= remaining % item_stack.item.max_stack();
            }
        }

        false
    }

    pub fn drop_item(&mut self, slot: usize) -> Option<ItemStack> {
        let item_stack = self.get_item(slot)?;
        self.consume_item_impl(slot, item_stack.stack)
    }

    pub fn empty_slots_count(&self) -> u8 {
        self.slots.iter().filter(|slot| slot.is_none()).count() as u8
    }
}

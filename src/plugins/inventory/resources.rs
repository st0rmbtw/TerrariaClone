use crate::items::ItemStack;

pub struct Inventory {
    pub(super) items: [Option<ItemStack>; 50],
    pub selected_slot: usize,
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

    pub fn select_item(&mut self, slot: usize) {
        assert!(slot <= 9);
        self.selected_slot = slot;
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
                    if (inv_item.stack + item.stack) < inv_item.item.max_stack() {
                        inv_item.stack += item.stack;
                    }
                    break;
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

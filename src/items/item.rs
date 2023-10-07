use super::{ItemTool, ItemSeed, ItemBlock};

pub(crate) type Stack = u16;

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Item {
    Tool(ItemTool),
    Block(ItemBlock),
    Seed(ItemSeed)
}

impl Item {
    pub(crate) const fn consumable(&self) -> bool {
        match self {
            Item::Tool(_) => false,
            _ => true
        }
    }

    pub(crate) const fn max_stack(&self) -> Stack {
        match self {
            Item::Tool(_) => 1,
            _ => 9999
        }
    }

    pub(crate) const fn swing_cooldown(&self) -> u32 {
        match self {
            Item::Tool(tool) => tool.swing_cooldown(),
            Item::Block(_) | Item::Seed(_) => 15,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct ItemStack {
    pub item: Item,
    pub stack: Stack
}

impl ItemStack {
    pub(crate) const fn new_block(block: ItemBlock) -> Self {
        ItemStack { item: Item::Block(block), stack: 1 }
    }

    pub(crate) const fn new_tool(tool: ItemTool) -> Self {
        ItemStack { item: Item::Tool(tool), stack: 1 }
    }

    pub(crate) const fn new_seed(seed: ItemSeed) -> Self {
        ItemStack { item: Item::Seed(seed), stack: 1 }
    }

    pub(crate) fn with_stack(mut self, stack: Stack) -> Self {
        debug_assert!(stack <= self.item.max_stack());
        self.stack = stack;
        self
    }

    #[inline(always)]
    pub(crate) fn with_max_stack(self) -> Self {
        self.with_stack(self.item.max_stack())
    }
}
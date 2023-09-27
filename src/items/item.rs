use crate::world::block::BlockType;

use super::{Tool, Seed};

type Stack = u16;

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Item {
    Tool(Tool),
    Block(BlockType),
    Seed(Seed)
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
    pub(crate) const fn new_block(block_type: BlockType) -> Self {
        ItemStack { item: Item::Block(block_type), stack: 1 }
    }

    pub(crate) const fn new_tool(tool: Tool) -> Self {
        ItemStack { item: Item::Tool(tool), stack: 1 }
    }

    pub(crate) const fn new_seed(seed: Seed) -> Self {
        ItemStack { item: Item::Seed(seed), stack: 1 }
    }

    pub(crate) fn with_stack(mut self, stack: Stack) -> Self {
        self.stack = stack;
        self
    }

    #[inline(always)]
    pub(crate) fn with_max_stack(self) -> Self {
        self.with_stack(self.item.max_stack())
    }
}
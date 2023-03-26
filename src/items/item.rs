use bevy::{math::vec2, prelude::Vec2};

use crate::plugins::world::Block;

use super::{Pickaxe, Tool, Axe};

type Stack = u16;

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum Item {
    Tool(Tool),
    Block(Block)
}

impl Item {
    pub(crate) fn consumable(&self) -> bool {
        match self {
            Item::Tool(_) => false,
            _ => true
        }
    }

    pub(crate) fn max_stack(&self) -> Stack {
        match self {
            Item::Tool(_) => 1,
            Item::Block(_) => 999
        }
    }

    pub(crate) fn swing_cooldown(&self) -> u32 {
        match self {
            Item::Tool(tool) => tool.swing_cooldown(),
            Item::Block(_) => 15,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct ItemStack {
    pub item: Item,
    pub stack: Stack
}

impl ItemStack {
    pub(crate) fn with_stack(self, stack: Stack) -> Self {
        Self { stack, ..self }
    }
}

pub(crate) struct Items;
impl Items {
    pub(crate) const COPPER_PICKAXE: ItemStack = ItemStack {
        item: Item::Tool(Tool::Pickaxe(Pickaxe::CopperPickaxe)),
        stack: 1,
    };

    pub(crate) const COPPER_AXE: ItemStack = ItemStack {
        item: Item::Tool(Tool::Axe(Axe::CopperAxe)),
        stack: 1,
    };

    pub(crate) const DIRT_BLOCK: ItemStack = ItemStack {
        item: Item::Block(Block::Dirt),
        stack: 1,
    };

    pub(crate) const STONE_BLOCK: ItemStack = ItemStack {
        item: Item::Block(Block::Stone),
        stack: 1,
    };
}

pub(crate) fn get_animation_points() -> [Vec2; 3] {
    [vec2(-7.5, 11.0), vec2(6.0, 7.5), vec2(7.0, -4.0)]
}
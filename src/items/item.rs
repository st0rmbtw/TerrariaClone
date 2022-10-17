use bevy::{math::vec2, prelude::Vec2};

use crate::{block::Block, language::LanguageContent};

use super::Pickaxe;

pub type ItemId = u16;
pub type Stack = u16;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Item {
    Pickaxe(Pickaxe),
    Block(Block)
}

impl Item {
    pub fn consumable(&self) -> bool {
        match self {
            Item::Pickaxe(_) => false,
            _ => true
        }
    }

    pub fn name(&self, language_content: &LanguageContent) -> String {
        match self {
            Item::Pickaxe(pickaxe) => language_content.pickaxe_name(*pickaxe),
            Item::Block(block) => language_content.block_name(*block),
        }
    }

    pub fn max_stack(&self) -> Stack {
        match self {
            Item::Pickaxe(_) => 1,
            Item::Block(_) => 999
        }
    }
}

#[derive(Clone, Copy)]
pub struct ItemStack {
    pub item: Item,
    pub stack: Stack
}

impl ItemStack {
    pub fn with_stack(self, stack: Stack) -> Self {
        Self { 
            stack,
            ..self
        }
    }
}

pub struct Items;

impl Items {
    pub const COPPER_PICKAXE: ItemStack = ItemStack {
        item: Item::Pickaxe(Pickaxe::CopperPickaxe),
        stack: 1,
    };

    pub const DIRT_BLOCK: ItemStack = ItemStack {
        item: Item::Block(Block::Dirt),
        stack: 1,
    };

    pub const STONE_BLOCK: ItemStack = ItemStack {
        item: Item::Block(Block::Stone),
        stack: 1,
    };
}

pub fn get_animation_points(item: Item) -> Vec<Vec2> {
    match item {
        _ => vec![vec2(-5., 4.), vec2(5., 3.), vec2(5., -7.)]
    }
}
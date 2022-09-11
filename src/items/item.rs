use std::collections::HashMap;

use bevy::{math::vec2, prelude::Vec2};

use crate::block::Block;

use super::Pickaxe;

pub type ItemId = u16;
pub type Amount = u16;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Item {
    Pickaxe(Pickaxe),
    Block(Block)
}

#[derive(Clone, Copy)]
pub struct ItemStack {
    pub item: Item,
    pub amount: Amount
}

impl ItemStack {
    pub fn with_stack(self, amount: Amount) -> Self {
        Self { 
            amount,
            ..self
        }
    }
}

pub struct ItemMeta {
    pub name: &'static str,
    pub max_amount: Amount
}

lazy_static! {
    pub static ref ITEM_DATA: HashMap<Item, ItemMeta> = HashMap::from([
        (Item::Block(Block::Dirt), ItemMeta {
            name: "Dirt Block",
            max_amount: 999
        }),
        (Item::Block(Block::Stone), ItemMeta {
            name: "Stone Block",
            max_amount: 999
        }),
        (Item::Pickaxe(Pickaxe::CopperPickaxe), ItemMeta {
            name: "Copper Pickaxe",
            max_amount: 1
        }),
    ]);
}

pub struct Items;

impl Items {
    pub const COPPER_PICKAXE: ItemStack = ItemStack {
        item: Item::Pickaxe(Pickaxe::CopperPickaxe),
        amount: 1,
    };

    pub const DIRT_BLOCK: ItemStack = ItemStack {
        item: Item::Block(Block::Dirt),
        amount: 1,
    };

    pub const STONE_BLOCK: ItemStack = ItemStack {
        item: Item::Block(Block::Stone),
        amount: 1,
    };
}

pub fn get_animation_points(item: Item) -> Vec<Vec2> {
    match item {
        _ => vec![vec2(-5., 4.), vec2(5., 3.), vec2(5., -7.)]
    }
}

pub fn get_item_data<'a>(item: &Item) -> &'a ItemMeta {
    ITEM_DATA.get(item).expect("Item not found")
}
use std::collections::HashMap;

use bevy::{math::vec2, prelude::Vec2};

use crate::block::Block;

pub type ItemId = u16;
pub type Stack = u16;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemType {
    Pickaxe,
    Block(Block)
}

#[derive(Clone, Copy)]
pub struct Item {
    pub id: ItemId,
    pub item_type: ItemType,
    pub stack: Stack
}

impl Item {
    pub fn with_stack(self, stack: Stack) -> Self {
        Self { 
            stack,
            ..self
        }
    }
}

pub struct ItemData {
    pub name: &'static str,
    pub max_stack: Stack
}

lazy_static! {
    pub static ref ITEM_DATA: HashMap<ItemId, ItemData> = HashMap::from([
        (2, ItemData {
            name: "Dirt Block",
            max_stack: 999
        }),
        (3, ItemData {
            name: "Stone Block",
            max_stack: 999
        }),
        (3509, ItemData {
            name: "Copper Pickaxe",
            max_stack: 1
        }),
    ]);
}

pub struct Items;

impl Items {
    pub const COPPER_PICKAXE: Item = Item {
        id: 3509,
        item_type: ItemType::Pickaxe,
        stack: 1,
    };

    pub const DIRT_BLOCK: Item = Item {
        id: 2,
        item_type: ItemType::Block(Block::Dirt),
        stack: 1,
    };

    pub const STONE_BLOCK: Item = Item {
        id: 3,
        item_type: ItemType::Block(Block::Stone),
        stack: 1,
    };
}

pub fn get_animation_points(item_type: ItemType) -> Vec<Vec2> {
    match item_type {
        _ => vec![vec2(-5., 4.), vec2(5., 3.), vec2(5., -7.)]
    }
}

pub fn get_item_data_by_id<'a>(id: &ItemId) -> &'a ItemData {
    ITEM_DATA.get(id).expect("Item not found")
}
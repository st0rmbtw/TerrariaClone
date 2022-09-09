use std::collections::HashMap;

use bevy::{math::vec2, prelude::Vec2};

pub type ItemId = u16;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemType {
    Pickaxe,
}

#[derive(Clone, Copy)]
pub struct Item {
    pub id: ItemId,
    pub item_type: ItemType,
    pub stack: u16,
}

pub struct ItemData {
    pub name: &'static str,
    pub max_stack: u16,
}

lazy_static! {
    pub static ref ITEM_ANIMATION_DATA: HashMap<ItemType, Vec<Vec2>> = HashMap::from([(
        ItemType::Pickaxe,
        vec![vec2(-5., 4.), vec2(5., 3.), vec2(5., -7.),]
    )]);
}

lazy_static! {
    pub static ref ITEM_DATA: HashMap<ItemId, ItemData> = HashMap::from([(
        3509,
        ItemData {
            name: "Copper Pickaxe",
            max_stack: 1
        }
    )]);
}

pub struct Items;

impl Items {
    pub const COPPER_PICKAXE: Item = Item {
        id: 3509,
        item_type: ItemType::Pickaxe,
        stack: 1,
    };
}

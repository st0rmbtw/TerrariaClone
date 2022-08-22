use std::collections::HashMap;

pub type ItemId = u16;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ItemType {
    Pickaxe
}

#[derive(Clone, Copy)]
pub struct Item {
    pub id: ItemId,
    pub item_type: ItemType
}

pub struct ItemData {
    pub name: &'static str
}

lazy_static! {
    pub static ref ITEM_DATA: HashMap<ItemId, ItemData> = HashMap::from([
        (
            3509, 
            ItemData {
                name: "Copper Pickaxe"
            }
        )
    ]);
}


pub const ITEM_COPPER_PICKAXE: Item = Item {
    id: 3509,
    item_type: ItemType::Pickaxe
};


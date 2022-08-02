pub struct Item {
    pub id: i32,
    pub name: String,
}

lazy_static! {
    pub static ref ITEM_COPPER_PICKAXE: Item = Item {
        id: 3509,
        name: "Copper Pickaxe".to_string(),
    };
}

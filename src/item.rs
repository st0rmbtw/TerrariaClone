pub struct Item {
    pub id: i32,
    pub name: String,
}

lazy_static! {
    pub static ref ITEM_WOODEN_PICKAXE: Item = Item {
        id: 1,
        name: "Wooden Pickaxe".to_string(),
    };
}
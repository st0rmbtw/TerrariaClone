pub struct Block {
    pub id: u32,
    pub name: &'static str,
}

pub static BLOCK_AIR: u32 = u32::MAX;
pub static BLOCK_DIRT: &Block = &Block {
    id: 0,
    name: "Dirt",
};
pub static BLOCK_STONE: &Block = &Block {
    id: 1,
    name: "Stone"
};
pub static BLOCK_GRASS: &Block = &Block {
    id: 2,
    name: "Grass"
};

pub fn get_block_by_id(id: u32) -> Option<&'static Block> {
    match id {
        0 => Some(BLOCK_DIRT),
        1 => Some(BLOCK_STONE),
        2 => Some(BLOCK_GRASS),
        _ => None
    }
}
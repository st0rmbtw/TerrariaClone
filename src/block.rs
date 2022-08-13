use std::fmt::Debug;

pub type BlockId = u16;

pub struct Block {
    pub id: BlockId,
    pub name: &'static str,
}

impl Debug for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Block").field("name", &self.name).finish()
    }
}

pub const BLOCK_DIRT_ID: BlockId = 0;
pub const BLOCK_STONE_ID: BlockId = 1;
pub const BLOCK_GRASS_ID: BlockId = 2;

pub static BLOCK_AIR: BlockId = BlockId::MAX;
pub static BLOCK_DIRT: &Block = &Block {
    id: BLOCK_DIRT_ID,
    name: "Dirt",
};
pub static BLOCK_STONE: &Block = &Block {
    id: BLOCK_STONE_ID,
    name: "Stone"
};
pub static BLOCK_GRASS: &Block = &Block {
    id: BLOCK_GRASS_ID,
    name: "Grass"
};

pub fn get_block_by_id(id: BlockId) -> Option<&'static Block> {
    match id {
        BLOCK_DIRT_ID => Some(BLOCK_DIRT),
        BLOCK_STONE_ID => Some(BLOCK_STONE),
        BLOCK_GRASS_ID => Some(BLOCK_GRASS),
        _ => None
    }
}
use std::collections::HashMap;

pub struct BlockData {
    name: &'static str
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum Block {
    Dirt,
    Stone,
    Grass
}

lazy_static! {
    pub static ref BLOCK_DATA: HashMap<Block, BlockData> = HashMap::from([
        (
            Block::Dirt,
            BlockData {
                name: "Dirt Block"
            }
        ),
        (
            Block::Stone,
            BlockData {
                name: "Stone Block"
            }
        )
    ]);
}
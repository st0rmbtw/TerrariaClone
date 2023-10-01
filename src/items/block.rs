use crate::world::block::BlockType;

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum ItemBlock {
    Dirt,
    Stone,
    Wood,
}

impl From<BlockType> for ItemBlock {
    fn from(block_type: BlockType) -> Self {
        match block_type {
            BlockType::Dirt | BlockType::Grass => Self::Dirt,
            BlockType::Stone => Self::Stone,
            BlockType::Tree(_) => Self::Wood,
            BlockType::Wood => Self::Wood,
        }
    }
}
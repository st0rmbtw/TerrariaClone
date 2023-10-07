use crate::world::block::BlockType;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ItemSeed {
    Grass
}

impl ItemSeed {
    pub(crate) const fn seeded_dirt(&self) -> BlockType {
        match self {
            ItemSeed::Grass => BlockType::Grass,
        }
    }
}
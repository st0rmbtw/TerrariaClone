use crate::world::block::BlockType;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Seed {
    Grass
}

impl Seed {
    pub(crate) fn seeded_dirt(&self) -> BlockType {
        match self {
            Seed::Grass => BlockType::Grass,
        }
    }
}
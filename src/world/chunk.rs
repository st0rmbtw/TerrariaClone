use bevy::prelude::{Component, UVec2};

use super::block::BlockType;
use super::tree::{TreeFrameType, Tree};

pub(crate) type ChunkPos = UVec2;

#[derive(Component)]
pub(crate) struct ChunkContainer {
    pub(crate) pos: ChunkPos
}

#[derive(Component, PartialEq, Eq)]
pub(crate) enum ChunkType {
    Tile,
    Wall,
    Tree,
    TreeBranch,
    TreeTop,
}

impl ChunkType {
    pub(crate) const fn from_block_type(block_type: BlockType) -> Self {
        match block_type {
            BlockType::Tree(Tree { frame_type: TreeFrameType::BranchLeftLeaves | TreeFrameType::BranchRightLeaves, .. }) => ChunkType::TreeBranch,
            BlockType::Tree(Tree { frame_type: TreeFrameType::TopLeaves, .. }) => ChunkType::TreeTop,
            BlockType::Tree(_) => ChunkType::Tree,
            _ => ChunkType::Tile
        }
    }
}

#[derive(Component)]
pub(crate) struct Chunk {
    pub(crate) chunk_type: ChunkType,
    pub(crate) pos: ChunkPos
}

impl Chunk {
    #[inline(always)]
    pub(crate) const fn new(pos: ChunkPos, chunk_type: ChunkType) -> Self {
        Self { pos, chunk_type }
    }
}
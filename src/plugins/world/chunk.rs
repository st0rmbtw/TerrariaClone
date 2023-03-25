use bevy::prelude::{Component, UVec2};

use super::{Tree, TreeFrameType, BlockType};

pub(super) type ChunkPos = UVec2;

#[derive(Component)]
pub(super) struct ChunkContainer {
    pub(super) pos: ChunkPos
}

#[derive(Component, PartialEq, Eq)]
pub(super) enum ChunkType {
    Tile,
    Wall,
    Tree,
    TreeBranch,
    TreeTop,
}

impl ChunkType {
    pub(super) const fn from_block_type(block_type: BlockType) -> Self {
        match block_type {
            BlockType::Tree(Tree { frame_type: TreeFrameType::BranchLeftLeaves | TreeFrameType::BranchRightLeaves, .. }) => ChunkType::TreeBranch,
            BlockType::Tree(Tree { frame_type: TreeFrameType::TopLeaves, .. }) => ChunkType::TreeTop,
            BlockType::Tree(_) => ChunkType::Tree,
            _ => ChunkType::Tile
        }
    }
}

#[derive(Component)]
pub(super) struct Chunk {
    pub(super) chunk_type: ChunkType,
    pub(super) pos: ChunkPos
}

impl Chunk {
    pub(super) const fn new(pos: ChunkPos, chunk_type: ChunkType) -> Self {
        Self { pos, chunk_type }
    }
}
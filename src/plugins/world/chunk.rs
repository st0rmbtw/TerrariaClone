use bevy::prelude::{Component, UVec2};

use super::{Block, Tree, TreeFrameType, BlockType};

pub type ChunkPos = UVec2;

#[derive(Component)]
pub struct ChunkContainer {
    pub pos: ChunkPos
}

#[derive(Component, PartialEq, Eq)]
pub enum ChunkType {
    Tile,
    Wall,
    Tree,
    TreeBranch,
    TreeTop,
}

#[derive(Component)]
pub struct Chunk {
    pub chunk_type: ChunkType,
    pub pos: ChunkPos
}

impl Chunk {
    pub const fn new(pos: ChunkPos, chunk_type: ChunkType) -> Self {
        Self { pos, chunk_type }
    }
}

impl Block {
    pub const fn chunk_type(&self) -> ChunkType {
        match self.block_type {
            BlockType::Tree(Tree { frame_type: TreeFrameType::BranchLeftLeaves | TreeFrameType::BranchRightLeaves, .. }) => ChunkType::TreeBranch,
            BlockType::Tree(Tree { frame_type: TreeFrameType::TopLeaves, .. }) => ChunkType::TreeTop,
            BlockType::Tree(_) => ChunkType::Tree,
            _ => ChunkType::Tile
        }
    }
}
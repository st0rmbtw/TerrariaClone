use bevy::prelude::{Component, UVec2};

pub type ChunkPos = UVec2;

#[derive(Component)]
pub struct Chunk {
    pub pos: ChunkPos
}

#[derive(Component)]
pub struct TileChunk {
    pub pos: ChunkPos
}

#[derive(Component)]
pub struct WallChunk {
    pub pos: ChunkPos
}
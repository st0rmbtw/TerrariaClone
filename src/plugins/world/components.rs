use bevy::prelude::{Component, IVec2};

#[derive(Component)]
pub struct Chunk {
    pub pos: IVec2
}

#[derive(Component)]
pub struct TileChunk {
    pub pos: IVec2
}

#[derive(Component)]
pub struct WallChunk {
    pub pos: IVec2
}
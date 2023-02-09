use bevy::{utils::HashSet, prelude::Resource};

use crate::world_generator::{CellArray};

use super::ChunkPos;

#[derive(Resource)]
pub struct WorldData {
    pub width: u16,
    pub height: u16,
    pub tiles: CellArray,
}

#[derive(Resource, Default)]
pub struct ChunkManager {
    pub spawned_chunks: HashSet<ChunkPos>
}
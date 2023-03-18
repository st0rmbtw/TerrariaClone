use bevy::{utils::HashSet, prelude::Resource};
use ndarray::Array2;

use super::ChunkPos;

#[derive(Resource)]
pub struct LightMap {
    pub colors: Array2<u8>,
}

#[derive(Resource, Default)]
pub(super) struct ChunkManager {
    pub spawned_chunks: HashSet<ChunkPos>
}
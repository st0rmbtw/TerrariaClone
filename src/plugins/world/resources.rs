use bevy::utils::HashSet;
use ndarray::Array2;

use crate::world_generator::Cell;

use super::ChunkPos;

pub struct WorldData {
    pub width: u16,
    pub height: u16,
    pub tiles: Array2<Cell>,
}

#[derive(Default)]
pub struct ChunkManager {
    pub spawned_chunks: HashSet<ChunkPos>
}
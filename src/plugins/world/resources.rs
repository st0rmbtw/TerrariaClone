use bevy::{utils::HashSet, prelude::Resource};
use ndarray::Array2;

use crate::world_generator::Cell;

use super::ChunkPos;

#[derive(Resource)]
pub struct WorldData {
    pub width: u16,
    pub height: u16,
    pub tiles: Array2<Cell>,
}

#[derive(Resource, Default)]
pub struct ChunkManager {
    pub spawned_chunks: HashSet<ChunkPos>
}
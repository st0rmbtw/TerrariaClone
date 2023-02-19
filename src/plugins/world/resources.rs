use bevy::{utils::HashSet, prelude::{Resource, UVec4}};
use ndarray::Array2;

use crate::world_generator::CellArray;

use super::ChunkPos;

#[derive(Resource)]
pub struct WorldData {
    pub width: u16,
    pub height: u16,
    pub tiles: CellArray,
}

#[derive(Resource)]
pub struct LightMap {
    pub width: u16,
    pub height: u16,
    pub colors: Array2<UVec4>,
}

#[derive(Resource, Default)]
pub struct ChunkManager {
    pub spawned_chunks: HashSet<ChunkPos>
}
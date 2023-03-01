use bevy::{utils::HashSet, prelude::Resource};
use bevy_ecs_tilemap::tiles::TilePos;
use ndarray::Array2;

use crate::world_generator::CellArray;

use super::ChunkPos;

#[derive(Resource)]
pub struct WorldData {
    pub width: u16,
    pub height: u16,
    pub tiles: CellArray,
    pub spawn_point: TilePos
}

#[derive(Resource, Default)]
pub struct LightMap {
    pub width: u16,
    pub height: u16,
    // f32 because WGSL doesn't support u8 yet
    pub colors: Array2<f32>,
}

#[derive(Resource, Default)]
pub struct ChunkManager {
    pub spawned_chunks: HashSet<ChunkPos>
}
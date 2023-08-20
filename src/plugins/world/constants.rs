use bevy_ecs_tilemap::prelude::{TilemapTileSize, TilemapSize};

pub(crate) const TILE_SIZE: f32 = 16.;
pub(crate) const WALL_SIZE: f32 = 32.;
pub(super) const TREE_SIZE: TilemapTileSize = TilemapTileSize { x: 20., y: 20. };
pub(super) const TREE_BRANCHES_SIZE: TilemapTileSize = TilemapTileSize { x: 50., y: 40. };
pub(super) const TREE_TOPS_SIZE: TilemapTileSize = TilemapTileSize { x: 88., y: 148. };

pub(super) const CHUNK_SIZE: f32 = 25.;
pub(super) const CHUNK_SIZE_U: u32 = CHUNK_SIZE as u32;

pub(super) const CHUNKMAP_SIZE: TilemapSize = TilemapSize {
    x: CHUNK_SIZE as u32,
    y: CHUNK_SIZE as u32,
};
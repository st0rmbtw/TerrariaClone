use bevy::prelude::{Vec2, OrthographicProjection, UVec2};
use bevy_ecs_tilemap::tiles::TilePos;

use crate::util::FRect;

use super::{CHUNK_SIZE_U, ChunkPos};

pub fn get_chunk_pos(pos: TilePos) -> ChunkPos {    
    UVec2::from(pos) / CHUNK_SIZE_U
}

pub fn get_chunk_tile_pos(map_tile_pos: TilePos) -> TilePos {
    TilePos { 
        x: map_tile_pos.x % CHUNK_SIZE_U, 
        y: CHUNK_SIZE_U - 1 - (map_tile_pos.y % CHUNK_SIZE_U)
    }
}

pub fn get_camera_fov(camera_pos: Vec2, projection: &OrthographicProjection) -> FRect {
    FRect {
        left: camera_pos.x + projection.left * projection.scale,
        right: camera_pos.x + projection.right * projection.scale,
        top: camera_pos.y + projection.top * projection.scale,
        bottom: camera_pos.y + projection.bottom * projection.scale,
    }
}
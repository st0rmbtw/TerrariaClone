use bevy::prelude::{Vec2, OrthographicProjection};
use bevy_ecs_tilemap::tiles::TilePos;

use crate::world::{chunk::ChunkPos, Size};

use super::{CHUNK_SIZE_U, CHUNK_SIZE, TILE_SIZE, CameraFov, ChunkRange};

pub(super) fn get_chunk_pos(pos: TilePos) -> ChunkPos {
    ChunkPos::new(pos.x, pos.y) / CHUNK_SIZE_U
}

pub(super) fn get_chunk_tile_pos(map_tile_pos: TilePos) -> TilePos {
    TilePos { 
        x: map_tile_pos.x % CHUNK_SIZE_U, 
        y: CHUNK_SIZE_U - 1 - (map_tile_pos.y % CHUNK_SIZE_U)
    }
}

pub(super) fn get_camera_fov(camera_pos: Vec2, projection: &OrthographicProjection) -> CameraFov {
    CameraFov {
        left: camera_pos.x + projection.area.min.x,
        right: camera_pos.x + projection.area.max.x,
        top: camera_pos.y + projection.area.max.y,
        bottom: camera_pos.y + projection.area.min.y,
    }
}

pub(super) fn get_chunk_range_by_camera_fov(camera_fov: CameraFov, world_size: Size) -> ChunkRange {
    let left = (camera_fov.left / (CHUNK_SIZE * TILE_SIZE)).floor() as u32;
    let mut right = (camera_fov.right / (CHUNK_SIZE * TILE_SIZE)).ceil() as u32;
    let top = (camera_fov.top / (CHUNK_SIZE * TILE_SIZE)).ceil().abs() as u32;
    let mut bottom = (camera_fov.bottom / (CHUNK_SIZE * TILE_SIZE)).floor().abs() as u32;

    let max_chunk_x = world_size.width as u32 / CHUNK_SIZE_U;
    let max_chunk_y = world_size.height as u32 / CHUNK_SIZE_U;

    if right > max_chunk_x {
        right = max_chunk_x;
    }
    
    if bottom > max_chunk_y {
        bottom = max_chunk_y;
    }

    ChunkRange {
        x: left..=right,
        y: top..=bottom
    }
}
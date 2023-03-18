use bevy::prelude::{Vec2, OrthographicProjection, UVec2};
use bevy_ecs_tilemap::tiles::TilePos;

use crate::common::rect::{FRect, URect};

use super::{CHUNK_SIZE_U, ChunkPos, CHUNK_SIZE, Size, TILE_SIZE};

pub(super) fn get_chunk_pos(pos: TilePos) -> ChunkPos {    
    UVec2::from(pos) / CHUNK_SIZE_U
}

pub(super) fn get_chunk_tile_pos(map_tile_pos: TilePos) -> TilePos {
    TilePos { 
        x: map_tile_pos.x % CHUNK_SIZE_U, 
        y: CHUNK_SIZE_U - 1 - (map_tile_pos.y % CHUNK_SIZE_U)
    }
}

pub(super) fn get_camera_fov(camera_pos: Vec2, projection: &OrthographicProjection) -> FRect {
    FRect {
        left: camera_pos.x + projection.area.min.x,
        right: camera_pos.x + projection.area.max.x,
        top: camera_pos.y + projection.area.max.y,
        bottom: camera_pos.y + projection.area.min.y,
    }
}

pub(super) fn get_chunk_position_by_camera_fov(camera_fov: FRect, world_size: Size) -> URect {
    let mut rect = URect { 
        left: (camera_fov.left / (CHUNK_SIZE * TILE_SIZE)).floor() as u32, 
        right: (camera_fov.right / (CHUNK_SIZE * TILE_SIZE)).ceil() as u32, 
        bottom: (camera_fov.bottom / (CHUNK_SIZE * TILE_SIZE)).floor().abs() as u32, 
        top: (camera_fov.top / (CHUNK_SIZE * TILE_SIZE)).ceil().abs() as u32,
    };

    let max_chunk_x: u32 = world_size.width as u32 / CHUNK_SIZE_U;
    let max_chunk_y: u32 = world_size.height as u32 / CHUNK_SIZE_U;

    if rect.right > max_chunk_x {
        rect.right = max_chunk_x;
    }
    
    if rect.bottom > max_chunk_y {
        rect.bottom = max_chunk_y;
    }

    rect
}
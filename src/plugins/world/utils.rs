use bevy::prelude::{Vec2, OrthographicProjection, UVec2};
use bevy_ecs_tilemap::tiles::TilePos;
use rand::{thread_rng, Rng};

use crate::{util::FRect, world_generator::{Neighbors, DirtConnections, DirtConnection}};

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
        bottom: camera_pos.y - projection.bottom * projection.scale,
    }
}

pub fn get_tile_sprite_index_by_neighbors(slope: Neighbors) -> u32 {
    let rand: u32 = thread_rng().gen_range(1..3);

    match slope {
        // All
        Neighbors::ALL => rand + 16,
        // None
        Neighbors::NONE => 16 * 3 + rand + 8,
        // Top
        Neighbors::TOP => 16 * 3 + rand + 5,
        // Bottom
        Neighbors::BOTTOM => rand + 6,
        // Left
        Neighbors::LEFT => (rand - 1) * 16 + 12,
        // Right
        Neighbors::RIGHT => (rand - 1) * 16 + 9,
        // Top Bottom
        Neighbors::TOP_BOTTOM => (rand - 1) * 16 + 5,
        // Top Left Right
        Neighbors::TOP_LEFT_RIGHT => 16 * 2 + rand + 1,
        // Bottom Left Right
        Neighbors::BOTTOM_LEFT_RIGHT => rand,
        // Left Right
        Neighbors::LEFT_RIGHT => 4 * 16 + 5 + rand,
        // Bottom Left
        Neighbors::BOTTOM_LEFT => 16 * 3 + 1 + (rand - 1) * 2,
        // Bottom Right
        Neighbors::BOTTOM_RIGHT => 16 * 3 + (rand - 1) * 2,
        // Top Left
        Neighbors::TOP_LEFT => 16 * 4 + 1 + (rand - 1) * 2,
        // Top Right
        Neighbors::TOP_RIGHT => 16 * 4 + (rand - 1) * 2,
        // Top Bottom Left
        Neighbors::TOP_BOTTOM_LEFT => (rand - 1) * 16 + 4,
        // Top Bottom Right
        Neighbors::TOP_BOTTOM_RIGHT => (rand - 1) * 16,
    }
}

pub fn get_tile_sprite_index_by_dirt_connections(dirt_connections: DirtConnections) -> u32 {
    let rand: u32 = thread_rng().gen_range(1..3);

    /* DirtConnections::ALL => 16 * 11 + rand + 5,
    DirtConnections::TOP => 16 * 6 + rand + 7,
    DirtConnections::BOTTOM => 16 * 5 + rand + 7,
    DirtConnections::LEFT => (7 + rand - 1) * 16 + 9,
    DirtConnections::RIGHT => (7 + rand - 1) * 16 + 8,
    DirtConnections::TOP_BOTTOM => (12 + rand - 1) * 16 + 6,
    DirtConnections::TOP_LEFT_RIGHT => (5 + rand - 1) * 16 + 11,
    DirtConnections::BOTTOM_LEFT_RIGHT => (8 + rand - 1) * 16 + 11,
    DirtConnections::LEFT_RIGHT => 11 * 16 + 8 + rand,
    DirtConnections::BOTTOM_LEFT => (6 + rand * 2) * 16 + 2,
    DirtConnections::BOTTOM_RIGHT => (6 + rand * 2) * 16 + 3,
    DirtConnections::TOP_LEFT => (5 + (rand - 1) * 2) * 16 + 2,
    DirtConnections::TOP_RIGHT => (5 + (rand - 1) * 2) * 16 + 3,
    DirtConnections::TOP_BOTTOM_LEFT => (5 + rand - 1) * 16 + 12,
    DirtConnections::TOP_BOTTOM_RIGHT => (8 + rand - 1) * 16 + 12, */

    match dirt_connections {
        DirtConnections { 
            top: DirtConnection::Connected, 
            bottom: DirtConnection::NotConnected(false),
            left: DirtConnection::NotConnected(false),
            right: DirtConnection::NotConnected(false), 
        } => 16 * 6 + rand + 7,
        DirtConnections { 
            top: DirtConnection::NotConnected(false), 
            bottom: DirtConnection::Connected,
            left: DirtConnection::NotConnected(false),
            right: DirtConnection::NotConnected(false), 
        } => 16 * 6 + rand + 7,
        DirtConnections { 
            top: DirtConnection::Connected, 
            bottom: DirtConnection::NotConnected(false),
            left: DirtConnection::NotConnected(false),
            right: DirtConnection::NotConnected(false), 
        } => 16 * 6 + rand + 7,
        DirtConnections { 
            top: DirtConnection::Connected, 
            bottom: DirtConnection::NotConnected(false),
            left: DirtConnection::NotConnected(false),
            right: DirtConnection::NotConnected(false), 
        } => 16 * 6 + rand + 7,

    }
}

pub fn get_wall_sprite_index(slope: Neighbors) -> u32 {
    let rand: u32 = thread_rng().gen_range(1..3);

    match slope {
        // All
        Neighbors::ALL => 13 + rand,
        // None
        Neighbors::NONE => 13 * 3 + 8 + rand,
        // Top
        Neighbors::TOP => 13 * 2 + rand,
        // Bottom
        Neighbors::BOTTOM => 6 + rand,
        // Top Bottom
        Neighbors::TOP_BOTTOM => (rand - 1) * 13 + 5,
        // Bottom Right
        Neighbors::BOTTOM_RIGHT => 13 * 3 + (rand - 1) * 2,
        // Bottom Left
        Neighbors::BOTTOM_LEFT => 13 * 3 + 1 + (rand - 1) * 2,
        // Top Right
        Neighbors::TOP_RIGHT => 13 * 4 + (rand - 1) * 2,
        // Top Left
        Neighbors::TOP_LEFT => 13 * 4 + 1 + (rand - 1) * 2,
        // Left Right
        Neighbors::LEFT_RIGHT => 13 * 4 + 5 + rand,
        // Bottom Left Right
        Neighbors::BOTTOM_LEFT_RIGHT => 1 + rand,
        // Top Bottom Right
        Neighbors::TOP_BOTTOM_RIGHT => 13 * (rand - 1),
        // Top Bottom Left
        Neighbors::TOP_BOTTOM_LEFT => 13 * (rand - 1) + 4,
        // Top Left Right
        Neighbors::TOP_LEFT_RIGHT => 13 * 2 + rand,
        _ => panic!("{:#?}", slope),
    }
}

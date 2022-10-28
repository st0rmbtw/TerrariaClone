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
        bottom: camera_pos.y - projection.bottom * projection.scale,
    }
}

// pub fn get_tile_sprite_index_by_dirt_connections(dirt_connections: DirtConnections) -> u32 {
//     let rand: u32 = thread_rng().gen_range(1..3);

//     match dirt_connections {
//         // ALL
//         DirtConnections {
//             top: DirtConnection::Connected,
//             bottom: DirtConnection::Connected,
//             left: DirtConnection::Connected,
//             right: DirtConnection::Connected, 
//         } => 16 * 11 + rand + 5,
//         // TOP
//         DirtConnections { 
//             top: DirtConnection::Connected, 
//             bottom: DirtConnection::NotConnected(..),
//             left: DirtConnection::NotConnected(true),
//             right: DirtConnection::NotConnected(true),
//         } => 16 * 6 + rand + 7,
//         // TOP
//         DirtConnections { 
//             top: DirtConnection::Connected, 
//             bottom: DirtConnection::NotConnected(..),
//             left: DirtConnection::NotConnected(false),
//             right: DirtConnection::NotConnected(false),
//         } => (7 + rand -1) * 16 + 6,
//         // BOTTOM
//         DirtConnections {
//             top: DirtConnection::NotConnected(true), 
//             bottom: DirtConnection::Connected,
//             left: DirtConnection::NotConnected(true),
//             right: DirtConnection::NotConnected(true), 
//         } => 16 * 5 + rand + 7,
//         DirtConnections {
//             top: DirtConnection::NotConnected(false), 
//             bottom: DirtConnection::Connected,
//             left: DirtConnection::NotConnected(true),
//             right: DirtConnection::NotConnected(true), 
//         } => (4 + rand - 1) * 16 + 6,
//         // LEFT
//         DirtConnections { 
//             top: DirtConnection::NotConnected(true), 
//             bottom: DirtConnection::NotConnected(true),
//             left: DirtConnection::Connected,
//             right: DirtConnection::NotConnected(true), 
//         } => (7 + rand - 1) * 16 + 9,
//         // RIGHT
//         DirtConnections { 
//             top: DirtConnection::NotConnected(true), 
//             bottom: DirtConnection::NotConnected(true),
//             left: DirtConnection::NotConnected(true),
//             right: DirtConnection::Connected, 
//         } => (7 + rand - 1) * 16 + 8,
//         // TOP AND BOTTOM
//         DirtConnections { 
//             top: DirtConnection::Connected,
//             bottom: DirtConnection::Connected,
//             left: DirtConnection::NotConnected(true),
//             right: DirtConnection::NotConnected(true)
//          } => (12 + rand - 1) * 16 + 6,
//         // TOP AND LEFT AND RIGHT
//         DirtConnections {
//             top: DirtConnection::Connected,
//             left: DirtConnection::Connected,
//             right: DirtConnection::Connected,
//             bottom: DirtConnection::NotConnected(true)
//         } => (5 + rand - 1) * 16 + 11,
//         // BOTTOM AND LEFT AND RIGHT
//         DirtConnections {
//             top: DirtConnection::NotConnected(true),
//             left: DirtConnection::Connected,
//             right: DirtConnection::Connected,
//             bottom: DirtConnection::Connected
//         } => (8 + rand - 1) * 16 + 11,
//         // LEFT AND RIGHT
//         DirtConnections {
//             top: DirtConnection::NotConnected(true),
//             left: DirtConnection::Connected,
//             right: DirtConnection::Connected,
//             bottom: DirtConnection::NotConnected(true)
//         } => 11 * 16 + 8 + rand,
//         // BOTTOM AND LEFT
//         DirtConnections {
//             top: DirtConnection::NotConnected(true),
//             left: DirtConnection::Connected,
//             right: DirtConnection::NotConnected(true),
//             bottom: DirtConnection::Connected
//         } => (6 + rand * 2) * 16 + 2,
//         // BOTTOM AND RIGHT
//         DirtConnections {
//             top: DirtConnection::NotConnected(true),
//             left: DirtConnection::NotConnected(true),
//             right: DirtConnection::Connected,
//             bottom: DirtConnection::Connected
//         } => (6 + rand * 2) * 16 + 3,
//         // TOP AND LEFT
//         DirtConnections {
//             top: DirtConnection::Connected,
//             bottom: DirtConnection::NotConnected(true),
//             left: DirtConnection::Connected,
//             right: DirtConnection::NotConnected(true),
//         } => (5 + (rand - 1) * 2) * 16 + 2,
//         // TOP AND RIGHT
//         DirtConnections {
//             top: DirtConnection::Connected,
//             bottom: DirtConnection::NotConnected(true),
//             left: DirtConnection::NotConnected(true),
//             right: DirtConnection::Connected,
//         } => (5 + (rand - 1) * 2) * 16 + 3,
//         // TOP AND BOTTOM AND LEFT
//         DirtConnections {
//             top: DirtConnection::Connected,
//             bottom: DirtConnection::Connected,
//             left: DirtConnection::Connected,
//             right: DirtConnection::NotConnected(true)
//         } => (5 + rand - 1) * 16 + 12,
//         // TOP AND BOTTOM AND RIGHT
//         DirtConnections {
//             top: DirtConnection::Connected,
//             bottom: DirtConnection::Connected,
//             left: DirtConnection::NotConnected(true),
//             right: DirtConnection::Connected
//         } => (8 + rand - 1) * 16 + 12,
//         // _ => panic!("{:#?}", dirt_connections)
//         _ => 16 * 3 + rand + 8
//     }
// }

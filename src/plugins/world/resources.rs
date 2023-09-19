use bevy::{utils::HashSet, prelude::{Resource, Query, Entity, UVec2, Commands, DespawnRecursiveExt, BuildChildren}};
use bevy_ecs_tilemap::tiles::{TilePos, TileStorage, TileTextureIndex};

use crate::world::{chunk::{ChunkPos, Chunk, ChunkType}, block::{BlockType, Block}};

use super::{systems::{spawn_block, spawn_cracks}, utils::{get_chunk_pos, get_chunk_tile_pos}};

#[derive(Resource, Clone, Copy, Default)]
pub(crate) struct WorldUndergroundLevel(pub(crate) u32);

#[derive(Resource, Default)]
pub(super) struct ChunkManager {
    pub(super) spawned_chunks: HashSet<ChunkPos>
}

impl ChunkManager {
    fn filter_chunk(chunk: &Chunk, chunk_pos: UVec2, block_type: BlockType) -> bool {
        chunk.pos == chunk_pos && chunk.chunk_type == ChunkType::from(block_type)
    }

    pub(super) fn get_block_entity(
        query_chunk: &Query<(&Chunk, &TileStorage)>,
        chunk_pos: UVec2,
        chunk_tile_pos: TilePos,
        block_type: BlockType
    ) -> Option<Entity> {
        let filtered_chunk = query_chunk
            .iter()
            .find(|(chunk, _)| {
                ChunkManager::filter_chunk(chunk, chunk_pos, block_type)
            });

        filtered_chunk.and_then(|(_, storage)| storage.get(&chunk_tile_pos))
    }

    pub(super) fn remove(
        commands: &mut Commands,
        query_chunk: &mut Query<(&Chunk, &mut TileStorage)>,
        tile_pos: TilePos,
        chunk_type: ChunkType
    ) {
        let chunk_pos = get_chunk_pos(tile_pos);
        let chunk_tile_pos = get_chunk_tile_pos(tile_pos);

        let filtered_chunk = query_chunk
            .iter_mut()
            .find(|(chunk, _)| {
                chunk.pos == chunk_pos && chunk.chunk_type == chunk_type
            });

        if let Some((_, mut tile_storage)) = filtered_chunk {
            if let Some(tile_entity) = tile_storage.get(&chunk_tile_pos) {
                commands.entity(tile_entity).despawn_recursive();
                tile_storage.remove(&chunk_tile_pos);
            }
        }
    }

    pub(super) fn spawn_block(
        commands: &mut Commands,
        query_chunk: &mut Query<(&Chunk, &mut TileStorage, Entity)>,
        tile_pos: TilePos,
        block: &Block,
        index: u32,
    ) {
        let chunk_pos = get_chunk_pos(tile_pos);
        let chunk_tile_pos = get_chunk_tile_pos(tile_pos);

        let filtered_chunk = query_chunk
            .iter_mut()
            .find(|(chunk, _, _)| {
                ChunkManager::filter_chunk(chunk, chunk_pos, block.block_type)
            });

        if let Some((_, mut tile_storage, tilemap_entity)) = filtered_chunk {
            let tile_entity = spawn_block(commands, *block, chunk_tile_pos, tilemap_entity, index);
            commands.entity(tilemap_entity).add_child(tile_entity);
            tile_storage.set(&chunk_tile_pos, tile_entity);
        }
    }

    pub(super) fn update_tile_cracks(
        commands: &mut Commands,
        query_chunk: &mut Query<(&Chunk, &mut TileStorage, Entity)>,
        query_tile: &mut Query<&mut TileTextureIndex>,
        tile_pos: TilePos,
        index: u32,
    ) {
        let chunk_pos = get_chunk_pos(tile_pos);
        let chunk_tile_pos = get_chunk_tile_pos(tile_pos);

        let filtered_chunk = query_chunk
            .iter_mut()
            .find(|(chunk, _, _)| {
                chunk.pos == chunk_pos && chunk.chunk_type == ChunkType::Cracks
            });

        if let Some((_, mut tile_storage, tilemap_entity)) = filtered_chunk {
            if let Some(e) = tile_storage.checked_get(&chunk_tile_pos) {
                if let Ok(mut tile_texture) = query_tile.get_mut(e) {
                    tile_texture.0 = index;
                }
            } else {
                let cracks_entity = spawn_cracks(commands, chunk_tile_pos, tilemap_entity, index);
                commands.entity(tilemap_entity).add_child(cracks_entity);
                tile_storage.set(&chunk_tile_pos, cracks_entity);
            }
        }
    }
}
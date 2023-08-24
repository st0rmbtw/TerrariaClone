use bevy::{utils::HashSet, prelude::{Resource, Query, Entity, UVec2, Commands, DespawnRecursiveExt, BuildChildren, Deref, DerefMut, Handle, Mesh}};
use bevy_ecs_tilemap::tiles::{TilePos, TileStorage};
use ndarray::Array2;

use crate::world::{chunk::{ChunkPos, Chunk, ChunkType}, block::{BlockType, Block}};

use super::{systems::spawn_block, utils::{get_chunk_pos, get_chunk_tile_pos}};

#[derive(Resource, Deref, DerefMut)]
pub(crate) struct LightMap(pub(crate) Array2<f32>);

impl LightMap {
    #[inline(always)]
    pub(crate) fn new(colors: Array2<f32>) -> Self {
        Self(colors)
    }
}

#[derive(Resource, Deref)]
pub(crate) struct LightMapChunkMesh(pub(crate) Handle<Mesh>);

#[derive(Resource, Default)]
pub(super) struct ChunkManager {
    pub(super) spawned_chunks: HashSet<ChunkPos>
}

impl ChunkManager {
    fn filter_chunk(chunk: &Chunk, chunk_pos: UVec2, block_type: BlockType) -> bool {
        chunk.pos == chunk_pos && chunk.chunk_type == ChunkType::from_block_type(block_type)
    }

    pub(super) fn get_block_entity(
        query_chunk: &Query<(&Chunk, &TileStorage)>,
        chunk_pos: UVec2,
        chunk_tile_pos: TilePos,
        block_type: BlockType
    ) -> Option<Entity> {
        let filtered_chunks = query_chunk
            .iter()
            .find(|(chunk, _)| {
                ChunkManager::filter_chunk(chunk, chunk_pos, block_type)
            });

        if let Some((_, tile_storage)) = filtered_chunks {
            tile_storage.get(&chunk_tile_pos)
        } else {
            None
        }
    }

    pub(super) fn remove_block(
        commands: &mut Commands,
        query_chunk: &mut Query<(&Chunk, &mut TileStorage)>,
        tile_pos: TilePos,
        block_type: BlockType
    ) {
        let chunk_pos = get_chunk_pos(tile_pos);
        let chunk_tile_pos = get_chunk_tile_pos(tile_pos);

        let filtered_chunks = query_chunk
            .iter_mut()
            .find(|(chunk, _)| {
                ChunkManager::filter_chunk(chunk, chunk_pos, block_type)
            });

        if let Some((_, mut tile_storage)) = filtered_chunks {
            let block_entity = tile_storage.get(&chunk_tile_pos).unwrap();
            commands.entity(block_entity).despawn_recursive();
            tile_storage.remove(&chunk_tile_pos);
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

        let filtered_chunks = query_chunk
            .iter_mut()
            .find(|(chunk, _, _)| {
                ChunkManager::filter_chunk(chunk, chunk_pos, block.block_type)
            });

        if let Some((_, mut tile_storage, tilemap_entity)) = filtered_chunks {
            let tile_entity = spawn_block(commands, *block, chunk_tile_pos, tilemap_entity, index);
            commands.entity(tilemap_entity).add_child(tile_entity);
            tile_storage.set(&chunk_tile_pos, tile_entity);
        }
    }
}
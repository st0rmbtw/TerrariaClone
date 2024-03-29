use std::time::{SystemTime, UNIX_EPOCH};

use bevy::{
    prelude::{
        EventReader, ResMut, Query, Commands, EventWriter, Entity, BuildChildren, Transform, 
        default, SpatialBundle, DespawnRecursiveExt, OrthographicProjection, Changed, 
        GlobalTransform, With, Res, UVec2, NextState, Name, Vec2,
    }, 
    math::Vec3Swizzles, render::view::NoFrustumCulling
};
use bevy_ecs_tilemap::{
    tiles::{
        TilePos, TileStorage, TileBundle, TileTextureIndex
    }, 
    prelude::{
        TilemapGridSize, TilemapTexture, TilemapTileSize, 
        TilemapSpacing, TilemapId, TilemapSize
    }, 
    TilemapBundle, helpers::square_grid::neighbors::Neighbors
};
use rand::{thread_rng, Rng};

use crate::{plugins::{assets::{BlockAssets, WallAssets}, camera::components::MainCamera, audio::{SoundType, AudioCommandsExt}, DespawnOnGameExit, item::ItemCommandsExt, particles::Particle}, common::{state::GameState, TextureAtlasPos, math::map_range_i32, helpers::{tile_to_world_pos, random_point_cone}}, world::{WorldSize, chunk::{Chunk, ChunkType, ChunkContainer, ChunkPos}, WorldData, block::{BlockType, Block}, wall::Wall, tree::TreeFrameType, generator::generate_world}, WALL_LAYER, TILES_LAYER, items::ItemStack};

use super::{
    utils::{get_chunk_pos, get_camera_fov, get_chunk_tile_pos, get_chunk_range_by_camera_fov, self}, 
    events::{UpdateNeighborsEvent, DigBlockEvent, UpdateBlockEvent, SeedEvent, UpdateCracksEvent, UpdateWallEvent, DigWallEvent, BreakTileEvent, PlaceTileEvent, TileRemovedEvent},
    resources::{ChunkManager, WorldUndergroundLevel}, 
    constants::{CHUNK_SIZE_U, WALL_SIZE, CHUNKMAP_SIZE, TREE_SIZE, TREE_BRANCHES_SIZE, TREE_TOPS_SIZE, CHUNK_SIZE, TILE_SIZE}, WORLD_RENDER_LAYER, TileType
};

#[cfg(feature = "debug")]
use crate::plugins::debug::DebugConfiguration;

pub(super) fn setup(mut commands: Commands) {
    commands.init_resource::<ChunkManager>();
}

pub(super) fn spawn_terrain(mut commands: Commands) {
    let _current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

    // let seed = current_time.as_millis() as u32;
    let seed = 2225406523;

    println!("The seed of the world is {}", seed);

    let world_data = generate_world(seed, WorldSize::Tiny);

    commands.insert_resource(WorldUndergroundLevel(world_data.layer.underground as u32));
    commands.insert_resource(super::WorldSize(world_data.area.size()));
    commands.insert_resource(world_data);
    commands.insert_resource(NextState(Some(GameState::InGame)));
}

pub(super) fn cleanup(mut commands: Commands) {
    commands.remove_resource::<WorldData>();
    commands.remove_resource::<ChunkManager>();
}

pub(super) fn spawn_block(
    commands: &mut Commands,
    block: Block,
    tile_pos: TilePos,
    tilemap_entity: Entity,
    index: u32
) -> Entity {
    commands
        .spawn((
            Name::new(format!("{:#?}", block.block_type)),
            block,
            TileBundle {
                position: tile_pos,
                tilemap_id: TilemapId(tilemap_entity),
                texture_index: TileTextureIndex(index),
                ..default()
            }
        ))
        .id()
}

pub(super) fn spawn_cracks(
    commands: &mut Commands,
    tile_pos: TilePos,
    tilemap_entity: Entity,
    index: u32
) -> Entity {
    commands.spawn(TileBundle {
        position: tile_pos,
        tilemap_id: TilemapId(tilemap_entity),
        texture_index: TileTextureIndex(index),
        ..default()
    })
    .id()
}

pub(super) fn spawn_wall(
    commands: &mut Commands,
    wall_pos: TilePos,
    wallmap_entity: Entity,
    index: u32
) -> Entity {
    commands
        .spawn(TileBundle {
            position: wall_pos,
            tilemap_id: TilemapId(wallmap_entity),
            texture_index: TileTextureIndex(index),
            ..default()
        })
        .id()
}

pub(super) fn spawn_chunks(
    mut commands: Commands,
    block_assets: Res<BlockAssets>,
    wall_assets: Res<WallAssets>,
    world_data: Res<WorldData>,
    mut chunk_manager: ResMut<ChunkManager>,
    query_camera: Query<
        (&GlobalTransform, &OrthographicProjection),
        (With<MainCamera>, Changed<GlobalTransform>),
    >
) {
    if let Ok((camera_transform, projection)) = query_camera.get_single() {
        let camera_fov = get_camera_fov(camera_transform.translation().xy(), projection);
        let chunk_range = get_chunk_range_by_camera_fov(camera_fov, world_data.area.size());

        for y in chunk_range.min.y..=chunk_range.max.y {
            for x in chunk_range.min.x..=chunk_range.max.x {
                let chunk_pos = UVec2::new(x, y);
                if chunk_manager.spawned_chunks.insert(chunk_pos) {
                    spawn_chunk(&mut commands, &block_assets, &wall_assets, &world_data, chunk_pos);
                }
            }
        }
    }
}

pub(super) fn despawn_chunks(
    mut commands: Commands,
    chunks: Query<(Entity, &ChunkContainer)>,
    mut chunk_manager: ResMut<ChunkManager>,
    query_camera: Query<
        (&GlobalTransform, &OrthographicProjection),
        (With<MainCamera>, Changed<GlobalTransform>),
    >,
    world_data: Res<WorldData>
) {
    if let Ok((camera_transform, projection)) = query_camera.get_single() {
        let camera_fov = get_camera_fov(camera_transform.translation().xy(), projection);
        let chunk_range = get_chunk_range_by_camera_fov(camera_fov, world_data.area.size());

        chunks.for_each(|(entity, ChunkContainer { pos })| {
            if !chunk_range.contains(*pos) {
                chunk_manager.spawned_chunks.remove(pos);
                commands.entity(entity).despawn_recursive();
            }
        });
    }
}

pub(super) fn spawn_chunk(
    commands: &mut Commands,
    block_assets: &BlockAssets,
    wall_assets: &WallAssets,
    world_data: &WorldData,
    chunk_pos: ChunkPos,
) { 
    let chunk = commands.spawn((
        Name::new(format!("ChunkContainer {}", chunk_pos)),
        ChunkContainer { pos: chunk_pos },
        DespawnOnGameExit,
        SpatialBundle {
            transform: Transform::from_xyz(
                chunk_pos.x as f32 * CHUNK_SIZE * TILE_SIZE + TILE_SIZE / 2.,
                -(chunk_pos.y as f32 + 1.) * CHUNK_SIZE * TILE_SIZE + TILE_SIZE / 2.,
                0.
            ),
            ..default()
        }
    ))
    .id();

    let tilemap_entity = commands.spawn_empty().id();
    let mut tile_storage = TileStorage::empty(CHUNKMAP_SIZE);

    let tile_crack_map_entity = commands.spawn_empty().id();
    let tile_crack_storage = TileStorage::empty(CHUNKMAP_SIZE);

    let wallmap_entity = commands.spawn_empty().id();
    let mut wall_storage = TileStorage::empty(CHUNKMAP_SIZE);

    let treemap_entity = commands.spawn_empty().id();
    let mut tree_storage = TileStorage::empty(CHUNKMAP_SIZE);

    let tree_branches_map_entity = commands.spawn_empty().id();
    let mut tree_branches_storage = TileStorage::empty(CHUNKMAP_SIZE);

    let tree_tops_map_entity = commands.spawn_empty().id();
    let mut tree_tops_storage = TileStorage::empty(CHUNKMAP_SIZE);

    for y in 0..CHUNK_SIZE_U {
        for x in 0..CHUNK_SIZE_U {
            let chunk_tile_pos = TilePos { 
                x, 
                y: CHUNK_SIZE_U - 1 - y
            };

            let map_tile_pos = TilePos {
                x: chunk_pos.x * CHUNK_SIZE_U + x,
                y: chunk_pos.y * CHUNK_SIZE_U + y
            };

            if let Some(&block) = world_data.get_block(map_tile_pos) {
                if let BlockType::Tree(tree) = block.block_type {
                    let index = tree.texture_atlas_pos(block.variant);
                    
                    match tree.frame_type {
                        TreeFrameType::BranchLeftLeaves | TreeFrameType::BranchRightLeaves => {
                            let tree_branch_entity = spawn_block(commands, block, chunk_tile_pos, tree_branches_map_entity, index);
                            commands.entity(tree_branches_map_entity).add_child(tree_branch_entity);
                            tree_branches_storage.set(&chunk_tile_pos, tree_branch_entity);
                        },
                        TreeFrameType::TopLeaves => {
                            let tree_top_entity = spawn_block(commands, block, chunk_tile_pos, tree_tops_map_entity, index);
                            commands.entity(tree_tops_map_entity).add_child(tree_top_entity);
                            tree_tops_storage.set(&chunk_tile_pos, tree_top_entity);
                        },
                        _ => {
                            let tree_entity = spawn_block(commands, block, chunk_tile_pos, treemap_entity, index);
                            commands.entity(treemap_entity).add_child(tree_entity);
                            tree_storage.set(&chunk_tile_pos, tree_entity);
                        }
                    }
                } else {
                    let index = Block::get_sprite_index(
                        &world_data.get_block_neighbors(map_tile_pos, block.is_solid()).map_ref(|b| b.block_type), 
                        &block
                    );

                    let tile_entity = spawn_block(commands, block, chunk_tile_pos, tilemap_entity, index);

                    commands.entity(tilemap_entity).add_child(tile_entity);
                    tile_storage.set(&chunk_tile_pos, tile_entity);
                }

                if let Some(index) = block.cracks_index {
                    let cracks_entity = spawn_cracks(commands, chunk_tile_pos, tilemap_entity, index);
                    commands.entity(tilemap_entity).add_child(cracks_entity);
                    tile_storage.set(&chunk_tile_pos, cracks_entity);
                }
            }

            if let Some(wall) = world_data.get_wall(map_tile_pos) {
                let index = Wall::get_sprite_index(
                    &world_data.get_wall_neighbors(map_tile_pos).map_ref(|w| w.wall_type),
                    wall
                ).to_wall_index();

                let wall_entity = spawn_wall(commands, chunk_tile_pos, wallmap_entity, index);

                commands.entity(wallmap_entity).add_child(wall_entity);
                wall_storage.set(&chunk_tile_pos, wall_entity);

                if let Some(index) = wall.cracks_index {
                    let cracks_entity = spawn_cracks(commands, chunk_tile_pos, tilemap_entity, index);
                    commands.entity(tilemap_entity).add_child(cracks_entity);
                    tile_storage.set(&chunk_tile_pos, cracks_entity);
                }
            }
        }
    }

    commands
        .entity(tilemap_entity)
        .insert((
            Name::new("TileMap"),
            Chunk::new(chunk_pos, ChunkType::Tile),
            WORLD_RENDER_LAYER,
            TilemapBundle {
                grid_size: TilemapGridSize {
                    x: TILE_SIZE,
                    y: TILE_SIZE,
                },
                size: CHUNKMAP_SIZE,
                storage: tile_storage,
                texture: TilemapTexture::Single(block_assets.tiles.clone_weak()),
                tile_size: TilemapTileSize {
                    x: TILE_SIZE,
                    y: TILE_SIZE,
                },
                spacing: TilemapSpacing {
                    x: 2.,
                    y: 2.
                },
                transform: Transform::from_xyz(0., 0., TILES_LAYER + 0.5),
                ..default()
            }
        ));

    commands
        .entity(tile_crack_map_entity)
        .insert((
            Name::new("TileCracksMap"),
            Chunk::new(chunk_pos, ChunkType::Cracks),
            WORLD_RENDER_LAYER,
            NoFrustumCulling,
            TilemapBundle {
                grid_size: TilemapGridSize {
                    x: TILE_SIZE,
                    y: TILE_SIZE,
                },
                size: CHUNKMAP_SIZE,
                storage: tile_crack_storage,
                texture: TilemapTexture::Single(block_assets.tile_cracks.clone_weak()),
                tile_size: TilemapTileSize {
                    x: TILE_SIZE,
                    y: TILE_SIZE,
                },
                transform: Transform::from_xyz(0., 0., TILES_LAYER + 0.6),
                ..Default::default()
            }
        ));

    commands
        .entity(wallmap_entity)
        .insert((
            Name::new("WallMap"),
            Chunk::new(chunk_pos, ChunkType::Wall),
            WORLD_RENDER_LAYER,
            NoFrustumCulling,
            TilemapBundle {
                grid_size: TilemapGridSize {
                    x: TILE_SIZE,
                    y: TILE_SIZE,
                },
                size: CHUNKMAP_SIZE,
                storage: wall_storage,
                texture: TilemapTexture::Single(wall_assets.walls.clone_weak()),
                tile_size: TilemapTileSize {
                    x: WALL_SIZE,
                    y: WALL_SIZE,
                },
                transform: Transform::from_xyz(0., 0., WALL_LAYER),
                ..Default::default()
            }
        ));

    commands
        .entity(treemap_entity)
        .insert((
            Name::new("TreeMap"),
            Chunk::new(chunk_pos, ChunkType::Tree),
            WORLD_RENDER_LAYER,
            NoFrustumCulling,
            TilemapBundle {
                grid_size: TilemapGridSize {
                    x: TILE_SIZE,
                    y: TILE_SIZE,
                },
                size: CHUNKMAP_SIZE,
                storage: tree_storage,
                texture: TilemapTexture::Single(block_assets.trees.clone_weak()),
                tile_size: TREE_SIZE,
                transform: Transform::from_xyz(0., 0., TILES_LAYER + 0.1),
                spacing: TilemapSpacing { 
                    x: 2.,
                    y: 2.,
                },
                ..Default::default()
            }
        ));

    commands
        .entity(tree_branches_map_entity)
        .insert((
            Name::new("TreeBranchesMap"),
            Chunk::new(chunk_pos, ChunkType::TreeBranch),
            WORLD_RENDER_LAYER,
            NoFrustumCulling,
            TilemapBundle {
                grid_size: TilemapGridSize {
                    x: TILE_SIZE,
                    y: TILE_SIZE,
                },
                size: CHUNKMAP_SIZE,
                storage: tree_branches_storage,
                texture: TilemapTexture::Single(block_assets.tree_branches_forest.clone_weak()),
                tile_size: TREE_BRANCHES_SIZE,
                transform: Transform::from_xyz(0., 0., TILES_LAYER + 0.2),
                spacing: TilemapSpacing { 
                    x: 2.,
                    y: 2.,
                },
                ..Default::default()
            }
        ));

    commands
        .entity(tree_tops_map_entity)
        .insert((
            Name::new("TreeTopsMap"),
            Chunk::new(chunk_pos, ChunkType::TreeTop),
            WORLD_RENDER_LAYER,
            NoFrustumCulling,
            TilemapBundle {
                grid_size: TilemapGridSize {
                    x: TILE_SIZE,
                    y: TILE_SIZE,
                },
                size: CHUNKMAP_SIZE,
                storage: tree_tops_storage,
                texture: TilemapTexture::Single(block_assets.tree_tops_forest.clone_weak()),
                tile_size: TREE_TOPS_SIZE,
                transform: Transform::from_xyz(0., 0., TILES_LAYER + 0.2),
                ..Default::default()
            }
        ));

    commands
        .entity(chunk)
        .push_children(
            &[tilemap_entity, wallmap_entity, treemap_entity, tree_branches_map_entity, tree_tops_map_entity, tile_crack_map_entity]
        );
}

pub(super) fn handle_break_tile_event(
    mut commands: Commands,
    mut query_chunk: Query<(&Chunk, &mut TileStorage)>,
    mut world_data: ResMut<WorldData>,
    mut break_tile: EventReader<BreakTileEvent>,
    mut update_neighbors: EventWriter<UpdateNeighborsEvent>,
    mut tile_removed: EventWriter<TileRemovedEvent>,
) {
    let mut rng = thread_rng();

    for &BreakTileEvent { tile_pos, tile_type } in break_tile.iter() {
        match tile_type {
            TileType::Block(_) => {
                let Some(block_type) = world_data.get_block(tile_pos).map(|b| b.block_type) else { continue; };

                if let BlockType::Tree(_) = block_type {
                    break_tree(&mut commands, &mut world_data, &mut query_chunk, &mut tile_removed, tile_pos, false);
                } else {
                    world_data.remove_block(tile_pos);

                    ChunkManager::remove(&mut commands, &mut query_chunk, tile_pos, ChunkType::from(block_type));
                    ChunkManager::remove(&mut commands, &mut query_chunk, tile_pos, ChunkType::Cracks);

                    if let Some(particle) = Particle::get_by_block(block_type) {
                        utils::spawn_particles_on_break(&mut commands, particle, tile_pos);
                    }

                    commands.spawn_dropped_item(
                        tile_to_world_pos(tile_pos),
                        Vec2::new(rng.gen_range(-0.5f32..0.5f32), rng.gen_range(0.5f32..1.0f32)) * 3.,
                        ItemStack::new_block(block_type.into()),
                        None
                    );
                }

                commands.play_sound(SoundType::BlockHit(block_type));
                update_neighbors.send(UpdateNeighborsEvent { tile_pos });
            },
            TileType::Wall(_) => {
                let Some(wall_type) = world_data.get_wall(tile_pos).map(|w| w.wall_type) else { continue; };

                world_data.remove_wall(tile_pos);

                ChunkManager::remove(&mut commands, &mut query_chunk, tile_pos, ChunkType::Wall);
                ChunkManager::remove(&mut commands, &mut query_chunk, tile_pos, ChunkType::Cracks);

                if let Some(particle) = Particle::get_by_wall(wall_type) {
                    utils::spawn_particles_on_break(&mut commands, particle, tile_pos);
                }

                commands.play_sound(SoundType::WallHit);
                update_neighbors.send(UpdateNeighborsEvent { tile_pos });
            },
        }
        tile_removed.send(TileRemovedEvent { tile_pos, tile_type });
    }
}

pub(super) fn handle_dig_block_event(
    mut commands: Commands,
    mut world_data: ResMut<WorldData>,
    mut break_block_events: EventWriter<BreakTileEvent>,
    mut update_block_events: EventWriter<UpdateBlockEvent>,
    mut update_cracks_events: EventWriter<UpdateCracksEvent>,
    mut update_neighbors_events: EventWriter<UpdateNeighborsEvent>,
    mut dig_block_events: EventReader<DigBlockEvent>,
) {
    let mut rng = thread_rng();

    for &DigBlockEvent { tile_pos, tool } in dig_block_events.iter() {
        if let Some(block) = world_data.get_block_mut(tile_pos) {
            block.hp -= tool.power();

            if block.hp <= 0 {
                break_block_events.send(BreakTileEvent {
                    tile_pos,
                    tile_type: TileType::Block(Some(block.block_type))
                });
            } else {
                if let Some(particle) = Particle::get_by_block(block.block_type) {
                    utils::spawn_particles_on_dig(&mut commands, particle, tile_pos);
                }

                if block.block_type == BlockType::Grass {
                    block.block_type = BlockType::Dirt;
                }

                let x = block.cracks_index.map(|index| index / 6).unwrap_or_else(|| rng.gen_range(0..6));
                let y = map_range_i32(block.max_hp(), 0, 0, 3, block.hp) as u32;
                let index = TextureAtlasPos::new(x, y).to_2d_index(6);

                block.cracks_index = Some(index);

                commands.play_sound(SoundType::BlockHit(block.block_type));
                update_block_events.send(UpdateBlockEvent { tile_pos, block: *block });
                update_neighbors_events.send(UpdateNeighborsEvent { tile_pos });
                update_cracks_events.send(UpdateCracksEvent { tile_pos, index });
            }
        }
    }
}

pub(super) fn handle_dig_wall_event(
    mut commands: Commands,
    mut world_data: ResMut<WorldData>,
    mut break_tile_events: EventWriter<BreakTileEvent>,
    mut update_wall_events: EventWriter<UpdateWallEvent>,
    mut update_cracks_events: EventWriter<UpdateCracksEvent>,
    mut update_neighbors_events: EventWriter<UpdateNeighborsEvent>,
    mut dig_wall_events: EventReader<DigWallEvent>,
) {
    let mut rng = thread_rng();

    for &DigWallEvent { tile_pos, tool } in dig_wall_events.iter() {
        if let Some(wall) = world_data.get_wall_mut(tile_pos) {
            wall.hp -= tool.power();

            if wall.hp <= 0 {
                break_tile_events.send(BreakTileEvent {
                    tile_pos,
                    tile_type: TileType::Wall(Some(wall.wall_type))
                });
            } else {
                if let Some(particle) = Particle::get_by_wall(wall.wall_type) {
                    utils::spawn_particles_on_dig(&mut commands, particle, tile_pos);
                }

                let x = wall.cracks_index.map(|index| index / 6).unwrap_or_else(|| rng.gen_range(0..6));
                let y = map_range_i32(wall.max_hp(), 0, 0, 3, wall.hp) as u32;
                let index = TextureAtlasPos::new(x, y).to_2d_index(6);

                wall.cracks_index = Some(index);

                commands.play_sound(SoundType::WallHit);
                update_wall_events.send(UpdateWallEvent { tile_pos, wall: *wall });
                update_neighbors_events.send(UpdateNeighborsEvent { tile_pos });
                update_cracks_events.send(UpdateCracksEvent { tile_pos, index });
            }
        }
    }
}

pub(super) fn handle_place_tile_event(
    mut commands: Commands,
    mut query_chunk: Query<(&Chunk, &mut TileStorage, Entity)>,
    mut world_data: ResMut<WorldData>,
    mut place_block: EventReader<PlaceTileEvent>,
    mut update_neighbors: EventWriter<UpdateNeighborsEvent>,
) {
    for &PlaceTileEvent { tile_pos, tile_type } in place_block.iter() {
        match tile_type {
            TileType::Block(Some(block_type)) => {
                if world_data.block_exists(tile_pos) { continue; }

                let new_block = Block::from(block_type);
                
                world_data.set_block(tile_pos, new_block);

                let neighbors = world_data
                    .get_block_neighbors(tile_pos, block_type.is_solid())
                    .map_ref(|b| b.block_type);
                
                let index = Block::get_sprite_index(&neighbors, &new_block);

                ChunkManager::spawn_block(&mut commands, &mut query_chunk, tile_pos, &new_block, index);

                update_neighbors.send(UpdateNeighborsEvent { tile_pos });
                commands.play_sound(SoundType::BlockHit(block_type));
            },
            TileType::Wall(Some(wall_type)) => {
                if world_data.wall_exists(tile_pos) { continue; }

                let new_wall = Wall::from(wall_type);
                
                world_data.set_wall(tile_pos, new_wall);

                let neighbors = world_data
                    .get_wall_neighbors(tile_pos)
                    .map_ref(|w| w.wall_type);
                
                let index = Wall::get_sprite_index(&neighbors, &new_wall).to_wall_index();

                ChunkManager::spawn_wall(&mut commands, &mut query_chunk, tile_pos, index);

                update_neighbors.send(UpdateNeighborsEvent { tile_pos });
                commands.play_sound(SoundType::WallHit);
            },
            _ => unreachable!()
        }
    }
}

pub(super) fn handle_update_neighbors_event(
    world_data: Res<WorldData>,
    mut events: EventReader<UpdateNeighborsEvent>,
    mut update_block_events: EventWriter<UpdateBlockEvent>,
    mut update_wall_events: EventWriter<UpdateWallEvent>,
) {
    for UpdateNeighborsEvent { tile_pos } in events.iter() {
        let map_size = TilemapSize::from(world_data.area.size());
        
        let neighbor_positions = Neighbors::get_square_neighboring_positions(tile_pos, &map_size, true);

        for pos in neighbor_positions.iter() {
            if let Some(block) = world_data.get_block(*pos) {
                update_block_events.send(UpdateBlockEvent { 
                    tile_pos: *pos,
                    block: *block,
                });
            }

            if let Some(wall) = world_data.get_wall(*pos) {
                update_wall_events.send(UpdateWallEvent { 
                    tile_pos: *pos,
                    wall: *wall,
                });
            }
        }
    }
}

pub(super) fn handle_update_block_event(
    mut update_block_events: EventReader<UpdateBlockEvent>,
    mut query_tile: Query<&mut TileTextureIndex>,
    query_chunk: Query<(&Chunk, &TileStorage)>,
    world_data: Res<WorldData>
) {
    for &UpdateBlockEvent { tile_pos, block } in update_block_events.iter() {
        let neighbors = world_data
            .get_block_neighbors(tile_pos, block.is_solid())
            .map_ref(|b| b.block_type);

        let chunk_pos = get_chunk_pos(tile_pos);
        let chunk_tile_pos = get_chunk_tile_pos(tile_pos);

        if let Some(block_entity) = ChunkManager::get_block_entity(&query_chunk, chunk_pos, chunk_tile_pos, block.block_type) {
            if let Ok(mut tile_texture) = query_tile.get_mut(block_entity) {
                tile_texture.0 = Block::get_sprite_index(&neighbors, &block);
            }
        }
    }
}

pub(super) fn handle_update_wall_event(
    mut update_wall_events: EventReader<UpdateWallEvent>,
    mut query_tile: Query<&mut TileTextureIndex>,
    query_chunk: Query<(&Chunk, &TileStorage)>,
    world_data: Res<WorldData>
) {
    for &UpdateWallEvent { tile_pos, wall } in update_wall_events.iter() {
        let neighbors = world_data
            .get_wall_neighbors(tile_pos)
            .map_ref(|b| b.wall_type);

        let chunk_pos = get_chunk_pos(tile_pos);
        let chunk_tile_pos = get_chunk_tile_pos(tile_pos);

        if let Some(wall_entity) = ChunkManager::get_wall_entity(&query_chunk, chunk_pos, chunk_tile_pos) {
            if let Ok(mut tile_texture) = query_tile.get_mut(wall_entity) {
                tile_texture.0 = Wall::get_sprite_index(&neighbors, &wall).to_wall_index();
            }
        }
    }
}

pub(super) fn handle_seed_event(
    mut commands: Commands,
    mut seed_events: EventReader<SeedEvent>,
    mut update_block_events: EventWriter<UpdateBlockEvent>,
    mut update_neighbors_events: EventWriter<UpdateNeighborsEvent>,
    mut world_data: ResMut<WorldData>,
) {
    for &SeedEvent { tile_pos, seed } in seed_events.iter() {
        if let Some(block) = world_data.get_block_with_type_mut(tile_pos, BlockType::Dirt) {
            commands.play_sound(SoundType::BlockPlace(block.block_type));
            
            block.block_type = seed.seeded_dirt();

            update_block_events.send(UpdateBlockEvent { 
                tile_pos,
                block: *block,
            });
            update_neighbors_events.send(UpdateNeighborsEvent { tile_pos });
        }
    }
}

pub(super) fn handle_update_cracks_event(
    mut commands: Commands,
    world_data: Res<WorldData>,
    mut update_cracks_events: EventReader<UpdateCracksEvent>,
    mut query_tile: Query<&mut TileTextureIndex>,
    mut query_chunk: Query<(&Chunk, &mut TileStorage, Entity)>,
) {
    for &UpdateCracksEvent { tile_pos, index } in update_cracks_events.iter() {
        if world_data.get_block(tile_pos).is_some_and(|b| b.cracks()) || world_data.wall_exists(tile_pos) {
            ChunkManager::update_tile_cracks(&mut commands, &mut query_chunk, &mut query_tile, tile_pos, index);
        }
    }
}

fn break_tree(
    commands: &mut Commands, 
    world_data: &mut ResMut<WorldData>,
    chunks: &mut Query<(&Chunk, &mut TileStorage)>,
    tile_removed: &mut EventWriter<TileRemovedEvent>,
    pos: TilePos,
    tree_falling: bool
) {
    if let Some(&block) = world_data.get_block(pos) {
        if let BlockType::Tree(tree) = block.block_type {
            world_data.remove_block(pos);

            ChunkManager::remove(commands, chunks, pos, ChunkType::from(block.block_type));
            ChunkManager::remove(commands, chunks, pos, ChunkType::Cracks);
            utils::spawn_particles_on_break(commands, Particle::Wood, pos);

            commands.spawn_dropped_item(
                tile_to_world_pos(pos),
                random_point_cone(Vec2::Y, 150.0, 1.) * 4.,
                ItemStack::new_block(block.block_type.into()),
                None
            );

            tile_removed.send(TileRemovedEvent { tile_pos: pos, tile_type: TileType::Block(Some(block.block_type)) });

            if tree.frame_type.is_trunk() || tree_falling {
                break_tree(commands, world_data, chunks, tile_removed, TilePos::new(pos.x + 1, pos.y), true);
                break_tree(commands, world_data, chunks, tile_removed, TilePos::new(pos.x - 1, pos.y), true);
                break_tree(commands, world_data, chunks, tile_removed, TilePos::new(pos.x, pos.y - 1), true);
            }
        }
    }
}

#[cfg(feature = "debug")]
use bevy::prelude::{Visibility, DetectChanges};

#[cfg(feature = "debug")]
pub(super) fn set_tiles_visibility(
    debug_config: Res<DebugConfiguration>,
    mut query_chunk: Query<(&mut Visibility, &Chunk)>
) {
    use crate::common::helpers::set_visibility;

    if debug_config.is_changed() {
        for (visibility, chunk) in &mut query_chunk {
            if chunk.chunk_type != ChunkType::Wall {
                set_visibility(visibility, debug_config.show_tiles);
            } else {
                set_visibility(visibility, debug_config.show_walls);
            }
        }
    }
}
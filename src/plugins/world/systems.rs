use std::time::{SystemTime, UNIX_EPOCH};

use bevy::{
    prelude::{
        EventReader, ResMut, Query, Commands, EventWriter, Entity, BuildChildren, Transform, 
        default, SpatialBundle, DespawnRecursiveExt, OrthographicProjection, Changed, 
        GlobalTransform, With, Res, UVec2, Audio, NextState, Vec2
    }, 
    math::Vec3Swizzles
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
use rand::{thread_rng, seq::SliceRandom};

use crate::{plugins::{world::{CHUNK_SIZE, TILE_SIZE}, assets::{BlockAssets, WallAssets, SoundAssets}, camera::{MainCamera, UpdateLightEvent}, player::{Player, PlayerRect}}, common::{state::GameState, helpers::tile_pos_to_world_coords, rect::FRect}, world::{WorldSize, chunk::{Chunk, ChunkType, ChunkContainer, ChunkPos}, WorldData, block::{BlockType, Block}, wall::Wall, tree::TreeFrameType, generator::generate_world}};

use super::{get_chunk_pos, CHUNK_SIZE_U, UpdateNeighborsEvent, WALL_SIZE, CHUNKMAP_SIZE, get_camera_fov, ChunkManager, get_chunk_tile_pos, BreakBlockEvent, DigBlockEvent, PlaceBlockEvent, TREE_SIZE, TREE_BRANCHES_SIZE, TREE_TOPS_SIZE, utils::get_chunk_range_by_camera_fov, UpdateBlockEvent, SeedEvent};

#[cfg(feature = "debug")]
use crate::plugins::debug::DebugConfiguration;

pub(super) fn spawn_terrain(mut commands: Commands) {
    let _current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

    // let seed = current_time.as_millis() as u32;
    let seed = 1837178180;

    println!("The world's seed is {}", seed);

    let world_data = generate_world(seed, WorldSize::Tiny);
    // let light_map = generate_light_map(&world_data);

    commands.insert_resource(world_data);
    // commands.insert_resource(LightMap::new(light_map));
    commands.insert_resource(NextState(Some(GameState::InGame)));
}

pub(super) fn spawn_block(
    commands: &mut Commands,
    block: Block,
    tile_pos: TilePos,
    tilemap_entity: Entity,
    index: u32
) -> Entity {
    commands
        .spawn(TileBundle {
            position: tile_pos,
            tilemap_id: TilemapId(tilemap_entity),
            texture_index: TileTextureIndex(index),
            ..default()
        })
        .insert(block)
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
        let chunk_range = get_chunk_range_by_camera_fov(camera_fov, world_data.size);

        for y in chunk_range.y {
            for x in chunk_range.x.clone() {
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
        let chunk_range = get_chunk_range_by_camera_fov(camera_fov, world_data.size);

        for (entity, ChunkContainer { pos: chunk_pos }) in chunks.iter() {
            if (chunk_pos.x < *chunk_range.x.start() || chunk_pos.x > *chunk_range.x.end()) ||
               (chunk_pos.y > *chunk_range.y.end() || chunk_pos.y < *chunk_range.y.start()) 
            {
                chunk_manager.spawned_chunks.remove(chunk_pos);
                commands.entity(entity).despawn_recursive();
            }
        }
    }
}

pub(super) fn spawn_chunk(
    commands: &mut Commands,
    block_assets: &BlockAssets,
    wall_assets: &WallAssets,
    world_data: &WorldData,
    chunk_pos: ChunkPos,
) { 
    let chunk = commands.spawn(SpatialBundle {
        transform: Transform::from_xyz(chunk_pos.x as f32 * CHUNK_SIZE * TILE_SIZE, -(chunk_pos.y as f32 + 1.) * CHUNK_SIZE * TILE_SIZE + TILE_SIZE, 0.),
        ..default()
        })
        .insert(ChunkContainer { pos: chunk_pos })
        .id();

    let tilemap_entity = commands.spawn_empty().id();
    let mut tile_storage = TileStorage::empty(CHUNKMAP_SIZE);

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
                x: (chunk_pos.x as f32 * CHUNK_SIZE) as u32 + x,
                y: (chunk_pos.y as f32 * CHUNK_SIZE) as u32 + y
            };

            if let Some(block) = world_data.get_block(map_tile_pos) {
                if let BlockType::Tree(tree) = block.block_type {
                    let index = tree.texture_atlas_pos();
                    
                    match tree.frame_type {
                        TreeFrameType::BranchLeftLeaves | TreeFrameType::BranchRightLeaves => {
                            let tree_branch_entity = spawn_block(commands, *block, chunk_tile_pos, tree_branches_map_entity, index);
                            commands.entity(tree_branches_map_entity).add_child(tree_branch_entity);
                            tree_branches_storage.set(&chunk_tile_pos, tree_branch_entity);
                        },
                        TreeFrameType::TopLeaves => {
                            let tree_top_entity = spawn_block(commands, *block, chunk_tile_pos, tree_tops_map_entity, index);
                            commands.entity(tree_tops_map_entity).add_child(tree_top_entity);
                            tree_tops_storage.set(&chunk_tile_pos, tree_top_entity);
                        },
                        _ => {
                            let tree_entity = spawn_block(commands, *block, chunk_tile_pos, treemap_entity, index);
                            commands.entity(treemap_entity).add_child(tree_entity);
                            tree_storage.set(&chunk_tile_pos, tree_entity);
                        }
                    }
                } else {
                    let index = Block::get_sprite_index(
                        &world_data.get_block_neighbors(map_tile_pos, block.is_solid()).map_ref(|b| b.block_type), 
                        block.block_type
                    );

                    let tile_entity = spawn_block(commands, *block, chunk_tile_pos, tilemap_entity, index);

                    commands.entity(tilemap_entity).add_child(tile_entity);
                    tile_storage.set(&chunk_tile_pos, tile_entity);
                }
            }

            if let Some(wall) = world_data.get_wall(map_tile_pos) {
                let index = Wall::get_sprite_index(
                    world_data.get_wall_neighbors(map_tile_pos).map_ref(|w| **w),
                    *wall
                ).to_wall_index();

                let wall_entity = spawn_wall(commands, chunk_tile_pos, wallmap_entity, index);

                commands.entity(wallmap_entity).add_child(wall_entity);
                wall_storage.set(&chunk_tile_pos, wall_entity);
            }
        }
    }

    commands
        .entity(tilemap_entity)
        .insert(Chunk::new(chunk_pos, ChunkType::Tile))
        .insert(TilemapBundle {
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
            transform: Transform::from_xyz(0., 0., 2.),
            ..Default::default()
        });

    commands
        .entity(wallmap_entity)
        .insert(Chunk::new(chunk_pos, ChunkType::Wall))
        .insert(TilemapBundle {
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
            transform: Transform::from_xyz(0., 0., 1.),
            ..Default::default()
        });

    commands
        .entity(treemap_entity)
        .insert(Chunk::new(chunk_pos, ChunkType::Tree))
        .insert(TilemapBundle {
            grid_size: TilemapGridSize {
                x: TILE_SIZE,
                y: TILE_SIZE,
            },
            size: CHUNKMAP_SIZE,
            storage: tree_storage,
            texture: TilemapTexture::Single(block_assets.trees.clone_weak()),
            tile_size: TREE_SIZE,
            transform: Transform::from_xyz(0., 0., 1.5),
            spacing: TilemapSpacing { 
                x: 2.,
                y: 2.,
            },
            ..Default::default()
        });

    commands
        .entity(tree_branches_map_entity)
        .insert(Chunk::new(chunk_pos, ChunkType::TreeBranch))
        .insert(TilemapBundle {
            grid_size: TilemapGridSize {
                x: TILE_SIZE,
                y: TILE_SIZE,
            },
            size: CHUNKMAP_SIZE,
            storage: tree_branches_storage,
            texture: TilemapTexture::Single(block_assets.tree_branches_forest.clone_weak()),
            tile_size: TREE_BRANCHES_SIZE,
            transform: Transform::from_xyz(0., 0., 1.6),
            spacing: TilemapSpacing { 
                x: 2.,
                y: 2.,
            },
            ..Default::default()
        });

    commands
        .entity(tree_tops_map_entity)
        .insert(Chunk::new(chunk_pos, ChunkType::TreeTop))
        .insert(TilemapBundle {
            grid_size: TilemapGridSize {
                x: TILE_SIZE,
                y: TILE_SIZE,
            },
            size: CHUNKMAP_SIZE,
            storage: tree_tops_storage,
            texture: TilemapTexture::Single(block_assets.tree_tops_forest.clone_weak()),
            tile_size: TREE_TOPS_SIZE,
            transform: Transform::from_xyz(0., 0., 1.6),
            ..Default::default()
        });

    commands
        .entity(chunk)
        .push_children(
            &[tilemap_entity, wallmap_entity, treemap_entity, tree_branches_map_entity, tree_tops_map_entity]
        );
}

pub(super) fn handle_break_block_event(
    mut commands: Commands,
    mut world_data: ResMut<WorldData>,
    mut break_block_events: EventReader<BreakBlockEvent>,
    mut update_light_events: EventWriter<UpdateLightEvent>,
    mut update_neighbors_ew: EventWriter<UpdateNeighborsEvent>,
    mut query_chunk: Query<(&Chunk, &mut TileStorage)>,
    sound_assets: Res<SoundAssets>,
    audio: Res<Audio>
) {
    let mut rng = thread_rng();

    for &BreakBlockEvent { tile_pos } in break_block_events.iter() {
        if let Some(&block) = world_data.get_block(tile_pos) {
            if let BlockType::Tree(_) = block.block_type {
                break_tree(&mut commands, &mut query_chunk, tile_pos, &mut world_data, false);
            } else {
                world_data.remove_block(tile_pos);

                ChunkManager::remove_block(&mut commands, &mut query_chunk, tile_pos, block.block_type);

                update_light_events.send(UpdateLightEvent);
            }

            audio.play(sound_assets.get_by_block(block.block_type, &mut rng));

            update_neighbors_ew.send(UpdateNeighborsEvent { tile_pos });
        }
    }
}

pub(super) fn handle_dig_block_event(
    mut world_data: ResMut<WorldData>,
    mut break_block_events: EventWriter<BreakBlockEvent>,
    mut update_block_events: EventWriter<UpdateBlockEvent>,
    mut dig_block_events: EventReader<DigBlockEvent>,
    sound_assets: Res<SoundAssets>,
    audio: Res<Audio>,
    #[cfg(feature = "debug")]
    debug_config: Res<DebugConfiguration>
) {
    let mut rng = thread_rng();

    for &DigBlockEvent { tile_pos, tool } in dig_block_events.iter() {
        if let Some(block) = world_data.get_block_mut(tile_pos) {
            if !block.check_required_tool(tool) { continue; }

            #[cfg(feature = "debug")]
            if debug_config.instant_break {
                break_block_events.send(BreakBlockEvent { tile_pos });
                return;
            }

            block.hp -= tool.power();

            if block.hp <= 0 {
                break_block_events.send(BreakBlockEvent { tile_pos });
            } else {
                audio.play(sound_assets.get_by_block(block.block_type, &mut rng));
                
                if block.block_type == BlockType::Grass {
                    block.block_type = BlockType::Dirt;

                    update_block_events.send(UpdateBlockEvent {
                        tile_pos,
                        block_type: block.block_type,
                        update_neighbors: true
                    });
                }
            }
        }
    }
}

pub(super) fn handle_place_block_event(
    mut commands: Commands,
    mut world_data: ResMut<WorldData>,
    mut place_block_events: EventReader<PlaceBlockEvent>,
    mut update_light_events: EventWriter<UpdateLightEvent>,
    mut update_neighbors_events: EventWriter<UpdateNeighborsEvent>,
    mut query_chunk: Query<(&Chunk, &mut TileStorage, Entity)>,
    query_player: Query<&PlayerRect, With<Player>>,
    sound_assets: Res<SoundAssets>,
    audio: Res<Audio>
) {
    let player_rect = query_player.single();

    for &PlaceBlockEvent { tile_pos, block } in place_block_events.iter() {
        if world_data.block_exists(tile_pos) { continue; }

        // Forbid to place a block inside the player 
        {
            let Vec2 { x, y } = tile_pos_to_world_coords(tile_pos);
            let tile_rect = FRect::new_center(x, y, TILE_SIZE, TILE_SIZE);
            if player_rect.intersects(&tile_rect) { continue; }
        }
        
        world_data.set_block(tile_pos, &block);

        let neighbors = world_data
            .get_block_neighbors(tile_pos, block.is_solid())
            .map_ref(|b| b.block_type);
        
        let index = Block::get_sprite_index(&neighbors, block.block_type);

        ChunkManager::spawn_block(&mut commands, &mut query_chunk, tile_pos, &block, index);

        update_neighbors_events.send(UpdateNeighborsEvent { tile_pos });
        update_light_events.send(UpdateLightEvent);

        audio.play(sound_assets.get_by_block(block.block_type, &mut thread_rng()));
    }
}

pub(super) fn handle_update_neighbors_event(
    world_data: Res<WorldData>,
    mut events: EventReader<UpdateNeighborsEvent>,
    mut update_block_events: EventWriter<UpdateBlockEvent>
) {
    for UpdateNeighborsEvent { tile_pos } in events.iter() {
        let map_size = TilemapSize {
            x: world_data.size.width as u32,
            y: world_data.size.height as u32,
        };
        
        let neighbor_positions = Neighbors::get_square_neighboring_positions(tile_pos, &map_size, false);

        for pos in neighbor_positions.iter() {
            if let Some(block) = world_data.get_block(*pos) {
                update_block_events.send(UpdateBlockEvent { 
                    tile_pos: *pos,
                    block_type: block.block_type,
                    update_neighbors: false
                });
            }
        }
    }
}

pub(super) fn handle_update_block_event(
    mut update_block_events: EventReader<UpdateBlockEvent>,
    mut update_neighbors_events: EventWriter<UpdateNeighborsEvent>,
    mut query_tile: Query<&mut TileTextureIndex>,
    query_chunk: Query<(&Chunk, &TileStorage)>,
    world_data: Res<WorldData>
) {
    for UpdateBlockEvent { tile_pos, block_type, update_neighbors } in update_block_events.iter() {
        let neighbors = world_data
            .get_block_neighbors(tile_pos, block_type.is_solid())
            .map_ref(|b| b.block_type);

        let chunk_pos = get_chunk_pos(*tile_pos);
        let chunk_tile_pos = get_chunk_tile_pos(*tile_pos);

        if let Some(block_entity) = ChunkManager::get_block_entity(&query_chunk, chunk_pos, chunk_tile_pos, *block_type) {
            if let Ok(mut tile_texture) = query_tile.get_mut(block_entity) {
                tile_texture.0 = Block::get_sprite_index(&neighbors, *block_type);
            }
        }

        if *update_neighbors {
            update_neighbors_events.send(UpdateNeighborsEvent { tile_pos: *tile_pos });
        }
    }
}

pub(super) fn handle_seed_event(
    mut seed_events: EventReader<SeedEvent>,
    mut update_block_events: EventWriter<UpdateBlockEvent>,
    mut world_data: ResMut<WorldData>,
    sound_assets: Res<SoundAssets>,
    audio: Res<Audio>
) {
    let mut rng = thread_rng();

    for &SeedEvent { tile_pos: world_pos, seed } in seed_events.iter() {
        if let Some(block) = world_data.get_block_with_type_mut(world_pos, BlockType::Dirt) {
            block.block_type = seed.seeded_dirt();

            update_block_events.send(UpdateBlockEvent { 
                tile_pos: world_pos,
                block_type: block.block_type,
                update_neighbors: true
            });

            audio.play(sound_assets.dig.choose(&mut rng).unwrap().clone_weak());
        }
    }
}

fn break_tree(
    commands: &mut Commands, 
    chunks: &mut Query<(&Chunk, &mut TileStorage)>, 
    pos: TilePos, 
    world_data: &mut ResMut<WorldData>,
    tree_falling: bool
) {
    if let Some(&block) = world_data.get_block(pos) {
        if let BlockType::Tree(tree) = block.block_type {
            world_data.remove_block(pos);

            ChunkManager::remove_block(commands, chunks, pos, block.block_type);

            if tree.frame_type.is_stem() || tree_falling {
                break_tree(commands, chunks, TilePos::new(pos.x + 1, pos.y), world_data, true);
                break_tree(commands, chunks, TilePos::new(pos.x - 1, pos.y), world_data, true);
                break_tree(commands, chunks, TilePos::new(pos.x, pos.y - 1), world_data, true);
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
    if debug_config.is_changed() {
        for (mut visibility, chunk) in &mut query_chunk {
            if chunk.chunk_type != ChunkType::Wall {
                if debug_config.show_tiles {
                    *visibility = Visibility::Inherited;
                } else {
                    *visibility = Visibility::Hidden;
                }
            }

            if chunk.chunk_type == ChunkType::Wall {
                if debug_config.show_walls {
                    *visibility = Visibility::Inherited;
                } else {
                    *visibility = Visibility::Hidden;
                }
            }
        }
    }
}
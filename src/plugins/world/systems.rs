use std::time::{SystemTime, UNIX_EPOCH};

use bevy::{
    prelude::{
        EventReader, ResMut, Query, Commands, EventWriter, Entity, BuildChildren, Transform, 
        default, SpatialBundle, DespawnRecursiveExt, OrthographicProjection, Changed, 
        GlobalTransform, With, Res, UVec2, NextState, Vec2, Name, Assets, Mesh, shape::Quad
    }, 
    math::Vec3Swizzles, sprite::{MaterialMesh2dBundle, Mesh2dHandle}, render::view::NoFrustumCulling
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

use crate::{plugins::{assets::{BlockAssets, WallAssets}, camera::{components::MainCamera, events::UpdateLightEvent}, player::{Player, PlayerRect}, audio::{PlaySoundEvent, SoundType}, world::resources::LightMap, DespawnOnGameExit}, common::{state::GameState, helpers::tile_pos_to_world_coords, rect::FRect}, world::{WorldSize, chunk::{Chunk, ChunkType, ChunkContainer, ChunkPos}, WorldData, block::{BlockType, Block}, wall::Wall, tree::TreeFrameType, generator::generate_world, light::generate_light_map}, lighting::compositing::{LightMapTexture, LightMapMaterial}, WALL_LAYER, TILES_LAYER, PLAYER_LAYER};

use super::{
    utils::{get_chunk_pos, get_camera_fov, get_chunk_tile_pos, get_chunk_range_by_camera_fov}, 
    events::{UpdateNeighborsEvent, BreakBlockEvent, DigBlockEvent, PlaceBlockEvent, UpdateBlockEvent, SeedEvent},
    resources::{ChunkManager, LightMapChunkMesh}, 
    constants::{CHUNK_SIZE_U, WALL_SIZE, CHUNKMAP_SIZE, TREE_SIZE, TREE_BRANCHES_SIZE, TREE_TOPS_SIZE, CHUNK_SIZE, TILE_SIZE}, WORLD_RENDER_LAYER
};

#[cfg(feature = "debug")]
use crate::plugins::debug::DebugConfiguration;

pub(super) fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>
) {
    commands.init_resource::<ChunkManager>();

    let handle = meshes.add(Quad::new(Vec2::splat(CHUNK_SIZE * TILE_SIZE)).into());

    commands.insert_resource(LightMapChunkMesh(handle));
}

pub(super) fn spawn_terrain(mut commands: Commands) {
    let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

    let seed = current_time.as_millis() as u32;

    println!("The world's seed is {}", seed);

    let world_data = generate_world(seed, WorldSize::Tiny);
    let light_map = generate_light_map(&world_data);

    commands.insert_resource(world_data);
    commands.insert_resource(LightMap::new(light_map));
    commands.insert_resource(NextState(Some(GameState::InGame)));
}

pub(super) fn cleanup(mut commands: Commands) {
    commands.remove_resource::<WorldData>();
    commands.remove_resource::<LightMap>();
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
    light_map_texture: Res<LightMapTexture>,
    light_map_mesh: Res<LightMapChunkMesh>,
    mut chunk_manager: ResMut<ChunkManager>,
    mut tile_materials: ResMut<Assets<LightMapMaterial>>,
    query_camera: Query<
        (&GlobalTransform, &OrthographicProjection),
        (With<MainCamera>, Changed<Transform>),
    >
) {
    if let Ok((camera_transform, projection)) = query_camera.get_single() {

        let camera_fov = get_camera_fov(camera_transform.translation().xy(), projection);
        let chunk_range = get_chunk_range_by_camera_fov(camera_fov, world_data.size);

        for y in chunk_range.min.y..=chunk_range.max.y {
            for x in chunk_range.min.x..=chunk_range.max.x {
                let chunk_pos = UVec2::new(x, y);
                if chunk_manager.spawned_chunks.insert(chunk_pos) {
                    spawn_chunk(&mut commands, &block_assets, &wall_assets, &world_data, &mut tile_materials, &light_map_texture, &light_map_mesh, chunk_pos);
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
    light_map_materials: &mut Assets<LightMapMaterial>,
    light_map_texture: &LightMapTexture,
    light_map_mesh: &LightMapChunkMesh,
    chunk_pos: ChunkPos,
) { 
    let chunk = commands.spawn((
        Name::new(format!("ChunkContainer {}", chunk_pos)),
        ChunkContainer { pos: chunk_pos },
        DespawnOnGameExit,
        SpatialBundle {
            transform: Transform::from_xyz(chunk_pos.x as f32 * CHUNK_SIZE * TILE_SIZE, -(chunk_pos.y as f32 + 1.) * CHUNK_SIZE * TILE_SIZE + TILE_SIZE, 0.),
            ..default()
        }
    ))
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
                x: chunk_pos.x * CHUNK_SIZE_U + x,
                y: chunk_pos.y * CHUNK_SIZE_U + y
            };

            if let Some(&block) = world_data.get_block(map_tile_pos) {
                if let BlockType::Tree(tree) = block.block_type {
                    let index = tree.texture_atlas_pos();
                    
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
                        block.block_type
                    );

                    let tile_entity = spawn_block(commands, block, chunk_tile_pos, tilemap_entity, index);

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

    let mesh = commands.spawn((
        MaterialMesh2dBundle {
            mesh: Mesh2dHandle(light_map_mesh.clone_weak()),
            material: light_map_materials.add(LightMapMaterial {
                light_map_image: light_map_texture.0.clone_weak(),
                chunk_pos,
            }),
            transform: Transform::from_xyz(
                CHUNK_SIZE / 2. * TILE_SIZE - TILE_SIZE / 2.,
                CHUNK_SIZE / 2. * TILE_SIZE - TILE_SIZE / 2.,
                PLAYER_LAYER + 1.
            ),
            ..default()
        },
        NoFrustumCulling,
        WORLD_RENDER_LAYER
    ))
    .id();

    commands.entity(chunk).add_child(mesh);

    commands
        .entity(tilemap_entity)
        .insert((
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
        .entity(wallmap_entity)
        .insert((
            Chunk::new(chunk_pos, ChunkType::Wall),
            WORLD_RENDER_LAYER,
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
            Chunk::new(chunk_pos, ChunkType::Tree),
            WORLD_RENDER_LAYER,
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
            Chunk::new(chunk_pos, ChunkType::TreeBranch),
            WORLD_RENDER_LAYER,
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
            Chunk::new(chunk_pos, ChunkType::TreeTop),
            WORLD_RENDER_LAYER,
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
            &[tilemap_entity, wallmap_entity, treemap_entity, tree_branches_map_entity, tree_tops_map_entity]
        );
}

pub(super) fn handle_break_block_event(
    mut commands: Commands,
    mut query_chunk: Query<(&Chunk, &mut TileStorage)>,
    mut world_data: ResMut<WorldData>,
    mut break_block: EventReader<BreakBlockEvent>,
    mut update_light: EventWriter<UpdateLightEvent>,
    mut update_neighbors: EventWriter<UpdateNeighborsEvent>,
    mut play_sound: EventWriter<PlaySoundEvent>
) {
    for &BreakBlockEvent { tile_pos } in break_block.iter() {
        if let Some(&block) = world_data.get_block(tile_pos) {
            if let BlockType::Tree(_) = block.block_type {
                break_tree(&mut commands, &mut world_data, &mut query_chunk, tile_pos, false);
            } else {
                world_data.remove_block(tile_pos);

                ChunkManager::remove_block(&mut commands, &mut query_chunk, tile_pos, block.block_type);

                update_light.send(UpdateLightEvent { tile_pos });
            }

            play_sound.send(PlaySoundEvent(SoundType::BlockHit(block.block_type)));
            update_neighbors.send(UpdateNeighborsEvent { tile_pos });
        }
    }
}

pub(super) fn handle_dig_block_event(
    mut world_data: ResMut<WorldData>,
    mut break_block_events: EventWriter<BreakBlockEvent>,
    mut update_block_events: EventWriter<UpdateBlockEvent>,
    mut dig_block_events: EventReader<DigBlockEvent>,
    mut play_sound: EventWriter<PlaySoundEvent>
) {
    for &DigBlockEvent { tile_pos, tool } in dig_block_events.iter() {
        if let Some(block) = world_data.get_block_mut(tile_pos) {
            block.hp -= tool.power();

            if block.hp <= 0 {
                break_block_events.send(BreakBlockEvent { tile_pos });
            } else {
                play_sound.send(PlaySoundEvent(SoundType::BlockHit(block.block_type)));
                
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
    query_player: Query<&PlayerRect, With<Player>>,
    mut query_chunk: Query<(&Chunk, &mut TileStorage, Entity)>,
    mut world_data: ResMut<WorldData>,
    mut place_block: EventReader<PlaceBlockEvent>,
    mut update_light: EventWriter<UpdateLightEvent>,
    mut update_neighbors: EventWriter<UpdateNeighborsEvent>,
    mut play_sound: EventWriter<PlaySoundEvent>
) {
    let player_rect = query_player.single();

    for &PlaceBlockEvent { tile_pos, block } in place_block.iter() {
        if world_data.block_exists(tile_pos) { continue; }

        let new_block = Block::new(block);

        // Forbid to place a block inside the player 
        {
            let Vec2 { x, y } = tile_pos_to_world_coords(tile_pos);
            let tile_rect = FRect::new_center(x, y, TILE_SIZE, TILE_SIZE);
            if player_rect.intersects(&tile_rect) { continue; }
        }
        
        world_data.set_block(tile_pos, &new_block);

        let neighbors = world_data
            .get_block_neighbors(tile_pos, block.is_solid())
            .map_ref(|b| b.block_type);
        
        let index = Block::get_sprite_index(&neighbors, block);

        ChunkManager::spawn_block(&mut commands, &mut query_chunk, tile_pos, &new_block, index);

        update_neighbors.send(UpdateNeighborsEvent { tile_pos });
        update_light.send(UpdateLightEvent { tile_pos });
        play_sound.send(PlaySoundEvent(SoundType::BlockHit(block)));
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
    for &UpdateBlockEvent { tile_pos, block_type, update_neighbors } in update_block_events.iter() {
        let neighbors = world_data
            .get_block_neighbors(tile_pos, block_type.is_solid())
            .map_ref(|b| b.block_type);

        let chunk_pos = get_chunk_pos(tile_pos);
        let chunk_tile_pos = get_chunk_tile_pos(tile_pos);

        if let Some(block_entity) = ChunkManager::get_block_entity(&query_chunk, chunk_pos, chunk_tile_pos, block_type) {
            if let Ok(mut tile_texture) = query_tile.get_mut(block_entity) {
                tile_texture.0 = Block::get_sprite_index(&neighbors, block_type);
            }
        }

        if update_neighbors {
            update_neighbors_events.send(UpdateNeighborsEvent { tile_pos });
        }
    }
}

pub(super) fn handle_seed_event(
    mut seed_events: EventReader<SeedEvent>,
    mut update_block_events: EventWriter<UpdateBlockEvent>,
    mut world_data: ResMut<WorldData>,
    mut play_sound: EventWriter<PlaySoundEvent>
) {
    for &SeedEvent { tile_pos, seed } in seed_events.iter() {
        if let Some(block) = world_data.get_block_with_type_mut(tile_pos, BlockType::Dirt) {
            play_sound.send(PlaySoundEvent(SoundType::BlockPlace(block.block_type)));
            
            block.block_type = seed.seeded_dirt();

            update_block_events.send(UpdateBlockEvent { 
                tile_pos,
                block_type: block.block_type,
                update_neighbors: true
            });
        }
    }
}

fn break_tree(
    commands: &mut Commands, 
    world_data: &mut ResMut<WorldData>,
    chunks: &mut Query<(&Chunk, &mut TileStorage)>,
    pos: TilePos,
    tree_falling: bool
) {
    if let Some(&block) = world_data.get_block(pos) {
        if let BlockType::Tree(tree) = block.block_type {
            world_data.remove_block(pos);

            ChunkManager::remove_block(commands, chunks, pos, block.block_type);

            if tree.frame_type.is_stem() || tree_falling {
                break_tree(commands, world_data, chunks, TilePos::new(pos.x + 1, pos.y), true);
                break_tree(commands, world_data, chunks, TilePos::new(pos.x - 1, pos.y), true);
                break_tree(commands, world_data, chunks, TilePos::new(pos.x, pos.y - 1), true);
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